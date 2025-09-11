//! Machine initialization with DNS support

use eyre::Result;

/// Initialize machine with DNS support for cluster manager resolution
pub async fn init_machine_with_dns_support(cluster_manager: String, cluster_alias: String) -> Result<()> {
    println!("🏗️  Initializing machine for cluster...");
    println!("🎯 Cluster: {} (alias: {})", cluster_manager, cluster_alias);
    
    // Resolve cluster manager ID52 (supports both domain and direct ID52)
    let cluster_manager_id52 = if cluster_manager.contains('.') {
        println!("🌐 Resolving cluster manager via DNS: {}", cluster_manager);
        match fastn_id52::PublicKey::resolve(&cluster_manager, "malai").await {
            Ok(public_key) => {
                println!("✅ DNS resolution successful");
                public_key.id52()
            }
            Err(e) => {
                println!("❌ DNS resolution failed: {}", e);
                println!("💡 Ensure domain has TXT record: {} TXT \"malai=<cluster-manager-id52>\"", cluster_manager);
                return Err(eyre::eyre!("DNS resolution failed for {}: {}", cluster_manager, e));
            }
        }
    } else {
        // Direct ID52
        println!("🆔 Using direct cluster manager ID52");
        cluster_manager.clone()
    };
    
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
    
    Ok(())
}

/// Resolve cluster manager ID52 from domain or direct ID52 (using fastn-id52 DNS)
pub async fn resolve_cluster_manager_id52(cluster_identifier: &str) -> Result<String> {
    if cluster_identifier.contains('.') {
        // Domain name - use fastn-id52 DNS resolution
        let public_key = fastn_id52::PublicKey::resolve(cluster_identifier, "malai").await?;
        Ok(public_key.id52())
    } else {
        // Direct ID52
        Ok(cluster_identifier.to_string())
    }
}