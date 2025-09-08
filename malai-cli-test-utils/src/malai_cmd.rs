//! Fluent malai command builder for testing

use crate::{CommandOutput, get_malai_binary};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Fluent builder for malai commands
#[derive(Debug, Clone)]
pub struct MalaiCommand {
    binary_path: PathBuf,
    malai_home: Option<PathBuf>,
    env_vars: HashMap<String, String>,
    timeout: std::time::Duration,
}

impl Default for MalaiCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl MalaiCommand {
    /// Create new malai command builder
    pub fn new() -> Self {
        Self {
            binary_path: get_malai_binary(),
            malai_home: None,
            env_vars: HashMap::new(),
            timeout: std::time::Duration::from_secs(30),
        }
    }

    /// Set MALAI_HOME directory
    pub fn malai_home<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.malai_home = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set environment variable
    pub fn env<K: AsRef<str>, V: AsRef<str>>(mut self, key: K, value: V) -> Self {
        self.env_vars.insert(key.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Set timeout for command execution
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Execute `malai keygen` command
    pub async fn keygen(self) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        self.execute_args(["keygen"]).await
    }

    /// Execute `malai keygen --file <path>` command
    pub async fn keygen_to_file<P: AsRef<Path>>(self, file_path: P) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        self.execute_args(["keygen", "--file", file_path.as_ref().to_str().unwrap()]).await
    }

    /// Execute `malai ssh create-cluster` command
    pub async fn ssh_create_cluster(self, alias: Option<&str>) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        let mut args = vec!["ssh", "create-cluster"];
        if let Some(alias) = alias {
            args.extend(["--alias", alias]);
        }
        self.execute_args(args).await
    }

    /// Execute `malai ssh cluster-info` command
    pub async fn ssh_cluster_info(self) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        self.execute_args(["ssh", "cluster-info"]).await
    }

    /// Execute `malai ssh agent` command
    pub async fn ssh_agent(self) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        self.execute_args(["ssh", "agent"]).await
    }

    /// Execute `malai ssh agent -e` command for environment setup
    pub async fn ssh_agent_environment(self) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        self.execute_args(["ssh", "agent", "-e"]).await
    }

    /// Execute `malai ssh exec` command
    pub async fn ssh_exec(self, machine: &str, command: &str, args: Vec<&str>) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        let mut cmd_args = vec!["ssh", "exec", machine, command];
        cmd_args.extend(args);
        self.execute_args(cmd_args).await
    }

    /// Execute `malai ssh shell` command
    pub async fn ssh_shell(self, machine: &str) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        self.execute_args(["ssh", "shell", machine]).await
    }

    /// Execute `malai ssh curl` command
    pub async fn ssh_curl(self, url: &str, args: Vec<&str>) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        let mut cmd_args = vec!["ssh", "curl", url];
        cmd_args.extend(args);
        self.execute_args(cmd_args).await
    }

    /// Execute `malai http` command
    pub async fn http(self, port: u16, public: bool) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        let port_str = port.to_string();
        let mut args = vec!["http", &port_str];
        if public {
            args.push("--public");
        }
        self.execute_args(args).await
    }

    /// Execute `malai folder` command  
    pub async fn folder<P: AsRef<Path>>(self, path: P, public: bool) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        let mut args = vec!["folder", path.as_ref().to_str().unwrap()];
        if public {
            args.push("--public");
        }
        self.execute_args(args).await
    }

    /// Execute arbitrary malai command with custom args
    pub async fn execute_args<I, S>(self, args: I) -> Result<CommandOutput, Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut cmd = std::process::Command::new(&self.binary_path);
        
        // Set MALAI_HOME if specified
        if let Some(home) = &self.malai_home {
            cmd.env("MALAI_HOME", home);
        }
        
        // Set additional environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }
        
        // Add command arguments
        for arg in args {
            cmd.arg(arg.as_ref());
        }

        // Execute with timeout
        let output = tokio::time::timeout(self.timeout, async {
            tokio::task::spawn_blocking(move || cmd.output()).await?
        }).await??;

        Ok(CommandOutput::from_output(output))
    }

    /// Spawn malai command as background process (for agents, servers, etc.)
    pub fn spawn_background<I, S>(self, args: I) -> Result<BackgroundProcess, Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut cmd = tokio::process::Command::new(&self.binary_path);
        
        // Set MALAI_HOME if specified
        if let Some(home) = &self.malai_home {
            cmd.env("MALAI_HOME", home);
        }
        
        // Set additional environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }
        
        // Add command arguments
        for arg in args {
            cmd.arg(arg.as_ref());
        }

        cmd.kill_on_drop(true);
        let child = cmd.spawn()?;

        Ok(BackgroundProcess {
            child: Some(child),
            timeout: self.timeout,
        })
    }
}

/// Background process handle with automatic cleanup
pub struct BackgroundProcess {
    child: Option<tokio::process::Child>,
    timeout: std::time::Duration,
}

impl BackgroundProcess {
    /// Wait for process to complete
    pub async fn wait(mut self) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        if let Some(child) = self.child.take() {
            let output = tokio::time::timeout(self.timeout, child.wait_with_output()).await??;
            Ok(CommandOutput::from_output(output))
        } else {
            Err("Process already consumed".into())
        }
    }

    /// Kill the background process
    pub async fn kill(mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut child) = self.child.take() {
            child.kill().await?;
            let _ = child.wait().await;
        }
        Ok(())
    }

    /// Check if process is still running
    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(_)) => false,  // Process has exited
                Ok(None) => true,      // Process still running
                Err(_) => false,       // Error checking status
            }
        } else {
            false
        }
    }
}

impl Drop for BackgroundProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.start_kill();
        }
    }
}