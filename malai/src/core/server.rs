use crate::core::config::Config;
use crate::core::protocol::*;
use eyre::Result;
use std::process::Stdio;
use tokio::process::Command;

/// SSH server that accepts incoming connections and executes commands
#[derive(Clone)]
pub struct Server {
    config: Config,
    machine_name: String,
    secret_key: fastn_id52::SecretKey,
}

impl Server {
    /// Create a new SSH server
    pub fn new(config: Config, machine_name: String, secret_key: fastn_id52::SecretKey) -> Self {
        Self {
            config,
            machine_name,
            secret_key,
        }
    }

    /// Start the SSH server
    pub async fn start(&self, graceful: fastn_p2p::Graceful) -> Result<()> {
        tracing::info!("Starting SSH server: {}", self.machine_name);

        // Get our machine configuration
        let machine_config = self.config.machines.get(&self.machine_name)
            .ok_or_else(|| eyre::eyre!("Machine '{}' not found in configuration", self.machine_name))?;

        tracing::info!("Machine ID52: {}", machine_config.id52);

        // Start listening for different SSH protocols
        let protocols = vec![
            SshProtocol::Execute,
            SshProtocol::Shell,
            SshProtocol::HttpProxy,
            SshProtocol::ConfigSync,
        ];

        use futures_util::stream::StreamExt;

        let request_stream = fastn_p2p::server::listen(self.secret_key.clone(), &protocols)?;
        let mut request_stream = std::pin::pin!(request_stream);

        fastn_p2p::spawn(async move {
            while let Some(request_result) = request_stream.next().await {
                match request_result {
                    Ok(request) => {
                        let server_clone = self.clone();
                        fastn_p2p::spawn(async move {
                            if let Err(e) = server_clone.handle_p2p_request(request).await {
                                tracing::error!("Error handling P2P request: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Error receiving P2P request: {}", e);
                    }
                }
            }
            Ok::<(), eyre::Error>(())
        });

        Ok(())
    }

    /// Handle incoming P2P request
    async fn handle_p2p_request(&self, request: fastn_p2p::server::Request<SshProtocol>) -> Result<()> {
        let peer_id = request.peer().id52();
        let protocol = request.protocol().clone();

        tracing::info!("Handling {} request from {}", protocol, peer_id);

        match protocol {
            SshProtocol::Execute => {
                self.handle_execute_request(request).await
            }
            SshProtocol::Shell => {
                self.handle_shell_request(request).await
            }
            SshProtocol::HttpProxy => {
                self.handle_http_proxy_request(request).await
            }
            SshProtocol::ConfigSync => {
                self.handle_config_sync_request(request).await
            }
        }
    }

    /// Handle execute command request
    async fn handle_execute_request(&self, request: fastn_p2p::server::Request<SshProtocol>) -> Result<()> {
        let (execute_request, handle): (ExecuteRequest, _) = request.get_input().await.map_err(|e| eyre::eyre!("Failed to get execute request: {}", e))?;

        tracing::info!("Execute request from {}: {} {:?}", 
                      execute_request.client_id52, execute_request.command, execute_request.args);

        // Check if client has permission to execute this command
        if !self.config.can_execute_command(&execute_request.client_id52, &self.machine_name, &execute_request.command) {
            let error = ExecuteError {
                message: format!("Permission denied: {} cannot execute '{}' on {}", 
                               execute_request.client_id52, execute_request.command, self.machine_name),
                error_code: ExecuteErrorCode::PermissionDenied,
            };
            return handle.send::<ExecuteResponse, ExecuteError>(Err(error)).await.map_err(|e| eyre::eyre!("Failed to send error: {}", e));
        }

        // Execute the command
        let mut cmd = Command::new(&execute_request.command);
        cmd.args(&execute_request.args)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        // Set working directory if provided
        if let Some(working_dir) = &execute_request.working_dir {
            cmd.current_dir(working_dir);
        }

        // Set environment variables
        for (key, value) in &execute_request.env {
            cmd.env(key, value);
        }

        let result = match cmd.spawn() {
            Ok(child) => {
                match child.wait_with_output().await {
                    Ok(output) => {
                        let response = ExecuteResponse {
                            stdout: output.stdout,
                            stderr: output.stderr,
                            exit_code: output.status.code().unwrap_or(-1),
                        };
                        Ok(response)
                    }
                    Err(e) => {
                        Err(ExecuteError {
                            message: format!("Failed to wait for command: {}", e),
                            error_code: ExecuteErrorCode::ExecutionFailed,
                        })
                    }
                }
            }
            Err(e) => {
                Err(ExecuteError {
                    message: format!("Failed to execute command: {}", e),
                    error_code: ExecuteErrorCode::CommandNotFound,
                })
            }
        };

        handle.send::<ExecuteResponse, ExecuteError>(result).await.map_err(|e| eyre::eyre!("Failed to send response: {}", e))?;
        Ok(())
    }

    /// Handle shell request
    async fn handle_shell_request(&self, request: fastn_p2p::server::Request<SshProtocol>) -> Result<()> {
        let (shell_request, handle): (ShellRequest, _) = request.get_input().await.map_err(|e| eyre::eyre!("Failed to get shell request: {}", e))?;

        tracing::info!("Shell request from {}", shell_request.client_id52);

        // Check if client has shell access
        if !self.config.can_execute_command(&shell_request.client_id52, &self.machine_name, "bash") {
            let error = ShellError {
                message: format!("Permission denied: {} cannot access shell on {}", 
                               shell_request.client_id52, self.machine_name),
                error_code: ShellErrorCode::PermissionDenied,
            };
            return handle.send::<ShellResponse, ShellError>(Err(error)).await.map_err(|e| eyre::eyre!("Failed to send error: {}", e));
        }

        // TODO: Implement interactive shell session
        // For now, return not implemented error
        let error = ShellError {
            message: "Interactive shell not yet implemented".to_string(),
            error_code: ShellErrorCode::SessionFailed,
        };
        handle.send::<ShellResponse, ShellError>(Err(error)).await.map_err(|e| eyre::eyre!("Failed to send error: {}", e))?;

        Ok(())
    }

    /// Handle HTTP proxy request
    async fn handle_http_proxy_request(&self, request: fastn_p2p::server::Request<SshProtocol>) -> Result<()> {
        let (proxy_request, handle): (HttpProxyRequest, _) = request.get_input().await.map_err(|e| eyre::eyre!("Failed to get proxy request: {}", e))?;

        tracing::info!("HTTP proxy request from {} for service {}", 
                      proxy_request.client_id52, proxy_request.service_name);

        // Check if client can access this service
        if !self.config.can_access_service(&proxy_request.client_id52, &self.machine_name, &proxy_request.service_name) {
            let error = HttpProxyError {
                message: format!("Permission denied: {} cannot access service {} on {}", 
                               proxy_request.client_id52, proxy_request.service_name, self.machine_name),
                error_code: HttpProxyErrorCode::PermissionDenied,
            };
            return handle.send::<HttpProxyResponse, HttpProxyError>(Err(error)).await.map_err(|e| eyre::eyre!("Failed to send error: {}", e));
        }

        // TODO: Implement HTTP proxy functionality
        let error = HttpProxyError {
            message: "HTTP proxy not yet implemented".to_string(),
            error_code: HttpProxyErrorCode::ServiceUnavailable,
        };
        handle.send::<HttpProxyResponse, HttpProxyError>(Err(error)).await.map_err(|e| eyre::eyre!("Failed to send error: {}", e))?;

        Ok(())
    }

    /// Handle configuration sync request
    async fn handle_config_sync_request(&self, request: fastn_p2p::server::Request<SshProtocol>) -> Result<()> {
        let (sync_request, handle): (ConfigSyncRequest, _) = request.get_input().await.map_err(|e| eyre::eyre!("Failed to get sync request: {}", e))?;

        tracing::info!("Config sync request from {}", sync_request.member_id52);

        // TODO: Implement configuration synchronization
        let response = ConfigSyncResponse {
            config_hash: "placeholder-hash".to_string(),
            config_data: None,
            needs_update: false,
        };
        handle.send::<ConfigSyncResponse, ConfigSyncError>(Ok(response)).await.map_err(|e| eyre::eyre!("Failed to send response: {}", e))?;

        Ok(())
    }

    /// Get list of HTTP services this machine exposes
    pub fn get_exposed_services(&self) -> Vec<(String, u16)> {
        if let Some(machine_config) = self.config.machines.get(&self.machine_name) {
            machine_config.services
                .iter()
                .map(|(name, service_config)| (name.clone(), service_config.port))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if a client can access an HTTP service
    pub fn can_client_access_service(&self, client_id52: &str, service_name: &str) -> bool {
        self.config.can_access_service(client_id52, &self.machine_name, service_name)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_config() -> Config {
        let toml_content = r#"
[cluster_manager]
id52 = "cluster-manager-id52"

[machine.web01]
id52 = "web01-id52"
accept_ssh = true
allow_from = "client1-id52,client2-id52"

[machine.web01.commands.ls]
allow_from = "readonly-client-id52"

[machine.web01.services.admin]
port = 8080
allow_from = "admin-client-id52"

[machine.client]
id52 = "client1-id52"
"#;

        toml::from_str(toml_content).unwrap()
    }

    #[test]
    fn test_server_creation() {
        let config = create_test_config();
        let secret_key = fastn_id52::SecretKey::generate();
        let server = Server::new(config, "web01".to_string(), secret_key);
        
        assert_eq!(server.machine_name, "web01");
        assert_eq!(server.config.machines.len(), 2);
    }

    #[test]
    fn test_service_access_control() {
        let config = create_test_config();
        let secret_key = fastn_id52::SecretKey::generate();
        let server = Server::new(config, "web01".to_string(), secret_key);

        // admin-client-id52 should be able to access admin service
        assert!(server.can_client_access_service("admin-client-id52", "admin"));
        
        // client1-id52 should not be able to access admin service
        assert!(!server.can_client_access_service("client1-id52", "admin"));
    }

    #[test]
    fn test_exposed_services() {
        let config = create_test_config();
        let secret_key = fastn_id52::SecretKey::generate();
        let server = Server::new(config, "web01".to_string(), secret_key);
        
        let services = server.get_exposed_services();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].0, "admin");
        assert_eq!(services[0].1, 8080);
    }
}