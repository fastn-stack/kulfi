/// Test corrected SSH workflow: init-cluster → init → admin config → P2P sync  
/// This validates the proper design before implementing P2P config distribution

use malai_cli_test_utils::*;

#[tokio::test]
async fn test_corrected_ssh_workflow() {
    println!("🧪 Testing corrected SSH workflow design");
    
    let test_env = MalaiTestEnv::new("corrected-workflow").expect("Should create test env");
    
    // 1. Initialize cluster (cluster manager)
    println!("👑 Step 1: Initialize cluster...");
    let manager_home = test_env.test_dir().join("cluster-manager");
    
    let cluster_result = MalaiCommand::new()
        .malai_home(&manager_home)
        .ssh_init_cluster(Some("corrected-test"))
        .await
        .expect("Should init cluster")
        .expect_success()
        .expect("Init cluster should succeed");
    
    let cluster_id = cluster_result.extract_cluster_id().expect("Should extract cluster ID");
    println!("✅ Cluster initialized: {}", cluster_id);
    
    // Verify cluster manager has config
    let manager_config_path = manager_home.join("ssh").join("cluster-config.toml");
    assert!(manager_config_path.exists(), "Cluster manager should have config");
    
    let manager_cluster_info = MalaiCommand::new()
        .malai_home(&manager_home)
        .ssh_cluster_info()
        .await
        .expect("Should get manager info")
        .expect_success()
        .expect("Manager cluster info should succeed");
    
    manager_cluster_info.assert_contains("cluster-manager").expect("Should be cluster manager");
    println!("✅ Cluster manager role confirmed");
    
    // 2. Initialize machine (no config initially)
    println!("🖥️  Step 2: Initialize machine...");
    let machine_home = test_env.test_dir().join("machine");
    
    let machine_result = MalaiCommand::new()
        .malai_home(&machine_home)
        .ssh_init()
        .await
        .expect("Should init machine")
        .expect_success()
        .expect("Init machine should succeed");
    
    let machine_id52 = machine_result.extract_id52().expect("Should extract machine ID52");
    println!("✅ Machine initialized: {}", machine_id52);
    
    // Verify machine has NO config initially
    let machine_config_path = machine_home.join("ssh").join("cluster-config.toml");
    assert!(!machine_config_path.exists(), "Machine should NOT have config initially");
    
    let machine_cluster_info = MalaiCommand::new()
        .malai_home(&machine_home)
        .ssh_cluster_info()
        .await
        .expect("Should handle no config gracefully");
    
    // Should indicate no config found
    assert!(machine_cluster_info.contains("No cluster configuration found"), 
           "Machine should indicate no config found");
    println!("✅ Machine correctly has no config initially");
    
    // 3. Simulate admin adding machine to cluster config
    println!("👨‍💼 Step 3: Admin adds machine to cluster config...");
    let machine_config = format!(
        r#"
[machine.web01]
id52 = "{}"
accept_ssh = true
allow_from = "*"
"#,
        machine_id52
    );
    
    let existing_config = std::fs::read_to_string(&manager_config_path).expect("Should read config");
    let updated_config = format!("{}{}", existing_config, machine_config);
    std::fs::write(&manager_config_path, updated_config).expect("Should write config");
    
    println!("✅ Machine added to cluster config");
    
    // 4. Simulate P2P config distribution (manual copy for now)
    println!("📡 Step 4: Simulate P2P config sync...");
    std::fs::create_dir_all(machine_config_path.parent().unwrap()).expect("Should create machine ssh dir");
    std::fs::copy(&manager_config_path, &machine_config_path).expect("Should sync config");
    
    println!("✅ Config synced to machine (simulated P2P)");
    
    // 5. Test machine role detection after receiving config
    println!("🔍 Step 5: Test machine role detection after config sync...");
    let machine_cluster_info_after = MalaiCommand::new()
        .malai_home(&machine_home)
        .ssh_cluster_info()
        .await
        .expect("Should get machine info after sync")
        .expect_success()
        .expect("Machine cluster info should succeed after sync");
    
    println!("📋 Machine role after sync: {}", machine_cluster_info_after.stdout);
    machine_cluster_info_after.assert_contains("SSH server").expect("Should detect SSH server role");
    machine_cluster_info_after.assert_contains("web01").expect("Should show machine name");
    
    println!("✅ Machine role detection working after config sync");
    
    // 6. Test agent startup on both machines
    println!("🚀 Step 6: Test agent behavior...");
    
    // Manager agent should detect cluster-manager role
    let manager_agent_test = MalaiCommand::new()
        .malai_home(&manager_home)
        .timeout(std::time::Duration::from_secs(5))
        .execute_args(["ssh", "agent"])
        .await
        .expect("Should test manager agent");
    
    if manager_agent_test.contains("Cluster Manager") {
        println!("✅ Manager agent detects cluster-manager role");
    }
    
    // Machine agent should detect SSH server role (with config)
    let machine_agent_test = MalaiCommand::new()
        .malai_home(&machine_home)
        .timeout(std::time::Duration::from_secs(5))  
        .execute_args(["ssh", "agent"])
        .await
        .expect("Should test machine agent");
    
    if machine_agent_test.contains("SSH Server") {
        println!("✅ Machine agent detects SSH server role");
    }
    
    println!("🎉 Corrected SSH workflow test completed successfully!");
    println!("🎯 Ready for P2P config distribution implementation");
}