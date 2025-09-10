//! Config management utilities - clean and simple

use eyre::Result;
use std::str::FromStr;

/// Validate config file syntax
pub fn validate_config_file(config_path: &str) -> Result<()> {
    println!("🔍 Validating config: {}", config_path);
    
    if !std::path::Path::new(config_path).exists() {
        return Err(eyre::eyre!("Config file not found: {}", config_path));
    }
    
    // Read and parse TOML
    let config_content = std::fs::read_to_string(config_path)?;
    let _parsed: toml::Value = toml::from_str(&config_content)
        .map_err(|e| eyre::eyre!("TOML syntax error: {}", e))?;
    
    println!("✅ Config syntax valid");
    
    // Basic validation checks
    if config_content.contains("[cluster_manager]") {
        println!("✅ Contains cluster manager section");
    }
    
    if config_content.contains("[machine.") {
        let machine_count = config_content.lines()
            .filter(|line| line.trim().starts_with("[machine.") && !line.trim().starts_with('#'))
            .count();
        println!("✅ Contains {} machine sections", machine_count);
    }
    
    Ok(())
}

/// Check all configs in MALAI_HOME
pub async fn check_all_configs() -> Result<()> {
    println!("🔍 Checking all configurations in MALAI_HOME...");
    
    let malai_home = crate::core_utils::get_malai_home();
    println!("📁 MALAI_HOME: {}", malai_home.display());
    
    let clusters_dir = malai_home.join("clusters");
    if !clusters_dir.exists() {
        println!("❌ No clusters directory found");
        return Ok(());
    }
    
    let mut total_configs = 0;
    let mut valid_configs = 0;
    
    // Check each cluster directory
    if let Ok(entries) = std::fs::read_dir(&clusters_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let cluster_alias = entry.file_name().to_string_lossy().to_string();
                let cluster_dir = entry.path();
                
                println!("\n📋 Cluster: {}", cluster_alias);
                
                // Check cluster-config.toml
                let cluster_config = cluster_dir.join("cluster-config.toml");
                if cluster_config.exists() {
                    total_configs += 1;
                    match validate_config_file(cluster_config.to_str().unwrap()) {
                        Ok(_) => {
                            println!("   ✅ cluster-config.toml valid");
                            valid_configs += 1;
                        }
                        Err(e) => {
                            println!("   ❌ cluster-config.toml invalid: {}", e);
                        }
                    }
                }
                
                // Check machine-config.toml
                let machine_config = cluster_dir.join("machine-config.toml");
                if machine_config.exists() {
                    total_configs += 1;
                    match validate_config_file(machine_config.to_str().unwrap()) {
                        Ok(_) => {
                            println!("   ✅ machine-config.toml valid");
                            valid_configs += 1;
                        }
                        Err(e) => {
                            println!("   ❌ machine-config.toml invalid: {}", e);
                        }
                    }
                }
                
                // Check identity files
                let identity_file = cluster_dir.join("identity.key");
                if identity_file.exists() {
                    match std::fs::read_to_string(&identity_file) {
                        Ok(key_content) => {
                            match fastn_id52::SecretKey::from_str(key_content.trim()) {
                                Ok(_) => println!("   ✅ identity.key valid"),
                                Err(e) => println!("   ❌ identity.key invalid: {}", e),
                            }
                        }
                        Err(e) => {
                            println!("   ❌ identity.key read error: {}", e);
                        }
                    }
                }
            }
        }
    }
    
    println!("\n📊 Configuration Summary:");
    println!("   Total configs: {}", total_configs);
    println!("   Valid configs: {}", valid_configs);
    
    if valid_configs == total_configs {
        println!("✅ All configurations valid");
    } else {
        return Err(eyre::eyre!("Some configurations invalid"));
    }
    
    Ok(())
}

/// Trigger config reload on running daemon
pub async fn reload_daemon_config() -> Result<()> {
    println!("🔄 Triggering config reload on running daemon...");
    
    // TODO: Send reload signal to daemon via Unix socket
    println!("⚠️ Config reload not yet implemented");
    println!("💡 For now, restart daemon to reload config");
    
    Ok(())
}