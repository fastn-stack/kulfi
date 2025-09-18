//! expose_http: Production-grade HTTP over P2P using new fastn-p2p API
//!
//! Exposes a local HTTP server over P2P network for remote access.
//! This is a fresh implementation using the new fastn-p2p API.
//!
//! Usage: expose_http <host> <port>

use std::collections::HashMap;

// Protocol definition for HTTP proxying
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum HttpProtocol {
    Forward,
}

// HTTP request metadata sent during connection
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub version: String,
}

// Custom error type for HTTP operations
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <host> <port>", args[0]);
        std::process::exit(1);
    }
    
    let (host, port) = malai_next::utils::parse_host_port(&args[1..])?;
    let secret_key = malai_next::utils::get_or_generate_key()?;
    
    println!("🌐 Exposing HTTP server {}:{} over P2P", host, port);
    println!("🔑 Your ID52: {}", secret_key.id52());
    println!("🔗 Connect with: http_bridge {}", secret_key.id52());
    
    // Production-grade HTTP over P2P server
    fastn_p2p::listen(secret_key)
        .handle_streams(HttpProtocol::Forward, move |mut session, http_request: HttpRequest| {
            let host = host.clone();
            async move {
                tracing::info!(
                    "📡 HTTP request: {} {} from {}",
                    http_request.method, http_request.path,
                    session.peer().id52()
                );
                
                // Connect to local HTTP server
                let upstream_addr = format!("{host}:{port}");
                let upstream_stream = tokio::net::TcpStream::connect(&upstream_addr).await
                    .map_err(|e| HttpError::Connection(format!("Failed to connect to {upstream_addr}: {e}")))?;
                
                // Reconstruct HTTP request for local server
                let mut request_lines = vec![
                    format!("{} {} {}", http_request.method, http_request.path, http_request.version)
                ];
                
                for (key, value) in http_request.headers {
                    request_lines.push(format!("{key}: {value}"));
                }
                request_lines.push(String::new()); // Empty line before body
                
                let request_header = request_lines.join("\r\n");
                let (upstream_read, mut upstream_write) = upstream_stream.into_split();
                
                // Send HTTP request header to local server
                tokio::io::AsyncWriteExt::write_all(&mut upstream_write, request_header.as_bytes()).await?;
                
                // Use copy_both for efficient bidirectional HTTP body streaming
                match session.copy_both(upstream_read, upstream_write).await {
                    Ok((from_local, to_local)) => {
                        tracing::info!("✅ HTTP request completed: {to_local} sent to local, {from_local} received");
                    }
                    Err(e) => {
                        tracing::error!("❌ Proxy stream error: {e}");
                        return Err(HttpError::Io(e));
                    }
                }
                
                Ok(())
            }
        })
        .await?;
    
    Ok(())
}