//! Real malai server - clean and simple like fastn-rig
//!
//! One listener per identity, handles all malai protocols.
//! Based on the proven working simple_server.rs pattern.

use eyre::Result;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Real malai protocols  
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MalaiProtocol {
    ConfigUpdate,    // Cluster manager → machine config distribution
    ExecuteCommand,  // Any machine → any machine command execution
}

impl std::fmt::Display for MalaiProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Config update request
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigRequest {
    pub sender_id52: String,
    pub config_content: String,
    pub timestamp: String,
}

/// Config update response
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigResponse {
    pub success: bool,
    pub message: String,
}

/// Config update error
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigError {
    pub message: String,
}

/// Command execution request
#[derive(Serialize, Deserialize, Debug)]
pub struct CommandRequest {
    pub client_id52: String,
    pub command: String,
    pub args: Vec<String>,
}

/// Command execution response
#[derive(Serialize, Deserialize, Debug)]
pub struct CommandResponse {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: i32,
}

/// Command execution error
#[derive(Serialize, Deserialize, Debug)]
pub struct CommandError {
    pub error_type: String,
    pub message: String,
}

/// Real malai server - follows proven simple_server.rs pattern
pub async fn run_malai_server(identity: fastn_id52::SecretKey) -> Result<()> {
    let id52 = identity.id52();
    println!("🔥 malai server starting for: {}", id52);
    
    // All protocols this device handles (real malai functionality)
    let protocols = vec![
        MalaiProtocol::ConfigUpdate,
        MalaiProtocol::ExecuteCommand,
    ];
    
    println!("📡 Listening for: {:?}", protocols);
    
    // ONE listener per identity - proven working pattern
    let mut stream = fastn_p2p::listen!(identity, &protocols);
    
    println!("✅ malai server ready");
    
    // Main server loop - clean and simple
    while let Some(request_result) = stream.next().await {
        let request = request_result?;
        
        println!("📨 Received: {} from {}", request.protocol, request.peer().id52());
        
        // Protocol dispatch - clean and readable
        match request.protocol {
            MalaiProtocol::ConfigUpdate => {
                let _ = request.handle(|config_req: ConfigRequest| async move {
                    handle_config_update(config_req).await
                }).await;
            }
            MalaiProtocol::ExecuteCommand => {
                let _ = request.handle(|cmd_req: CommandRequest| async move {
                    handle_command_execution(cmd_req).await
                }).await;
            }
        }
    }
    
    Ok(())
}

/// Handle config update (real implementation)
async fn handle_config_update(config_req: ConfigRequest) -> Result<ConfigResponse, ConfigError> {
    println!("📥 Config from: {}", config_req.sender_id52);
    
    // Save config to machine-config.toml in current directory (simplified)
    let config_path = std::path::PathBuf::from("machine-config.toml");
    
    match std::fs::write(&config_path, &config_req.config_content) {
        Ok(_) => {
            println!("💾 Config saved to: {}", config_path.display());
            println!("🔄 Machine now accepts command execution");
            
            Ok(ConfigResponse {
                success: true,
                message: "Config received and saved successfully".to_string(),
            })
        }
        Err(e) => {
            Err(ConfigError {
                message: format!("Failed to save config: {}", e),
            })
        }
    }
}

/// Handle command execution (with basic ACL)
async fn handle_command_execution(cmd_req: CommandRequest) -> Result<CommandResponse, CommandError> {
    println!("💻 Command from: {}", cmd_req.client_id52);
    println!("🔧 Executing: {} {:?}", cmd_req.command, cmd_req.args);
    
    // Basic ACL check - must have config file  
    let config_path = std::path::PathBuf::from("machine-config.toml");
    if !config_path.exists() {
        println!("❌ No config found - rejecting command");
        return Err(CommandError {
            error_type: "no_config".to_string(),
            message: "Machine has no configuration".to_string(),
        });
    }
    
    println!("✅ Config found - executing command");
    
    // Execute real command
    use tokio::process::Command;
    
    match Command::new(&cmd_req.command).args(&cmd_req.args).output().await {
        Ok(output) => {
            println!("✅ Command executed: exit_code={}", output.status.code().unwrap_or(-1));
            Ok(CommandResponse {
                stdout: output.stdout,
                stderr: output.stderr,
                exit_code: output.status.code().unwrap_or(-1),
            })
        }
        Err(e) => {
            Err(CommandError {
                error_type: "execution_failed".to_string(),
                message: e.to_string(),
            })
        }
    }
}

/// Send config to machine (client function)
pub async fn send_config(
    sender_identity: fastn_id52::SecretKey,
    target_id52: &str,
    config_content: &str,
) -> Result<()> {
    let target_key = fastn_id52::PublicKey::from_str(target_id52)?;
    
    let request = ConfigRequest {
        sender_id52: sender_identity.id52(),
        config_content: config_content.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    match fastn_p2p::call::<MalaiProtocol, ConfigRequest, ConfigResponse, ConfigError>(
        sender_identity,
        &target_key,
        MalaiProtocol::ConfigUpdate,
        request,
    ).await {
        Ok(Ok(response)) => {
            println!("✅ Config sent: {}", response.message);
            Ok(())
        }
        Ok(Err(e)) => {
            Err(eyre::eyre!("Config error: {}", e.message))
        }
        Err(e) => {
            Err(eyre::eyre!("P2P failed: {}", e))
        }
    }
}

/// Send command to machine (client function)
pub async fn send_command(
    sender_identity: fastn_id52::SecretKey,
    target_id52: &str,
    command: &str,
    args: Vec<String>,
) -> Result<()> {
    let target_key = fastn_id52::PublicKey::from_str(target_id52)?;
    
    let request = CommandRequest {
        client_id52: sender_identity.id52(),
        command: command.to_string(),
        args,
    };
    
    match fastn_p2p::call::<MalaiProtocol, CommandRequest, CommandResponse, CommandError>(
        sender_identity,
        &target_key,
        MalaiProtocol::ExecuteCommand,
        request,
    ).await {
        Ok(Ok(response)) => {
            // Display command output
            print!("{}", String::from_utf8_lossy(&response.stdout));
            eprint!("{}", String::from_utf8_lossy(&response.stderr));
            println!("✅ Command completed: exit_code={}", response.exit_code);
            Ok(())
        }
        Ok(Err(e)) => {
            Err(eyre::eyre!("Command error: {}: {}", e.error_type, e.message))
        }
        Err(e) => {
            Err(eyre::eyre!("P2P failed: {}", e))
        }
    }
}