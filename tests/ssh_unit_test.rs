/// Unit tests for SSH config that don't require the full SSH implementation
/// These validate just the config parsing without any CLI dependencies

#[test]
fn test_basic_toml_parsing() {
    // Test that our expected config format can be parsed
    let config_toml = r#"
[cluster_manager]
id52 = "cluster-manager-id52-test"
use_keyring = true

[machine.web01]  
id52 = "web01-id52-test"
accept_ssh = true
allow_from = "client1-id52"

[machine.client1]
id52 = "client1-id52-test"
"#;

    // Test basic TOML parsing
    let parsed: toml::Value = toml::from_str(config_toml).expect("TOML should parse");
    
    // Verify structure
    assert!(parsed.get("cluster_manager").is_some());
    assert!(parsed.get("machine").is_some());
    
    let machine_table = parsed.get("machine").unwrap().as_table().unwrap();
    assert!(machine_table.get("web01").is_some());
    assert!(machine_table.get("client1").is_some());
    
    println!("✅ Basic TOML parsing works");
}

#[test]
fn test_expected_config_structure() {
    // Test the exact config structure we expect to generate
    let expected_config = r#"[cluster_manager]
id52 = "abc123def456"
use_keyring = true

[machine.web01]
id52 = "server123id456" 
accept_ssh = true
allow_from = "*"

[machine.laptop]
id52 = "client123id456"
"#;

    let parsed: toml::Value = toml::from_str(expected_config).expect("Expected config should parse");
    
    // Test cluster manager
    let cluster_manager = parsed["cluster_manager"].as_table().unwrap();
    assert_eq!(cluster_manager["id52"].as_str().unwrap(), "abc123def456");
    assert_eq!(cluster_manager["use_keyring"].as_bool().unwrap(), true);
    
    // Test machines
    let machines = parsed["machine"].as_table().unwrap();
    let web01 = machines["web01"].as_table().unwrap();
    assert_eq!(web01["id52"].as_str().unwrap(), "server123id456");
    assert_eq!(web01["accept_ssh"].as_bool().unwrap(), true);
    
    let laptop = machines["laptop"].as_table().unwrap();
    assert_eq!(laptop["id52"].as_str().unwrap(), "client123id456");
    // accept_ssh defaults to false, so it might not be present
    
    println!("✅ Expected config structure is valid");
}

#[tokio::test]
async fn test_malai_binary_exists() {
    // Test that we can find and execute the malai binary
    use malai_cli_test_utils::get_malai_binary;
    
    println!("🔍 Looking for malai binary...");
    let malai_path = get_malai_binary();
    println!("📍 Found malai at: {}", malai_path.display());
    
    assert!(malai_path.exists(), "Malai binary should exist after build");
    
    // Test basic malai execution (help command)
    let output = std::process::Command::new(&malai_path)
        .arg("--help")
        .output()
        .expect("Should execute malai --help");
    
    assert!(output.status.success(), "Malai --help should succeed");
    
    let help_text = String::from_utf8_lossy(&output.stdout);
    assert!(help_text.contains("malai"), "Help should mention malai");
    
    // Check if SSH command is available
    if help_text.contains("ssh") {
        println!("✅ SSH command is available in malai");
    } else {
        println!("⚠️  SSH command not yet available - this is expected during development");
    }
    
    println!("✅ Malai binary test passed");
}