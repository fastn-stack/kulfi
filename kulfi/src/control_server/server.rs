pub async fn handle_connection(
    stream: tokio::net::TcpStream,
    graceful: fastn_net::Graceful,
    id_map: fastn_net::IDMap,
    client_pools: fastn_net::HttpConnectionPools,
    peer_connections: fastn_net::PeerStreamSenders,
) {
    kulfi::OPEN_CONTROL_CONNECTION_COUNT.incr();
    kulfi::CONTROL_CONNECTION_COUNT.incr();

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
                // http/1.1 allows https://en.wikipedia.org/wiki/HTTP_pipelining
                // but hyper does not, https://github.com/hyperium/hyper/discussions/2747:
                //
                // > hyper does not support HTTP/1.1 pipelining, since it's a deprecated HTTP
                // > feature. it's better to use HTTP/2.
                //
                // so we will never have IN_FLIGHT_REQUESTS > OPEN_CONNECTION_COUNT.
                //
                // for hostn-edge contacting hostn-document / hostn-wasm, it may have been useful to
                // send multiple requests on the same connection as they are independent of each
                // other. without pipelining, we will end up having effectively more open
                // connections between edge and js/wasm.
                hyper::service::service_fn(|r| handle_request(r, id_map.clone(), client_pools.clone(), peer_connections.clone(), graceful.clone())),
            );
    }

    if let Err(e) = tokio::select! {
        _ = graceful.cancelled() => {
            conn.as_mut().graceful_shutdown();
            conn.await
        }
        r = &mut conn => r,
    } {
        tracing::error!("connection error1: {e:?}");
    }

    kulfi::OPEN_CONTROL_CONNECTION_COUNT.decr();
}

async fn handle_request(
    r: hyper::Request<hyper::body::Incoming>,
    id_map: fastn_net::IDMap,
    client_pools: fastn_net::HttpConnectionPools,
    peer_connections: fastn_net::PeerStreamSenders,
    graceful: fastn_net::Graceful,
) -> fastn_net::http::ProxyResult<eyre::Error> {
    kulfi::CONTROL_REQUEST_COUNT.incr();
    kulfi::IN_FLIGHT_REQUESTS.incr();
    let r = handle_request_(r, id_map, client_pools, peer_connections, graceful).await;
    kulfi::IN_FLIGHT_REQUESTS.decr();
    r
}

async fn handle_request_(
    r: hyper::Request<hyper::body::Incoming>,
    id_map: fastn_net::IDMap,
    client_pools: fastn_net::HttpConnectionPools,
    peer_connections: fastn_net::PeerStreamSenders,
    graceful: fastn_net::Graceful,
) -> fastn_net::http::ProxyResult<eyre::Error> {
    let id = match r
        .headers()
        .get("Host")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.split_once('.'))
    {
        Some((first, _)) => first,
        None => {
            tracing::error!("got http request without Host header");
            return Ok(fastn_net::bad_request!(
                "got http request without Host header"
            ));
        }
    };

    tracing::info!("got request for {id}");

    // if this is an identity, if so forward the request to fastn corresponding to that identity
    if let Some(fastn_port) = find_identity(id, id_map.clone()).await? {
        let addr = format!("127.0.0.1:{fastn_port}");
        return kulfi::control_server::proxy_pass(r, find_pool(client_pools, &addr).await?, &addr)
            .await;
    }

    // TODO: maybe we should try all the identities not just default
    let (default_id, default_port) = default_identity(id_map.clone()).await?;
    match what_to_do(default_port, id).await {
        // if the id belongs to a friend of an identity, send the request to the friend over iroh
        Ok(WhatToDo::ForwardToPeer { peer_id }) => {
            let self_endpoint = get_endpoint(default_id.as_str(), id_map).await?;
            fastn_net::http_to_peer(
                fastn_net::Protocol::Http.into(),
                r,
                self_endpoint,
                peer_id.as_str(),
                peer_connections,
                graceful,
            )
            .await
        }
        // if not identity, find if the id is an http device owned by identity, if so proxy-pass the
        // request to that device
        Ok(WhatToDo::ProxyPass { port }) => {
            let addr = format!("127.0.0.1:{port}");
            kulfi::control_server::proxy_pass(r, find_pool(client_pools, &addr).await?, &addr).await
        }
        Ok(WhatToDo::UnknownPeer) => {
            tracing::error!("unknown peer: {id}");
            Ok(fastn_net::server_error!("unknown peer"))
        }
        Err(e) => {
            tracing::error!("proxy error: {e}");
            Ok(fastn_net::server_error!(
                "failed to contact default identity service"
            ))
        }
    }
}

pub async fn find_pool(
    client_pools: fastn_net::HttpConnectionPools,
    addr: &str,
) -> eyre::Result<fastn_net::HttpConnectionPool> {
    {
        let pools = client_pools.lock().await;
        if let Some(v) = pools.get(addr) {
            return Ok(v.to_owned());
        }
    }

    let c = fastn_net::HttpConnectionPool::builder()
        .build(fastn_net::HttpConnectionManager::new(addr.to_string()))
        .await?;

    {
        client_pools
            .lock()
            .await
            .insert(addr.to_string(), c.clone());
    }

    Ok(c)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum WhatToDo {
    ForwardToPeer { peer_id: String },
    ProxyPass { port: u16 },
    UnknownPeer,
}

async fn what_to_do(_port: u16, id: &str) -> eyre::Result<WhatToDo> {
    // request to fastn server at /-/kulfi/v1/control/what-to-do/<id>/
    Ok(WhatToDo::ForwardToPeer {
        peer_id: id.to_string(),
    })
}

async fn find_identity(id: &str, id_map: fastn_net::IDMap) -> eyre::Result<Option<u16>> {
    for (i, (port, _ep)) in id_map.lock().await.iter() {
        // if i.starts_with(id) {
        if i == id {
            return Ok(Some(*port));
        }
    }

    Ok(None)
}

async fn default_identity(id_map: fastn_net::IDMap) -> eyre::Result<(String, u16)> {
    Ok(id_map
        .lock()
        .await
        .first()
        .map(|(ident, (port, _ep))| (ident.to_string(), *port))
        .expect("kulfi ensures there is at least one identity at the start"))
}

async fn get_endpoint(
    self_id52: &str,
    id_map: fastn_net::IDMap,
) -> eyre::Result<iroh::endpoint::Endpoint> {
    let map = id_map.lock().await;

    for (id, (_port, ep)) in map.iter() {
        if id == self_id52 {
            return Ok(ep.clone());
        }
    }

    tracing::error!("no entry for {self_id52} in the id_map: {id_map:?}");
    Err(eyre::anyhow!(
        "no entry for {self_id52} in the id_map: {id_map:?}"
    ))
}
