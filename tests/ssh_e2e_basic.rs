use malai_cli_test_utils::*;

/// Simple malai SSH end-to-end test using test utilities
/// Test: Create cluster, add machines, execute command
#[tokio::test]
async fn test_basic_ssh_cluster() {
    println!("🧪 Starting basic SSH cluster test");

    // 1. Set up complete SSH cluster environment
    println!("🏗️  Setting up SSH cluster environment...");
    let (mut env, cluster) = match ClusterTestHelper::setup_basic_ssh_cluster(
        "basic-ssh",
        "test-cluster"
    ).await {
        Ok(result) => {
            println!("✅ Cluster setup successful");
            result
        }
        Err(e) => {
            println!("❌ CRITICAL: Cluster setup failed: {}", e);
            println!("🔍 CI DEBUG: This will help identify SSH infrastructure issues");
            panic!("CRITICAL: SSH cluster setup failed: {}", e);
        }
    };

    println!("✅ Cluster setup complete:");
    println!("   Cluster ID: {}", cluster.cluster_id());
    println!("   Manager: {}", cluster.manager());  
    println!("   Server: {}", cluster.server());
    println!("   Client: {}", cluster.client());

    // Wait longer in CI environment
    let wait_time = if std::env::var("CI").is_ok() { 15 } else { 5 };
    println!("⏳ Waiting {}s for agents to initialize (CI needs more time)", wait_time);
    env.wait_for_agents(std::time::Duration::from_secs(wait_time)).await;

    // 2. Validate basic SSH functionality
    ClusterTestHelper::validate_basic_ssh(&env, &cluster)
        .await
        .expect("Basic SSH validation failed");

    // 3. Test specific SSH command execution  
    println!("🧪 Testing SSH command execution...");
    match SshTestHelper::test_ssh_execution(
        &env,
        cluster.client(),
        &format!("web01.{}", cluster.cluster_id()),
        "echo",
        vec!["Hello SSH from test!"],
        "Hello SSH from test!",
    ).await {
        Ok(()) => {
            println!("✅ CRITICAL: SSH command execution successful");
        }
        Err(e) => {
            println!("❌ CRITICAL: SSH command execution failed: {}", e);
            println!("🔍 CI DEBUG: This will help identify P2P communication issues");
            panic!("CRITICAL: SSH command execution failed: {}", e);
        }
    }

    // 4. Test cluster info command
    println!("📋 Testing cluster info...");
    match env.malai_cmd(cluster.manager())?
        .ssh_cluster_info()
        .await?
        .expect_success()
    {
        Ok(cluster_info) => {
            println!("🔍 DEBUG: Cluster info output: {}", cluster_info.stdout);
            cluster_info.assert_contains("cluster-manager")?;
            println!("✅ Cluster info validation successful");
        }
        Err(e) => {
            println!("❌ CRITICAL: Cluster info failed: {}", e);
            panic!("CRITICAL: Cluster info command failed: {}", e);
        }
    }

    // Clean up automatically via Drop implementations
    env.stop_all_agents().await.expect("Failed to stop agents");
    
    println!("✅ Basic SSH cluster test passed!");
}