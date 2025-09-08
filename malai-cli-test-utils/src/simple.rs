//! Simple, generic utilities for malai CLI testing

use std::path::PathBuf;
use std::process::Command;

/// Get path to malai binary with automatic building
pub fn get_malai_binary() -> PathBuf {
    let target_dir = detect_target_dir();
    let malai_path = target_dir.join("malai");
    
    // Always ensure malai is built fresh to avoid stale binary issues
    let _ = ensure_malai_built();
    
    if !malai_path.exists() {
        panic!("Malai binary not found at {}", malai_path.display());
    }
    
    malai_path
}

/// Build malai binary using cargo (ensures fresh binary)
pub fn ensure_malai_built() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔨 Building fresh malai binary to avoid stale binary issues...");
    let output = Command::new("cargo")
        .args(["build", "--bin", "malai", "--workspace"])
        .output()?;
    
    if !output.status.success() {
        return Err(format!(
            "Failed to build malai: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }
    
    println!("✅ Fresh malai binary ready for testing");
    Ok(())
}

/// Detect target directory (supports workspace and local builds)
pub fn detect_target_dir() -> PathBuf {
    // Strategy 1: Check current directory (workspace root)
    let workspace_target = std::env::current_dir()
        .expect("Could not get current directory")
        .join("target")
        .join("debug");
    
    // Strategy 2: Check if we're in malai subdirectory
    let parent_target = std::env::current_dir()
        .expect("Could not get current directory")
        .parent()
        .map(|p| p.join("target").join("debug"));

    // Check in order of preference
    for candidate in [&workspace_target, &parent_target.unwrap_or(workspace_target.clone())] {
        if candidate.join("malai").exists() {
            return candidate.clone();
        }
    }
    
    // Fallback - return workspace target (build will create it)
    workspace_target
}

/// Output from a malai CLI command execution
#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub exit_code: Option<i32>,
}

impl CommandOutput {
    /// Create from std::process::Output
    pub fn from_output(output: std::process::Output) -> Self {
        Self {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
            exit_code: output.status.code(),
        }
    }

    /// Assert command succeeded
    pub fn expect_success(self) -> Result<Self, Box<dyn std::error::Error>> {
        if self.success {
            Ok(self)
        } else {
            Err(format!(
                "Command failed with exit code {:?}\nstdout: {}\nstderr: {}",
                self.exit_code, self.stdout, self.stderr
            ).into())
        }
    }

    /// Assert command failed
    pub fn expect_failure(self) -> Result<Self, Box<dyn std::error::Error>> {
        if !self.success {
            Ok(self)
        } else {
            Err(format!(
                "Command unexpectedly succeeded\nstdout: {}\nstderr: {}",
                self.stdout, self.stderr
            ).into())
        }
    }

    /// Extract ID52 from keygen output (checks both stdout and stderr)
    pub fn extract_id52(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Check stderr first (where keygen puts the ID52)
        for line in self.stderr.lines() {
            if line.contains("ID52") && line.contains(":") {
                if let Some(id52_part) = line.split(':').nth(1) {
                    return Ok(id52_part.trim().to_string());
                }
            }
        }
        
        // Fallback to stdout
        for line in self.stdout.lines() {
            if line.contains("ID52") && line.contains(":") {
                if let Some(id52_part) = line.split(':').nth(1) {
                    return Ok(id52_part.trim().to_string());
                }
            }
        }
        
        Err(format!("Could not extract ID52 from output\nstdout: {}\nstderr: {}", self.stdout, self.stderr).into())
    }

    /// Extract cluster ID from create-cluster output
    pub fn extract_cluster_id(&self) -> Result<String, Box<dyn std::error::Error>> {
        for line in self.stdout.lines() {
            if line.contains("Cluster created with ID:") {
                if let Some(cluster_id) = line.split(':').nth(1) {
                    return Ok(cluster_id.trim().to_string());
                }
            }
        }
        Err(format!("Could not extract cluster ID from output: {}", self.stdout).into())
    }

    /// Check if output contains expected text
    pub fn contains(&self, text: &str) -> bool {
        self.stdout.contains(text) || self.stderr.contains(text)
    }

    /// Assert output contains expected text
    pub fn assert_contains(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.contains(text) {
            Ok(())
        } else {
            Err(format!(
                "Output does not contain '{}'\nstdout: {}\nstderr: {}",
                text, self.stdout, self.stderr
            ).into())
        }
    }
}