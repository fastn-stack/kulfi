/// Daemon startup and management

use eyre::Result;

/// Start malai daemon with file locking and service orchestration
pub async fn start_malai_daemon(environment: bool, foreground: bool) -> Result<()> {
    if environment {
        // Print environment variables for shell integration
        let malai_home = crate::core::get_malai_home();
        println!("MALAI_HOME={}", malai_home.display());
        println!("MALAI_DAEMON_SOCK={}", malai_home.join("malai.sock").display());
        return Ok(());
    }
    
    let malai_home = crate::core::get_malai_home();
    println!("🚀 Starting malai daemon...");
    println!("📁 MALAI_HOME: {}", malai_home.display());
    
    // Acquire exclusive lock (following fastn-rig pattern)
    let lock_path = malai_home.join("malai.lock");
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)?;
    
    match lock_file.try_lock() {
        Ok(()) => {
            println!("🔒 Lock acquired: {}", lock_path.display());
        }
        Err(_) => {
            println!("❌ Another malai daemon already running at {}", malai_home.display());
            return Ok(());
        }
    }
    
    let _lock_guard = lock_file; // Hold lock for daemon lifetime
    
    // Daemonize unless in foreground mode
    if !foreground {
        println!("🔄 Daemonizing (use --foreground to stay in terminal)...");
        // TODO: Implement actual fork/daemonize  
        println!("📋 For now running in foreground (daemonization not yet implemented)");
    } else {
        println!("📋 Running in foreground mode");
    }
    
    // Load and validate ALL configs before starting services
    let validated_configs = crate::core::daemon::load_and_validate_all_configs(&malai_home).await?;
    println!("✅ All configurations validated successfully");
    
    // Start services based on validated configs
    crate::core::daemon::start_services_from_configs(validated_configs).await?;
    
    println!("✅ malai daemon started");
    println!("💡 Use 'malai daemon -e' for environment variables");
    println!("📨 malai daemon running. Press Ctrl+C to stop.");
    
    // Wait for graceful shutdown using fastn-p2p global singleton
    fastn_p2p::cancelled().await;
    
    println!("👋 malai daemon stopped gracefully");
    Ok(())
}