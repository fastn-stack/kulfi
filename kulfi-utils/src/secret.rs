use eyre::WrapErr;

fn get(id52: &str) -> eyre::Result<iroh::SecretKey> {
    let entry = keyring_entry(id52)?;

    let secret = entry
        .get_secret()
        .wrap_err_with(|| format!("keyring: secret not found for {id52}"))?;

    if secret.len() != 32 {
        return Err(eyre::anyhow!(
            "keyring: secret has invalid length: {}",
            secret.len()
        ));
    }

    let bytes: [u8; 32] = secret.try_into().expect("already checked for length");
    Ok(iroh::SecretKey::from_bytes(&bytes))
}

fn save(_secret_key: &iroh::SecretKey) -> eyre::Result<()> {
    // Ok(self.keyring_entry()?.set_secret(&secret_key.to_bytes())?)

    todo!()
}

fn generate(mut rng: impl rand_core::CryptoRngCore) -> eyre::Result<iroh::PublicKey> {
    let secret_key = iroh::SecretKey::generate(&mut rng);
    // we do not want to keep secret key in memory, only in keychain
    let _id52 = kulfi_utils::public_key_to_id52(&secret_key.public());
    // store
    //     .save(&secret_key)
    //     .wrap_err_with(|| "failed to store secret key to keychain")?;
    // todo!()
    Ok(secret_key.public())
}

pub fn generate_public_key(_rng: impl rand_core::CryptoRngCore) ->  eyre::Result<iroh::PublicKey> {
    todo!()
}

fn keyring_entry(id52: &str) -> eyre::Result<keyring::Entry> {
    keyring::Entry::new("kulfi", id52)
        .wrap_err_with(|| format!("failed to create keyring Entry for {id52}"))
}

pub async fn read_or_create_key() -> eyre::Result<(String, iroh::SecretKey)> {
    match tokio::fs::read_to_string(".malai.id52").await {
        Ok(v) => Ok((v, todo!())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::info!("no key found, creating new one");
            let public_key = generate(rand::rngs::OsRng)?;
            let public_key = kulfi_utils::public_key_to_id52(&public_key);
            tokio::fs::write(".malai.id52", public_key.as_str()).await?;
            Ok((public_key, todo!()))
        }
        Err(e) => {
            tracing::error!("failed to read key: {e}");
            Err(e.into())
        }
    }
}
