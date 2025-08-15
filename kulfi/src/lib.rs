#![deny(unused_extern_crates)]
#![deny(unused_crate_dependencies)]
#![deny(unsafe_code)]

extern crate self as kulfi; // TODO: Remove this and separate kulfi binary from library to get rid of unused_crate_dependencies
// lint check errors
// Only the binary is using the following crates:
use clap as _;
use clap_verbosity_flag as _;
use directories as _;
use thiserror as _;
use tracing_subscriber as _;

mod browse;
mod config;
pub mod control_server;
mod counters;
pub mod dot_kulfi;
mod identity;
pub mod peer_server;
mod start;
mod tauri;
pub mod utils;

// pub use identity::{Identity, PeerIdentity};
pub use browse::browse;
pub use config::Config;
pub use counters::{
    CONTROL_CONNECTION_COUNT, CONTROL_REQUEST_COUNT, IN_FLIGHT_REQUESTS,
    OPEN_CONTROL_CONNECTION_COUNT,
};
pub use identity::Identity;
pub use start::start;
pub use tauri::ui;
