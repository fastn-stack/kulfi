use eyre::WrapErr;

impl kulfi::Identity {
    #[tracing::instrument(skip_all)]
    pub async fn run(
        self,
        graceful: kulfi_utils::Graceful,
        id_map: kulfi_utils::IDMap,
        data_dir: &std::path::Path,
    ) -> eyre::Result<()> {
        let port = start_fastn(
            std::sync::Arc::clone(&id_map),
            graceful.clone(),
            &self.id52,
            data_dir,
        )
        .await
        .wrap_err_with(|| "failed to start fastn")
        .unwrap_or_else(|e| {
            tracing::error!("failed to start fastn: {e:?}, using 8000 for now");
            8000
        });
        tracing::info!("fastn started on port {port}");

        let secret_key = todo!();
        let ep = kulfi_utils::get_endpoint(secret_key)
            .await
            .wrap_err_with(|| "failed to bind to iroh network")?;

        {
            id_map.lock().await.push((self.id52, (port, ep.clone())));
        }

        kulfi::peer_server::run(ep, port, self.client_pools.clone(), graceful).await
    }
}

/// launch fastn from the package directory and return the port
#[tracing::instrument(skip_all)]
async fn start_fastn(
    _id_map: kulfi_utils::IDMap,
    _graceful: kulfi_utils::Graceful,
    id52: &str,
    data_dir: &std::path::Path,
) -> eyre::Result<u16> {
    tracing::info!("Running `fastn serve` for {id52}");
    let path = data_dir.join("identities").join(id52).join("package");
    let (port, _child) = spawn_fastn_serve_and_get_port(&path).await?;

    // TODO: store the child process as well. Use child.kill() to kill it when shutting down

    Ok(port)
}

/// Spawn `fastn serve` in [dir] and return the port and the child process.
/// The returned child process can be used to kill the fastn server later.
#[tracing::instrument]
pub async fn spawn_fastn_serve_and_get_port(
    dir: &std::path::Path,
) -> eyre::Result<(u16, tokio::process::Child)> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};

    let mut cmd = tokio::process::Command::new("fastn");
    cmd.current_dir(dir);
    cmd.args(["serve", "--offline"]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());

    let mut child = cmd.spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| eyre::eyre!("Failed to capture stdout"))?;

    let mut reader = BufReader::new(stdout).lines();
    let prefix = "Go to: http://127.0.0.1:";

    tracing::info!("Waiting for fastn server to start...");

    while let Some(line) = reader
        .next_line()
        .await
        .wrap_err_with(|| "failed to read next line")?
    {
        tracing::info!("fastn output: {}", line);
        if let Some(port_str) = line.trim().strip_prefix(prefix) {
            let port: u16 = port_str
                .trim()
                .parse()
                .map_err(|e| eyre::eyre!("Failed to parse port: {}", e))?;

            tracing::info!("Fastn server started on port {}", port);

            // Return the port and keep the child alive
            return Ok((port, child));
        }
    }

    Err(eyre::eyre!("Did not find port in fastn output"))
}
