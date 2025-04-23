pub async fn http_to_peer<T>(
    req: hyper::Request<T>,
    self_endpoint: iroh::Endpoint,
    remote_node_id52: &str,
    peer_connections: kulfi_utils::PeerStreamSenders,
    _patch: ftnet_sdk::RequestPatch,
    graceful: kulfi_utils::Graceful,
) -> kulfi_utils::http::ProxyResult
where
    T: hyper::body::Body + Unpin + Send,
    T::Data: Into<hyper::body::Bytes> + Send,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    use http_body_util::{BodyDataStream, BodyExt};
    use tokio_stream::StreamExt;

    tracing::info!("peer_proxy: {remote_node_id52}");

    let (mut send, mut recv) = kulfi_utils::get_stream(
        self_endpoint,
        kulfi_utils::Protocol::Http,
        remote_node_id52.to_string(),
        peer_connections.clone(),
        graceful,
    )
    .await?;

    tracing::info!("wrote protocol");

    let (head, body) = req.into_parts();
    send.write_all(&serde_json::to_vec(&crate::http::Request::from(head))?)
        .await?;
    send.write_all(b"\n").await?;

    tracing::info!("sent request header");

    let mut stream = BodyDataStream::new(body);

    while let Some(chunk) = stream.next().await {
        let bytes: hyper::body::Bytes = chunk?.into(); // requires T::Data: Into<Bytes>
        send.write_all(&bytes).await?;
    }

    tracing::info!("sent body");

    let r: kulfi_utils::http::Response = match recv.next().await {
        Some(Ok(v)) => serde_json::from_str(&v)?,
        Some(Err(e)) => {
            tracing::error!("failed to get bidirectional stream: {e:?}");
            return Err(eyre::anyhow!("failed to get bidirectional stream: {e:?}"));
        }
        None => {
            tracing::error!("failed to read from incoming connection");
            return Err(eyre::anyhow!("failed to read from incoming connection"));
        }
    };

    tracing::info!("got response header: {r:?}");

    let mut body = recv.read_buffer().to_owned();
    let mut recv = recv.into_inner();

    tracing::trace!("reading body");

    while let Some(v) = match recv.read_chunk(1024 * 64, true).await {
        Ok(v) => Ok(v),
        Err(e) => {
            tracing::error!("error reading chunk: {e:?}");
            Err(eyre::anyhow!("read_chunk error: {e:?}"))
        }
    }? {
        body.extend_from_slice(&v.bytes);
        tracing::trace!(
            "reading body, partial: {}, new body size: {} bytes",
            v.bytes.len(),
            body.len()
        );
    }

    let body = body.freeze();
    tracing::debug!("got {} bytes of body", body.len());

    let mut res = hyper::Response::new(
        http_body_util::Full::new(body)
            .map_err(|e| match e {})
            .boxed(),
    );
    *res.status_mut() = hyper::http::StatusCode::from_u16(r.status)?;
    for (k, v) in r.headers {
        res.headers_mut().insert(
            hyper::http::header::HeaderName::from_bytes(k.as_bytes())?,
            hyper::http::header::HeaderValue::from_bytes(&v)?,
        );
    }

    tracing::info!("all done");
    Ok(res)
}
