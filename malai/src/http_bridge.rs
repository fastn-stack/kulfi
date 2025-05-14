pub async fn http_bridge(
    port: u16,
    proxy_target: Option<String>,
    graceful: kulfi_utils::Graceful,
    post_start: impl FnOnce(u16) -> eyre::Result<()>,
) -> eyre::Result<()> {
    use eyre::WrapErr;

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .wrap_err_with(|| {
            format!("can not listen on port {port}, is it busy, or you do not have root access?")
        })?;

    // because the caller can pass the port as 0 if they want to bind to a random port
    let port = listener.local_addr()?.port();

    post_start(port)?;

    println!("Listening on http://127.0.0.1:{port}");

    let peer_connections = kulfi_utils::PeerStreamSenders::default();

    let mut g = graceful.clone();
    let g2 = graceful.clone();
    g.spawn(async move { listener_loop(listener, g2, peer_connections, proxy_target).await });

    loop {
        tokio::select! {
            () = graceful.cancelled() => {
                tracing::info!("Stopping control server.");
                break;
            }
            r = g.show_info() => {
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
        }
    }

    Ok(())
}

async fn listener_loop(
    listener: tokio::net::TcpListener,
    graceful: kulfi_utils::Graceful,
    peer_connections: kulfi_utils::PeerStreamSenders,
    proxy_target: Option<String>,
) {
    loop {
        tracing::trace!("waiting for connection");

        match listener.accept().await {
            Ok((stream, _addr)) => {
                tracing::info!("got connection");
                let g = graceful.clone();
                let peer_connections = peer_connections.clone();
                let proxy_target = proxy_target.clone();
                graceful.spawn(async move {
                    let self_endpoint = malai::global_iroh_endpoint().await;
                    handle_connection(self_endpoint, stream, g, peer_connections, proxy_target)
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

pub async fn handle_connection(
    self_endpoint: iroh::Endpoint,
    stream: tokio::net::TcpStream,
    graceful: kulfi_utils::Graceful,
    peer_connections: kulfi_utils::PeerStreamSenders,
    proxy_target: Option<String>,
) {
    let io = hyper_util::rt::TokioIo::new(stream);

    let builder =
        hyper_util::server::conn::auto::Builder::new(hyper_util::rt::tokio::TokioExecutor::new());
    // the following builder runs only http2 service, whereas the hyper_util auto Builder runs an
    // http1.1 server that upgrades to http2 if the client requests.
    // let builder = hyper::server::conn::http2::Builder::new(hyper_util::rt::tokio::TokioExecutor::new());
    tokio::pin! {
        let conn = builder
            .serve_connection(
                io,
                hyper::service::service_fn(|r| handle_request(r, self_endpoint.clone(), peer_connections.clone(), proxy_target.clone(), graceful.clone())),
            );
    }

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
    r: hyper::Request<hyper::body::Incoming>,
    self_endpoint: iroh::Endpoint,
    peer_connections: kulfi_utils::PeerStreamSenders,
    proxy_target: Option<String>,
    graceful: kulfi_utils::Graceful,
) -> kulfi_utils::http::ProxyResult {
    let peer_id = match get_peer_id52_from_host(
        r.headers().get("Host").and_then(|h| h.to_str().ok()),
        proxy_target,
    ) {
        Ok(peer_id) => peer_id,
        Err(e) => {
            tracing::error!("failed to get peer id from request: {e:?}");
            return Ok(kulfi_utils::bad_request!(
                "failed to get peer id from request"
            ));
        }
    };

    tracing::info!("got request for {peer_id}");

    kulfi_utils::http_to_peer(
        kulfi_utils::http::incoming_to_bytes(r).await?,
        self_endpoint,
        &peer_id,
        peer_connections,
        Default::default(), /* RequestPatch */
        graceful,
    )
    .await
}

fn get_peer_id52_from_host(
    host: Option<&str>,
    proxy_target: Option<String>,
) -> eyre::Result<String> {
    let first = match host.and_then(|h| h.split_once('.')) {
        Some((first, _)) => first,
        None => {
            tracing::error!("got http request without Host header");
            return Err(eyre::anyhow!("got http request without Host header"));
        }
    };

    if first == "127" {
        if let Some(target) = proxy_target {
            return Ok(target);
        }
    }

    if first.len() != 52 && proxy_target.is_none() {
        tracing::error!(peer_id = %first, "request received for invalid peer id");
        return Err(eyre::anyhow!("got http request with invalid peer id"));
    }

    if let Some(target) = proxy_target {
        if first != target {
            tracing::error!(peer_id = %first, proxy_target = %target, "request for peer_id is not allowed");
            return Err(eyre::anyhow!("got http request with invalid peer id"));
        }
    }

    Ok(first.to_string())
}
