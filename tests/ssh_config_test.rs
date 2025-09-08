/// Test malai SSH config parsing and generation
/// This validates our TOML config structure works correctly

use malai::ssh::config::{Config, MachineRole};
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_config_parsing_basic() {
    let config_toml = r#"
[cluster_manager]
id52 = "cluster-manager-id52-test"

[machine.web01]
id52 = "web01-id52-test"
accept_ssh = true
allow_from = "*"

[machine.laptop]
id52 = "laptop-id52-test"

[group.servers]
members = "web01"
"#;

    let config: Config = toml::from_str(config_toml).expect("Config should parse");
    
    // Test basic structure
    assert_eq!(config.cluster_manager.id52, "cluster-manager-id52-test");
    assert_eq!(config.machines.len(), 2);
    assert_eq!(config.groups.len(), 1);
    
    // Test machine properties
    let web01 = config.machines.get("web01").expect("web01 should exist");
    assert_eq!(web01.id52, "web01-id52-test");
    assert!(web01.accept_ssh);
    assert_eq!(web01.allow_from.as_ref().unwrap(), "*");
    
    let laptop = config.machines.get("laptop").expect("laptop should exist");
    assert_eq!(laptop.id52, "laptop-id52-test");
    assert!(!laptop.accept_ssh);  // defaults to false
    
    println!("✅ Basic config parsing test passed");
}

#[test]
fn test_role_detection() {
    let config_toml = r#"
[cluster_manager]
id52 = "manager-id52"

[machine.server1]
id52 = "server1-id52"
accept_ssh = true

[machine.client1]
id52 = "client1-id52"
"#;

    let config: Config = toml::from_str(config_toml).expect("Config should parse");
    
    // Test role detection
    assert_eq!(config.get_local_role("manager-id52"), MachineRole::ClusterManager);
    assert_eq!(config.get_local_role("server1-id52"), MachineRole::SshServer("server1".to_string()));
    assert_eq!(config.get_local_role("client1-id52"), MachineRole::ClientOnly("client1".to_string()));
    assert_eq!(config.get_local_role("unknown-id52"), MachineRole::Unknown);
    
    println!("✅ Role detection test passed");
}

#[test]
fn test_permission_checking() {
    let config_toml = r#"
[cluster_manager]
id52 = "manager-id52"

[machine.web01]
id52 = "web01-id52"
accept_ssh = true
allow_from = "client1-id52,admin-id52"

[machine.web01.command.ls]
allow_from = "readonly-id52"

[machine.web01.service.api]
port = 8080
allow_from = "client1-id52"

[machine.client1]
id52 = "client1-id52"
"#;

    let config: Config = toml::from_str(config_toml).expect("Config should parse");
    
    // Test SSH access permissions
    assert!(config.can_execute_command("client1-id52", "web01", "bash"));
    assert!(!config.can_execute_command("unknown-id52", "web01", "bash"));
    assert!(!config.can_execute_command("client1-id52", "client1", "bash")); // client1 doesn't accept SSH
    
    // Test command-specific permissions
    assert!(config.can_execute_command("readonly-id52", "web01", "ls"));
    assert!(!config.can_execute_command("client1-id52", "web01", "ls")); // needs specific permission for ls
    
    // Test service access permissions
    assert!(config.can_access_service("client1-id52", "web01", "api"));
    assert!(!config.can_access_service("unknown-id52", "web01", "api"));
    
    println!("✅ Permission checking test passed");
}

#[test]
fn test_config_file_operations() {
    // Test saving and loading config files
    let config = Config {
        cluster_manager: malai::ssh::config::ClusterManagerConfig {
            id52: "test-manager-id52".to_string(),
            use_keyring: true,
            private_key_file: None,
            private_key: None,
        },
        machines: std::collections::HashMap::new(),
        groups: std::collections::HashMap::new(),
    };
    
    // Save to temporary file
    let mut temp_file = NamedTempFile::new().expect("Should create temp file");
    let file_path = temp_file.path().to_str().unwrap();
    
    config.save_to_file(file_path).expect("Should save config");
    
    // Load it back
    let loaded_config = Config::load_from_file(file_path).expect("Should load config");
    assert_eq!(loaded_config.cluster_manager.id52, "test-manager-id52");
    
    println!("✅ Config file operations test passed");
}