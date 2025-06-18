pub async fn expose_http(host: String, port: u16, bridge: String, graceful: kulfi_utils::Graceful) {
    let (id52, secret_key) = match kulfi_utils::read_or_create_key().await {
        Ok(v) => v,
        Err(e) => {
            malai::identity_read_err_msg(e);
            std::process::exit(1);
        }
    };

    let ep = match kulfi_utils::get_endpoint(secret_key).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to bind to iroh network:");
            eprintln!("{e:?}");
            std::process::exit(1);
        }
    };

    InfoMode::Startup.print(&host, port, &id52, &bridge);

    let client_pools = kulfi_utils::HttpConnectionPools::default();

    let mut graceful_mut = graceful.clone();

    loop {
        tokio::select! {
            _ = graceful_mut.show_info() => {
                InfoMode::OnExit.print(&host, port, &id52, &bridge);
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

                let client_pools = client_pools.clone();
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
                    if let Err(e) = handle_connection(conn, client_pools, host, port).await {
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

impl InfoMode {
    fn print(&self, host: &str, port: u16, id52: &str, bridge: &str) {
        use colored::Colorize;

        // Malai: Sharing http://127.0.0.1:3000 at
        // https://68tr15k68lu9f05tk03j9nnjcn1n0fqb5vdb1c3205nj8nv974ng.kulfi.site/
        // To avoid the public proxy, run your own with: malai http-bridge
        // Or use: malai browse kulfi://68tr15k68lu9f05tk03j9nnjcn1n0fqb5vdb1c3205nj8nv974ng

        if self == &InfoMode::OnExit {
            println!();
        }

        println!(
            "{}: Sharing {} at",
            "Malai".on_green().black(),
            format!("http://{host}:{port}").yellow()
        );

        println!("{}", format!("https://{id52}.{bridge}").yellow(),);

        if self != &InfoMode::OnExit {
            println!("To avoid the public proxy, run your own with: malai http-bridge");
        }

        println!(
            "\nOr use: {}",
            format!("malai browse kulfi://{id52}").yellow()
        );

        if self == &InfoMode::OnExit {
            println!("Press ctrl+c again to exit.");
        }
    }
}
