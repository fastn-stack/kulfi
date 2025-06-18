use eyre::WrapErr;

pub const SECRET_KEY_ENV_VAR: &str = "KULFI_SECRET_KEY";
pub const SECRET_KEY_FILE: &str = ".malai.secret-key";
pub const ID52_FILE: &str = ".malai.id52";

pub fn generate_secret_key() -> eyre::Result<(String, iroh::SecretKey)> {
    let secret_key = iroh::SecretKey::generate(&mut rand::rngs::OsRng);
    let public_key = secret_key.public();
    let id52 = kulfi_utils::public_key_to_id52(&public_key);
    Ok((id52, secret_key))
}

pub async fn generate_and_save_key() -> eyre::Result<(String, iroh::SecretKey)> {
    let (id52, secret_key) = generate_secret_key()?;
    let e = keyring_entry(&id52)?;
    e.set_secret(&secret_key.to_bytes())
        .wrap_err_with(|| format!("failed to save secret key for {id52}"))?;
    tokio::fs::write(ID52_FILE, &id52).await?;
    Ok((id52, secret_key))
}

fn keyring_entry(id52: &str) -> eyre::Result<keyring::Entry> {
    keyring::Entry::new("kulfi", id52)
        .wrap_err_with(|| format!("failed to create keyring Entry for {id52}"))
}

fn handle_secret(secret: &str) -> eyre::Result<(String, iroh::SecretKey)> {
    use std::str::FromStr;

    let secret_key = iroh::SecretKey::from_str(secret)
        .wrap_err_with(|| "failed to parse secret key from string")?;
    let public_key = secret_key.public();
    let id52 = kulfi_utils::public_key_to_id52(&public_key);
    Ok((id52, secret_key))
}

pub fn get_secret_key(_id52: &str, _path: &str) -> eyre::Result<iroh::SecretKey> {
    // intentionally left unimplemented as design is changing in kulfi
    // this is not used in malai
    todo!("implement for kulfi")
}

#[tracing::instrument]
pub async fn read_or_create_key() -> eyre::Result<(String, iroh::SecretKey)> {
    if let Ok(secret) = std::env::var(SECRET_KEY_ENV_VAR) {
        tracing::info!("Using secret key from environment variable {SECRET_KEY_ENV_VAR}");
        return handle_secret(&secret);
    } else {
        match tokio::fs::read_to_string(SECRET_KEY_FILE).await {
            Ok(secret) => {
                tracing::info!("Using secret key from file {SECRET_KEY_FILE}");
                return handle_secret(&secret);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                tracing::error!("failed to read {SECRET_KEY_FILE}: {e}");
                return Err(e.into());
            }
        }
    }

    tracing::info!("No secret key found in environment or file, trying {ID52_FILE}");
    match tokio::fs::read_to_string(ID52_FILE).await {
        Ok(id52) => {
            let e = keyring_entry(&id52)?;
            match e.get_secret() {
                Ok(secret) => {
                    if secret.len() != 32 {
                        return Err(eyre::anyhow!(
                            "keyring: secret for {id52} has invalid length: {}",
                            secret.len()
                        ));
                    }

                    let bytes: [u8; 32] = secret.try_into().expect("already checked for length");
                    let secret_key = iroh::SecretKey::from_bytes(&bytes);
                    Ok((
                        kulfi_utils::public_key_to_id52(&secret_key.public()),
                        secret_key,
                    ))
                }
                Err(e) => {
                    tracing::error!("failed to read secret for {id52} from keyring: {e}");
                    Err(e.into())
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => generate_and_save_key().await,
        Err(e) => {
            tracing::error!("failed to read {ID52_FILE}: {e}");
            Err(e.into())
        }
    }
}
