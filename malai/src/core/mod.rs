pub mod agent;
pub mod cli;
pub mod client;
pub mod cluster;
pub mod config;
// pub mod daemon;  // daemon.rs is in src/, not src/core/
pub mod protocol;
pub mod server;

// CLI module exports
pub use cli::execute_direct_command;