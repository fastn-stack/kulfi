//! Comprehensive malai CLI testing utilities
//!
//! This crate provides testing infrastructure for malai commands with:
//! - Automatic binary discovery with build support  
//! - Process lifecycle with RAII cleanup
//! - MALAI_HOME isolation for multi-instance testing
//! - Cluster setup and management
//! - Identity generation and management
//! - Fluent API for readable test scenarios
//!
//! ## Tested Binaries: malai
//! ⚠️ When adding binaries: update simple.rs + .github/workflows/malai-ssh-tests.yml

use std::time::Duration;

pub mod cluster;
pub mod malai_cmd;
pub mod simple;
pub mod ssh;
pub mod test_env;

pub use simple::{CommandOutput, get_malai_binary, ensure_malai_built};
pub use malai_cmd::{MalaiCommand, BackgroundProcess};
pub use ssh::{SshTestHelper, BasicCluster};
pub use test_env::{MalaiTestEnv, MachineHandle};
pub use cluster::ClusterTestHelper;

/// malai-specific test configuration
#[derive(Debug, Clone)]
pub struct MalaiCliConfig {
    pub pre_build: bool,
    pub cleanup_on_drop: bool,
    pub default_timeout: Duration,
    pub skip_keyring: bool,
}

impl Default for MalaiCliConfig {
    fn default() -> Self {
        Self {
            pre_build: true,
            cleanup_on_drop: true,
            default_timeout: Duration::from_secs(30),
            skip_keyring: true,
        }
    }
}