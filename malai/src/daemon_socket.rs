//! Unix socket communication between daemon and CLI
//!
//! Implements simple protocol for daemon-CLI communication:
//! - CLI sends rescan commands to daemon via Unix socket
//! - Daemon responds with success/failure status
//! - Used for malai rescan and automatic init command triggers

use eyre::Result;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

/// Message types for daemon-CLI communication
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DaemonMessage {
    /// Request full rescan of all clusters
    RescanAll,
    /// Request selective rescan of specific cluster
    RescanCluster(String),
    /// Response: operation successful
    Success,
    /// Response: operation failed with error message
    Error(String),
}

/// Start Unix socket listener and return handle for background processing
pub async fn start_daemon_socket_listener(malai_home: PathBuf) -> Result<tokio::task::JoinHandle<()>> {
    let socket_path = malai_home.join("malai.socket");
    
    // Remove old socket file if exists
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }
    
    let listener = UnixListener::bind(&socket_path)?;
    println!("🔌 Daemon socket listener started: {}", socket_path.display());
    
    // Return handle to background task that processes connections
    let handle = tokio::spawn(async move {
        // Handle incoming connections
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    // Handle connection in background task
                    tokio::spawn(async move {
                        if let Err(e) = handle_socket_connection(stream).await {
                            println!("❌ Socket connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    println!("❌ Socket accept error: {}", e);
                    break;
                }
            }
        }
    });
    
    Ok(handle)
}

/// Handle single socket connection from CLI
async fn handle_socket_connection(mut stream: UnixStream) -> Result<()> {
    // Read message from CLI
    let mut buffer = vec![0u8; 1024];
    let n = stream.read(&mut buffer).await?;
    
    if n == 0 {
        return Ok(()); // Connection closed
    }
    
    // Parse message
    let message_str = String::from_utf8_lossy(&buffer[..n]);
    let message: DaemonMessage = serde_json::from_str(&message_str)?;
    
    println!("📨 Received daemon message: {:?}", message);
    
    // Process message and generate response
    let response = match message {
        DaemonMessage::RescanAll => {
            println!("🔄 Processing full rescan request...");
            // TODO: Implement actual rescan logic in daemon
            match perform_daemon_rescan(None).await {
                Ok(_) => {
                    println!("✅ Full rescan completed successfully");
                    DaemonMessage::Success
                }
                Err(e) => {
                    println!("❌ Full rescan failed: {}", e);
                    DaemonMessage::Error(e.to_string())
                }
            }
        }
        DaemonMessage::RescanCluster(cluster_name) => {
            println!("🔄 Processing selective rescan request for: {}", cluster_name);
            // TODO: Implement actual selective rescan logic in daemon
            match perform_daemon_rescan(Some(cluster_name.clone())).await {
                Ok(_) => {
                    println!("✅ Selective rescan completed for: {}", cluster_name);
                    DaemonMessage::Success
                }
                Err(e) => {
                    println!("❌ Selective rescan failed for {}: {}", cluster_name, e);
                    DaemonMessage::Error(e.to_string())
                }
            }
        }
        _ => DaemonMessage::Error("Invalid message type".to_string()),
    };
    
    // Send response back to CLI
    let response_str = serde_json::to_string(&response)?;
    stream.write_all(response_str.as_bytes()).await?;
    stream.flush().await?;
    
    Ok(())
}

/// Send rescan command to running daemon
pub async fn send_daemon_rescan_command(malai_home: PathBuf, cluster_name: Option<String>) -> Result<()> {
    let socket_path = malai_home.join("malai.socket");
    
    if !socket_path.exists() {
        return Err(eyre::eyre!("Daemon not running (no Unix socket found)"));
    }
    
    // Connect to daemon socket
    let mut stream = UnixStream::connect(&socket_path).await?;
    
    // Prepare message
    let message = match cluster_name {
        Some(cluster) => DaemonMessage::RescanCluster(cluster),
        None => DaemonMessage::RescanAll,
    };
    
    // Send message
    let message_str = serde_json::to_string(&message)?;
    stream.write_all(message_str.as_bytes()).await?;
    stream.flush().await?;
    
    // Read response
    let mut buffer = vec![0u8; 1024];
    let n = stream.read(&mut buffer).await?;
    
    if n == 0 {
        return Err(eyre::eyre!("No response from daemon"));
    }
    
    // Parse response
    let response_str = String::from_utf8_lossy(&buffer[..n]);
    let response: DaemonMessage = serde_json::from_str(&response_str)?;
    
    match response {
        DaemonMessage::Success => {
            println!("✅ Daemon rescan completed successfully");
            Ok(())
        }
        DaemonMessage::Error(error_msg) => {
            Err(eyre::eyre!("Daemon rescan failed: {}", error_msg))
        }
        _ => Err(eyre::eyre!("Invalid response from daemon")),
    }
}

/// Perform actual rescan in daemon (NOT IMPLEMENTED)
async fn perform_daemon_rescan(_cluster_name: Option<String>) -> Result<()> {
    // ❌ FAKE SUCCESS REMOVED: This function was printing success without doing anything!
    // This explains why E2E tests passed while P2P was broken.
    
    panic!(
        "DAEMON RESCAN NOT IMPLEMENTED: This function was returning Ok() and printing success \
        without actually rescanning anything! This is why E2E tests gave false confidence. \
        
        Need to implement:
        1. Stop old P2P listeners for affected clusters
        2. Re-read cluster configurations 
        3. Start new P2P listeners with updated configs
        4. Handle errors gracefully (continue with working clusters)
        
        Fix this before any rescan functionality will work."
    );
}