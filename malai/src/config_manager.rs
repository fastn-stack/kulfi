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
                
                // Check cluster.toml
                let cluster_config = cluster_dir.join("cluster.toml");
                if cluster_config.exists() {
                    total_configs += 1;
                    match validate_config_file(cluster_config.to_str().unwrap()) {
                        Ok(_) => {
                            println!("   ✅ cluster.toml valid");
                            valid_configs += 1;
                        }
                        Err(e) => {
                            println!("   ❌ cluster.toml invalid: {}", e);
                        }
                    }
                }
                
                // Check machine.toml
                let machine_config = cluster_dir.join("machine.toml");
                if machine_config.exists() {
                    total_configs += 1;
                    match validate_config_file(machine_config.to_str().unwrap()) {
                        Ok(_) => {
                            println!("   ✅ machine.toml valid");
                            valid_configs += 1;
                        }
                        Err(e) => {
                            println!("   ❌ machine.toml invalid: {}", e);
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

/// Check configuration for specific cluster
pub async fn check_cluster_config(cluster_name: &str) -> Result<()> {
    println!("🔍 Checking configuration for cluster: {}", cluster_name);
    
    let malai_home = crate::core_utils::get_malai_home();
    let cluster_dir = malai_home.join("clusters").join(cluster_name);
    
    if !cluster_dir.exists() {
        return Err(eyre::eyre!("Cluster '{}' not found in {}", cluster_name, cluster_dir.display()));
    }
    
    // Check cluster config
    let cluster_config = cluster_dir.join("cluster.toml");
    if cluster_config.exists() {
        validate_config_file(&cluster_config.to_string_lossy())?;
        println!("✅ {}/cluster.toml valid", cluster_name);
    }
    
    // Check machine config if exists
    let machine_config = cluster_dir.join("machine.toml");
    if machine_config.exists() {
        validate_config_file(&machine_config.to_string_lossy())?;
        println!("✅ {}/machine.toml valid", cluster_name);
    }
    
    println!("✅ Cluster '{}' configuration valid", cluster_name);
    Ok(())
}

/// Trigger selective config reload on running daemon
pub async fn reload_daemon_config_selective(cluster_name: String) -> Result<()> {
    println!("🔄 Triggering selective config reload for cluster: {}", cluster_name);
    
    let malai_home = crate::core_utils::get_malai_home();
    let socket_path = malai_home.join("malai.socket");
    
    if !socket_path.exists() {
        println!("❌ Daemon not running (no Unix socket found)");
        println!("💡 Start daemon with: malai daemon");
        return Ok(());
    }
    
    // TODO: Send selective reload signal to daemon via Unix socket
    println!("⚠️ Selective config reload not yet implemented");
    println!("💡 For now, restart daemon to reload config");
    
    Ok(())
}

/// Trigger config reload on running daemon
pub async fn reload_daemon_config() -> Result<()> {
    println!("🔄 Triggering config reload on running daemon...");
    
    let malai_home = crate::core_utils::get_malai_home();
    let socket_path = malai_home.join("malai.socket");
    
    if !socket_path.exists() {
        println!("❌ Daemon not running (no Unix socket found)");
        println!("💡 Start daemon with: malai daemon");
        return Ok(());
    }
    
    // TODO: Send reload signal to daemon via Unix socket
    println!("⚠️ Config reload not yet implemented");
    println!("💡 For now, restart daemon to reload config");
    
    Ok(())
}
/// Role detection for cluster directory
#[derive(Debug, Clone, PartialEq)]
pub enum ClusterRole {
    ClusterManager,  // cluster.toml exists, machine.toml missing
    Machine,         // machine.toml exists, cluster.toml missing  
    Waiting,         // neither file exists
}

/// Detect role for cluster directory (with error checking)
pub fn detect_cluster_role(cluster_dir: &std::path::Path) -> Result<ClusterRole> {
    let cluster_config = cluster_dir.join("cluster.toml");
    let machine_config = cluster_dir.join("machine.toml");
    
    let has_cluster = cluster_config.exists();
    let has_machine = machine_config.exists();
    
    match (has_cluster, has_machine) {
        (true, true) => {
            Err(eyre::eyre!(
                "CONFIGURATION ERROR: Both cluster.toml and machine.toml exist in {}\n\
                 This is not supported. Each cluster directory must have exactly one config:\n\
                 - cluster.toml: For cluster manager role\n\
                 - machine.toml: For machine role\n\
                 Remove one of the files to fix this error.",
                cluster_dir.display()
            ))
        }
        (true, false) => {
            println!("   👑 Cluster Manager role detected");
            Ok(ClusterRole::ClusterManager)
        }
        (false, true) => {
            println!("   🖥️  Machine role detected");
            Ok(ClusterRole::Machine)
        }
        (false, false) => {
            println!("   📋 Waiting for configuration");
            Ok(ClusterRole::Waiting)
        }
    }
}

/// Scan all clusters and detect roles (with validation)
pub async fn scan_cluster_roles() -> Result<Vec<(String, fastn_id52::SecretKey, ClusterRole)>> {
    let malai_home = crate::core_utils::get_malai_home();
    let clusters_dir = malai_home.join("clusters");
    
    if !clusters_dir.exists() {
        println!("📂 No clusters directory");
        return Ok(Vec::new());
    }
    
    let mut cluster_identities = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&clusters_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let cluster_alias = entry.file_name().to_string_lossy().to_string();
                let cluster_dir = entry.path();
                
                println!("\n📋 Scanning cluster: {}", cluster_alias);
                
                // Detect role (will crash if both configs exist)
                let role = detect_cluster_role(&cluster_dir)?;
                
                // Load identity based on role (design-compliant)
                let identity_path = match role {
                    ClusterRole::ClusterManager => cluster_dir.join("cluster.private-key"),
                    ClusterRole::Machine => cluster_dir.join("machine.private-key"),
                    ClusterRole::Waiting => cluster_dir.join("identity.key"), // Generic for waiting
                };
                
                if identity_path.exists() {
                    let key_content = std::fs::read_to_string(&identity_path)?;
                    let identity = fastn_id52::SecretKey::from_str(key_content.trim())?;
                    
                    println!("   🔑 Identity: {}", identity.id52());
                    cluster_identities.push((cluster_alias, identity, role));
                } else {
                    println!("   ❌ No private key found for role: {:?}", role);
                }
            }
        }
    }
    
    Ok(cluster_identities)
}
