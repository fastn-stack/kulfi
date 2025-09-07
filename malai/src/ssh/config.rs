use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub cluster_manager: ClusterManagerConfig,
    pub servers: HashMap<String, ServerConfig>,
    pub devices: HashMap<String, DeviceConfig>,
    pub groups: HashMap<String, GroupConfig>,
}

/// Cluster manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterManagerConfig {
    pub id52: String,
    #[serde(default = "default_true")]
    pub use_keyring: bool,
    pub private_key_file: Option<String>,
    pub private_key: Option<String>,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub id52: String,
    pub allow_from: Option<String>,
    pub commands: HashMap<String, CommandConfig>,
    pub services: HashMap<String, ServiceConfig>,
}

/// Device configuration (client-only nodes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub id52: String,
}

/// Group configuration for easier management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    pub members: String, // comma-separated list
}

/// Command-specific access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    pub allow_from: String, // comma-separated id52 list
}

/// HTTP service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub http: u16, // port number
    pub allow_from: String, // comma-separated id52 list or "*"
}

fn default_true() -> bool {
    true
}

impl Config {
    /// Load configuration from TOML file
    pub fn load_from_file(path: &str) -> eyre::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to TOML file
    pub fn save_to_file(&self, path: &str) -> eyre::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get servers that a device/client can access
    pub fn get_accessible_servers(&self, client_id52: &str) -> Vec<String> {
        let mut accessible = Vec::new();
        
        for (server_name, server_config) in &self.servers {
            if let Some(allow_from) = &server_config.allow_from {
                if allow_from.contains(client_id52) || allow_from.contains('*') {
                    accessible.push(server_name.clone());
                }
            }
        }
        
        accessible
    }

    /// Check if a client can execute a command on a server
    pub fn can_execute_command(&self, client_id52: &str, server_name: &str, command: &str) -> bool {
        if let Some(server) = self.servers.get(server_name) {
            // Check server-level access first
            if let Some(allow_from) = &server.allow_from {
                if allow_from.contains(client_id52) || allow_from.contains('*') {
                    return true;
                }
            }
            
            // Check command-specific access
            if let Some(cmd_config) = server.commands.get(command) {
                return cmd_config.allow_from.contains(client_id52) || cmd_config.allow_from.contains('*');
            }
        }
        false
    }

    /// Check if a client can access an HTTP service
    pub fn can_access_service(&self, client_id52: &str, server_name: &str, service_name: &str) -> bool {
        if let Some(server) = self.servers.get(server_name) {
            if let Some(service) = server.services.get(service_name) {
                return service.allow_from.contains(client_id52) || service.allow_from.contains('*');
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let toml_content = r#"
[cluster_manager]
id52 = "test-cluster-manager-id52"
use_keyring = true

[servers.web01]
id52 = "web01-id52"
allow_from = "device1-id52,device2-id52"

[servers.web01.commands.ls]
allow_from = "readonly-device-id52"

[servers.web01.services.admin]
http = 8080
allow_from = "admin-device-id52"

[devices.laptop]
id52 = "laptop-id52"

[groups.web_servers]
members = "web01,web02"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.cluster_manager.id52, "test-cluster-manager-id52");
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.groups.len(), 1);
    }

    #[test]
    fn test_access_control() {
        let toml_content = r#"
[cluster_manager]
id52 = "cluster-manager-id52"

[servers.web01]
id52 = "web01-id52"
allow_from = "device1-id52,device2-id52"

[servers.web01.commands.ls]
allow_from = "readonly-device-id52"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        
        // Test server access
        assert!(config.can_execute_command("device1-id52", "web01", "bash"));
        assert!(!config.can_execute_command("device3-id52", "web01", "bash"));
        
        // Test command-specific access
        assert!(config.can_execute_command("readonly-device-id52", "web01", "ls"));
        assert!(!config.can_execute_command("readonly-device-id52", "web01", "bash"));
    }
}