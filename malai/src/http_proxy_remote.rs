pub async fn http_proxy_remote(graceful: kulfi_utils::Graceful) -> eyre::Result<()> {
    use eyre::WrapErr;

    let (id52, secret_key) = kulfi_utils::read_or_create_key().await?;
    let ep = kulfi_utils::get_endpoint(secret_key)
        .await
        .wrap_err_with(|| "failed to bind to iroh network")?;

    let http_connection_pools = kulfi_utils::HttpConnectionPools::default();
    InfoMode::Startup.print(&id52);

    let mut graceful_mut = graceful.clone();

    loop {
        tokio::select! {
            _ = graceful_mut.show_info() => {
                InfoMode::OnExit.print(&id52);
            }
            _ = graceful.cancelled() => {
                tracing::info!("Stopping http-proxy server.");
                break;
            }
            conn = ep.accept() => {
                let conn = match conn {
                    Some(conn) => conn,
                    None => {
                        tracing::info!("no connection");
                        break;
                    }
                };

                let graceful_for_handle_connection = graceful.clone();
                let http_connection_pools = http_connection_pools.clone();
                graceful.spawn(async move {
                    let start = std::time::Instant::now();
                    let conn = match conn.await {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::error!("failed to convert incoming to connection: {e:?}");
                            return;
                        }
                    };
                    if let Err(e) = handle_connection(conn, http_connection_pools, graceful_for_handle_connection).await {
                        tracing::error!("connection error3: {e:?}");
                    }
                    tracing::info!("connection handled in {:?}", start.elapsed());
                });
            }
        }
    }

    ep.close().await;
    Ok(())
}

async fn handle_connection(
    conn: iroh::endpoint::Connection,
    http_connection_pools: kulfi_utils::HttpConnectionPools,
    graceful: kulfi_utils::Graceful,
) -> eyre::Result<()> {
    let remote_id52 = kulfi_utils::get_remote_id52(&conn)
        .await
        .inspect_err(|e| tracing::error!("failed to get remote id: {e:?}"))?;

    tracing::info!("new client: {remote_id52}, waiting for bidirectional stream");
    loop {
        let (extra, mut send, recv): (malai::ProxyData, _, _) =
            kulfi_utils::accept_bi_with(&conn, kulfi_utils::Protocol::HttpProxy)
                .await
                .inspect_err(|e| tracing::error!("failed to accept bidirectional stream: {e:?}"))?;
        tracing::info!("got connection from {remote_id52}, extra: {extra:?}");

        let http_connection_pools = http_connection_pools.clone();
        graceful.spawn(async move {
            if let Err(e) = match extra {
                malai::ProxyData::Connect { addr } => {
                    kulfi_utils::peer_to_tcp(&addr, send, recv).await
                }
                malai::ProxyData::Http { addr } => {
                    kulfi_utils::peer_to_http(&addr, http_connection_pools, &mut send, recv).await
                }
            } {
                tracing::error!("failed to proxy tcp: {e:?}");
            }
            tracing::info!("closing send stream");
        });
    }
}

#[derive(PartialEq, Debug)]
enum InfoMode {
    Startup,
    OnExit,
}

impl InfoMode {
    fn print(&self, id52: &str) {
        use colored::Colorize;

        // Malai: Running Public HTTP Proxy at 68tr15k68lu9f05tk03j9nnjcn1n0fqb5vdb1c3205nj8nv974ng.
        // Run `malai http-proxy-bridge 68tr15k68lu9f05tk03j9nnjcn1n0fqb5vdb1c3205nj8nv974ng` on
        // any machine to access this proxy server.

        if self == &InfoMode::OnExit {
            println!();
        }

        println!(
            "{cli}: Running Public HTTP Proxy at {id52}.",
            cli = "Malai".on_green().black(),
            id52 = id52.yellow(),
        );

        println!(
            "Run {cli} on any machine to access this proxy server.",
            cli = format!("malai http-proxy {id52}").yellow(),
        );
    }
}
