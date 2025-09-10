pub mod agent;
pub mod client;
pub mod cluster;
pub mod config;
pub mod daemon;
pub mod protocol;
pub mod server;

// Re-export daemon functions for backwards compatibility
pub use daemon::{
    get_malai_home, get_default_malai_home, load_and_validate_all_configs,
    start_services_from_configs, show_detailed_status, show_cluster_info,
    MalaiProtocol, RemoteAccessRequest, RemoteAccessResponse, RemoteAccessError,
    find_machine_id52_in_config
};