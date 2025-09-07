// use crate::ssh::client::Client; // Not used yet
use eyre::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::net::{UnixListener, UnixStream};

/// SSH agent that manages connections and provides HTTP proxy functionality
pub struct Agent {
    socket_path: PathBuf,
    data_dir: PathBuf,
    client_id52: String,
    connections: HashMap<String, Connection>, // server_id52 -> connection
    http_proxy_port: Option<u16>,
}

impl Agent {
    /// Create a new SSH agent
    pub fn new(data_dir: PathBuf, client_id52: String) -> Self {
        let socket_path = data_dir.join("ssh").join("agent.sock");
        
        Self {
            socket_path,
            data_dir,
            client_id52,
            connections: HashMap::new(),
            http_proxy_port: None,
        }
    }

    /// Start the SSH agent
    pub async fn start(&mut self, graceful: kulfi_utils::Graceful, enable_http_proxy: bool) -> Result<()> {
        tracing::info!("Starting SSH agent");

        // Ensure socket directory exists
        if let Some(socket_dir) = self.socket_path.parent() {
            tokio::fs::create_dir_all(socket_dir).await?;
        }

        // Remove existing socket if it exists
        let _ = tokio::fs::remove_file(&self.socket_path).await;

        // Bind Unix socket for agent communication
        let listener = UnixListener::bind(&self.socket_path)?;
        tracing::info!("Agent listening on socket: {:?}", self.socket_path);

        // Start HTTP proxy if enabled
        if enable_http_proxy {
            self.start_http_proxy(graceful.clone()).await?;
        }

        // Start connection manager
        self.start_connection_manager(graceful.clone()).await?;

        // Handle client connections
        graceful.spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, _addr)) => {
                                let graceful_for_connection = graceful.clone();
                                graceful.spawn(async move {
                                    if let Err(e) = Self::handle_client_connection(stream).await {
                                        tracing::error!("Error handling client connection: {}", e);
                                    }
                                    Ok::<(), eyre::Error>(())
                                });
                            }
                            Err(e) => {
                                tracing::error!("Error accepting connection: {}", e);
                                break;
                            }
                        }
                    }
                    _ = graceful.cancelled() => {
                        tracing::info!("SSH agent shutting down");
                        break;
                    }
                }
            }
            Ok::<(), eyre::Error>(())
        });

        Ok(())
    }

    /// Handle a client connection to the agent
    async fn handle_client_connection(_stream: UnixStream) -> Result<()> {
        tracing::debug!("Handling new client connection");

        // TODO: Implement agent protocol
        // For now, just close the connection
        
        Ok(())
    }

    /// Start the HTTP proxy server
    async fn start_http_proxy(&mut self, graceful: kulfi_utils::Graceful) -> Result<()> {
        // Find available port
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let proxy_port = listener.local_addr()?.port();
        self.http_proxy_port = Some(proxy_port);

        tracing::info!("Starting HTTP proxy on port {}", proxy_port);

        graceful.spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                tracing::debug!("HTTP proxy connection from: {}", addr);
                                let graceful_for_request = graceful.clone();
                                graceful.spawn(async move {
                                    if let Err(e) = Self::handle_http_request(stream).await {
                                        tracing::error!("Error handling HTTP request: {}", e);
                                    }
                                    Ok::<(), eyre::Error>(())
                                });
                            }
                            Err(e) => {
                                tracing::error!("Error accepting HTTP connection: {}", e);
                                break;
                            }
                        }
                    }
                    _ = graceful.cancelled() => {
                        tracing::info!("HTTP proxy shutting down");
                        break;
                    }
                }
            }
            Ok::<(), eyre::Error>(())
        });

        Ok(())
    }

    /// Handle an HTTP proxy request
    async fn handle_http_request(_stream: tokio::net::TcpStream) -> Result<()> {
        // TODO: Parse HTTP request
        // TODO: Extract host header to determine target service
        // TODO: Resolve service to server and establish connection
        // TODO: Proxy request/response
        
        tracing::debug!("HTTP request handling not yet implemented");
        Ok(())
    }

    /// Start the connection manager
    async fn start_connection_manager(&self, graceful: kulfi_utils::Graceful) -> Result<()> {
        graceful.spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // TODO: Cleanup idle connections
                        // TODO: Refresh cluster configurations
                        tracing::trace!("Connection manager tick");
                    }
                    _ = graceful.cancelled() => {
                        tracing::info!("Connection manager shutting down");
                        break;
                    }
                }
            }
            Ok::<(), eyre::Error>(())
        });

        Ok(())
    }

    /// Print environment variables for shell integration
    pub fn print_environment(&self, lockdown_mode: bool, http_proxy: bool) -> Result<()> {
        // Print MALAI_SSH_AGENT
        println!("MALAI_SSH_AGENT={}", self.socket_path.display());

        // Print MALAI_LOCKDOWN_MODE if enabled
        if lockdown_mode {
            println!("MALAI_LOCKDOWN_MODE=true");
        }

        // Print HTTP_PROXY if HTTP proxy is enabled and running
        if http_proxy {
            if let Some(port) = self.http_proxy_port {
                println!("HTTP_PROXY=http://127.0.0.1:{}", port);
            }
        }

        Ok(())
    }

    /// Get the socket path for this agent
    pub fn get_socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Check if agent is running
    pub async fn is_running(&self) -> bool {
        // Try to connect to the socket
        match UnixStream::connect(&self.socket_path).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

/// Represents a persistent connection to a server
#[derive(Debug)]
struct Connection {
    server_id52: String,
    last_used: std::time::Instant,
    // TODO: Add actual connection handle (iroh connection)
}

impl Connection {
    fn new(server_id52: String) -> Self {
        Self {
            server_id52,
            last_used: std::time::Instant::now(),
        }
    }

    fn update_last_used(&mut self) {
        self.last_used = std::time::Instant::now();
    }

    fn is_idle(&self, idle_timeout: std::time::Duration) -> bool {
        self.last_used.elapsed() > idle_timeout
    }
}

/// Agent utilities
pub struct AgentUtils;

impl AgentUtils {
    /// Start agent if not already running and return environment variables
    pub async fn ensure_agent_running(lockdown_mode: bool, http_proxy: bool) -> Result<()> {
        // TODO: Get client identity and data directory
        let client_id52 = "temp-client-id52".to_string(); // Placeholder
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join("malai");

        let agent = Agent::new(data_dir, client_id52);

        // Check if agent is already running
        if agent.is_running().await {
            // Agent is running, just print environment
            agent.print_environment(lockdown_mode, http_proxy)?;
        } else {
            // Need to start agent
            tracing::info!("Starting SSH agent in background");
            
            // TODO: Start agent as background process
            // For now, just print environment assuming agent will start
            agent.print_environment(lockdown_mode, http_proxy)?;
        }

        Ok(())
    }

    /// Get agent socket path from environment or default location
    pub fn get_agent_socket_path() -> Option<PathBuf> {
        if let Ok(socket_path) = std::env::var("MALAI_SSH_AGENT") {
            Some(PathBuf::from(socket_path))
        } else {
            None
        }
    }

    /// Check if lockdown mode is enabled
    pub fn is_lockdown_mode() -> bool {
        std::env::var("MALAI_LOCKDOWN_MODE")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_agent_creation() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().to_path_buf();
        let client_id52 = "test-client-id52".to_string();

        let agent = Agent::new(data_dir.clone(), client_id52);
        
        assert_eq!(agent.socket_path, data_dir.join("ssh").join("agent.sock"));
        assert_eq!(agent.client_id52, "test-client-id52");
        assert!(agent.connections.is_empty());
        assert!(agent.http_proxy_port.is_none());
    }

    #[test]
    fn test_connection_idle_detection() {
        let mut connection = Connection::new("test-server-id52".to_string());
        
        // Initially not idle
        assert!(!connection.is_idle(std::time::Duration::from_secs(60)));
        
        // Update last used time to simulate old connection
        connection.last_used = std::time::Instant::now() - std::time::Duration::from_secs(120);
        
        // Should be idle now
        assert!(connection.is_idle(std::time::Duration::from_secs(60)));
    }

    #[test]
    fn test_agent_utils_lockdown_detection() {
        // Test without environment variable
        assert!(!AgentUtils::is_lockdown_mode());
        
        // Test with environment variable set to true
        std::env::set_var("MALAI_LOCKDOWN_MODE", "true");
        assert!(AgentUtils::is_lockdown_mode());
        
        // Test with environment variable set to false
        std::env::set_var("MALAI_LOCKDOWN_MODE", "false");
        assert!(!AgentUtils::is_lockdown_mode());
        
        // Cleanup
        std::env::remove_var("MALAI_LOCKDOWN_MODE");
    }
}