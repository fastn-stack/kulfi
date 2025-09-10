//! Direct CLI execution mode - works without daemon dependency

use eyre::Result;
use std::str::FromStr;

/// Execute command in direct CLI mode (MVP primary mode)
pub async fn execute_direct_command(machine_address: &str, command: &str, args: Vec<String>) -> Result<()> {
    println!("🎯 Direct CLI mode: {}", machine_address);
    
    // Parse machine.cluster format
    let parts: Vec<&str> = machine_address.split('.').collect();
    if parts.len() != 2 {
        return Err(eyre::eyre!("Invalid format. Use: machine.cluster"));
    }
    
    let machine_alias = parts[0];
    let cluster_alias = parts[1];
    
    // Read MALAI_HOME directly (no daemon dependency)
    let malai_home = if let Ok(home) = std::env::var("MALAI_HOME") {
        std::path::PathBuf::from(home)
    } else {
        dirs::data_dir().unwrap_or_default().join("malai")
    };
    
    println!("📁 Reading from: {}", malai_home.display());
    
    // Find our identity for this cluster (auto-select machine)
    let cluster_dir = malai_home.join("clusters").join(cluster_alias);
    if !cluster_dir.exists() {
        return Err(eyre::eyre!("Cluster {} not found in MALAI_HOME", cluster_alias));
    }
    
    // Auto-select local machine identity (design: pick the one machine in cluster)
    let cluster_private_key = cluster_dir.join("cluster.private-key");
    let machine_private_key = cluster_dir.join("machine.private-key");
    
    let our_identity = if cluster_private_key.exists() && !machine_private_key.exists() {
        // Cluster manager mode
        println!("🔑 Using cluster manager identity");
        let key_content = std::fs::read_to_string(&cluster_private_key)?;
        fastn_id52::SecretKey::from_str(key_content.trim())?
    } else if machine_private_key.exists() && !cluster_private_key.exists() {
        // Machine mode  
        println!("🔑 Using machine identity");
        let key_content = std::fs::read_to_string(&machine_private_key)?;
        fastn_id52::SecretKey::from_str(key_content.trim())?
    } else if cluster_private_key.exists() && machine_private_key.exists() {
        return Err(eyre::eyre!("Configuration error: Both cluster.private-key and machine.private-key exist in {}", cluster_dir.display()));
    } else {
        return Err(eyre::eyre!("No identity found for cluster: {}", cluster_alias));
    };
    
    // Read cluster config to find target machine
    let cluster_config_path = cluster_dir.join("cluster.toml");
    let machine_config_path = cluster_dir.join("machine.toml");
    
    let config_content = if cluster_config_path.exists() {
        // Read from cluster.toml (cluster manager)
        std::fs::read_to_string(&cluster_config_path)?
    } else if machine_config_path.exists() {
        // Read from machine.toml (machine)
        std::fs::read_to_string(&machine_config_path)?
    } else {
        return Err(eyre::eyre!("No config found for cluster: {}", cluster_alias));
    };
    
    let config: toml::Value = toml::from_str(&config_content)?;
    
    // Find target machine ID52
    let target_id52 = if let Some(machine_section) = config.get("machine") {
        if let Some(machines) = machine_section.as_table() {
            if let Some(target_machine) = machines.get(machine_alias) {
                if let Some(target_table) = target_machine.as_table() {
                    if let Some(id52_value) = target_table.get("id52") {
                        id52_value.as_str().unwrap_or("").to_string()
                    } else {
                        return Err(eyre::eyre!("No id52 for machine: {}", machine_alias));
                    }
                } else {
                    return Err(eyre::eyre!("Invalid machine config"));
                }
            } else {
                return Err(eyre::eyre!("Machine {} not found in cluster {}", machine_alias, cluster_alias));
            }
        } else {
            return Err(eyre::eyre!("No machines table in config"));
        }
    } else {
        return Err(eyre::eyre!("No machine section in config"));
    };
    
    println!("🎯 Target machine: {}", target_id52);
    
    // Self-command optimization (design requirement)
    if target_id52 == our_identity.id52() {
        println!("🔄 Self-command detected - executing locally");
        
        // Execute locally without P2P
        use tokio::process::Command;
        match Command::new(command).args(&args).output().await {
            Ok(output) => {
                print!("{}", String::from_utf8_lossy(&output.stdout));
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
                
                if output.status.success() {
                    println!("✅ Self-command completed");
                } else {
                    println!("❌ Command failed with exit code: {}", output.status.code().unwrap_or(-1));
                }
                Ok(())
            }
            Err(e) => {
                Err(eyre::eyre!("Failed to execute command: {}", e))
            }
        }
    } else {
        // Remote execution via fresh P2P connection
        println!("📡 Remote execution - creating fresh P2P connection");
        crate::malai_server::send_command(our_identity, &target_id52, command, args).await
    }
}