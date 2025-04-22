/// PeerConnections stores the iroh connections for every peer.
///
/// when a connection is broken, etc., we remove the connection from the map.
pub type PeerStreamSenders = std::sync::Arc<
    tokio::sync::Mutex<std::collections::HashMap<(SelfID52, RemoteID52), StreamRequestSender>>,
>;

type Stream = (iroh::endpoint::SendStream, kulfi_utils::FrameReader);
type StreamResult = eyre::Result<Stream>;
type ReplyChannel = tokio::sync::oneshot::Sender<StreamResult>;
type RemoteID52 = String;
type SelfID52 = String;

type StreamRequest = (kulfi_utils::Protocol, ReplyChannel);

type StreamRequestSender = tokio::sync::mpsc::Sender<StreamRequest>;
type StreamRequestReceiver = tokio::sync::mpsc::Receiver<StreamRequest>;

/// get_stream tries to check if the bidirectional stream is healthy, as simply opening
/// a bidirectional stream, or even simply writing on it does not guarantee that the stream is
/// open. only the read request times out to tell us something is wrong. this is why get_stream
/// takes the protocol as well, as every outgoing bi-direction stream must have a protocol. it
/// sends the protocol and waits for an ack. if the ack is not received within a certain time, it
/// assumes the connection is not healthy, and tries to recreate the connection.
///
/// for managing connection, we use a spawned task. this task listens for incoming stream requests
/// and manages the connection as part of the task local data.
pub async fn get_stream(
    self_endpoint: iroh::Endpoint,
    protocol: kulfi_utils::Protocol,
    remote_node_id52: RemoteID52,
    peer_stream_senders: PeerStreamSenders,
) -> eyre::Result<(iroh::endpoint::SendStream, kulfi_utils::FrameReader)> {
    use eyre::WrapErr;

    let stream_request_sender =
        get_stream_request_sender(self_endpoint, remote_node_id52, peer_stream_senders).await;

    let (reply_channel, receiver) = tokio::sync::oneshot::channel();

    stream_request_sender
        .send((protocol, reply_channel))
        .await
        .wrap_err_with(|| "failed to send on stream_request_sender")?;

    receiver.await?
}

async fn get_stream_request_sender(
    self_endpoint: iroh::Endpoint,
    remote_node_id52: RemoteID52,
    peer_stream_senders: PeerStreamSenders,
) -> StreamRequestSender {
    let self_id52 = kulfi_utils::public_key_to_id52(&self_endpoint.node_id());
    let mut senders = peer_stream_senders.lock().await;

    if let Some(sender) = senders.get(&(self_id52.clone(), remote_node_id52.clone())) {
        return sender.clone();
    }

    // TODO: figure out if the mpsc::channel is the right size
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    senders.insert(
        (self_id52.clone(), remote_node_id52.clone()),
        sender.clone(),
    );
    drop(senders);

    tokio::spawn(async move {
        connection_manager(receiver, self_endpoint, remote_node_id52.clone()).await;

        // cleanup the peer_stream_senders map, so no future tasks will try to use this.
        let mut senders = peer_stream_senders.lock().await;
        senders.remove(&(self_id52.clone(), remote_node_id52));
    });

    sender
}

async fn connection_manager(
    mut receiver: StreamRequestReceiver,
    self_endpoint: iroh::Endpoint,
    remote_node_id52: RemoteID52,
) {
    let e = match connection_manager_(&mut receiver, self_endpoint, remote_node_id52.clone()).await
    {
        Ok(()) => {
            tracing::info!("connection manager closed");
            return;
        }
        Err(e) => e,
    };

    // what is our error handling strategy?
    //
    // since an error has just occurred on our connection, it is best to cancel all concurrent
    // tasks that depend on this connection, and let the next task recreate the connection, this
    // way things are clean.
    //
    // we can try to keep the concurrent tasks open, and retry connection, but it increases the
    // complexity of implementation, and it is not worth it for now.
    //
    // also note that connection_manager() and it's caller, get_stream(), are called to create the
    // initial stream only, this error handling strategy will work for concurrent requests that are
    // waiting for the stream to be created. the tasks that already got the stream will not be
    // affected by this. tho, since something wrong has happened with the connection, they will
    // eventually fail too.
    tracing::error!("connection manager worker error: {e:?}");

    // once we close the receiver, any tasks that have gotten access to the corresponding sender
    // will fail when sending.
    receiver.close();

    // send an error to all the tasks that are waiting for stream for this receiver.
    while let Some((_protocol, reply_channel)) = receiver.recv().await {
        if reply_channel
            .send(Err(eyre::anyhow!("failed to create connection: {e:?}")))
            .is_err()
        {
            tracing::error!("failed to send error reply: {e:?}");
        }
    }
}

