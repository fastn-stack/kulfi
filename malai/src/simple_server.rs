//! Simple, clean malai server - ONE LISTENER PER IDENTITY
//!
//! This is what malai is really about: a simple P2P server that handles requests.
//! No complexity, no multiple listeners, just the core functionality.

use eyre::Result;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Simple protocol - just like fastn-p2p-test
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MalaiProtocol {
    Echo,           // Simple test
    Config,         // Config updates
    Execute,        // Command execution
}

impl std::fmt::Display for MalaiProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EchoRequest {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EchoResponse {
    pub response: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EchoError {
    pub error: String,
}

/// The main malai server - simple and clean like fastn-rig
pub async fn run_simple_malai_server(identity: fastn_id52::SecretKey) -> Result<()> {
    let id52 = identity.id52();
    println!("🔥 Simple malai server starting for: {}", id52);
    
    // Simple protocols
    let protocols = vec![MalaiProtocol::Echo, MalaiProtocol::Config, MalaiProtocol::Execute];
    
    // ONE listener (following fundamental rule)
    let mut stream = fastn_p2p::listen!(identity, &protocols);
    println!("📡 Listening for requests...");
    
    // Simple server loop
    while let Some(request_result) = stream.next().await {
        let request = request_result?;
        
        println!("📨 Received: {} from {}", request.protocol, request.peer().id52());
        
        // Simple protocol dispatch
        match request.protocol {
            MalaiProtocol::Echo => {
                let _ = request.handle(|req: EchoRequest| async move {
                    println!("📨 Echo: {}", req.message);
                    Result::<EchoResponse, EchoError>::Ok(EchoResponse {
                        response: format!("Echo: {}", req.message),
                    })
                }).await;
            }
            MalaiProtocol::Config => {
                let _ = request.handle(|req: EchoRequest| async move {
                    println!("📥 Config: {}", req.message);
                    Result::<EchoResponse, EchoError>::Ok(EchoResponse {
                        response: "Config received".to_string(),
                    })
                }).await;
            }
            MalaiProtocol::Execute => {
                let _ = request.handle(|req: EchoRequest| async move {
                    println!("💻 Execute: {}", req.message);
                    Result::<EchoResponse, EchoError>::Ok(EchoResponse {
                        response: format!("Executed: {}", req.message),
                    })
                }).await;
            }
        }
    }
    
    Ok(())
}

/// Simple test function
pub async fn test_simple_server() -> Result<()> {
    println!("🧪 Testing simple server...");
    
    // Generate two identities
    let server_key = fastn_id52::SecretKey::generate();
    let client_key = fastn_id52::SecretKey::generate();
    
    let server_id52 = server_key.id52();
    let client_id52 = client_key.id52();
    
    println!("🔑 Server: {}", server_id52);
    println!("🔑 Client: {}", client_id52);
    
    // Start server
    fastn_p2p::spawn(async move {
        if let Err(e) = run_simple_malai_server(server_key).await {
            println!("❌ Server failed: {}", e);
        }
    });
    
    // Wait a moment for server to start
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    
    // Test client call
    let request = EchoRequest {
        message: "Hello from simple test".to_string(),
    };
    
    let server_public_key = fastn_id52::PublicKey::from_str(&server_id52)?;
    
    match fastn_p2p::call::<MalaiProtocol, EchoRequest, EchoResponse, EchoError>(
        client_key,
        &server_public_key,
        MalaiProtocol::Echo,
        request,
    ).await {
        Ok(Ok(response)) => {
            println!("✅ Response: {}", response.response);
        }
        Ok(Err(e)) => {
            println!("❌ Server error: {:?}", e);
        }
        Err(e) => {
            println!("❌ P2P error: {}", e);
        }
    }
    
    Ok(())
}