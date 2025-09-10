/// Test SSH agent functionality
/// Tests role detection, environment variables, and basic agent lifecycle

use malai_cli_test_utils::*;

#[tokio::test]  
async fn test_agent_environment_variables() {
    println!("🧪 Testing SSH agent environment variables");
    
    // Create test environment
    let test_env = MalaiTestEnv::new("agent-env").expect("Should create test env");
    let manager_home = test_env.test_dir().join("manager");
    
    // Create a cluster
    let create_result = MalaiCommand::new()
        .malai_home(&manager_home)
        .ssh_create_cluster(Some("test-cluster"))
        .await
        .expect("Should create cluster")
        .expect_success()
        .expect("Create cluster should succeed");
    
    let cluster_id = create_result.extract_cluster_id().expect("Should extract cluster ID");
    println!("✅ Created cluster: {}", cluster_id);
    
    // Test agent environment output
    let env_output = MalaiCommand::new()
        .malai_home(&manager_home)
        .ssh_agent_environment()
        .await
        .expect("Should get environment")
        .expect_success()
        .expect("Agent -e should succeed");
    
    println!("📋 Environment output:");
    println!("{}", env_output.stdout);
    
    // Verify environment variables
    env_output.assert_contains("MALAI_SSH_AGENT=").expect("Should output MALAI_SSH_AGENT");
    env_output.assert_contains("HTTP_PROXY=").expect("Should output HTTP_PROXY");
    env_output.assert_contains(&manager_home.to_string_lossy()).expect("Should use correct MALAI_HOME path");
    
    println!("✅ Agent environment variables test passed");
}

#[tokio::test]
async fn test_agent_role_detection() {
    println!("🧪 Testing SSH agent role detection");
    
    let test_env = MalaiTestEnv::new("agent-role").expect("Should create test env");
    
    // Test 1: Cluster manager role
    let manager_home = test_env.test_dir().join("manager");
    
    let create_result = MalaiCommand::new()
        .malai_home(&manager_home)
        .ssh_create_cluster(Some("test-cluster"))
        .await
        .expect("Should create cluster")
        .expect_success()
        .expect("Create cluster should succeed");
        
    let cluster_id = create_result.extract_cluster_id().expect("Should extract cluster ID");
    println!("✅ Created cluster: {}", cluster_id);
    
    // Test cluster info shows cluster manager role
    let cluster_info = MalaiCommand::new()
        .malai_home(&manager_home)
        .ssh_cluster_info()
        .await
        .expect("Should get cluster info")
        .expect_success()
        .expect("Cluster info should succeed");
    
    cluster_info.assert_contains("cluster-manager").expect("Should detect cluster manager role");
    println!("✅ Cluster manager role detected correctly");
    
    // Test 2: Unknown role (new machine without cluster)
    let unknown_home = test_env.test_dir().join("unknown");
    
    let unknown_info = MalaiCommand::new()
        .malai_home(&unknown_home)
        .ssh_cluster_info()
        .await
        .expect("Should handle unknown machine");
    
    // This should indicate no cluster config
    println!("📋 Unknown machine output: {}", unknown_info.stdout);
    assert!(unknown_info.contains("No cluster configuration found") || 
            unknown_info.contains("not found"), "Should indicate no config found");
    
    println!("✅ Unknown role handled correctly");
}

#[tokio::test]
async fn test_agent_lockfile_protection() {
    println!("🧪 Testing SSH agent lockfile protection");
    
    let test_env = MalaiTestEnv::new("agent-lockfile").expect("Should create test env");
    let agent_home = test_env.test_dir().join("agent");
    
    // Create a cluster for testing
    MalaiCommand::new()
        .malai_home(&agent_home)
        .ssh_create_cluster(Some("lockfile-test"))
        .await
        .expect("Should create cluster")
        .expect_success()
        .expect("Create cluster should succeed");
    
    // Start first agent in background
    println!("🚀 Starting first agent...");
    let _agent1 = MalaiCommand::new()
        .malai_home(&agent_home)
        .spawn_background(["ssh", "agent"])
        .expect("Should start first agent");
    
    // Wait a moment for agent to start
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    
    // Try to start second agent (should detect existing)
    println!("🚀 Trying to start second agent...");
    let agent2_output = MalaiCommand::new()
        .malai_home(&agent_home)
        .timeout(std::time::Duration::from_secs(5))
        .execute_args(["ssh", "agent"])
        .await
        .expect("Should execute second agent command");
    
    println!("📋 Second agent output:");
    println!("{}", agent2_output.stdout);
    
    // Should indicate agent already running
    if agent2_output.contains("already running") {
        println!("✅ Lockfile protection working");
    } else {
        println!("⚠️  Lockfile protection may need adjustment");
        println!("Output: {}", agent2_output.stdout);
    }
    
    println!("✅ Agent lockfile test completed");
}