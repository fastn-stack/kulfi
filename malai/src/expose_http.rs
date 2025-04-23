pub async fn expose_http(
    host: String,
    port: u16,
    bridge: String,
    _graceful: kulfi_utils::Graceful,
    mut show_info_rx: tokio::sync::watch::Receiver<bool>,
) -> eyre::Result<()> {
    use eyre::WrapErr;
    use kulfi_utils::SecretStore;

    let id52 = kulfi_utils::read_or_create_key().await?;
    let secret_key = kulfi_utils::KeyringSecretStore::new(id52.clone()).get()?;
    let ep = kulfi_utils::get_endpoint(secret_key)
        .await
        .wrap_err_with(|| "failed to bind to iroh network")?;

    print_id52_info(&host, port, &id52, &bridge, InfoMode::Startup);

    let client_pools = kulfi_utils::HttpConnectionPools::default();

    loop {
        tokio::select! {
            _ = show_info_rx.changed() => {
                print_id52_info(&host, port, &id52, &bridge, InfoMode::OnExit);
            }
            conn = ep.accept() => {
                let conn = match conn {
                    Some(conn) => conn,
                    None => {
                        tracing::info!("no connection");
                        break;
                    }
                };

                let client_pools = client_pools.clone();
                let host = host.clone();

                tokio::spawn(async move {
                    let start = std::time::Instant::now();
                    let conn = match conn.await {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::error!("failed to convert incoming to connection: {:?}", e);
                            return;
                        }
                    };
                    if let Err(e) = handle_connection(conn, client_pools, host, port).await {
                        tracing::error!("connection error3: {:?}", e);
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
    client_pools: kulfi_utils::HttpConnectionPools,
    host: String,
    port: u16,
) -> eyre::Result<()> {
    let remote_id52 = kulfi_utils::get_remote_id52(&conn)
        .await
        .inspect_err(|e| tracing::error!("failed to get remote id: {e:?}"))?;

    tracing::info!("new client: {remote_id52}, waiting for bidirectional stream");
    loop {
        let (mut send, recv) = kulfi_utils::accept_bi(&conn, kulfi_utils::Protocol::Http)
            .await
            .inspect_err(|e| tracing::error!("failed to accept bidirectional stream: {e:?}"))?;
        tracing::info!("{remote_id52}");
        let client_pools = client_pools.clone();
        if let Err(e) =
            kulfi_utils::peer_to_http(&format!("{host}:{port}"), client_pools, &mut send, recv)
                .await
        {
            tracing::error!("failed to proxy http: {e:?}");
        }
        tracing::info!("closing send stream");
        send.finish()?;
    }
}

#[derive(PartialEq, Debug)]
enum InfoMode {
    Startup,
    OnExit,
}

fn print_id52_info(host: &str, port: u16, id52: &str, bridge: &str, mode: InfoMode) {
    use colored::Colorize;

    if mode == InfoMode::Startup {
        println!(
            "{} is now serving {}",
            "Malai".on_green().black(),
            format!("http://{host}:{port}").yellow()
        );
    }

    if mode == InfoMode::OnExit {
        // an extra empty line to make the output more readable
        // otherwise the first line is missed with keyboard input
        println!("\nServing: {}", format!("http://{host}:{port}").yellow());
    }

    println!("ID52: {}", id52.yellow());
    println!(
        "HTTP Address {}",
        format!("https://{id52}.{bridge}").yellow(),
    );

    if mode == InfoMode::OnExit {
        println!("Press ctrl+c again to exit.");
    }
}
