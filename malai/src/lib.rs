#![deny(unused_extern_crates)]
#![deny(unused_crate_dependencies)]
#![deny(unsafe_code)]

extern crate self as malai;

use chrono as _;
use clap as _;
use clap_verbosity_flag as _;
use dirs as _;
use fastn_id52 as _;
use fastn_net as _;
use fastn_p2p as _;
use kulfi_id52 as _;
use libc as _;
use toml as _;
use tracing_subscriber as _;

#[cfg(test)]
use malai_cli_test_utils as _;
#[cfg(test)]
use tempfile as _;

mod browse;
mod expose_http;
mod expose_tcp;
mod folder;
mod http_bridge;
mod http_proxy;
mod http_proxy_remote;
mod keygen;
mod run;
// mod core;  // Temporarily disabled - fixing module reorganization
mod core_utils;  // Core malai utilities
mod tcp_bridge;

pub use browse::browse;
pub use expose_http::expose_http;
pub use expose_tcp::expose_tcp;
pub use folder::folder;
pub use http_bridge::http_bridge;
pub use http_proxy::{ProxyData, http_proxy};
pub use http_proxy_remote::http_proxy_remote;
pub use keygen::keygen;
pub use run::run;
pub use core_utils::{
    create_cluster, execute_ssh_command, init_cluster, init_machine_for_cluster, 
    init_machine_for_cluster_with_alias, show_cluster_info, start_ssh_agent, 
    start_ssh_cluster, start_unified_malai
};
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
