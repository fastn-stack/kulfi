pub async fn generate(file: Option<String>) -> eyre::Result<()> {
    use std::io::Write;

    let (id52, secret_key) = kulfi_utils::generate_secret_key()?;
    eprintln!("Generated Public Key (ID52): {id52}");

    match file {
        Some(ref file) => {
            writeln!(std::fs::File::create(file)?, "{secret_key}")?;
            println!("Private key saved to `{file}`.");
        }
        None => {
            println!("{secret_key}");
        }
    }

    Ok(())
}
