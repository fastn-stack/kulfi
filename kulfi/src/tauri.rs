#[allow(unexpected_cfgs)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn ui() -> eyre::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri::WebviewUrl;

            let _window = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::App("browser.html".into()),
            )
            .title("Kulfi")
            .center()
            .inner_size(800.0, 600.0)
            .build()?;

            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol("kulfi", |_ctx, request, responder| {
            tauri::async_runtime::spawn(async move {
                let mut request = kulfi_utils::http::vec_u8_to_bytes(request);

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

                let graceful = kulfi_utils::Graceful::default();
                let peer_connections = kulfi_utils::PeerStreamSenders::default();
                let response = kulfi_utils::http_to_peer_non_streaming(
                    kulfi_utils::Protocol::Http.into(),
                    request,
                    kulfi_utils::global_iroh_endpoint().await,
                    &id52,
                    peer_connections,
                    graceful,
                )
                .await;

                let response = kulfi_utils::http::response_to_static(response)
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
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
