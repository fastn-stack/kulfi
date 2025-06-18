mod render_folder;

use render_folder::render_folder;

/// folder() exposes a folder on the kulfi network
///
/// the folder needs a little bit of user interface, the directory listing page. there are many
/// ways to implement the UI, we can hard code some minimal HTML template, and call it a day.
///
/// we are going to use fastn to build the UI though. partially, this is because we create the fastn
/// support in malai, which then will help us when we are building the kulfi app, which also uses
/// fastn internally for all sorts of UI.
///
/// using fastn means people can actually customize the folder browsing user interface, if we hard
/// code some HTML, we will have to make it configurable, and possibly use some sort of template
/// library. and if they want to do more, add logo, JS/css etc., it will no longer be just a single
/// html template, but we will need some way to include a folder, and we will end up either
/// re-inventing a poor man's web framework, or make this simple.
///
/// simple is in general good, but UI is a very important part of software, and giving it
/// second-rate treatment here, for folder, and not using fastn is a mistake. or so I feel as I
/// write this.
///
/// so how will this work? where would the fastn package be created? also which fastn template will
/// be used to create the fastn package?
///
/// at the highest level, as we have discussed in kulfi/src/config/mod.rs, we will have a kulfi
/// folder, which we will re-use for malai as well. why maintain two folders?
///
/// having said all that, the first version of malai browsing will be a simple HTML page, and we
/// will compile `folder.html` template as part of the build process.
pub async fn folder(path: String, bridge: String, graceful: kulfi_utils::Graceful) {
    let path = match validate_path(&path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to validate path: {e}");
            std::process::exit(1);
        }
    };

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to port: {e}");
            std::process::exit(1);
        }
    };

    let port = match listener.local_addr() {
        Ok(addr) => addr.port(),
        Err(e) => {
            eprintln!("Failed to get local address: {e}");
            std::process::exit(1);
        }
    };
    println!("Serving {path:?} on http://127.0.0.1:{port}");

    let graceful_for_expose_http = graceful.clone();

    graceful.spawn(async move {
        malai::expose_http(
            "127.0.0.1".to_string(),
            port,
            bridge,
            graceful_for_expose_http,
        )
        .await
    });

    let mut graceful_mut = graceful.clone();

    loop {
        tokio::select! {
            _ = graceful.cancelled() => {
                tracing::info!("Stopping control server.");
                break;
            }
            _ = graceful_mut.show_info() => {
                println!("Listening on http://127.0.0.1:{port}");
                println!("Press ctrl+c again to exit.");
            }
            conn = listener.accept() => {
                match conn {
                    Ok((stream, _)) => {
                        let graceful_for_handle_connection = graceful.clone();
                        let path = path.clone();
                        graceful.spawn(async move { handle_connection(stream, path, graceful_for_handle_connection).await });
                    }
                    Err(e) => {
                        tracing::error!("failed to accept: {e:?}");
                    }
                }
            }
        }
    }
}

