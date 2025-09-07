use crate::ssh::protocol::*;
use eyre::Result;
use std::path::PathBuf;

/// SSH client for connecting to remote servers
pub struct Client {
    secret_key: fastn_id52::SecretKey,
    data_dir: PathBuf,
}

impl Client {
    /// Create a new SSH client
    pub fn new(secret_key: fastn_id52::SecretKey, data_dir: PathBuf) -> Self {
        Self {
            secret_key,
            data_dir,
        }
    }

    /// Connect to a server and execute a command
    pub async fn execute_command(&self, server_address: &str, command: &str, args: Vec<String>) -> Result<ExecuteResponse> {
        tracing::info!("Executing command '{}' on server {}", command, server_address);

        // Parse server address (could be domain-based or ID-based)
        let (server_public_key, _cluster_info) = self.parse_server_address(server_address)?;

        let request = ExecuteRequest {
            client_id52: self.secret_key.public_key().id52(),
            command: command.to_string(),
            args,
            env: std::env::vars().collect(),
            working_dir: std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
        };

        match fastn_p2p::call(
            self.secret_key.clone(),
            &server_public_key,
            SshProtocol::Execute,
            request,
        )
        .await
        {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(error)) => Err(eyre::eyre!("SSH execute error: {}", error)),
            Err(e) => Err(eyre::eyre!("Failed to call remote server: {}", e)),
        }
    }

    /// Start an interactive shell session
    pub async fn start_shell(&self, server_address: &str) -> Result<()> {
        tracing::info!("Starting interactive shell on server {}", server_address);

        let (server_public_key, _cluster_info) = self.parse_server_address(server_address)?;

        let request = ShellRequest {
            client_id52: self.secret_key.public_key().id52(),
            terminal_size: None, // TODO: Get actual terminal size
            env: std::env::vars().collect(),
        };

        match fastn_p2p::call(
            self.secret_key.clone(),
            &server_public_key,
            SshProtocol::Shell,
            request,
        )
        .await
        {
            Ok(Ok(_response)) => {
                // TODO: Handle interactive shell session
                println!("Interactive shell not yet implemented");
            }
            Ok(Err(error)) => {
                eprintln!("Error: {}", error);
            }
            Err(e) => {
                return Err(eyre::eyre!("Failed to call remote server: {}", e));
            }
        }

        Ok(())
    }

    /// Parse server address into server public key and cluster information
    fn parse_server_address(&self, address: &str) -> Result<(fastn_id52::PublicKey, ClusterInfo)> {
        // Handle different addressing formats:
        // 1. Domain-based: node-alias.cluster-domain.com
        // 2. ID-based: node-alias.cluster-id52
        // 3. Full ID: node-id52.cluster-id52

        if address.contains('.') {
            let parts: Vec<&str> = address.split('.').collect();
            if parts.len() >= 2 {
                let node_part = parts[0];
                let cluster_part = parts[1..].join(".");
                
                // TODO: Implement proper address resolution
                // For now, create a mock public key - generate a temporary secret and use its public key
                let mock_secret = fastn_id52::SecretKey::generate();
                let mock_public_key = mock_secret.public_key();
                
                return Ok((
                    mock_public_key,
                    ClusterInfo {
                        cluster_id: cluster_part,
                        is_domain: parts.len() > 2,
                    },
                ));
            }
        }

        Err(eyre::eyre!("Invalid server address format: {}", address))
    }

    /// Get client data directory for a specific cluster
    pub fn get_cluster_dir(&self, cluster_id: &str) -> PathBuf {
        self.data_dir.join("ssh").join("clusters").join(cluster_id)
    }

    /// Load cluster configuration
    pub fn load_cluster_config(&self, cluster_id: &str) -> Result<crate::ssh::config::Config> {
        let config_path = self.get_cluster_dir(cluster_id).join("cluster-config.toml");
        crate::ssh::config::Config::load_from_file(config_path.to_str().unwrap())
    }

    /// Get available servers in a cluster
    pub fn get_available_servers(&self, cluster_id: &str) -> Result<Vec<String>> {
        let config = self.load_cluster_config(cluster_id)?;
        Ok(config.get_accessible_servers(&self.secret_key.public_key().id52()))
    }
}

/// Cluster information parsed from server address
#[derive(Debug, Clone)]
struct ClusterInfo {
    cluster_id: String,
    is_domain: bool, // true if domain-based, false if ID-based
}

/// Utility functions for SSH operations
pub struct SshUtils;

impl SshUtils {
    /// Execute a single SSH command and print output
    pub async fn exec_and_print(server_address: &str, command: &str, args: Vec<String>) -> Result<i32> {
        // TODO: Get client secret key from keyring or config
        let secret_key = fastn_id52::SecretKey::generate(); // Placeholder
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join("malai");

        let client = Client::new(secret_key, data_dir);
        let response = client.execute_command(server_address, command, args).await?;

        // Print stdout
        if !response.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&response.stdout));
        }
        
        // Print stderr to stderr
        if !response.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&response.stderr));
        }
        
        Ok(response.exit_code)
    }

    /// Start an interactive SSH session
    pub async fn start_interactive_session(server_address: &str) -> Result<()> {
        // TODO: Get client secret key from keyring or config
        let secret_key = fastn_id52::SecretKey::generate(); // Placeholder
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join("malai");

        let client = Client::new(secret_key, data_dir);
        client.start_shell(server_address).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_address_parsing() {
        let secret_key = fastn_id52::SecretKey::generate();
        let client = Client::new(secret_key, PathBuf::from("/tmp"));

        // Test domain-based address
        let result = client.parse_server_address("web01.company.com");
        assert!(result.is_ok());
        let (_server_key, cluster_info) = result.unwrap();
        assert!(cluster_info.is_domain);
        assert_eq!(cluster_info.cluster_id, "company.com");

        // Test ID-based address  
        let result = client.parse_server_address("web01.cluster-id52");
        assert!(result.is_ok());
        let (_server_key, cluster_info) = result.unwrap();
        assert!(!cluster_info.is_domain);
        assert_eq!(cluster_info.cluster_id, "cluster-id52");
    }

    #[test]
    fn test_cluster_directory_path() {
        let secret_key = fastn_id52::SecretKey::generate();
        let client = Client::new(secret_key, PathBuf::from("/home/user/.local/share/malai"));
        
        let cluster_dir = client.get_cluster_dir("test-cluster");
        assert_eq!(cluster_dir, PathBuf::from("/home/user/.local/share/malai/ssh/clusters/test-cluster"));
    }

    #[test]
    fn test_client_creation() {
        let secret_key = fastn_id52::SecretKey::generate();
        let data_dir = PathBuf::from("/tmp/malai");
        let client = Client::new(secret_key.clone(), data_dir.clone());
        
        assert_eq!(client.data_dir, data_dir);
        assert_eq!(client.secret_key.public_key(), secret_key.public_key());
    }
}