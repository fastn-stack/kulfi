/// Remote access command execution

use eyre::Result;

/// Send remote access command via P2P
pub async fn send_remote_access_command(machine_address: &str, command: &str, args: Vec<String>) -> Result<()> {
    println!("🧪 Executing remote command...");
    println!("📍 Target: {}", machine_address);
    println!("💻 Command: {} {:?}", command, args);
    
    // Parse machine address to get machine alias and cluster
    let parts: Vec<&str> = machine_address.split('.').collect();
    if parts.len() < 2 {
        return Err(eyre::eyre!("Invalid machine address format: {}", machine_address));
    }
    
    let machine_alias = parts[0];
    let cluster_alias = parts[1];
    
    // Find cluster config to get target machine ID52
    let malai_home = crate::core::get_malai_home();
    let cluster_dir = malai_home.join("clusters").join(cluster_alias);
    let cluster_config_path = cluster_dir.join("cluster-config.toml");
    
    if !cluster_config_path.exists() {
        return Err(eyre::eyre!("No cluster config found for: {}", cluster_alias));
    }
    
    // Parse cluster config to get target machine ID52  
    let config_content = std::fs::read_to_string(&cluster_config_path)?;
    let config: toml::Value = toml::from_str(&config_content)?;
    
    let target_machine_id52 = crate::core::daemon::find_machine_id52_in_config(&config, machine_alias)?;
    println!("🎯 Target machine ID52: {}", target_machine_id52);
    
    // Get local identity (cluster manager uses keys/identity.key)
    let identity_file = malai_home.join("keys").join("identity.key");
    if !identity_file.exists() {
        return Err(eyre::eyre!("No local identity found"));
    }
    
    let secret_key_hex = std::fs::read_to_string(&identity_file)?;
    let local_secret = fastn_id52::SecretKey::from_str(secret_key_hex.trim())?;
    
    // Convert target machine ID52 to public key
    let target_public_key = fastn_id52::PublicKey::from_str(&target_machine_id52)?;
    
    // Create remote access request
    let request = crate::core::daemon::RemoteAccessRequest {
        client_id52: local_secret.id52(),
        machine_alias: machine_alias.to_string(),
        command: command.to_string(),
        args: args.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    println!("📡 Sending remote access command via fastn_p2p::call...");
    
    // Send command via fastn_p2p (with timeout like fastn-rig)
    let call_result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        fastn_p2p::call::<crate::core::daemon::MalaiProtocol, crate::core::daemon::RemoteAccessRequest, crate::core::daemon::RemoteAccessResponse, crate::core::daemon::RemoteAccessError>(
            local_secret,
            &target_public_key,
            crate::core::daemon::MalaiProtocol::ExecuteCommand,
            request,
        ),
    ).await;
    
    match call_result {
        Ok(Ok(Ok(response))) => {
            // Display command output
            if !response.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&response.stdout));
            }
            if !response.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&response.stderr));
            }
            
            if response.exit_code == 0 {
                println!("✅ Remote command executed successfully ({}ms)", response.execution_time_ms);
            } else {
                println!("❌ Remote command failed with exit code: {}", response.exit_code);
            }
            
            Ok(())
        }
        Ok(Ok(Err(error))) => {
            Err(eyre::eyre!("Remote access error: {}", error))
        }
        Ok(Err(call_error)) => {
            Err(eyre::eyre!("P2P communication failed: {}", call_error))
        }
        Err(_timeout) => {
            Err(eyre::eyre!("Remote command timed out after 30 seconds"))
        }
    }
}