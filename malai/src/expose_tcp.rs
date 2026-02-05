pub async fn expose_tcp(
    host: String,
    port: u16,
    id52: String,
    secret_key: kulfi_id52::SecretKey,
    graceful: kulfi_utils::Graceful,
) {
    let ep = match kulfi_utils::get_endpoint(secret_key).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to bind to iroh network:");
            eprintln!("{e:?}");
            std::process::exit(1);
        }
    };

    InfoMode::Startup.print(port, &id52);

    let mut graceful_mut = graceful.clone();
    loop {
        let graceful_for_handle_connection = graceful.clone();

        tokio::select! {
            _ = graceful_mut.show_info() => {
                InfoMode::OnExit.print(port, &id52);
            }
            _ = graceful.cancelled() => {
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
                    if let Err(e) = handle_connection(conn, host, port, graceful_for_handle_connection).await {
                        tracing::error!("connection error3: {:?}", e);
                    }
                    tracing::info!("connection handled in {:?}", start.elapsed());
                });
            }
        }
    }

    ep.close().await;
}

async fn handle_connection(
    conn: iroh::endpoint::Connection,
    host: String,
    port: u16,
    graceful: kulfi_utils::Graceful,
) -> eyre::Result<()> {
    let remote_id52 = kulfi_utils::get_remote_id52(&conn);

    tracing::info!("new client: {remote_id52}, waiting for bidirectional stream");
    loop {
        let (send, recv) = kulfi_utils::accept_bi(&conn, kulfi_utils::Protocol::Tcp)
            .await
            .inspect_err(|e| tracing::error!("failed to accept bidirectional stream: {e:?}"))?;
        tracing::info!("{remote_id52}");
        let addr = format!("{host}:{port}");
        graceful.spawn(async move {
            if let Err(e) = kulfi_utils::peer_to_tcp(&addr, send, recv).await {
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
