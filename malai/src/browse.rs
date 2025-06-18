pub async fn browse(url: String, graceful: kulfi_utils::Graceful) {
    let (id52, path) = match parse_url(&url) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = ?e, url, "Failed to parse URL");
            eprintln!("Failed to parse URL: {e}");
            return;
        }
    };

    malai::http_bridge(0, Some(id52.to_string()), graceful, |port| {
        let url = format!("http://127.0.0.1:{port}/{path}");
        webbrowser::open(&url).map_err(Into::into)
    })
    .await
}

/// This function extracts the id52 and the path from the URL
///
/// the path is the part after the first / in the URL
fn parse_url(url: &str) -> eyre::Result<(&str, &str)> {
    // check if url starts with kulfi://
    let rest = match url.split_once("kulfi://") {
        Some(("", rest)) => rest,
        Some((e, _rest)) => {
            return Err(eyre::anyhow!(
                "URL must start with kulfi://, got {e} in the beginning"
            ));
        }
        None => {
            return Err(eyre::anyhow!("URL must start with kulfi://"));
        }
    };

    Ok(rest.split_once('/').unwrap_or((rest, "")))
}
