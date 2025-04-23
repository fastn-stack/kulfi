/// start kulfi service
///
/// on startup, we first check if another instance is running if so we exit.
///
/// the main job of this function is to run an instance of `fastn` for every identity in the
/// identities folder, and set-up http device driver for each of them.
///
/// it also has to start the device "drivers" for every device in the <identities>/devices folder.
pub async fn start(
    _fg: bool,
    data_dir: std::path::PathBuf,
    control_port: u16,
    graceful: kulfi_utils::Graceful,
) -> eyre::Result<()> {
    use eyre::WrapErr;

    let client_pools = kulfi_utils::HttpConnectionPools::default();
    let peer_connections = kulfi_utils::PeerStreamSenders::default();

    let config = kulfi::Config::read(&data_dir, client_pools.clone())
        .await
        .wrap_err_with(|| "failed to run config")?;

    let _lock = config
        .lock()
        .await
        .wrap_err_with(|| "looks like there is another instance of kulfi running")?;

    let identities = config.identities(client_pools.clone()).await?;
    tracing::info!(
        "kulfi started with {identities}.",
        identities = identities
            .iter()
            .map(|i| i.id52.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let first = identities
        .first()
        .map(|v| v.id52.clone())
        .ok_or_else(|| eyre::eyre!("no identities found"))?;

    let id_map = kulfi_utils::IDMap::default();

    for identity in identities {
        use std::sync::Arc;

        let g = graceful.clone();
        let id_map = Arc::clone(&id_map);
        let data_dir = data_dir.clone();
        graceful.tracker.spawn(async move {
            let public_key = identity.public_key;
            if let Err(e) = identity.run(g, id_map, &data_dir).await {
                tracing::error!("failed to run identity: {public_key}: {e:?}");
            }
        });
    }

    let g = graceful.clone();
    graceful.tracker.spawn(async move {
        tracing::info!("Starting control server with identity: {first}");
        kulfi::control_server::start(
            control_port,
            first,
            g,
            id_map,
            client_pools,
            peer_connections,
        )
        .await
        .unwrap()
    });

    Ok(())
}
