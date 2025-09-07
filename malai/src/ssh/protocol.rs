use serde::{Deserialize, Serialize};

/// SSH protocol for fastn-p2p communication
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SshProtocol {
    /// Execute a command on the server
    Execute,
    /// Start an interactive shell session
    Shell,
    /// HTTP service proxy request
    HttpProxy,
    /// Configuration synchronization
    ConfigSync,
}

impl std::fmt::Display for SshProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SshProtocol::Execute => write!(f, "ssh-execute"),
            SshProtocol::Shell => write!(f, "ssh-shell"),
            SshProtocol::HttpProxy => write!(f, "ssh-http-proxy"),
            SshProtocol::ConfigSync => write!(f, "ssh-config-sync"),
        }
    }
}

/// Request to execute a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteRequest {
    pub client_id52: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>, // Environment variables
    pub working_dir: Option<String>,
}

/// Response from command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: i32,
}

/// Error during command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteError {
    pub message: String,
    pub error_code: ExecuteErrorCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecuteErrorCode {
    PermissionDenied,
    CommandNotFound,
    ExecutionFailed,
    InvalidRequest,
}

/// Request to start interactive shell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellRequest {
    pub client_id52: String,
    pub terminal_size: Option<(u16, u16)>, // (width, height)
    pub env: Vec<(String, String)>,
}

/// Shell session response (for initial handshake)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellResponse {
    pub session_id: String,
    pub banner: Option<String>,
}

/// Error during shell session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellError {
    pub message: String,
    pub error_code: ShellErrorCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellErrorCode {
    PermissionDenied,
    SessionFailed,
    InvalidRequest,
}

/// HTTP proxy request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpProxyRequest {
    pub client_id52: String,
    pub service_name: String,
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// HTTP proxy response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpProxyResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// Error during HTTP proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpProxyError {
    pub message: String,
    pub error_code: HttpProxyErrorCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpProxyErrorCode {
    PermissionDenied,
    ServiceNotFound,
    ServiceUnavailable,
    InvalidRequest,
    ProxyError,
}

/// Configuration sync request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSyncRequest {
    pub member_id52: String,
    pub current_config_hash: Option<String>,
}

/// Configuration sync response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSyncResponse {
    pub config_hash: String,
    pub config_data: Option<String>, // TOML config, only sent if hash differs
    pub needs_update: bool,
}

/// Error during config sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSyncError {
    pub message: String,
    pub error_code: ConfigSyncErrorCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigSyncErrorCode {
    MemberNotFound,
    InvalidHash,
    SyncFailed,
}

impl std::fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.message, self.error_code)
    }
}

impl std::error::Error for ExecuteError {}

impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.message, self.error_code)
    }
}

impl std::error::Error for ShellError {}

impl std::fmt::Display for HttpProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.message, self.error_code)
    }
}

impl std::error::Error for HttpProxyError {}

impl std::fmt::Display for ConfigSyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.message, self.error_code)
    }
}

impl std::error::Error for ConfigSyncError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_serialization() {
        let protocol = SshProtocol::Execute;
        let json = serde_json::to_string(&protocol).unwrap();
        let deserialized: SshProtocol = serde_json::from_str(&json).unwrap();
        assert_eq!(protocol, deserialized);
    }

    #[test]
    fn test_execute_request_serialization() {
        let request = ExecuteRequest {
            client_id52: "test-client".to_string(),
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            env: vec![("PATH".to_string(), "/usr/bin".to_string())],
            working_dir: Some("/tmp".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ExecuteRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request.client_id52, deserialized.client_id52);
        assert_eq!(request.command, deserialized.command);
        assert_eq!(request.args, deserialized.args);
    }

    #[test]
    fn test_protocol_display() {
        assert_eq!(SshProtocol::Execute.to_string(), "ssh-execute");
        assert_eq!(SshProtocol::Shell.to_string(), "ssh-shell");
        assert_eq!(SshProtocol::HttpProxy.to_string(), "ssh-http-proxy");
        assert_eq!(SshProtocol::ConfigSync.to_string(), "ssh-config-sync");
    }
}