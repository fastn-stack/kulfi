pub async fn http_proxy(
    port: u16,
    remote: String,
    graceful: kulfi_utils::Graceful,
    post_start: impl FnOnce(u16) -> eyre::Result<()>,
) {
    use eyre::WrapErr;

    let listener = match tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await.wrap_err_with(|| {
        format!("can not listen on port {port}, is it busy, or you do not have root access?")
    }) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to port {port}: {e:?}");
            std::process::exit(1);
        }
    };

    let port = match listener.local_addr() {
        Ok(addr) => addr.port(),
        Err(e) => {
            eprintln!("Failed to get local address: {e:?}");
            std::process::exit(1);
        }
    };

    if let Err(e) = post_start(port) {
        eprintln!("Failed to run post start function: {e:?}");
    }

    println!("Listening on http://127.0.0.1:{port}");

    let peer_connections = kulfi_utils::PeerStreamSenders::default();

    let mut graceful_mut = graceful.clone();
    loop {
        tokio::select! {
            () = graceful.cancelled() => {
                tracing::info!("Stopping control server.");
                break;
            }
            r = graceful_mut.show_info() => {
                match r {
                    Ok(_) => {
                        println!("Listening on http://127.0.0.1:{port}");
                        println!("Press ctrl+c again to exit.");
                    }
                    Err(e) => {
                        tracing::error!("failed to show info: {e:?}");
                    }
                }
            }
            r = listener.accept() => {
                match r {
                    Ok((stream, _addr)) => {
                        tracing::info!("got connection");
                        let graceful_for_handle_connection = graceful.clone();
                        let peer_connections = peer_connections.clone();
                        let remote = remote.clone();
                        graceful.spawn(async move {
                            let self_endpoint = kulfi_utils::global_iroh_endpoint().await;
                            handle_connection(
                                self_endpoint,
                                stream,
                                graceful_for_handle_connection,
                                peer_connections,
                                remote,
                            )
                            .await
                        });
                    }
                    Err(e) => {
                        tracing::error!("failed to accept: {e:?}");
                        break;
                    }
                }
            }
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ProxyData {
    Connect { addr: String },
    Http { addr: String },
}

pub async fn handle_connection(
    self_endpoint: iroh::Endpoint,
    stream: tokio::net::TcpStream,
    graceful: kulfi_utils::Graceful,
    peer_connections: kulfi_utils::PeerStreamSenders,
    remote: String,
) {
    let io = hyper_util::rt::TokioIo::new(stream);

    let conn = hyper::server::conn::http1::Builder::new().serve_connection(
        io,
        hyper::service::service_fn(|r| {
            handle_request(
                r,
                self_endpoint.clone(),
                peer_connections.clone(),
                remote.clone(),
                graceful.clone(),
            )
        }),
    );

    let mut conn = conn.with_upgrades();
    let mut conn = std::pin::Pin::new(&mut conn);

    if let Err(e) = tokio::select! {
        _ = graceful.cancelled() => {
            conn.as_mut().graceful_shutdown();
            conn.await
        }
        r = &mut conn => r,
    } {
        tracing::error!("connection error2: {e:?}");
    }
}

async fn handle_request(
    mut r: hyper::Request<hyper::body::Incoming>,
    self_endpoint: iroh::Endpoint,
    peer_connections: kulfi_utils::PeerStreamSenders,
    remote: String,
    graceful: kulfi_utils::Graceful,
) -> kulfi_utils::http::ProxyResult<eyre::Error> {
    tracing::info!("got request for {remote}");

    let graceful_for_upgrade = graceful.clone();
    let host = match r
        .uri()
        .host()
        .map(|h| format!("{h}:{}", r.uri().port_u16().unwrap_or(80)))
    {
        Some(v) => v,
        None => {
            tracing::error!("got http request without Host header");
            return Ok(kulfi_utils::bad_request!(
                "got http request without Host header"
            ));
        }
    };

    tracing::trace!("host: {host}, method: {}, url: {}", r.method(), r.uri());

    if r.method() == hyper::Method::CONNECT {
        tracing::trace!("upgrading connection");
        // set up a future that will eventually receive the upgraded
        // connection and talk a new protocol, and spawn the future
        // into the runtime.
        //
        // note: this can't possibly be fulfilled until the 101 response (SWITCHING_PROTOCOLS)
        // is returned below, so it's better to spawn this future instead of
        // waiting for it to complete to then return a response.
        graceful.spawn(async move {
            if let Err(e) = handle_upgrade(
                r,
                host,
                self_endpoint,
                remote,
                peer_connections,
                graceful_for_upgrade,
            )
            .await
            {
                tracing::error!("failed to handle: {e}")
            }
        });

        let mut res = hyper::Response::default();
        *res.status_mut() = hyper::http::StatusCode::OK;
        Ok(res)
    } else {
        tracing::trace!("regular (non upgrade) http request");
        r.headers_mut().remove(hyper::header::CONNECTION);
        kulfi_utils::http_to_peer(
            kulfi_utils::ProtocolHeader {
                protocol: kulfi_utils::Protocol::HttpProxy,
                extra: Some(serde_json::to_string(&ProxyData::Http {
                    addr: host.to_string(),
                })?),
            },
            r,
            self_endpoint,
            &remote,
            peer_connections,
            Default::default(), /* RequestPatch */
            graceful,
        )
        .await
    }
}

async fn handle_upgrade(
    mut r: hyper::Request<hyper::body::Incoming>,
    host: String,
    self_endpoint: iroh::Endpoint,
    remote: String,
    peer_connections: kulfi_utils::PeerStreamSenders,
    graceful: kulfi_utils::Graceful,
) -> eyre::Result<()> {
    // todo: what all can we upgrade to?

    tracing::info!("upgrading connection to {host} for {remote}");

    let upgraded = match hyper::upgrade::on(&mut r).await {
        Ok(upgraded) => upgraded,
        Err(e) => {
            return Err(eyre::anyhow!("failed to upgrade connection: {e}"));
        }
    };

    tracing::trace!("upgraded connection to {host}");

    let upgraded = hyper_util::rt::TokioIo::new(upgraded);
    let (tcp_recv, tcp_send) = tokio::io::split(upgraded);

    let (send, recv) = kulfi_utils::get_stream(
        self_endpoint,
        kulfi_utils::ProtocolHeader {
            protocol: kulfi_utils::Protocol::HttpProxy,
            extra: Some(serde_json::to_string(&ProxyData::Connect {
                addr: host.to_string(),
            })?),
        },
        remote.to_string(),
        peer_connections.clone(),
        graceful,
    )
    .await?;

    tracing::trace!("got stream for {remote}");
    kulfi_utils::pipe_tcp_stream_over_iroh(tcp_recv, tcp_send, send, recv).await?;
    tracing::trace!("finished handling upgrade for {remote}");

    Ok(())
}