async fn connection_manager_(
    receiver: &mut StreamRequestReceiver,
    self_endpoint: iroh::Endpoint,
    remote_node_id52: RemoteID52,
) -> eyre::Result<()> {
    let conn = match self_endpoint
        .connect(
            kulfi_utils::id52_to_public_key(&remote_node_id52)?,
            kulfi_utils::APNS_IDENTITY,
        )
        .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("failed to create connection: {e:?}");
            return Err(eyre::anyhow!("failed to create connection: {e:?}"));
        }
    };

    let timeout = std::time::Duration::from_secs(12);
    let mut idle_counter = 0;

    loop {
        if idle_counter > 4 {
            tracing::info!("connection idle timeout, returning");
            // this ensures we keep a connection open only for 12 * 5 seconds = 1 min
            break;
        }

        tokio::select! {
            _ = tokio::time::sleep(timeout) => {
                if let Err(e) = kulfi_utils::ping(&conn).await {
                    tracing::error!("pinging failed: {e:?}");
                    break;
                }
                idle_counter += 1;
            },
            Some((protocol, reply_channel)) = receiver.recv() => {
                tracing::info!("connection ping: {protocol:?}, idle counter: {idle_counter}");
                idle_counter = 0;
                // is this a good idea to serialize this part? if 10 concurrent requests come in, we will
                // handle each one sequentially. the other alternative is to spawn a task for each request.
                // so which is better?
                //
                // in general, if we do it in parallel via spawning, we will have better throughput.
                //
                // and we are not worried about having too many concurrent tasks, tho iroh has a limit on
                // concurrent tasks[1], with a default of 100[2]. it is actually a todo to find out what
                // happens when we hit this limit, do they handle it by queueing the tasks, or do they
                // return an error. if they queue then we wont have to implement queue logic.
                //
                // [1]: https://docs.rs/iroh/0.34.1/iroh/endpoint/struct.TransportConfig.html#method.max_concurrent_bidi_streams
                // [2]: https://docs.rs/iroh-quinn-proto/0.13.0/src/iroh_quinn_proto/config/transport.rs.html#354
                //
                // but all that is besides the point, we are worried about resilience right now, not
                // throughput per se (throughput is secondary goal, resilience primary).
                //
                // say we have 10 concurrent requests and lets say if we spawned a task for each, what
                // happens in error case? say connection failed, the device switched from wifi to 4g, or
                // whatever? in the handler task, we are putting a timeout on the read. in the serial case
                // the first request will timeout, and all subsequent requests will get immediately an
                // error. its predictable, its clean.
                //
                // if the tasks were spawned, each will timeout independently.
                //
                // we can also no longer rely on this function, connection_manager_, returning an error for
                // them, so our connection_handler strategy will interfere, we would have read more requests
                // off of receiver.
                //
                // do note that this is not a clear winner problem, this is a tradeoff, we lose throughput,
                // as in best case scenario, 10 concurrent tasks will be better. we will have to revisit
                // this in future when we are performance optimising things.
                if let Err(e) = handle_request(&conn, protocol, reply_channel).await {
                    tracing::error!("failed to handle request: {e:?}");
                    // note: we are intentionally not calling conn.close(). why? so that if some existing
                    // stream is still open, if we explicitly call close on the connection, that stream will
                    // immediately fail as well, and we do not want that. we want to let the stream fail
                    // on its own, maybe it will work, maybe it will not.
                    return Err(e);
                }
            }
            else => break,
        }
    }

    Ok(())
}

async fn handle_request(
    conn: &iroh::endpoint::Connection,
    protocol: kulfi_utils::Protocol,
    reply_channel: ReplyChannel,
) -> eyre::Result<()> {
    use eyre::WrapErr;
    use tokio_stream::StreamExt;

    tracing::trace!("handling request: {protocol:?}");

    let (mut send, recv) = match conn.open_bi().await {
        Ok(v) => {
            tracing::trace!("opened bi-stream");
            v
        }
        Err(e) => {
            tracing::error!("failed to open_bi: {e:?}");
            return Err(eyre::anyhow!("failed to open_bi: {e:?}"));
        }
    };

    send.write_all(
        &serde_json::to_vec(&protocol)
            .wrap_err_with(|| format!("failed to serialize protocol: {protocol:?}"))?,
    )
    .await?;
    send.write(b"\n")
        .await
        .wrap_err_with(|| "failed to write newline")?;
    let mut recv = kulfi_utils::frame_reader(recv);

    let msg = match recv.next().await {
        Some(v) => v.wrap_err_with(|| "failed to receive response for protocol")?,
        None => {
            tracing::error!("failed to read from incoming connection");
            return Err(eyre::anyhow!("failed to read from incoming connection"));
        }
    };

    if msg != kulfi_utils::ACK {
        tracing::error!("failed to read ack: {msg:?}");
        return Err(eyre::anyhow!("failed to read ack: {msg:?}"));
    }

    tracing::trace!("received ack");

    reply_channel.send(Ok((send, recv))).unwrap_or_else(|e| {
        tracing::error!("failed to send reply: {e:?}");
    });

    tracing::trace!("handle_request done");

    Ok(())
}
