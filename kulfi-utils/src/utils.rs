pub fn id52_to_public_key(id: &str) -> eyre::Result<iroh::PublicKey> {
    use eyre::WrapErr;

    let bytes = data_encoding::BASE32_DNSSEC.decode(id.as_bytes())?;
    if bytes.len() != 32 {
        return Err(eyre::anyhow!(
            "read: id has invalid length: {}",
            bytes.len()
        ));
    }

    let bytes: [u8; 32] = bytes.try_into().unwrap(); // unwrap ok as already asserted

    iroh::PublicKey::from_bytes(&bytes).wrap_err_with(|| "failed to parse id to public key")
}

pub fn public_key_to_id52(key: &iroh::PublicKey) -> String {
    data_encoding::BASE32_DNSSEC.encode(key.as_bytes())
}

pub type FrameReader =
    tokio_util::codec::FramedRead<iroh::endpoint::RecvStream, tokio_util::codec::LinesCodec>;

pub fn frame_reader(recv: iroh::endpoint::RecvStream) -> FrameReader {
    FrameReader::new(recv, tokio_util::codec::LinesCodec::new())
}

pub async fn get_remote_id52(conn: &iroh::endpoint::Connection) -> eyre::Result<String> {
    let remote_node_id = match conn.remote_node_id() {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("could not read remote node id: {e}, closing connection");
            // TODO: is this how we close the connection in error cases or do we send some error
            //       and wait for other side to close the connection?
            let e2 = conn.closed().await;
            tracing::info!("connection closed: {e2}");
            // TODO: send another error_code to indicate bad remote node id?
            conn.close(0u8.into(), &[]);
            return Err(eyre::anyhow!("could not read remote node id: {e}"));
        }
    };

    Ok(public_key_to_id52(&remote_node_id))
}

async fn ack(send: &mut iroh::endpoint::SendStream) -> eyre::Result<()> {
    send.write_all(format!("{}\n", kulfi_utils::ACK).as_bytes())
        .await?;
    Ok(())
}

pub async fn accept_bi(
    conn: &iroh::endpoint::Connection,
    expected: kulfi_utils::Protocol,
) -> eyre::Result<(iroh::endpoint::SendStream, FrameReader)> {
    loop {
        match accept_bi_(conn).await? {
            (mut send, recv, kulfi_utils::Protocol::Ping) => {
                tracing::info!("got ping");
                if !recv.read_buffer().is_empty() {
                    send.write_all(b"error: ping message should not have payload\n")
                        .await?;
                    return Err(eyre::anyhow!("ping got extra data"));
                }
                tracing::info!("sending PONG");
                send.write_all(kulfi_utils::PONG)
                    .await
                    .inspect_err(|e| tracing::error!("failed to write PONG: {e:?}"))?;
                tracing::info!("sent PONG");
            }
            (s, r, found) => {
                if found != expected {
                    return Err(eyre::anyhow!("expected: {expected:?}, got {found:?}"));
                }
                return Ok((s, r));
            }
        }
    }
}

async fn accept_bi_(
    conn: &iroh::endpoint::Connection,
) -> eyre::Result<(
    iroh::endpoint::SendStream,
    FrameReader,
    kulfi_utils::Protocol,
)> {
    use tokio_stream::StreamExt;

    let (mut send, recv) = conn.accept_bi().await?;
    tracing::info!("got bidirectional stream");
    let mut recv = frame_reader(recv);
    let msg = match recv.next().await {
        Some(v) => v?,
        None => {
            tracing::error!("failed to read from incoming connection");
            return Err(eyre::anyhow!("failed to read from incoming connection"));
        }
    };
    let msg = serde_json::from_str::<kulfi_utils::Protocol>(&msg)
        .inspect_err(|e| tracing::error!("json error for {msg}: {e}"))?;

    ack(&mut send).await?;
    Ok((send, recv, msg))
}
