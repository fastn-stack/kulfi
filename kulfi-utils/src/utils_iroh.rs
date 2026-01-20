// Functions that work with iroh types

pub fn get_remote_id52(conn: &iroh::endpoint::Connection) -> String {
    let remote_id = conn.remote_id();

    // Convert iroh::EndpointId to ID52 string
    let bytes = remote_id.as_bytes();
    data_encoding::BASE32_DNSSEC.encode(bytes)
}

async fn ack(send: &mut iroh::endpoint::SendStream) -> eyre::Result<()> {
    tracing::trace!("sending ack");
    send.write_all(format!("{}\n", crate::ACK).as_bytes())
        .await?;
    tracing::trace!("sent ack");
    Ok(())
}

pub async fn accept_bi(
    conn: &iroh::endpoint::Connection,
    expected: crate::Protocol,
) -> eyre::Result<(iroh::endpoint::SendStream, iroh::endpoint::RecvStream)> {
    loop {
        tracing::trace!("accepting bidirectional stream");
        match accept_bi_(conn).await? {
            (mut send, _recv, crate::Protocol::Ping) => {
                tracing::trace!("got ping");
                tracing::trace!("sending PONG");
                send.write_all(crate::PONG)
                    .await
                    .inspect_err(|e| tracing::error!("failed to write PONG: {e:?}"))?;
                tracing::trace!("sent PONG");
            }
            (s, r, found) => {
                tracing::trace!("got bidirectional stream: {found:?}");
                if found != expected {
                    return Err(eyre::anyhow!("expected: {expected:?}, got {found:?}"));
                }
                return Ok((s, r));
            }
        }
    }
}

pub async fn accept_bi_with<T: serde::de::DeserializeOwned>(
    conn: &iroh::endpoint::Connection,
    expected: crate::Protocol,
) -> eyre::Result<(T, iroh::endpoint::SendStream, iroh::endpoint::RecvStream)> {
    let (send, mut recv) = accept_bi(conn, expected).await?;
    let next = next_json(&mut recv)
        .await
        .inspect_err(|e| tracing::error!("failed to read next message: {e}"))?;

    Ok((next, send, recv))
}

async fn accept_bi_(
    conn: &iroh::endpoint::Connection,
) -> eyre::Result<(
    iroh::endpoint::SendStream,
    iroh::endpoint::RecvStream,
    crate::Protocol,
)> {
    tracing::trace!("accept_bi_ called");
    let (mut send, mut recv) = conn.accept_bi().await?;
    tracing::trace!("accept_bi_ got send and recv");

    let msg: crate::Protocol = next_json(&mut recv)
        .await
        .inspect_err(|e| tracing::error!("failed to read next message: {e}"))?;

    tracing::trace!("msg: {msg:?}");

    ack(&mut send).await?;

    tracing::trace!("ack sent");
    Ok((send, recv, msg))
}

/// Read until a newline character is encountered, then deserialize the buffer as JSON
pub async fn next_json<T: serde::de::DeserializeOwned>(
    recv: &mut iroh::endpoint::RecvStream,
) -> eyre::Result<T> {
    // NOTE: the capacity is just a guess to avoid reallocations
    let mut buffer = Vec::with_capacity(1024);

    loop {
        let mut byte = [0u8];
        let n = recv.read(&mut byte).await?;

        if n == Some(0) || n.is_none() {
            return Err(eyre::anyhow!(
                "connection closed while reading response header"
            ));
        }

        if byte[0] == b'\n' {
            break;
        } else {
            buffer.push(byte[0]);
        }
    }

    Ok(serde_json::from_slice(&buffer)?)
}

/// Read until a newline character is encountered, then deserialize the buffer as JSON
pub async fn next_string(recv: &mut iroh::endpoint::RecvStream) -> eyre::Result<String> {
    // NOTE: the capacity is just a guess to avoid reallocations
    let mut buffer = Vec::with_capacity(1024);

    loop {
        let mut byte = [0u8];
        let n = recv.read(&mut byte).await?;

        if n == Some(0) || n.is_none() {
            return Err(eyre::anyhow!(
                "connection closed while reading response header"
            ));
        }

        if byte[0] == b'\n' {
            break;
        } else {
            buffer.push(byte[0]);
        }
    }

    String::from_utf8(buffer).map_err(|e| eyre::anyhow!("failed to convert bytes to string: {e}"))
}

pub async fn global_iroh_endpoint() -> iroh::Endpoint {
    async fn new_iroh_endpoint() -> iroh::Endpoint {
        // TODO: read secret key from ENV VAR
        iroh::Endpoint::builder()
            .discovery(iroh::discovery::pkarr::PkarrPublisher::n0_dns())
            .discovery(iroh::discovery::dns::DnsDiscovery::n0_dns())
            .discovery(iroh::discovery::mdns::MdnsDiscovery::builder())
            .alpns(vec![crate::APNS_IDENTITY.into()])
            .bind()
            .await
            .expect("failed to create iroh Endpoint")
    }

    static IROH_ENDPOINT: tokio::sync::OnceCell<iroh::Endpoint> =
        tokio::sync::OnceCell::const_new();
    IROH_ENDPOINT.get_or_init(new_iroh_endpoint).await.clone()
}
