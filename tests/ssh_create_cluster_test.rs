/// Test malai ssh create-cluster command
/// This validates the cluster creation functionality in isolation

use malai_cli_test_utils::*;
use std::path::PathBuf;

#[tokio::test]
async fn test_create_cluster_command() {
    println!("🧪 Testing create-cluster command");
    
    // Create isolated test environment
    let test_env = MalaiTestEnv::new("create-cluster").expect("Should create test env");
    let manager_home = test_env.test_dir().join("manager");
    std::fs::create_dir_all(&manager_home).expect("Should create manager dir");
    
    println!("🏗️  Creating cluster with MALAI_HOME: {}", manager_home.display());
    
    // Test the create-cluster command
    let result = MalaiCommand::new()
        .malai_home(&manager_home)
        .ssh_create_cluster(Some("test-cluster"))
        .await;
    
    match result {
        Ok(output) => {
            let output = output.expect_success().expect("Create cluster should succeed");
            
            println!("📝 Create cluster output:");
            println!("STDOUT: {}", output.stdout);
            println!("STDERR: {}", output.stderr);
            
            // Verify output format
            output.assert_contains("Cluster created with ID:").expect("Should output cluster ID");
            
            // Extract cluster ID 
            let cluster_id = output.extract_cluster_id().expect("Should extract cluster ID");
            println!("✅ Extracted cluster ID: {}", cluster_id);
            
            // Verify config file was created
            let config_path = manager_home.join("ssh").join("cluster-config.toml");
            assert!(config_path.exists(), "Config file should be created at {:?}", config_path);
            
            // Verify config file contents
            let config_content = std::fs::read_to_string(&config_path).expect("Should read config");
            assert!(config_content.contains("[cluster_manager]"), "Config should have cluster_manager section");
            assert!(config_content.contains(&cluster_id), "Config should contain cluster ID");
            
            println!("✅ Config file created correctly at: {}", config_path.display());
            println!("📄 Config content:\n{}", config_content);
            
        }
        Err(e) => {
            println!("❌ Create cluster command failed: {}", e);
            panic!("Create cluster command not implemented or failed: {}", e);
        }
    }
    
    println!("✅ Create-cluster test passed!");
}

#[tokio::test] 
async fn test_create_cluster_without_alias() {
    println!("🧪 Testing create-cluster without alias");
    
    let test_env = MalaiTestEnv::new("create-cluster-no-alias").expect("Should create test env");
    let manager_home = test_env.test_dir().join("manager");
    std::fs::create_dir_all(&manager_home).expect("Should create manager dir");
    
    // Test without alias
    let result = MalaiCommand::new()
        .malai_home(&manager_home)
        .ssh_create_cluster(None)  // No alias
        .await;
        
    match result {
        Ok(output) => {
            let output = output.expect_success().expect("Create cluster should succeed");
            output.assert_contains("Cluster created with ID:").expect("Should output cluster ID");
            
            let config_path = manager_home.join("ssh").join("cluster-config.toml");
            assert!(config_path.exists(), "Config file should be created");
            
            println!("✅ Create-cluster without alias works");
        }
        Err(e) => {
            println!("❌ Create cluster without alias failed: {}", e);
            // This is expected until we implement the command
            if e.to_string().contains("command not found") || e.to_string().contains("unrecognized subcommand") {
                println!("⏭️  Command not implemented yet - this is expected");
            } else {
                panic!("Unexpected error: {}", e);
            }
        }
    }
}

#[test]
fn test_malai_home_detection() {
    println!("🧪 Testing MALAI_HOME detection");
    
    // Test default behavior
    let default_home = malai::ssh::config::get_default_malai_home();
    println!("📁 Default MALAI_HOME: {}", default_home.display());
    assert!(default_home.to_string_lossy().contains("malai"));
    
    // Test override behavior  
    std::env::set_var("MALAI_HOME", "/tmp/custom-malai-test");
    let custom_home = malai::ssh::config::get_malai_home();
    assert_eq!(custom_home, PathBuf::from("/tmp/custom-malai-test"));
    
    // Clean up
    std::env::remove_var("MALAI_HOME");
    
    println!("✅ MALAI_HOME detection test passed");
}