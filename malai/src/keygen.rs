pub fn keygen(path: Option<String>) -> eyre::Result<()> {
    let secret_key_path = match path {
        Some(v) => std::path::PathBuf::from(v),
        None => directories::BaseDirs::new()
            .expect("Failed to get valid HOME dir")
            .data_dir()
            .join("malai")
            .join("default.key"),
    };

    if secret_key_path.exists() {
        eprintln!(
            "Path {secret_key_path:?} already exists. Please provide a different path or delete the existing file."
        );
        std::process::exit(1);
    }

    if secret_key_path.is_dir() {
        eprintln!("Secret key path is a directory. Please provide a valid file path.");
        std::process::exit(1);
    }

    std::fs::create_dir_all(
        secret_key_path
            .parent()
            .expect("Failed to get parent directory"),
    )?;

    match kulfi_utils::generate_secret_key(&secret_key_path) {
        Err(e) => {
            tracing::error!(error = ?e, "Failed to generate secret key");
            std::process::exit(1);
        }
        Ok(_) => {
            tracing::info!(path = ?secret_key_path, "Secret key generated successfully.");
            println!("Secret key generated successfully at {secret_key_path:?}");
            std::process::exit(0);
        }
    }
}
