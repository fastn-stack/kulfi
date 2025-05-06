pub fn read_or_create_secret_key(path: &std::path::Path) -> eyre::Result<iroh::SecretKey> {
    match get_secret_key(path) {
        Ok(v) => Ok(v),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(generate_secret_key(path)?),
        Err(e) => {
            tracing::error!("failed to read key: {e}");
            Err(e.into())
        }
    }
}

pub fn generate_secret_key(path: &std::path::Path) -> eyre::Result<iroh::SecretKey> {
    let secret_key = iroh::SecretKey::generate(rand::rngs::OsRng);
    std::fs::write(path, secret_key.to_bytes())?;
    Ok(secret_key)
}

pub fn get_secret_key(path: &std::path::Path) -> std::io::Result<iroh::SecretKey> {
    let secret = std::fs::read(path)?;

    assert_eq!(secret.len(), 32, "Secret key length must be 32");

    let bytes: [u8; 32] = secret.try_into().expect("already checked for length");
    Ok(iroh::SecretKey::from_bytes(&bytes))
}
