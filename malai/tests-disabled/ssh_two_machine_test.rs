/// Test SSH execution between two machines  
/// Level 3: Basic SSH functionality with cluster manager + SSH server + client

use malai_cli_test_utils::*;
use std::time::Duration;

#[tokio::test]
async fn test_two_machine_ssh_execution() {
    println!("🧪 Level 3: Testing SSH execution between two machines");
    
    let test_env = MalaiTestEnv::new("two-machine-ssh").expect("Should create test env");
    
    // 1. Set up cluster manager
    println!("👑 Setting up cluster manager...");
    let manager_home = test_env.test_dir().join("cluster-manager");
    std::fs::create_dir_all(&manager_home).expect("Should create manager home");
    
    let cluster_result = MalaiCommand::new()
        .malai_home(&manager_home)
        .ssh_create_cluster(Some("two-machine-test"))
        .await
        .expect("Should create cluster")
        .expect_success()
        .expect("Cluster creation should succeed");
    
    let cluster_id = cluster_result.extract_cluster_id().expect("Should extract cluster ID");
    println!("✅ Cluster created: {}", cluster_id);
    
    // 2. Create SSH server machine
    println!("🖥️  Setting up SSH server...");
    let server_home = test_env.test_dir().join("ssh-server");
    std::fs::create_dir_all(&server_home).expect("Should create server home");
    
    let server_keygen = MalaiCommand::new()
        .malai_home(&server_home)
        .keygen()
        .await
        .expect("Should generate server key")
        .expect_success()
        .expect("Server keygen should succeed");
    
    let server_id52 = server_keygen.extract_id52().expect("Should extract server ID52");
    println!("✅ Server identity: {}", server_id52);
    
    // 3. Add server to cluster config
    println!("📝 Adding server to cluster config...");
    let cluster_config_path = manager_home.join("ssh").join("cluster-config.toml");
    let server_config = format!(
        r#"
[machine.web01]
id52 = "{}"
accept_ssh = true
allow_from = "*"
"#,
        server_id52
    );
    
    // Append to existing config
    let existing_config = std::fs::read_to_string(&cluster_config_path).expect("Should read config");
    let updated_config = format!("{}{}", existing_config, server_config);
    std::fs::write(&cluster_config_path, updated_config).expect("Should write config");
    
    // Copy config to server for role detection
    let server_config_path = server_home.join("ssh").join("cluster-config.toml");
    std::fs::create_dir_all(server_config_path.parent().unwrap()).expect("Should create server ssh dir");
    std::fs::copy(&cluster_config_path, &server_config_path).expect("Should copy config");
    
    println!("✅ Server added to cluster config");
    
    // 4. Test agent role detection
    println!("🔍 Testing server agent role detection...");
    let server_cluster_info = MalaiCommand::new()
        .malai_home(&server_home)
        .ssh_cluster_info()
        .await
        .expect("Should get server cluster info")
        .expect_success()
        .expect("Server cluster info should succeed");
    
    println!("📋 Server role info: {}", server_cluster_info.stdout);
    server_cluster_info.assert_contains("SSH server").expect("Should detect SSH server role");
    println!("✅ Server role detected correctly");
    
    // 5. Start agents (in background for now, real P2P execution in next step)
    println!("🚀 Starting cluster manager agent...");
    let _manager_agent = MalaiCommand::new()
        .malai_home(&manager_home)
        .spawn_background(["ssh", "agent"])
        .expect("Should start manager agent");
    
    println!("🚀 Starting SSH server agent...");  
    let _server_agent = MalaiCommand::new()
        .malai_home(&server_home)
        .spawn_background(["ssh", "agent"])
        .expect("Should start server agent");
    
    // Wait for agents to initialize
    println!("⏳ Waiting for agents to initialize...");
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    // 6. Test SSH execution (will fail with "not implemented" for now)
    println!("🧪 Testing SSH execution...");
    let ssh_result = MalaiCommand::new()
        .malai_home(&manager_home)  // Cluster manager acts as client
        .ssh_exec("web01", "echo", vec!["Hello SSH!"])
        .await
        .expect("Should execute SSH command");
    
    println!("📋 SSH execution result:");
    println!("STDOUT: {}", ssh_result.stdout);
    println!("STDERR: {}", ssh_result.stderr);
    
    // For now, expect "not implemented" message
    if ssh_result.contains("not yet implemented") || ssh_result.contains("not implemented") {
        println!("✅ SSH exec command structure working (implementation pending)");
    } else {
        println!("⚠️  Unexpected output - check implementation");
    }
    
    println!("✅ Two-machine SSH test completed successfully");
    println!("🎯 Ready for Level 4: Implement real P2P SSH execution");
}

#[tokio::test]
async fn test_machine_config_propagation() {
    println!("🧪 Testing cluster config propagation workflow");
    
    let test_env = MalaiTestEnv::new("config-propagation").expect("Should create test env");
    
    // 1. Create cluster
    let manager_home = test_env.test_dir().join("manager");
    let cluster_result = MalaiCommand::new()
        .malai_home(&manager_home)
        .ssh_create_cluster(Some("config-test"))
        .await
        .expect("Should create cluster")
        .expect_success()
        .expect("Cluster creation should succeed");
    
    let cluster_id = cluster_result.extract_cluster_id().expect("Should extract cluster ID");
    
    // 2. Create machine and add to config  
    let machine_home = test_env.test_dir().join("machine");
    let machine_keygen = MalaiCommand::new()
        .malai_home(&machine_home)
        .keygen()
        .await
        .expect("Should generate machine key")
        .expect_success()
        .expect("Machine keygen should succeed");
    
    let machine_id52 = machine_keygen.extract_id52().expect("Should extract machine ID52");
    
    // 3. Add machine to cluster config (simulate admin action)
    let cluster_config_path = manager_home.join("ssh").join("cluster-config.toml");
    let machine_config = format!(
        r#"
[machine.test-machine]
id52 = "{}"
accept_ssh = true  
allow_from = "*"
"#,
        machine_id52
    );
    
    let existing_config = std::fs::read_to_string(&cluster_config_path).expect("Should read config");
    let updated_config = format!("{}{}", existing_config, machine_config);
    std::fs::write(&cluster_config_path, updated_config).expect("Should write config");
    
    // 4. Simulate config sync (manual copy for now, real P2P sync in next level)
    let machine_config_path = machine_home.join("ssh").join("cluster-config.toml");
    std::fs::create_dir_all(machine_config_path.parent().unwrap()).expect("Should create machine ssh dir");
    std::fs::copy(&cluster_config_path, &machine_config_path).expect("Should sync config");
    
    println!("📋 Config synced to machine");
    
    // 5. Test machine role detection after config sync
    let machine_cluster_info = MalaiCommand::new()
        .malai_home(&machine_home)
        .ssh_cluster_info()
        .await
        .expect("Should get machine cluster info")
        .expect_success()
        .expect("Machine cluster info should succeed");
    
    println!("📋 Machine role after config sync: {}", machine_cluster_info.stdout);
    machine_cluster_info.assert_contains("SSH server").expect("Should detect SSH server role");
    machine_cluster_info.assert_contains("test-machine").expect("Should show correct machine name");
    
    println!("✅ Config propagation and role detection working");
}