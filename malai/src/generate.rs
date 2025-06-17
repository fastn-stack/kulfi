pub async fn generate(file: Option<String>) -> eyre::Result<()> {
    use std::io::Write;

    let (id52, private_key) = kulfi_utils::generate_private_key()?;
    eprintln!("Generated Public Key (ID52): {id52}");

    match file {
        Some(ref file) => {
            writeln!(std::fs::File::create(file)?, "{private_key}")?;
            println!("Private key saved to `{file}`.");
        }
        None => {
            println!("{private_key}");
        }
    }

    Ok(())
}
