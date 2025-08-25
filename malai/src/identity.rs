use std::path::{Path, PathBuf};

fn get_identity_path(path: Option<String>) -> Option<PathBuf> {
    let path = match path {
        Some(path) => path,
        None => return None,
    };
    let path = Path::new(&path).to_path_buf();
    if path.is_dir() {
        Some(path.join(kulfi_utils::secret::ID52_FILE))
    } else {
        Some(path)
    }
}

pub fn create_identity(path: Option<String>) -> eyre::Result<()> {
    let path = get_identity_path(path);
    let (id52, _) = kulfi_utils::secret::generate_and_save_key(path)?;
    println!(
        "Identity(ID52) created: {}. And the secret key has been saved to system keyring.",
        id52
    );
    Ok(())
}

pub fn delete_identity(id52: Option<String>, path: Option<String>) -> eyre::Result<()> {
    if let Some(id52) = id52 {
        kulfi_utils::secret::delete_identity(&id52)?;
        println!("Identity(ID52) deleted: {}", id52);
    }

    let path = get_identity_path(path);
    let path = match path {
        Some(path) => path,
        None => PathBuf::from(kulfi_utils::secret::ID52_FILE),
    };
    if path.exists() {
        let id52 = std::fs::read_to_string(&path)?;
        if let Err(e) = kulfi_utils::secret::delete_identity(&id52) {
            eprint!(
                "Unable to delete identity(ID52): {} in file {}. Maybe you want to clean this file. Error: {}",
                id52,
                path.display(),
                e
            );
        }
        println!(
            "Identity(ID52) {} at file {} deleted.",
            id52,
            path.display()
        );
    } else {
        println!("Identity(ID52) file {} not found.", path.display());
    }
    Ok(())
}
