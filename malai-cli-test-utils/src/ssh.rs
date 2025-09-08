//! SSH-specific test helpers

use crate::{MalaiTestEnv, MachineHandle};

/// SSH cluster testing helper
pub struct SshTestHelper;

impl SshTestHelper {
    /// Create a basic SSH cluster with cluster manager, server, and client
    pub async fn create_basic_cluster(
        env: &mut MalaiTestEnv,
        cluster_alias: &str,
    ) -> Result<BasicCluster, Box<dyn std::error::Error>> {
        // 1. Create cluster manager
        let _manager = env.create_machine("cluster-manager").await?;
        let cluster_output = env.malai_cmd("cluster-manager")?
            .ssh_create_cluster(Some(cluster_alias))
            .await?
            .expect_success()?;
        
        let cluster_id = cluster_output.extract_cluster_id()?;

        // 2. Create SSH server machine
        let server = env.create_machine("ssh-server").await?;
        let server_id52 = server.id52.clone();
        
        // 3. Create client machine
        let client = env.create_machine("client").await?;
        let client_id52 = client.id52.clone();

        // 4. Update cluster config with server and client
        let manager_handle = env.get_machine("cluster-manager").unwrap();
        Self::add_machine_to_cluster(
            manager_handle,
            "web01",
            &server_id52,
            true,  // accept_ssh = true
            "*",   // allow_from = "*"
        )?;

        Self::add_machine_to_cluster(
            manager_handle,
            "client1", 
            &client_id52,
            false, // client-only
            "",    // no allow_from needed
        )?;

        Ok(BasicCluster {
            cluster_id,
            manager_name: "cluster-manager".to_string(),
            server_name: "ssh-server".to_string(),
            client_name: "client".to_string(),
        })
    }

    /// Add a machine to cluster configuration
    pub fn add_machine_to_cluster(
        cluster_manager: &MachineHandle,
        machine_alias: &str,
        machine_id52: &str,
        accept_ssh: bool,
        allow_from: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = cluster_manager.home_path.join("ssh").join("cluster-config.toml");
        
        let machine_config = if accept_ssh {
            format!(
                r#"
[machine.{}]
id52 = "{}"
accept_ssh = true
allow_from = "{}"
"#,
                machine_alias, machine_id52, allow_from
            )
        } else {
            format!(
                r#"
[machine.{}]
id52 = "{}"
"#,
                machine_alias, machine_id52
            )
        };

        // Append to existing config file
        std::fs::write(&config_path, machine_config)?;

        Ok(())
    }

    /// Add HTTP service to machine configuration
    pub fn add_http_service(
        cluster_manager: &MachineHandle,
        machine_alias: &str,
        service_name: &str,
        port: u16,
        allow_from: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = cluster_manager.home_path.join("ssh").join("cluster-config.toml");
        
        let service_config = format!(
            r#"
[machine.{}.service.{}]
port = {}
allow_from = "{}"
"#,
            machine_alias, service_name, port, allow_from
        );

        // Read existing config and append
        let existing = std::fs::read_to_string(&config_path).unwrap_or_default();
        let updated = format!("{}{}", existing, service_config);
        std::fs::write(&config_path, updated)?;

        Ok(())
    }

    /// Test SSH command execution
    pub async fn test_ssh_execution(
        env: &MalaiTestEnv,
        client_machine: &str,
        target_machine: &str,
        command: &str,
        args: Vec<&str>,
        expected_output: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = env.malai_cmd(client_machine)?
            .ssh_exec(target_machine, command, args)
            .await?
            .expect_success()?;

        output.assert_contains(expected_output)?;
        Ok(())
    }

    /// Test SSH command failure (should be denied)
    pub async fn test_ssh_denied(
        env: &MalaiTestEnv,
        client_machine: &str,
        target_machine: &str,
        command: &str,
        args: Vec<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = env.malai_cmd(client_machine)?
            .ssh_exec(target_machine, command, args)
            .await?
            .expect_failure()?;

        // Should contain permission denied or similar error
        assert!(output.contains("Permission denied") || output.contains("denied") || output.contains("not allowed"));
        Ok(())
    }

    /// Test HTTP service access through SSH proxy
    pub async fn test_http_service_access(
        env: &MalaiTestEnv,
        client_machine: &str,
        service_url: &str,
        expected_content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = env.malai_cmd(client_machine)?
            .ssh_curl(service_url, vec![])
            .await?
            .expect_success()?;

        output.assert_contains(expected_content)?;
        Ok(())
    }
}

/// Result of creating a basic SSH cluster setup
#[derive(Debug)]
pub struct BasicCluster {
    pub cluster_id: String,
    pub manager_name: String,
    pub server_name: String,
    pub client_name: String,
}

impl BasicCluster {
    /// Get cluster manager machine name
    pub fn manager(&self) -> &str {
        &self.manager_name
    }

    /// Get SSH server machine name
    pub fn server(&self) -> &str {
        &self.server_name
    }

    /// Get client machine name
    pub fn client(&self) -> &str {
        &self.client_name
    }

    /// Get cluster ID
    pub fn cluster_id(&self) -> &str {
        &self.cluster_id
    }
}