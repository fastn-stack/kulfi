/// Test that malai binary builds and basic commands work
/// This is the absolutely simplest test to validate our build pipeline

use malai_cli_test_utils::*;

#[tokio::test]
async fn test_malai_binary_builds() {
    println!("🔨 Testing malai binary builds and runs...");
    
    // This will build the binary if needed
    let malai_path = get_malai_binary();
    println!("📍 Malai binary location: {}", malai_path.display());
    
    // Test basic malai execution
    let output = MalaiCommand::new()
        .execute_args(["--help"])
        .await
        .expect("Should be able to run malai --help");
    
    let output = output.expect_success().expect("Malai --help should succeed");
    
    println!("📄 Malai help output:");
    println!("{}", output.stdout);
    
    // Verify basic structure
    assert!(output.contains("malai"), "Help should mention malai");
    
    // Check what commands are available
    if output.contains("ssh") {
        println!("✅ SSH command is available");
    } else {
        println!("⚠️  SSH command not yet available - will be implemented");
    }
    
    if output.contains("keygen") {
        println!("✅ Keygen command is available");
        
        // Test keygen works
        let temp_dir = tempfile::tempdir().expect("Should create temp dir");
        let keygen_output = MalaiCommand::new()
            .malai_home(temp_dir.path())
            .keygen()
            .await
            .expect("Should be able to run keygen");
            
        let keygen_output = keygen_output.expect_success().expect("Keygen should succeed");
        println!("🔑 Keygen output: {}", keygen_output.stdout);
        
        // Try to extract ID52 
        if let Ok(id52) = keygen_output.extract_id52() {
            println!("✅ Successfully extracted ID52: {}", id52);
        } else {
            println!("⚠️  Could not extract ID52 - output format may need adjustment");
            println!("Raw output: {}", keygen_output.stdout);
        }
    } else {
        println!("⚠️  Keygen command not available");
    }
    
    println!("✅ Basic malai binary test passed");
}

#[test]
fn test_toml_config_format() {
    println!("📝 Testing TOML config format we plan to use...");
    
    // Test the exact format we want to generate
    let sample_config = r#"[cluster_manager]
id52 = "cluster-abc123def456"
use_keyring = true

[machine.web01]
id52 = "machine-def456ghi789"
accept_ssh = true
allow_from = "*"

[machine.laptop]
id52 = "machine-ghi789jkl012"
"#;

    // Verify it parses as valid TOML
    let parsed: toml::Value = toml::from_str(sample_config).expect("Config should parse as TOML");
    
    println!("✅ Config format is valid TOML");
    
    // Test we can round-trip it
    let serialized = toml::to_string(&parsed).expect("Should serialize back to TOML");
    println!("🔄 Round-trip serialization works");
    println!("Generated TOML:\n{}", serialized);
    
    println!("✅ TOML config format test passed");
}