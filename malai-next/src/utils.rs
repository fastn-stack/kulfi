//! Shared utilities for malai-next

use eyre::Result;
use std::path::PathBuf;

/// Parse CLI arguments for expose_http/http_bridge pattern
pub fn parse_host_port(args: &[String]) -> Result<(String, u16)> {
    if args.len() < 2 {
        eyre::bail!("Usage: <host> <port>");
    }
    
    let host = args[0].clone();
    let port: u16 = args[1].parse()
        .map_err(|_| eyre::eyre!("Invalid port: {}", args[1]))?;
    
    Ok((host, port))
}

/// Get or generate a secret key (simplified for now)
pub fn get_or_generate_key() -> Result<fastn_p2p::SecretKey> {
    // For now, generate a new key each time
    // TODO: Implement proper key storage/retrieval
    Ok(fastn_p2p::SecretKey::generate())
}