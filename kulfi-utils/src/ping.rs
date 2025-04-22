pub const PONG: &[u8] = b"pong\n";
pub const ACK_PONG: &[u8] = b"ack\npong\n";

pub async fn ping(conn: &iroh::endpoint::Connection) -> eyre::Result<()> {
    tracing::info!("ping called");
    let (mut send_stream, mut recv_stream) = conn.open_bi().await?;
    tracing::info!("got bi, sending ping");
    send_stream
        .write_all(&serde_json::to_vec(&kulfi_utils::Protocol::Ping)?)
        .await?;
    tracing::info!("sent ping, sending newline");
    send_stream.write_all("\n".as_bytes()).await?;
    tracing::info!("newline sent, waiting for reply");
    let msg = recv_stream
        .read_to_end(1000)
        .await
        .inspect_err(|e| tracing::error!("failed to read: {e}"))?;
    tracing::info!("got {:?}, {PONG:?}", str::from_utf8(&msg));
    if msg != ACK_PONG {
        return Err(eyre::anyhow!("expected {PONG:?}, got {msg:?}"));
    }
    tracing::info!("got reply, finishing stream");
    send_stream.finish()?;
    tracing::info!("finished stream");
    Ok(())
}
