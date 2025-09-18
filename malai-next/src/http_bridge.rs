//! http_bridge: Production-grade HTTP bridge using new fastn-p2p API
//!
//! Creates a bridge that accepts HTTP requests and forwards them to a
//! remote HTTP server exposed via P2P using expose_http.
//!
//! Usage: http_bridge <target_id52> [local_port]

use std::collections::HashMap;

// Protocol definition for HTTP proxying (same as expose_http)
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

// Custom error type for HTTP bridge
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("P2P error: {0}")]
    P2P(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <target_id52> [local_port]", args[0]);
        std::process::exit(1);
    }
    
    let target_id52 = &args[1];
    let local_port: u16 = args.get(2)
        .unwrap_or(&"8080".to_string())
        .parse()
        .unwrap_or(8080);
    
    let secret_key = malai_next::utils::get_or_generate_key()?;
    let target_key: fastn_p2p::PublicKey = target_id52.parse()?;
    
    println!("🌉 Starting HTTP bridge on port {local_port}");
    println!("🔗 Forwarding to P2P target: {target_id52}");
    
    // Start local HTTP server
    let addr: std::net::SocketAddr = format!("127.0.0.1:{local_port}").parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    println!("🌐 Local HTTP server running on http://localhost:{local_port}");
    println!("🔄 All requests will be forwarded via P2P");
    
    // Accept HTTP connections and forward via P2P
    loop {
        match listener.accept().await {
            Ok((tcp_stream, _)) => {
                let target_key = target_key.clone();
                let secret_key = secret_key.clone();
                
                tokio::spawn(async move {
                    if let Err(e) = handle_http_request(tcp_stream, secret_key, target_key).await {
                        tracing::error!("HTTP request failed: {e}");
                    }
                });
            }
            Err(e) => {
                tracing::error!("Failed to accept connection: {e}");
            }
        }
    }
}

async fn handle_http_request(
    mut tcp_stream: tokio::net::TcpStream,
    secret_key: fastn_p2p::SecretKey,
    target_key: fastn_p2p::PublicKey,
) -> Result<(), BridgeError> {
    // Parse HTTP request from TCP stream
    let mut request_buffer = Vec::new();
    tokio::io::AsyncReadExt::read_to_end(&mut tcp_stream, &mut request_buffer).await?;
    
    if request_buffer.is_empty() {
        return Ok(());
    }
    
    // Parse HTTP request headers
    let request_str = String::from_utf8_lossy(&request_buffer);
    let lines: Vec<&str> = request_str.lines().collect();
    
    if lines.is_empty() {
        return Ok(());
    }
    
    // Parse request line
    let request_line = lines[0];
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 3 {
        return Ok(());
    }
    
    let method = parts[0].to_string();
    let path = parts[1].to_string();
    let version = parts[2].to_string();
    
    // Parse headers
    let mut headers = HashMap::new();
    for line in &lines[1..] {
        if line.is_empty() {
            break; // End of headers
        }
        if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_string(), value.to_string());
        }
    }
    
    let http_request = HttpRequest {
        method: method.clone(),
        path: path.clone(),
        headers,
        version,
    };
    
    tracing::info!("📤 Forwarding {method} {path} via P2P");
    
    // Connect to P2P target with HTTP request headers
    let mut session = fastn_p2p::client::connect(
        secret_key,
        target_key,
        HttpProtocol::Forward,
        http_request,
    ).await
    .map_err(|e| BridgeError::P2P(format!("Failed to connect: {e:?}")))?;
    
    // Use copy_both for bidirectional HTTP streaming
    let (tcp_read, tcp_write) = tcp_stream.into_split();
    match session.copy_both(tcp_read, tcp_write).await {
        Ok((sent, received)) => {
            tracing::info!("✅ HTTP bridge completed: {sent} sent, {received} received");
        }
        Err(e) => {
            tracing::error!("❌ Bridge stream error: {e}");
            return Err(BridgeError::Io(e));
        }
    }
    
    Ok(())
}