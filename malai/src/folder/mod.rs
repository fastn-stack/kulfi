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
pub async fn folder(
    path: String,
    bridge: String,
    secret_key_path: std::path::PathBuf,
    graceful: kulfi_utils::Graceful,
) -> eyre::Result<()> {
    use eyre::WrapErr;

    let path = validate_path(&path)?;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .wrap_err_with(|| "can not listen, is it busy, or you do not have root access?")?;

    let port = listener.local_addr()?.port();
    println!("Serving {path:?} on http://127.0.0.1:{port}");

    let g = graceful.clone();

    graceful.spawn(async move {
        malai::expose_http("127.0.0.1".to_string(), port, bridge, &secret_key_path, g).await
    });

    let mut g = graceful.clone();

    loop {
        tokio::select! {
            _ = graceful.cancelled() => {
                tracing::info!("Stopping control server.");
                break;
            }
            _ = g.show_info() => {
                println!("Listening on http://127.0.0.1:{port}");
                println!("Press ctrl+c again to exit.");
            }
            Ok((stream, _addr)) = listener.accept() => {
                let g = graceful.clone();
                let path = path.clone();
                graceful.spawn(async move { handle_connection(stream, path, g).await });
            }
            Err(e) = listener.accept() => {
                tracing::error!("failed to accept: {e:?}");
            },
        }
    }

    Ok(())
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
) -> kulfi_utils::http::ProxyResult {
    let path = r.uri().path().to_string();
    let path = join_path(&base_path, &path)?;

    if path.is_dir() {
        return Ok(kulfi_utils::http::bytes_to_resp(
            malai::folder::render_folder(&path, &base_path)?.into_bytes(),
            hyper::StatusCode::OK,
        ));
    }

    let mime = path
        .extension()
        .and_then(|v| v.to_str())
        .map(mime_guess::from_ext)
        .and_then(|v| v.first())
        .unwrap_or(mime_guess::mime::APPLICATION_OCTET_STREAM);

    let mut r = // TODO: streaming response
        kulfi_utils::http::bytes_to_resp(tokio::fs::read(path).await?, hyper::StatusCode::OK);

    r.headers_mut()
        .insert("content-type", mime.to_string().parse()?);

    Ok(r)
}

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
