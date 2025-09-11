//! Machine initialization with security-first design

use eyre::Result;

/// Initialize machine for cluster (direct ID52 only, no DNS for security)
pub async fn init_machine_for_cluster(cluster_manager: String, cluster_alias: String) -> Result<()> {
    println!("🏗️  Initializing machine for cluster...");
    println!("🎯 Cluster: {} (alias: {})", cluster_manager, cluster_alias);
    
    // Use cluster manager ID52 directly (security-first approach)
    if cluster_manager.contains('.') {
        return Err(eyre::eyre!("Domain names not supported for security reasons. Use cluster manager ID52 directly."));
    }
    
    println!("🆔 Using cluster manager ID52: {}", cluster_manager);
    let cluster_manager_id52 = cluster_manager.clone();
    
    println!("📝 Cluster manager ID52: {}", cluster_manager_id52);
    
    // Get MALAI_HOME  
    let malai_home = if let Ok(home) = std::env::var("MALAI_HOME") {
        std::path::PathBuf::from(home)
    } else {
        dirs::data_dir().unwrap_or_default().join("malai")
    };
    
    // Generate machine identity
    let machine_secret = fastn_id52::SecretKey::generate();
    let machine_id52 = machine_secret.id52();
    
    println!("🔑 Generated machine identity: {}", machine_id52);
    
    // Create cluster directory and save identity
    let cluster_dir = malai_home.join("clusters").join(&cluster_alias);
    std::fs::create_dir_all(&cluster_dir)?;
    
    // Save machine private key (design-compliant)
    let machine_key_path = cluster_dir.join("machine.private-key");
    std::fs::write(&machine_key_path, machine_secret.to_string())?;
    
    // Save cluster info for future reference
    let cluster_info = format!(
        r#"# Cluster registration information
cluster_alias = "{}"
cluster_manager_id52 = "{}"
machine_id52 = "{}"
domain = "{}"
"#,
        cluster_alias, 
        cluster_manager_id52, 
        machine_id52,
        if cluster_manager.contains('.') { cluster_manager.clone() } else { "".to_string() }
    );
    
    std::fs::write(cluster_dir.join("cluster-info.toml"), cluster_info)?;
    
    println!("✅ Machine initialized successfully");
    println!("Machine created with ID: {}", machine_id52);
    println!("📋 Next steps:");
    println!("1. Cluster admin should add this machine to cluster config:");
    println!("   [machine.{}]", cluster_alias);
    println!("   id52 = \"{}\"", machine_id52);
    println!("   allow_from = \"*\"");
    println!("2. Start daemon to accept commands: malai daemon");
    
    // Trigger selective rescan for this cluster if daemon running
    println!("🔄 Checking for running daemon to rescan cluster...");
    match crate::config_manager::reload_daemon_config_selective(cluster_alias.clone()).await {
        Ok(_) => {
            println!("✅ Daemon notified of new machine");
        }
        Err(e) if e.to_string().contains("no Unix socket found") => {
            // This is expected - daemon not running during init is normal
            println!("💡 Daemon not running - run 'malai rescan {}' after admin adds this machine", cluster_alias);
        }
        Err(e) => {
            // Real daemon communication failure - this should fail loudly
            return Err(eyre::eyre!("Failed to notify daemon of new machine: {}", e));
        }
    }
    
    Ok(())
}

