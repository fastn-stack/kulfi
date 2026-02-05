use std::path::PathBuf;

use eyre::WrapErr;

pub const SECRET_KEY_ENV_VAR: &str = "KULFI_SECRET_KEY";
pub const SECRET_KEY_FILE: &str = ".malai.secret-key";
pub const ID52_FILE: &str = ".malai.id52";

pub fn generate_secret_key() -> eyre::Result<(String, kulfi_id52::SecretKey)> {
    let secret_key = kulfi_id52::SecretKey::generate();
    let id52 = secret_key.id52();
    Ok((id52, secret_key))
}

pub fn generate_and_save_key(
    file: Option<PathBuf>,
) -> eyre::Result<(String, kulfi_id52::SecretKey)> {
    let (id52, secret_key) = generate_secret_key()?;
    let e = keyring_entry(&id52)?;
    e.set_secret(&secret_key.to_bytes())
        .wrap_err_with(|| format!("failed to save secret key for {id52}"))?;
    if let Some(file) = &file {
        std::fs::write(file, &id52)
            .wrap_err_with(|| format!("failed to save secret key to {}", &file.display()))?;
        println!("ID52 saved to {}", file.display());
    }
    Ok((id52, secret_key))
}

pub fn delete_identity(id52: &str) -> eyre::Result<()> {
    let e = keyring_entry(id52)?;
    e.delete_credential()?;
    Ok(())
}

fn keyring_entry(id52: &str) -> eyre::Result<keyring::Entry> {
    keyring::Entry::new("kulfi", id52)
        .wrap_err_with(|| format!("failed to create keyring Entry for {id52}"))
}

pub fn handle_secret(secret: &str) -> eyre::Result<(String, kulfi_id52::SecretKey)> {
    use std::str::FromStr;
    let secret_key = kulfi_id52::SecretKey::from_str(secret).map_err(|e| eyre::anyhow!("{}", e))?;
    let id52 = secret_key.id52();
    Ok((id52, secret_key))
}

pub fn get_secret_key(_id52: &str, _path: &str) -> eyre::Result<kulfi_id52::SecretKey> {
    // intentionally left unimplemented as design is changing in kulfi
    // this is not used in malai
    todo!("implement for kulfi")
}

pub fn handle_identity(id52: String) -> eyre::Result<(String, kulfi_id52::SecretKey)> {
    let e = kulfi_utils::secret::keyring_entry(&id52)?;
    match e.get_secret() {
        Ok(secret) => {
            if secret.len() != 32 {
                return Err(eyre::anyhow!(
                    "keyring: secret for {id52} has invalid length: {}",
                    secret.len()
                ));
            }

            let bytes: [u8; 32] = secret.try_into().expect("already checked for length");
            let secret_key = kulfi_id52::SecretKey::from_bytes(&bytes);
            let id52 = secret_key.id52();
            Ok((id52, secret_key))
        }
        Err(e) => {
            tracing::error!("failed to read secret for {id52} from keyring: {e}");
            Err(e.into())
        }
    }
}

#[tracing::instrument]
pub async fn read_or_create_key() -> eyre::Result<(String, kulfi_id52::SecretKey)> {
    if let Ok(secret) = std::env::var(SECRET_KEY_ENV_VAR) {
        tracing::info!("Using secret key from environment variable {SECRET_KEY_ENV_VAR}");
        return handle_secret(&secret);
    }
    match tokio::fs::read_to_string(SECRET_KEY_FILE).await {
        Ok(secret) => {
            tracing::info!("Using secret key from file {SECRET_KEY_FILE}");
            let secret = secret.trim_end();
            return handle_secret(secret);
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            tracing::error!("failed to read {SECRET_KEY_FILE}: {e}");
            return Err(e.into());
        }
    }

    tracing::info!("No secret key found in environment or file, trying {ID52_FILE}");
    match tokio::fs::read_to_string(ID52_FILE).await {
        Ok(id52) => handle_identity(id52),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            generate_and_save_key(Some(PathBuf::from(ID52_FILE)))
        }
        Err(e) => {
            tracing::error!("failed to read {ID52_FILE}: {e}");
            Err(e.into())
        }
    }
}
