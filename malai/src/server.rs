//! Main malai server - the core of everything
//!
//! This is the heart of malai: one P2P listener per identity that handles all protocols.
//! Clean, simple, readable.

use eyre::Result;
use futures_util::stream::StreamExt;

/// The main malai server - ONE LISTENER PER IDENTITY
pub async fn run_malai_server(identity: fastn_id52::SecretKey) -> Result<()> {
    let id52 = identity.id52();
    println!("🔥 Starting malai server for identity: {}", id52);
    
    // ALL protocols this device handles
    let protocols = vec![
        crate::core_utils::MalaiProtocol::ConfigUpdate,   // Receive config
        crate::core_utils::MalaiProtocol::ExecuteCommand, // Execute commands
    ];
    
    println!("📡 Listening on protocols: {:?}", protocols);
    
    // ONE listener for this identity - follows fundamental P2P rule
    let mut request_stream = fastn_p2p::listen!(identity.clone(), &protocols);
    
    println!("✅ Server started, waiting for requests...");
    
    // Main server loop - simple and clean
    loop {
        tokio::select! {
            request_result = request_stream.next() => {
                match request_result {
                    Some(Ok(request)) => {
                        println!("📨 Received: {}", request.protocol);
                        
                        // Route to appropriate handler
                        match request.protocol {
                            crate::core_utils::MalaiProtocol::ConfigUpdate => {
                                handle_config_reception(request).await;
                            }
                            crate::core_utils::MalaiProtocol::ExecuteCommand => {
                                handle_command_execution(request).await;
                            }
                        }
                    }
                    Some(Err(e)) => {
                        println!("❌ Request error: {}", e);
                    }
                    None => {
                        println!("📡 Server stream ended");
                        break;
                    }
                }
            }
            _ = fastn_p2p::cancelled() => {
                println!("🛑 Server {} stopping gracefully", id52);
                break;
            }
        }
    }
    
    Ok(())
}

/// Handle incoming config (simple)
async fn handle_config_reception(request: fastn_p2p::Request<crate::core_utils::MalaiProtocol>) {
    if let Err(e) = request.handle(|config: crate::core_utils::ConfigSyncRequest| async move {
        println!("📥 Config from: {}", config.sender_id52);
        
        // Save config (TODO: implement proper saving)
        println!("💾 Config saved (TODO: implement)");
        
        Result::<crate::core_utils::ConfigSyncResponse, crate::core_utils::ConfigSyncError>::Ok(
            crate::core_utils::ConfigSyncResponse {
                success: true,
                message: "Config received".to_string(),
            }
        )
    }).await {
        println!("❌ Config handling failed: {}", e);
    }
}

/// Handle command execution (simple)
async fn handle_command_execution(request: fastn_p2p::Request<crate::core_utils::MalaiProtocol>) {
    if let Err(e) = request.handle(|cmd: crate::core_utils::RemoteAccessRequest| async move {
        println!("📥 Command from: {}", cmd.client_id52);
        println!("💻 Running: {} {:?}", cmd.command, cmd.args);
        
        // Execute command (TODO: implement ACL + real execution)
        let output = format!("Executed: {} {:?}", cmd.command, cmd.args);
        
        Result::<crate::core_utils::RemoteAccessResponse, crate::core_utils::RemoteAccessError>::Ok(
            crate::core_utils::RemoteAccessResponse {
                stdout: output.into_bytes(),
                stderr: Vec::new(),
                exit_code: 0,
                execution_time_ms: 0,
            }
        )
    }).await {
        println!("❌ Command handling failed: {}", e);
    }
}