//! The kulfi folder
//!
//! The location of this folder is platform-specific, on Linux it is either
//! $HOME/.local/share/kulfi or $XDG_DATA_HOME/kulfi, on MacOS it is $HOME/Library/Application
//! Support/com.FifthTry.kulfi and on Windows: {FOLDERID_RoamingAppData}\kulfi\data which is usually
//! C:\Users\Alice\AppData\Roaming\FifthTry\kulfi\data.
//!
//! The folder contains a lock file, `$kulfi/kulfi.lock, which is used to ensure only one instance
//! of `kulfi` is running.
//!
//! The folder contains more folders like `identities`, `logs` and maybe `config.json` etc. in
//! the future.
//!
//! The identities folder is the most interesting one, it contains one folder for every identity
//! that exists on this machine. The content of single `identity` folder is described
//! in `identity/create.rs`.

mod init_if_required;
mod lock;

pub use init_if_required::init_if_required;
pub use lock::{KULFI_LOCK, MALAI_LOCK, exclusive, kulfi_lock_file, malai_lock_file};
