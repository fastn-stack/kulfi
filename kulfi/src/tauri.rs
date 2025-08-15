const BROWSER_WEBVIEW: &str = "browser_view";
const NAV_WEBVIEW: &str = "navigation";

#[allow(unexpected_cfgs)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn ui() -> eyre::Result<()> {
    const BROWSER_INIT_SCRIPT: &str = r#"
        console.log("Browser Init Script Loaded");

        const listen = window.__TAURI__.event.listen;
        const emitTo = window.__TAURI__.event.emitTo;

        // https://developer.mozilla.org/en-US/docs/Web/API/Window/pageshow_event
        window.addEventListener("pageshow", () => {
            console.info("Page show event triggered");
            emitUrlChange(window.location.href);
        });

        listen("nav-back", () => {
            console.log("going back one page");
            history.back();
        });

        listen("nav-forward", () => {
            console.log("going forward one page");
            history.forward();
        });


        /**
         * @param {string} url
         */
        function emitUrlChange(url) {
            console.log("Current URL:", url);
            if (url.startsWith("tauri://")) {
                console.info("URL starts with tauri://, ignoring emitUrlChange");
                return;
            }

            emitTo("navigation", "url-changed", url).
              then(() => {
                console.log("URL change emitted to navigation webview");
              })
              .catch(err => {
                console.error("Failed to emit URL change:", err);
              });
        }
    "#;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri::{LogicalPosition, LogicalSize, WebviewUrl};

            let width = 800.0;
            let height = 600.0;
            let bottom_height = 45.0;
            let top_height = height - bottom_height;

            let window = tauri::window::WindowBuilder::new(app, "main")
                .title("Kulfi")
                .center()
                .inner_size(width, height)
                .build()?;

            let _browser_window = window.add_child(
                tauri::webview::WebviewBuilder::new(BROWSER_WEBVIEW, tauri::WebviewUrl::App("init_view.html".into()))
                    .auto_resize()
                    .initialization_script(BROWSER_INIT_SCRIPT),
                LogicalPosition::new(0., 0.),
                LogicalSize::new(width, top_height), // TODO:
            )?;

            let _webview2 = window.add_child(
                tauri::webview::WebviewBuilder::new(NAV_WEBVIEW, WebviewUrl::App("navigation.html".into()))
                    .auto_resize(),
                LogicalPosition::new(0., top_height),
                LogicalSize::new(width, bottom_height), // TODO:
            )?;

            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol("kulfi", |_ctx, request, responder| {
            tauri::async_runtime::spawn(async move {
                let mut request = fastn_net::http::vec_u8_to_bytes(request);

                let (new_uri, id52) = kulfi_uri_to_path_and_id52(request.uri());

                *request.uri_mut() = new_uri.parse().expect("failed to parse new URI");

                // will get 400 Bad Request if the host is not set
                request.headers_mut().insert(
                    "HOST",
                    format!("kulfi://{id52}")
                        .parse()
                        .expect("failed to parse header value"),
                );

                let request = request; // remove mut bind

                tracing::info!(?request, "Sending Request");

                let graceful = fastn_net::Graceful::default();
                let peer_connections = fastn_net::PeerStreamSenders::default();
                let response = fastn_net::http_to_peer_non_streaming(
                    fastn_net::Protocol::Http.into(),
                    request,
                    fastn_net::global_iroh_endpoint().await,
                    &id52,
                    peer_connections,
                    graceful,
                )
                .await;

                let response = fastn_net::http::response_to_static(response)
                    .await
                    .expect("failed to convert response to static");

                if response.status().is_redirection() {
                    // NOTE: webview seems to be refusing to follow LOCATION of redirection if it's
                    // the kulfi:// protocol. We handle it manually here.

                    tracing::info!("Response is a redirection: {}", response.status());

                    let location = response
                        .headers()
                        .get(hyper::header::LOCATION)
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string());

                    tracing::info!("Location header: {:?}", location);
                    if let Some(location) = location {
                        let new_location = format!("kulfi://{id52}{}", location);

                        tracing::info!("Redirecting to new location: {}", new_location);

                        let html = format!(
                            r#"
                                <html>
                                <head>
                                    <meta http-equiv="refresh" content="0;url=kulfi://another/path" />
                                    <script>location.href = "{new_location}";</script>
                                </head>
                                <body>Redirecting...</body>
                                </html>
                            "#
                        );

                        responder.respond(
                            hyper::Response::builder()
                                .header("Content-Type", "text/html")
                                .status(200)
                                .body(html.as_bytes().to_vec())
                                .unwrap(),
                        );
                    }
                } else {
                    responder.respond(response);
                }
            });
        })
        .invoke_handler(tauri::generate_handler![open_url])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

/// Called from `navigation.html` frontend when the user enters a kulfi url and hits the enter key
#[tauri::command]
async fn open_url(app_handle: tauri::AppHandle, url: String) -> Result<(), OpenUrlError> {
    use tauri::Manager;

    tracing::info!("{:?}", app_handle.webviews().get("browser_view"));

    app_handle
        .get_webview(BROWSER_WEBVIEW)
        .ok_or(OpenUrlError::NoWebview)?
        .navigate(url.parse().map_err(|_| OpenUrlError::InvalidUrl)?)
        .map_err(|_| OpenUrlError::Navigation)
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
enum OpenUrlError {
    #[error("No webview found to open the URL")]
    NoWebview,
    #[error("Invalid URL provided")]
    InvalidUrl,
    #[error("Failed to navigate to the URL")]
    Navigation,
}

fn kulfi_uri_to_path_and_id52(uri: &hyper::Uri) -> (String, String) {
    // TODO: handle the following assert as error
    let uri_str = uri.to_string();
    assert!(uri_str.starts_with("kulfi://"));

    let id52 = uri.host().expect("URI must have a host");

    assert!(
        id52.len() == 52,
        "ID must be 52 characters long, got: {id52}"
    );

    // TODO: id52 must be alphanumeric only. should not have a dot (.)

    let new_uri = uri_str
        .strip_prefix(format!("kulfi://{id52}").as_str())
        .expect("already assert for kulfi://");

    let new_uri = if new_uri.is_empty() {
        "/".to_string()
    } else if !new_uri.starts_with('/') {
        format!("/{}", new_uri)
    } else {
        new_uri.to_string()
    };

    (new_uri, id52.to_string())
}
