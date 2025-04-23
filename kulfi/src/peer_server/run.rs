pub async fn run(
    ep: iroh::Endpoint,
    fastn_port: u16,
    client_pools: kulfi_utils::HttpConnectionPools,
    graceful: kulfi_utils::Graceful,
) -> eyre::Result<()> {
    loop {
        let conn = match ep.accept().await {
            Some(conn) => conn,
            None => {
                tracing::info!("no connection");
                break;
            }
        };
        let client_pools = client_pools.clone();
        graceful.spawn(async move {
            let start = std::time::Instant::now();
            let conn = match conn.await {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("failed to convert incoming to connection: {:?}", e);
                    return;
                }
            };
            // if let Err(e) = enqueue_connection(conn.clone(), peer_connections).await {
            //     tracing::error!("failed to enqueue connection: {:?}", e);
            //     return;
            // }
            if let Err(e) = handle_connection(conn, client_pools, fastn_port).await {
                tracing::error!("connection error3: {:?}", e);
            }
            tracing::info!("connection handled in {:?}", start.elapsed());
        });
    }

    ep.close().await;
    Ok(())
}

// async fn enqueue_connection(
//     conn: iroh::endpoint::Connection,
//     peer_connections: kulfi_utils::get_stream2::PeerStreamSenders,
// ) -> eyre::Result<()> {
//     let public_key = match conn.remote_node_id() {
//         Ok(v) => v,
//         Err(e) => {
//             tracing::error!("can not get remote id: {e:?}");
//             return Err(eyre::anyhow!("can not get remote id: {e:?}"));
//         }
//     };
//     let id = kulfi_utils::public_key_to_id52(&public_key);
//     let mut connections = peer_connections.lock().await;
//     connections.insert(id.clone(), conn);
//
//     Ok(())
// }

pub async fn handle_connection(
    conn: iroh::endpoint::Connection,
    client_pools: kulfi_utils::HttpConnectionPools,
    fastn_port: u16,
) -> eyre::Result<()> {
    tracing::info!("got connection from: {:?}", conn.remote_node_id());
    let remote_id52 = kulfi_utils::get_remote_id52(&conn)
        .await
        .inspect_err(|e| tracing::error!("failed to get remote id: {e:?}"))?;
    tracing::info!("new client: {remote_id52}, waiting for bidirectional stream");
    loop {
        let client_pools = client_pools.clone();
        // TODO: graceful shutdown
        let (mut send, recv) = kulfi_utils::accept_bi(&conn, kulfi_utils::Protocol::Http)
            .await
            .inspect_err(|e| tracing::error!("failed to accept bidirectional stream: {e:?}"))?;
        tracing::info!("{remote_id52}");
        if let Err(e) = kulfi_utils::peer_to_http(
            &format!("127.0.0.1:{fastn_port}"),
            client_pools,
            &mut send,
            recv,
        )
        .await
        {
            tracing::error!("failed to proxy http: {e:?}");
        }
        tracing::info!("closing send stream");
        send.finish()?;
    }
}
