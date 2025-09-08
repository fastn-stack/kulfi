use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub cluster_manager: ClusterManagerConfig,
    pub machines: HashMap<String, MachineConfig>,
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

/// Machine configuration (unified for all machine types)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    pub id52: String,
    #[serde(default)]
    pub accept_ssh: bool,                      // true = can accept SSH connections
    pub allow_from: Option<String>,           // SSH access control
    #[serde(default)]
    pub commands: HashMap<String, CommandConfig>, // Command-specific access
    #[serde(default)]
    pub services: HashMap<String, ServiceConfig>, // HTTP services
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
    pub port: u16,             // port number (renamed from 'http')
    pub allow_from: String,    // comma-separated id52 list or "*"
}

/// Role that a machine plays in the cluster
#[derive(Debug, Clone, PartialEq)]
pub enum MachineRole {
    ClusterManager,                    // This machine manages the cluster
    SshServer(String),                // This machine accepts SSH (machine name)  
    ClientOnly(String),               // This machine is client-only (machine name)
    Unknown,                          // Machine not found in config
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

    /// Get machines that a client can access via SSH
    pub fn get_accessible_machines(&self, client_id52: &str) -> Vec<String> {
        let mut accessible = Vec::new();
        
        for (machine_name, machine_config) in &self.machines {
            if machine_config.accept_ssh {
                if let Some(allow_from) = &machine_config.allow_from {
                    if allow_from.contains(client_id52) || allow_from.contains('*') {
                        accessible.push(machine_name.clone());
                    }
                }
            }
        }
        
        accessible
    }

    /// Check if a client can execute a command on a machine
    pub fn can_execute_command(&self, client_id52: &str, machine_name: &str, command: &str) -> bool {
        if let Some(machine) = self.machines.get(machine_name) {
            // Machine must accept SSH connections
            if !machine.accept_ssh {
                return false;
            }
            
            // Check machine-level access first
            if let Some(allow_from) = &machine.allow_from {
                if allow_from.contains(client_id52) || allow_from.contains('*') {
                    return true;
                }
            }
            
            // Check command-specific access
            if let Some(cmd_config) = machine.commands.get(command) {
                return cmd_config.allow_from.contains(client_id52) || cmd_config.allow_from.contains('*');
            }
        }
        false
    }

    /// Check if a client can access an HTTP service
    pub fn can_access_service(&self, client_id52: &str, machine_name: &str, service_name: &str) -> bool {
        if let Some(machine) = self.machines.get(machine_name) {
            if let Some(service) = machine.services.get(service_name) {
                return service.allow_from.contains(client_id52) || service.allow_from.contains('*');
            }
        }
        false
    }

    /// Get role of local machine by matching identity
    pub fn get_local_role(&self, local_id52: &str) -> MachineRole {
        // Check if this is the cluster manager
        if self.cluster_manager.id52 == local_id52 {
            return MachineRole::ClusterManager;
        }

        // Check if this is a configured machine
        for (machine_name, machine_config) in &self.machines {
            if machine_config.id52 == local_id52 {
                if machine_config.accept_ssh {
                    return MachineRole::SshServer(machine_name.clone());
                } else {
                    return MachineRole::ClientOnly(machine_name.clone());
                }
            }
        }

        MachineRole::Unknown
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

[machine.web01]
id52 = "web01-id52"
accept_ssh = true
allow_from = "laptop-id52,admin-id52"

[machine.web01.commands.ls]
allow_from = "readonly-id52"

[machine.web01.services.admin]
port = 8080
allow_from = "admin-id52"

[machine.laptop]
id52 = "laptop-id52"

[groups.web_servers]
members = "web01,web02"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.cluster_manager.id52, "test-cluster-manager-id52");
        assert_eq!(config.machines.len(), 2);
        assert_eq!(config.groups.len(), 1);
        assert!(config.machines.get("web01").unwrap().accept_ssh);
        assert!(!config.machines.get("laptop").unwrap().accept_ssh);
    }

    #[test]
    fn test_access_control() {
        let toml_content = r#"
[cluster_manager]
id52 = "cluster-manager-id52"

[machine.web01]
id52 = "web01-id52"
accept_ssh = true
allow_from = "client1-id52,client2-id52"

[machine.web01.commands.ls]
allow_from = "readonly-client-id52"

[machine.laptop]
id52 = "laptop-id52"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        
        // Test machine SSH access
        assert!(config.can_execute_command("client1-id52", "web01", "bash"));
        assert!(!config.can_execute_command("client3-id52", "web01", "bash"));
        
        // Test command-specific access
        assert!(config.can_execute_command("readonly-client-id52", "web01", "ls"));
        assert!(!config.can_execute_command("readonly-client-id52", "web01", "bash"));
        
        // Test client-only machine cannot accept SSH
        assert!(!config.can_execute_command("client1-id52", "laptop", "bash"));
    }

    #[test]
    fn test_role_detection() {
        let toml_content = r#"
[cluster_manager]
id52 = "manager-id52"

[machine.web01]
id52 = "web01-id52"
accept_ssh = true

[machine.laptop]  
id52 = "laptop-id52"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        
        assert_eq!(config.get_local_role("manager-id52"), MachineRole::ClusterManager);
        assert_eq!(config.get_local_role("web01-id52"), MachineRole::SshServer("web01".to_string()));
        assert_eq!(config.get_local_role("laptop-id52"), MachineRole::ClientOnly("laptop".to_string()));
        assert_eq!(config.get_local_role("unknown-id52"), MachineRole::Unknown);
    }
}