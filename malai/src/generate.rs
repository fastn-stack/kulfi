pub async fn generate(file: Option<String>) -> eyre::Result<()> {
    use std::io::Write;

    let private_key = iroh::SecretKey::generate(&mut rand::rngs::OsRng);
    let public_key = private_key.public();
    let id52 = kulfi_utils::public_key_to_id52(&public_key);
    eprintln!("Generated Public Key (ID52): {id52}");

    match file {
        Some(ref file) => {
            write!(std::fs::File::create(file)?, "{private_key}\n")?;
            println!("Private key saved to `{file}`.");
        }
        None => {
            println!("{private_key}");
        }
    }

    Ok(())
}
