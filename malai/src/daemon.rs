//! Real malai daemon - MVP implementation
//!
//! Single daemon that manages all cluster identities and roles.
//! Clean, simple implementation using proven malai_server.rs patterns.

use eyre::Result;
use futures_util::stream::StreamExt;

/// Start the real malai daemon - MVP implementation
pub async fn start_real_daemon(foreground: bool) -> Result<()> {
    let malai_home = crate::core_utils::get_malai_home();
    
    // Production logging for cluster admins
    tracing::info!("Starting malai daemon - MALAI_HOME: {}", malai_home.display());
    println!("🔥 Starting malai daemon (MVP)");
    println!("📁 MALAI_HOME: {}", malai_home.display());
    
    // File locking (proven working pattern)
    let lock_path = malai_home.join("malai.lock");
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)?;
    
    match lock_file.try_lock() {
        Ok(()) => {
            tracing::info!("Daemon lock acquired successfully: {}", lock_path.display());
            println!("🔒 Lock acquired: {}", lock_path.display());
        }
        Err(_) => {
            tracing::warn!("Daemon startup failed: another instance already running at {}", malai_home.display());
            println!("❌ Another malai daemon already running at {}", malai_home.display());
            return Ok(());
        }
    }
    
    let _lock_guard = lock_file; // Hold lock for daemon lifetime
    
    // Daemonize if not foreground (TODO for later)
    if !foreground {
        println!("📋 Running in foreground mode (daemonization TODO)");
    }
    
    // Scan all cluster identities and roles
    let cluster_roles = crate::config_manager::scan_cluster_roles().await?;
    
    if cluster_roles.is_empty() {
        tracing::warn!("No clusters found in MALAI_HOME: {}", malai_home.display());
        println!("❌ No clusters found in MALAI_HOME");
        println!("💡 Initialize a cluster: malai cluster init <name>");
        return Ok(());
    }
    
    tracing::info!("Found {} cluster identities for daemon startup", cluster_roles.len());
    println!("✅ Found {} cluster identities", cluster_roles.len());
    
    // Start Unix socket listener for daemon-CLI communication (wait for it to be ready)
    let _socket_handle = crate::daemon_socket::start_daemon_socket_listener(malai_home.clone()).await?;
    
    // Start one P2P listener per identity
    for (cluster_alias, identity, role) in cluster_roles {
        let id52 = identity.id52();
        tracing::info!("Starting P2P listener for cluster: {} (role: {:?}, id52: {})", cluster_alias, role, id52);
        println!("🚀 Starting P2P listener for: {} ({:?})", cluster_alias, role);
        
        let cluster_alias_clone = cluster_alias.clone();
        let cluster_alias_log = cluster_alias.clone();
        fastn_p2p::spawn(async move {
            if let Err(e) = run_cluster_listener(cluster_alias_clone, identity, role).await {
                tracing::error!("Cluster listener failed for {}: {}", cluster_alias_log, e);
                println!("❌ Cluster listener failed for {}: {}", cluster_alias_log, e);
            }
        });
    }
    
    tracing::info!("malai daemon fully started - all cluster listeners active");
    println!("✅ malai daemon started - all cluster listeners active");
    println!("📨 Press Ctrl+C to stop gracefully");
    
    // Wait for graceful shutdown
    fastn_p2p::cancelled().await;
    tracing::info!("malai daemon shutting down gracefully");
    println!("👋 malai daemon stopped gracefully");
    
    Ok(())
}

/// Run P2P listener for one cluster identity - follows malai_server.rs pattern
async fn run_cluster_listener(
    cluster_alias: String,
    identity: fastn_id52::SecretKey,
    role: crate::config_manager::ClusterRole,
) -> Result<()> {
    let id52 = identity.id52();
    println!("🎧 Cluster listener starting: {} ({})", cluster_alias, id52);
    
    // All protocols this identity handles
    let protocols = vec![
        crate::malai_server::MalaiProtocol::ConfigUpdate,
        crate::malai_server::MalaiProtocol::ExecuteCommand,
    ];
    
    // ONE listener per identity - proven pattern
    let mut stream = fastn_p2p::listen!(identity.clone(), &protocols);
    
    println!("📡 {} listening for: {:?}", cluster_alias, protocols);
    
    // Main listener loop - clean and simple
    while let Some(request_result) = stream.next().await {
        let request = request_result?;
        
        println!("📨 {} received: {} from {}", 
                cluster_alias, request.protocol, request.peer().id52());
        
        // Route to appropriate handler based on role and protocol
        let cluster_alias_clone = cluster_alias.clone();
        let role_clone = role.clone();
        
        match request.protocol {
            crate::malai_server::MalaiProtocol::ConfigUpdate => {
                let _ = request.handle(|config_req: crate::malai_server::ConfigRequest| async move {
                    handle_config_for_cluster(config_req, cluster_alias_clone, role_clone).await
                }).await;
            }
            crate::malai_server::MalaiProtocol::ExecuteCommand => {
                let _ = request.handle(|cmd_req: crate::malai_server::CommandRequest| async move {
                    handle_command_for_cluster(cmd_req, cluster_alias_clone, role_clone).await
                }).await;
            }
        }
    }
    
    Ok(())
}

