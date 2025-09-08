//! Complete test environment for malai testing with machine management

use crate::{MalaiCommand, MalaiCliConfig};
use crate::malai_cmd::BackgroundProcess;
use std::path::PathBuf;
use tempfile::TempDir;

/// Complete test environment for malai testing with machine management
pub struct MalaiTestEnv {
    temp_dir: TempDir,
    machines: Vec<MachineHandle>,
    config: MalaiCliConfig,
}

impl MalaiTestEnv {
    /// Create new test environment
    pub fn new(test_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempfile::Builder::new()
            .prefix(&format!("malai-test-{test_name}-"))
            .tempdir()?;

        Ok(Self {
            temp_dir,
            machines: Vec::new(),
            config: MalaiCliConfig::default(),
        })
    }

    /// Create with custom configuration
    pub fn with_config(
        test_name: &str,
        config: MalaiCliConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut env = Self::new(test_name)?;
        env.config = config;
        Ok(env)
    }

    /// Create a new machine (generates identity and MALAI_HOME)
    pub async fn create_machine(
        &mut self,
        name: &str,
    ) -> Result<&MachineHandle, Box<dyn std::error::Error>> {
        let machine_home = self.temp_dir.path().join(name);
        std::fs::create_dir_all(&machine_home)?;

        // Generate identity for this machine
        let output = MalaiCommand::new()
            .malai_home(&machine_home)
            .keygen()
            .await?
            .expect_success()?;

        let id52 = output.extract_id52()?;

        let machine = MachineHandle {
            name: name.to_string(),
            home_path: machine_home,
            id52,
            agent_process: None,
        };

        self.machines.push(machine);
        Ok(self.machines.last().unwrap())
    }

    /// Start SSH agent for a machine
    pub async fn start_ssh_agent(&mut self, machine_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let machine_index = self
            .machines
            .iter()
            .position(|m| m.name == machine_name)
            .ok_or(format!("Machine {machine_name} not found"))?;

        let machine = &self.machines[machine_index];
        let agent_process = MalaiCommand::new()
            .malai_home(&machine.home_path)
            .spawn_background(["ssh", "agent"])?;

        // Update the machine with the agent process
        self.machines[machine_index].agent_process = Some(agent_process);

        Ok(())
    }

    /// Get machine by name
    pub fn get_machine(&self, name: &str) -> Option<&MachineHandle> {
        self.machines.iter().find(|m| m.name == name)
    }

    /// Get machine by name (mutable)
    pub fn get_machine_mut(&mut self, name: &str) -> Option<&mut MachineHandle> {
        self.machines.iter_mut().find(|m| m.name == name)
    }

    /// Get all machines
    pub fn machines(&self) -> &[MachineHandle] {
        &self.machines
    }

    /// Get test directory path
    pub fn test_dir(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Create a malai command for a specific machine
    pub fn malai_cmd(&self, machine_name: &str) -> Result<MalaiCommand, Box<dyn std::error::Error>> {
        let machine = self.get_machine(machine_name)
            .ok_or(format!("Machine {machine_name} not found"))?;

        Ok(MalaiCommand::new().malai_home(&machine.home_path))
    }

    /// Wait for all background processes to stabilize
    pub async fn wait_for_agents(&mut self, duration: std::time::Duration) {
        tokio::time::sleep(duration).await;
    }

    /// Stop all background processes
    pub async fn stop_all_agents(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for machine in &mut self.machines {
            if let Some(agent) = machine.agent_process.take() {
                agent.kill().await?;
            }
        }
        Ok(())
    }
}

impl Drop for MalaiTestEnv {
    fn drop(&mut self) {
        // Background processes will be killed automatically via their Drop implementations
    }
}

/// Handle for a test machine with identity and optional running agent
pub struct MachineHandle {
    pub name: String,
    pub home_path: PathBuf,
    pub id52: String,
    pub agent_process: Option<BackgroundProcess>,
}

impl MachineHandle {
    /// Get MALAI_HOME path for this machine
    pub fn home_path(&self) -> &PathBuf {
        &self.home_path
    }

    /// Get machine's ID52
    pub fn id52(&self) -> &str {
        &self.id52
    }

    /// Check if agent is running
    pub fn is_agent_running(&mut self) -> bool {
        if let Some(agent) = &mut self.agent_process {
            agent.is_running()
        } else {
            false
        }
    }

    /// Create malai command for this machine
    pub fn malai_cmd(&self) -> MalaiCommand {
        MalaiCommand::new().malai_home(&self.home_path)
    }
}