pub async fn tcp_bridge(
    port: u16,
    proxy_target: String,
    graceful: kulfi_utils::Graceful,
) -> eyre::Result<()> {
    use eyre::WrapErr;

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .wrap_err_with(
            || "can not listen to port 80, is it busy, or you do not have root access?",
        )?;

    println!("Listening on http://127.0.0.1:{port}");

    let peer_connections = kulfi_utils::PeerStreamSenders::default();

    loop {
        tokio::select! {
            _ = graceful.cancelled() => {
                tracing::info!("Stopping control server.");
                break;
            }
            val = listener.accept() => {
                let self_endpoint = malai::global_iroh_endpoint().await;
                let g = graceful.clone();
                let peer_connections = peer_connections.clone();
                let proxy_target = proxy_target.clone();
                match val {
                    Ok((stream, _addr)) => {
                        graceful.tracker.spawn(async move { handle_connection(self_endpoint, stream, g, peer_connections, proxy_target).await });
                    },
                    Err(e) => {
                        tracing::error!("failed to accept: {e:?}");
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn handle_connection(
    _self_endpoint: iroh::Endpoint,
    _stream: tokio::net::TcpStream,
    _graceful_shutdown_rx: kulfi_utils::Graceful,
    _peer_connections: kulfi_utils::PeerStreamSenders,
    _proxy_target: String,
) {
    todo!()
}
