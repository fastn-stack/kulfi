use crate::ssh::config::Config;
use eyre::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Manages cluster configuration and member coordination
pub struct ClusterManager {
    config: Config,
    config_path: PathBuf,
    config_hash: String,
    member_hashes: HashMap<String, String>, // id52 -> last known config hash
}

impl ClusterManager {
    /// Create a new cluster manager
    pub fn new(config_path: PathBuf) -> Result<Self> {
        let config = Config::load_from_file(config_path.to_str().unwrap())?;
        let config_hash = Self::calculate_config_hash(&config)?;
        
        Ok(Self {
            config,
            config_path,
            config_hash,
            member_hashes: HashMap::new(),
        })
    }

    /// Start the cluster manager
    pub async fn start(&mut self, graceful: kulfi_utils::Graceful) -> Result<()> {
        tracing::info!("Starting SSH cluster manager");

        // Start configuration monitoring
        let config_monitor = self.start_config_monitor(graceful.clone());
        
        // Start member synchronization
        let sync_task = self.start_member_sync(graceful.clone());

        // Wait for shutdown
        graceful.spawn(async move {
            tokio::select! {
                _ = config_monitor => {
                    tracing::info!("Config monitor stopped");
                }
                _ = sync_task => {
                    tracing::info!("Member sync stopped");
                }
            }
            Ok::<(), eyre::Error>(())
        });

        Ok(())
    }

    /// Monitor configuration file for changes
    async fn start_config_monitor(&mut self, graceful: kulfi_utils::Graceful) -> Result<()> {
        let config_path = self.config_path.clone();
        
        graceful.spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Check if config file has changed
                        if let Ok(new_config) = Config::load_from_file(config_path.to_str().unwrap()) {
                            if let Ok(new_hash) = Self::calculate_config_hash(&new_config) {
                                // TODO: Compare with current hash and update if needed
                                tracing::trace!("Config hash: {}", new_hash);
                            }
                        }
                    }
                    _ = graceful.cancelled() => {
                        break;
                    }
                }
            }
            Ok::<(), eyre::Error>(())
        });

        Ok(())
    }

    /// Start member synchronization task
    async fn start_member_sync(&self, graceful: kulfi_utils::Graceful) -> Result<()> {
        graceful.spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // TODO: Sync configuration with cluster members
                        tracing::trace!("Syncing configuration with cluster members");
                    }
                    _ = graceful.cancelled() => {
                        break;
                    }
                }
            }
            Ok::<(), eyre::Error>(())
        });

        Ok(())
    }

    /// Calculate hash of configuration
    fn calculate_config_hash(config: &Config) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let serialized = toml::to_string(config)?;
        let mut hasher = DefaultHasher::new();
        serialized.hash(&mut hasher);
        Ok(format!("{:x}", hasher.finish()))
    }

    /// Get filtered configuration for a specific member
    pub fn get_member_config(&self, _member_id52: &str) -> Config {
        let mut filtered_config = self.config.clone();
        
        // Remove sensitive data
        filtered_config.cluster_manager.private_key = None;
        filtered_config.cluster_manager.private_key_file = None;
        
        // TODO: Filter config based on what the member needs to know
        // For example, servers only need to know about their own services
        // and which clients can access them
        
        filtered_config
    }

    /// Check if a member's configuration is up to date
    pub fn is_member_config_current(&self, member_id52: &str) -> bool {
        if let Some(member_hash) = self.member_hashes.get(member_id52) {
            member_hash == &self.config_hash
        } else {
            false
        }
    }

    /// Update member's last known configuration hash
    pub fn update_member_hash(&mut self, member_id52: String, hash: String) {
        self.member_hashes.insert(member_id52, hash);
    }

    /// Get list of all cluster members
    pub fn get_all_members(&self) -> Vec<String> {
        let mut members = Vec::new();
        
        // Add all servers
        members.extend(self.config.servers.keys().cloned());
        
        // Add all devices
        members.extend(self.config.devices.keys().cloned());
        
        members
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_cluster_manager_creation() {
        let toml_content = r#"
[cluster_manager]
id52 = "cluster-manager-id52"
use_keyring = true

[servers.web01]
id52 = "web01-id52"
allow_from = "device1-id52"

[devices.laptop]
id52 = "laptop-id52"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        
        let manager = ClusterManager::new(temp_file.path().to_path_buf()).unwrap();
        assert_eq!(manager.config.cluster_manager.id52, "cluster-manager-id52");
        assert_eq!(manager.config.servers.len(), 1);
        assert_eq!(manager.config.devices.len(), 1);
    }

    #[test]
    fn test_member_config_filtering() {
        let toml_content = r#"
[cluster_manager]
id52 = "cluster-manager-id52"
use_keyring = true
private_key = "secret-key-here"

[servers.web01]
id52 = "web01-id52"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        
        let manager = ClusterManager::new(temp_file.path().to_path_buf()).unwrap();
        let member_config = manager.get_member_config("web01-id52");
        
        // Sensitive data should be removed
        assert!(member_config.cluster_manager.private_key.is_none());
        assert!(member_config.cluster_manager.private_key_file.is_none());
    }
}