/// Handle config update for cluster (role-aware)
async fn handle_config_for_cluster(
    config_req: crate::malai_server::ConfigRequest,
    cluster_alias: String,
    role: crate::config_manager::ClusterRole,
) -> Result<crate::malai_server::ConfigResponse, crate::malai_server::ConfigError> {
    println!("📥 Config update for cluster: {}", cluster_alias);
    
    match role {
        crate::config_manager::ClusterRole::ClusterManager => {
            // Cluster managers don't receive config updates (they create them)
            Err(crate::malai_server::ConfigError {
                message: "Cluster managers don't receive config updates".to_string(),
            })
        }
        crate::config_manager::ClusterRole::Machine | crate::config_manager::ClusterRole::Waiting => {
            // Save received config to machine.toml
            let malai_home = crate::core_utils::get_malai_home();
            let machine_config_path = malai_home
                .join("clusters")
                .join(&cluster_alias)
                .join("machine.toml");
                
            match std::fs::write(&machine_config_path, &config_req.config_content) {
                Ok(_) => {
                    println!("💾 Saved machine config for: {}", cluster_alias);
                    Ok(crate::malai_server::ConfigResponse {
                        success: true,
                        message: format!("Config received for cluster {}", cluster_alias),
                    })
                }
                Err(e) => {
                    Err(crate::malai_server::ConfigError {
                        message: format!("Failed to save config: {}", e),
                    })
                }
            }
        }
    }
}

/// Handle command execution for cluster (role-aware with ACL)
async fn handle_command_for_cluster(
    cmd_req: crate::malai_server::CommandRequest,
    cluster_alias: String,
    role: crate::config_manager::ClusterRole,
) -> Result<crate::malai_server::CommandResponse, crate::malai_server::CommandError> {
    println!("💻 Command for cluster: {}", cluster_alias);
    println!("🔧 Command: {} {:?}", cmd_req.command, cmd_req.args);
    
    // Get config source based on role
    let config_content = match role {
        crate::config_manager::ClusterRole::ClusterManager => {
            // Read ACL from cluster.toml
            let malai_home = crate::core_utils::get_malai_home();
            let cluster_config_path = malai_home
                .join("clusters")
                .join(&cluster_alias)
                .join("cluster.toml");
            
            match std::fs::read_to_string(&cluster_config_path) {
                Ok(content) => content,
                Err(e) => {
                    return Err(crate::malai_server::CommandError {
                        error_type: "no_config".to_string(),
                        message: format!("No cluster.toml found: {}", e),
                    });
                }
            }
        }
        crate::config_manager::ClusterRole::Machine => {
            // Read ACL from machine.toml
            let malai_home = crate::core_utils::get_malai_home();
            let machine_config_path = malai_home
                .join("clusters")
                .join(&cluster_alias)
                .join("machine.toml");
            
            match std::fs::read_to_string(&machine_config_path) {
                Ok(content) => content,
                Err(e) => {
                    return Err(crate::malai_server::CommandError {
                        error_type: "no_config".to_string(),
                        message: format!("No machine.toml found: {}", e),
                    });
                }
            }
        }
        crate::config_manager::ClusterRole::Waiting => {
            return Err(crate::malai_server::CommandError {
                error_type: "waiting_config".to_string(),
                message: "Machine waiting for configuration".to_string(),
            });
        }
    };
    
    // Basic ACL validation (TODO: implement group expansion for MVP)
    if !validate_basic_acl(&config_content, &cmd_req.client_id52) {
        return Err(crate::malai_server::CommandError {
            error_type: "permission_denied".to_string(),
            message: "Access denied".to_string(),
        });
    }
    
    // Execute command
    execute_command_real(&cmd_req.command, &cmd_req.args).await
}

/// Basic ACL validation (MVP implementation)
fn validate_basic_acl(config_content: &str, client_id52: &str) -> bool {
    // Simple wildcard support for MVP
    config_content.contains("allow_from = \"*\"") || config_content.contains(client_id52)
}

/// Real command execution
async fn execute_command_real(command: &str, args: &[String]) -> Result<crate::malai_server::CommandResponse, crate::malai_server::CommandError> {
    use tokio::process::Command;
    
    match Command::new(command).args(args).output().await {
        Ok(output) => {
            Ok(crate::malai_server::CommandResponse {
                stdout: output.stdout,
                stderr: output.stderr,
                exit_code: output.status.code().unwrap_or(-1),
            })
        }
        Err(e) => {
            Err(crate::malai_server::CommandError {
                error_type: "execution_failed".to_string(),
                message: e.to_string(),
            })
        }

    }
}
