pub fn keygen(filename: Option<String>) {
    use std::io::Write;

    let (id52, secret_key) = match kulfi_utils::generate_secret_key() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to generate secret key: {e}");
            std::process::exit(1);
        }
    };

    println!("Generated Public Key (ID52): {id52}");

    match filename {
        Some(ref filename) => {
            if std::path::Path::new(filename).exists() {
                eprintln!("File `{filename}` already exists. Please choose a different file name.");
                std::process::exit(1);
            }
            let mut file = match std::fs::File::create(filename) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to create file `{filename}`: {e}");
                    std::process::exit(1);
                }
            };

            match write!(file, "{secret_key}") {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to write secret key to file `{filename}`: {e}");
                    std::process::exit(1);
                }
            }

            println!("Private key saved to `{filename}`.");
        }
        None => {
            println!("{secret_key}");
        }
    }
}
