#![deny(unused_extern_crates)]
#![deny(unused_crate_dependencies)]
#![deny(unsafe_code)]

extern crate self as malai;

use clap as _;
use clap_verbosity_flag as _;
use tracing_subscriber as _;

mod browse;
mod expose_http;
mod expose_tcp;
mod folder;
mod keygen;
mod http_bridge;
mod http_proxy;
mod http_proxy_remote;
mod run;
mod tcp_bridge;

pub use browse::browse;
pub use expose_http::expose_http;
pub use expose_tcp::expose_tcp;
pub use folder::folder;
pub use keygen::keygen;
pub use http_bridge::http_bridge;
pub use http_proxy::{ProxyData, http_proxy};
pub use http_proxy_remote::http_proxy_remote;
pub use run::run;
pub use tcp_bridge::tcp_bridge;

#[cfg(feature = "ui")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn ui() -> eyre::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

pub fn public_check(public: bool, service: &str, cmd: &str) -> bool {
    use colored::Colorize;

    if !public {
        tracing::info!("--public not passed. Quitting!");
        eprintln!(
            "You need to pass --public to expose the {service}. \
                    This is a security feature to prevent exposing your service \
                    to the public without your knowledge."
        );
        eprintln!("Instead, run: {}", cmd.yellow());
        eprintln!("In future, we will add a way to add access control.");
    }

    public
}

pub fn identity_read_err_msg(e: eyre::Report) {
    eprintln!("failed to get identity");
    eprintln!("malai uses your system keyring for storing identities securely.");
    eprintln!("use `malai keygen` if system keyring is not available.");
    eprintln!("full error:");
    eprintln!("{e:?}");
}
