pub async fn expose_tcp(
    host: String,
    port: u16,
    secret_key_path: &std::path::Path,
    mut graceful: kulfi_utils::Graceful,
) -> eyre::Result<()> {
    use eyre::WrapErr;

    let secret_key = kulfi_utils::read_or_create_secret_key(secret_key_path)?;
    let id52 = kulfi_utils::public_key_to_id52(&secret_key.public());
    let ep = kulfi_utils::get_endpoint(secret_key)
        .await
        .wrap_err_with(|| "failed to bind to iroh network")?;

    InfoMode::Startup.print(port, &id52);

    let g = graceful.clone();
    loop {
        tokio::select! {
            _ = graceful.show_info() => {
                InfoMode::OnExit.print(port, &id52);
            }
            _ = g.cancelled() => {
                tracing::info!("Stopping control server.");
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
                let host = host.clone();

                graceful.spawn(async move {
                    let start = std::time::Instant::now();
                    let conn = match conn.await {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::error!("failed to convert incoming to connection: {:?}", e);
                            return;
                        }
                    };
                    if let Err(e) = handle_connection(conn, host, port).await {
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
    host: String,
    port: u16,
) -> eyre::Result<()> {
    let remote_id52 = kulfi_utils::get_remote_id52(&conn)
        .await
        .inspect_err(|e| tracing::error!("failed to get remote id: {e:?}"))?;

    tracing::info!("new client: {remote_id52}, waiting for bidirectional stream");
    loop {
        let (mut send, recv) = kulfi_utils::accept_bi(&conn, kulfi_utils::Protocol::Tcp)
            .await
            .inspect_err(|e| tracing::error!("failed to accept bidirectional stream: {e:?}"))?;
        tracing::info!("{remote_id52}");
        if let Err(e) =
            kulfi_utils::peer_to_tcp(&remote_id52, &format!("{host}:{port}"), &mut send, recv).await
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

impl InfoMode {
    fn print(&self, port: u16, id52: &str) {
        use colored::Colorize;

        // Malai: Sharing port <port>
        // Run malai tcp-bridge <id52> <some-port> to connect to it from any machine.
        // Press ctrl+c again to exit.

        if self == &InfoMode::OnExit {
            println!();
        }

        if self == &InfoMode::Startup {
            println!("{}: Sharing port {port}", "Malai".on_green().black(),);
        }

        println!(
            "Run {}",
            format!("malai tcp-bridge {id52} <some-port>").yellow()
        );
        println!("to connect to it from any machine.");

        if self == &InfoMode::OnExit {
            println!("Press ctrl+c again to exit.");
        }
    }
}