pub async fn handle_connection(
    stream: tokio::net::TcpStream,
    path: std::path::PathBuf,
    graceful: kulfi_utils::Graceful,
) {
    let io = hyper_util::rt::TokioIo::new(stream);

    let path = std::sync::Arc::new(path);

    let builder =
        hyper_util::server::conn::auto::Builder::new(hyper_util::rt::tokio::TokioExecutor::new());
    // the following builder runs only http2 service, whereas the hyper_util auto Builder runs an
    // http1.1 server that upgrades to http2 if the client requests.
    // let builder = hyper::server::conn::http2::Builder::new(hyper_util::rt::tokio::TokioExecutor::new());
    tokio::pin! {
        let conn = builder
            .serve_connection(
                io,
                hyper::service::service_fn(|r| handle_request(r, path.clone())),
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
}

async fn handle_request(
    r: hyper::Request<hyper::body::Incoming>,
    base_path: std::sync::Arc<std::path::PathBuf>,
) -> eyre::Result<
    hyper::Response<http_body_util::combinators::BoxBody<hyper::body::Bytes, std::io::Error>>,
> {
    use futures_util::TryStreamExt;
    use http_body_util::BodyExt;
    use tokio::io::AsyncSeekExt;

    let path = r.uri().path().to_string();
    let path = percent_encoding::percent_decode_str(&path)
        .decode_utf8()
        .map_err(|e| {
            tracing::error!(?e, "failed to decode path");
            eyre::anyhow!("invalid path")
        })?;

    tracing::info!(?path, "request path");
    tracing::info!(?base_path, "base path");

    let path = join_path(&base_path, &path)?;
    tracing::info!(?path, "joined path");

    if path.is_dir() {
        tracing::info!("rendering folder");
        return Ok(kulfi_utils::http::bytes_to_resp::<std::io::Error>(
            malai::folder::render_folder(&path, &base_path)?.into_bytes(),
            hyper::StatusCode::OK,
        ));
    }

    tracing::info!("serving file");

    let mime = path
        .extension()
        .and_then(|v| v.to_str())
        .map(mime_guess::from_ext)
        .and_then(|v| v.first())
        .unwrap_or(mime_guess::mime::APPLICATION_OCTET_STREAM);

    let mut file = tokio::fs::File::open(path).await?;
    let metadata = file.metadata().await?;
    let total_size = metadata.len();

    let range = r
        .headers()
        .get(hyper::header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(parse_range);

    let resp = if let Some((start, end)) = range {
        let end = end.unwrap_or(total_size - 1);
        let end = end.min(total_size - 1);
        tracing::info!(start, end, "range request");
        let length = end - start + 1;

        file.seek(std::io::SeekFrom::Start(start)).await.unwrap();

        let reader_stream = tokio_util::io::ReaderStream::new(file);

        let stream_body =
            http_body_util::StreamBody::new(reader_stream.map_ok(hyper::body::Frame::data));
        let boxed_body = stream_body.boxed();

        hyper::Response::builder()
            .status(hyper::StatusCode::PARTIAL_CONTENT)
            .header(hyper::header::ACCEPT_RANGES, "bytes")
            .header(hyper::header::CONTENT_LENGTH, length.to_string())
            .header(hyper::header::CONTENT_TYPE, mime.as_ref())
            .header(
                hyper::header::CONTENT_RANGE,
                format!("bytes {}-{}/{}", start, end, total_size),
            )
            .body(boxed_body)?
    } else {
        tracing::info!("full file request");
        let reader_stream = tokio_util::io::ReaderStream::new(file);

        let stream_body =
            http_body_util::StreamBody::new(reader_stream.map_ok(hyper::body::Frame::data));
        let boxed_body = stream_body.boxed();

        hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .header(hyper::header::CONTENT_TYPE, mime.as_ref())
            .header(hyper::header::ACCEPT_RANGES, "bytes")
            .header(hyper::header::CONTENT_LENGTH, total_size.to_string())
            .body(boxed_body)?
    };

    Ok(resp)
}

#[tracing::instrument]
fn join_path(base_path: &std::path::Path, o_path: &str) -> eyre::Result<std::path::PathBuf> {
    let path = base_path
        .join(o_path.strip_prefix('/').unwrap_or(o_path))
        .canonicalize()?;

    if !path.starts_with(base_path) {
        eprintln!("{o_path} to a folder outside {base_path:?}");
        return Err(eyre::anyhow!("oops"));
    }

    Ok(path)
}

fn validate_path(path: &str) -> eyre::Result<std::path::PathBuf> {
    let path = std::path::PathBuf::from(path);

    if !path.exists() {
        return Err(eyre::anyhow!("{path:?} doesn't exist"));
    }

    if !path.is_dir() {
        return Err(eyre::anyhow!("{path:?} is not a directory"));
    }

    Ok(path.canonicalize()?)
}

/// Parse HTTP Range header value
/// "bytes=1000-2000" to Some((start, Some(end)))
/// "bytes=1000-" to Some((start, None))
/// range is inclusive (start..=end)
/// https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Range
fn parse_range(header_value: &str) -> Option<(u64, Option<u64>)> {
    if let Some(val) = header_value.strip_prefix("bytes=") {
        let parts: Vec<_> = val.split('-').collect();
        match parts.as_slice() {
            [start, end] if !start.is_empty() && !end.is_empty() => {
                let start = start.parse::<u64>().ok()?;
                let end = end.parse::<u64>().ok()?;
                return Some((start, Some(end)));
            }
            [start, ""] if !start.is_empty() => {
                let start = start.parse::<u64>().ok()?;
                return Some((start, None));
            }
            [start] if !start.is_empty() => {
                let start = start.parse::<u64>().ok()?;
                return Some((start, None));
            }
            _ => return None,
        }
    }
    None
}

#[cfg(test)]
mod test {
    use super::parse_range;

    #[test]
    fn test_parse_range() {
        assert_eq!(parse_range("bytes=1000-2000"), Some((1000, Some(2000))));
        assert_eq!(parse_range("bytes=1000-"), Some((1000, None)));
        assert_eq!(parse_range("bytes=-2000"), None);
        assert_eq!(parse_range("bytes=1000-1000"), Some((1000, Some(1000))));
        assert_eq!(parse_range("bytes=abc-def"), None);
        assert_eq!(parse_range("bytes="), None);
        assert_eq!(parse_range("bytes=1000-2000,3000-4000"), None);
    }
}
