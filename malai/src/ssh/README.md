# `malai ssh`

`malai ssh` provides a secure, P2P SSH-like system for managing clusters of machines and services over the kulfi network.

## Overview

The SSH functionality enables:
- Creating and managing machine clusters
- Secure remote command execution
- HTTP service proxying over P2P connections
- Centralized configuration and access control
- Agent-based connection management

## Clusters

`malai ssh` organizes machines into clusters. Each cluster has:

- **Cluster Manager**: A designated server that manages cluster configuration and coordinates member communication
- **Unique Identity**: Each cluster is identified by the cluster manager's id52
- **Domain Aliases**: Optional domain-based aliases for easier identification
- **DNS Integration**: Cluster manager id52 can be stored in DNS TXT records for domain-based discovery

Machines can belong to multiple clusters simultaneously, each with their own id52 keypair.

## Machine Types

All cluster members are "machines" with different capabilities based on configuration:

### Cluster Manager Machine
- Manages cluster configuration and member coordination
- Automatically created by `malai ssh create-cluster`
- Can manually edit `cluster-config.toml`
- Distributes config updates to all cluster machines

### Server Machines  
- Accept incoming SSH connections and execute commands
- Can expose HTTP services with access control
- Defined in config as `[machine.web01]` with `accept_ssh = true`
- Must be reachable by other cluster machines

### Client-Only Machines
- Can initiate SSH connections to server machines
- Cannot accept incoming connections (`accept_ssh = false` or not specified)
- Suitable for laptops, mobile devices, or firewalled machines

## Addressing and Aliases

### Machine Addressing
Each machine has multiple addressing options:

- **Domain-based**: `machine-alias.cluster-domain.com` (when domain is available)
- **ID-based**: `machine-alias.cluster-id52` (always works)
- **Full ID**: `machine-id52.cluster-id52` (direct addressing)

### Service Addressing
HTTP services on machines can be addressed as:

- **Domain-based**: `service-alias.machine-alias.cluster-domain.com`
- **ID-based**: `service-alias.machine-alias.cluster-id52`
- **Full ID**: `service-alias.machine-id52.cluster-id52`

Example: A Django admin service on machine `web01` in cluster `company.com` could be reached at `admin.web01.company.com`.

## HTTP Service Proxying

Machines can expose HTTP services through the malai network:

- **Service Registration**: Each HTTP service gets a unique alias
- **Access Control**: Configure which cluster machines can access each service
- **Transparent Proxying**: Services appear as if running locally to authorized clients
- **Port Flexibility**: No need to manage port conflicts across the cluster

## Config File Format

```toml
# Cluster manager configuration
[cluster-manager]
id52 = "cluster-manager-id52-here"

# Machine definitions
[machine.web01]
id52 = "web01-id52-here"
accept_ssh = true                     # This machine accepts SSH connections
allow_from = "laptop-id52,admin-id52" # Full SSH access from these machines

# Command-specific access control
[machine.web01.command.ls]
allow_from = "readonly-id52"          # Only this machine can run 'ls'

# HTTP service exposure
[machine.web01.service.admin]
port = 8080
allow_from = "admin-id52,manager-id52"

[machine.web01.service.api]  
port = 3000
allow_from = "*"                      # All cluster machines can access

# Client-only machine (no accept_ssh = true)
[machine.laptop]
id52 = "laptop-id52-here"

# Groups for easier management
[group.web-servers]
members = "web01,web02,web03"

[group.admins] 
members = "laptop,admin-desktop"
```

**Config Management Rules:**
- **Cluster Manager Machine**: Admin manually edits `$MALAI_HOME/ssh/cluster-config.toml`
  - Use any editor: vim, nano, cp, etc.
  - Agent watches for changes and auto-distributes to all cluster machines
- **All Other Machines**: 
  - Receive config from cluster manager via P2P sync
  - Agent automatically overwrites `$MALAI_HOME/ssh/cluster-config.toml`
  - **NEVER manually edit** - changes will be lost on next sync
  - Config is read-only for end users on non-cluster-manager machines

## Configuration Management

### Automatic Sync
The cluster manager automatically distributes configuration updates:

1. **Change Detection**: Monitors `cluster-config.toml` file hash changes
2. **Full Distribution**: All machines receive the complete cluster configuration
3. **Auto-Overwrite**: Machines automatically overwrite their local config file
4. **Role Detection**: Each machine's agent reads config to determine its role

### Machine Role Detection
Each machine's agent automatically detects its role by:

1. **Reading** local `cluster-config.toml` 
2. **Matching** local identity ID52 against config sections
3. **Determining role**:
   - If ID52 matches `[cluster-manager].id52` → cluster manager
   - If ID52 matches `[machine.X]` with `accept_ssh = true` → SSH server
   - If ID52 matches `[machine.X]` without `accept_ssh` → client-only
4. **Starting services** automatically based on detected role 

## Usage

### Basic SSH Command
```bash
# Connect to a server and run a command
malai ssh web01.company.com "ps aux"

# Interactive SSH session
malai ssh web01.company.com

# Using ID-based addressing
malai ssh web01.cluster-id52 "systemctl status nginx"
```

### Multi-Cluster Support
- Each device can participate in multiple clusters
- Separate id52 keypair for each cluster
- Automatic cluster detection from server address
- Configuration stored per cluster in `DATADIR[malai]/ssh/clusters/<cluster-alias>/`

### Cluster Directory Structure
```
DATADIR[malai]/ssh/clusters/<cluster-alias>/
├── cluster-config.toml    # Local cluster configuration
├── keypair.key           # This device's private key for this cluster
├── known-hosts          # Verified server public keys
└── logs/                # Connection and sync logs
```

## Command Reference

### Cluster Management Commands
```bash
# Initialize a new cluster (generates cluster manager identity)
malai ssh init-cluster [--alias company-cluster]
# Outputs: "Cluster created with ID: <cluster-manager-id52>"
# Creates: $MALAI_HOME/ssh/cluster-config.toml with this machine as cluster manager

# Initialize machine for SSH (generates identity only, NO config)
malai ssh init
# Outputs: "Machine created with ID: <machine-id52>"
# Creates: $MALAI_HOME/keys/identity.key (identity only)
# Machine will receive config from cluster manager via P2P sync

# List cluster information  
malai ssh cluster-info
# Shows: role (cluster-manager/machine/unknown), cluster ID, machine alias from config
```

### Client Commands
```bash
# Execute single command
malai ssh <server-address> <command>

# Interactive session
malai ssh <server-address>

# With explicit cluster
malai ssh --cluster company.com web01 "uptime"
```

### Agent Commands
```bash
# Start agent in background (handles all SSH functionality automatically)
# Requires MALAI_HOME to be set or uses default location
malai ssh agent

# Get environment setup commands for shell integration
malai ssh agent -e

# Agent automatically:
# - Uses MALAI_HOME for all data (config, identity, socket, lockfile)
# - Detects role from local identity vs cluster config
# - Starts cluster manager, server, or client-only mode as appropriate
# - Handles HTTP proxy and configuration sync
# - Manages connections and permissions
# - Uses system log directories for logging (never writes to MALAI_HOME/logs)
```

**Important:** Agent requires `MALAI_HOME` environment variable or uses platform default.

## SSH Agent

The SSH agent provides persistent connection management and improved performance.

### Operation Modes
1. **With Agent**: Commands are routed through the background agent process
2. **Without Agent**: Direct connections for each command (slower but simpler)

### Agent Benefits
- **Connection Reuse**: Maintains persistent connections to frequently accessed servers
- **Performance**: Faster command execution through connection pooling
- **HTTP Proxy**: Enables transparent HTTP service access
- **Background Sync**: Handles configuration updates automatically

### Agent Communication
- **Discovery**: Agent socket path via `MALAI_SSH_AGENT` environment variable
- **Fallback**: Direct connections when agent is unavailable
- **Logging**: All logs stored in `LOGDIR[malai]/ssh/` (stdout/stderr reserved for command output)

## Security Modes

### Lockdown Mode
Enable with `MALAI_LOCKDOWN_MODE=true`:

- **Key Isolation**: Private keys only accessible to the SSH agent
- **Mandatory Agent**: All SSH operations must go through the agent
- **Enhanced Security**: Reduces key exposure to individual command processes
- **Audit Trail**: Centralized logging of all SSH operations 

## HTTP Integration

### Transparent HTTP Access
The SSH agent provides transparent HTTP access to cluster services:

```bash
# These commands work transparently when agent is running
curl admin.web01.company.com/status
wget api.web01.company.com/data.json
```

### Mechanism
1. **HTTP Proxy**: Agent runs a local HTTP proxy
2. **Environment Setup**: `HTTP_PROXY` points to the agent's proxy
3. **Service Resolution**: Proxy resolves cluster service addresses
4. **P2P Tunneling**: HTTP requests tunneled through malai network 

### Explicit HTTP Commands
```bash
# Force HTTP access through malai network
malai ssh curl admin.web01.company.com/api

# Equivalent to:
# HTTP_PROXY=<agent-proxy> curl admin.web01.company.com/api
```

## Agent Environment Setup

### Shell Integration
The agent outputs environment variables in `ENV=value` format for shell evaluation:

```bash
# Start agent and configure environment
eval $(malai ssh agent -e)

# With specific options
eval $(malai ssh agent -e --lockdown --http)

# Disable HTTP proxy
eval $(malai ssh agent -e --http=false)
```

### Persistent Setup
Add to your shell profile (`.bashrc`, `.zshrc`, etc.):
```bash
# Enable malai ssh agent on shell startup
eval $(malai ssh agent -e)
```

### Environment Variables Set
- `MALAI_SSH_AGENT`: Unix socket path for agent communication
- `MALAI_LOCKDOWN_MODE`: Enable/disable lockdown mode (default: true)
- `HTTP_PROXY`: Local HTTP proxy for transparent service access (default: enabled)

## Environment Variables

### MALAI_HOME
The `MALAI_HOME` environment variable controls where malai stores its configuration and data files. This is crucial for running multiple clusters and testing scenarios.

**Default Locations:**
- Linux/macOS: `~/.local/share/malai`
- Windows: `%APPDATA%/malai`

**Override with MALAI_HOME:**
```bash
export MALAI_HOME=/path/to/custom/malai/data
# Create cluster or machine, then agent handles everything automatically
eval $(malai ssh agent -e)  # Agent auto-detects role and starts appropriate services
```

**Directory Structure:**
```
$MALAI_HOME/
├── ssh/
│   ├── cluster-config.toml      # Cluster configuration
│   │                           # • Cluster manager: manually edited by admin
│   │                           # • Other machines: auto-synced from cluster manager
│   ├── agent.sock              # Agent communication socket
│   └── agent.lock              # Lockfile to prevent multiple agents
└── keys/
    └── identity.key            # This machine's identity

# Logs stored in standard system log directories:
# - Linux/macOS: ~/.local/state/malai/ssh/logs/
# - Windows: %LOCALAPPDATA%/malai/ssh/logs/
```

**Agent Lockfile:**
- `$MALAI_HOME/ssh/agent.lock` prevents multiple agents with same MALAI_HOME
- If lockfile exists and process is running, new agent instances exit gracefully
- Enables safe testing with multiple MALAI_HOME directories

## Multi-Cluster Agent

A single agent manages all clusters:

### Unified Management
- **Single Process**: One agent handles all configured clusters
- **Shared HTTP Proxy**: Single proxy endpoint for all cluster services
- **Cross-Cluster**: Seamless access to services across different clusters
- **Resource Efficiency**: Minimal overhead regardless of cluster count

### Service Discovery
- **Automatic Scanning**: Agent discovers all clusters in `DATADIR[malai]/ssh/clusters/`
- **Dynamic Updates**: New clusters are automatically detected and integrated
- **Conflict Resolution**: Service name conflicts resolved by cluster precedence

## Multi-Instance Testing

The `MALAI_HOME` environment variable enables comprehensive testing of multi-cluster, multi-server scenarios on a single machine by creating isolated environments.

### Single Machine Multi-Cluster Setup

**1. Create Test Directories:**
```bash
mkdir -p /tmp/malai-test/{cluster1,cluster2,server1,server2,device1,device2}
```

**2. Initialize Cluster (Terminal 1):**
```bash
export MALAI_HOME=/tmp/malai-test/cluster1
malai ssh init-cluster --alias test-cluster
# Outputs: "Cluster created with ID: abc123..."
eval $(malai ssh agent -e)  # Start agent (automatically runs as cluster manager)
```

**3. Initialize Server Machine (Terminal 2):**
```bash
export MALAI_HOME=/tmp/malai-test/server1
malai ssh init  # Generate machine identity (NO config yet)
# Outputs: "Machine created with ID: def456..."
```

**4. Update Cluster Config (Terminal 1 - Cluster Manager):**
```bash
# Edit $MALAI_HOME/ssh/cluster-config.toml to add:
# [machine.web01]
# id52 = "def456..."  # The ID from step 3
# accept_ssh = true
# allow_from = "*"
# 
# Config automatically syncs to Terminal 2's machine
```

**5. Start Server Agent (Terminal 2):**
```bash
eval $(malai ssh agent -e)  # Agent receives config and auto-detects SSH server role
```

**6. Create Client Machine (Terminal 3):**
```bash
export MALAI_HOME=/tmp/malai-test/client1
malai keygen  # Generate client identity
# Outputs: "Identity created with ID52: ghi789..."
```

**7. Update Cluster Config for Client (Terminal 1):**
```bash
# Edit cluster config to add:
# [machine.laptop]
# id52 = "ghi789..."  # The ID from step 6
# (no accept_ssh = client-only by default)
```

**8. Test SSH (Terminal 3):**
```bash
eval $(malai ssh agent -e)  # Start agent (automatically runs as client)
malai ssh web01.test-cluster "echo 'Hello from remote server!'"
```

**5. Test HTTP Service Access:**
```bash
# In server terminal, start a local HTTP service
python3 -m http.server 8080 &

# In client terminal
curl admin.web01.cluster.local:8080/
```

### Multi-Cluster Testing

Test cross-cluster scenarios by setting up multiple independent clusters:

**Company Cluster:**
```bash
export MALAI_HOME=/tmp/malai-test/company-cluster
malai ssh create-cluster --alias company-cluster
eval $(malai ssh agent -e)  # Runs as cluster manager automatically
```

**Test Cluster:**
```bash
export MALAI_HOME=/tmp/malai-test/test-cluster
malai ssh create-cluster --alias test-cluster
eval $(malai ssh agent -e)  # Runs as different cluster manager
```

**Client with Access to Both:**
```bash
export MALAI_HOME=/tmp/malai-test/multi-client
eval $(malai ssh agent -e)
malai ssh web01.company.com "uptime"
malai ssh test-server.test.local "ps aux"
```

### Test Scenarios

**1. Permission Testing:**
```bash
# Test command restrictions
malai ssh restricted-server.cluster.local "ls"  # Should work
malai ssh restricted-server.cluster.local "rm file"  # Should fail
```

**2. Service Access Testing:**
```bash
# Test HTTP service permissions
curl api.server1.cluster.local/public    # Should work
curl admin.server1.cluster.local/secret  # Should fail without permission
```

**3. Agent Functionality Testing:**
```bash
# Test agent environment setup
eval $(malai ssh agent -e --lockdown --http)
echo $MALAI_SSH_AGENT
echo $HTTP_PROXY
echo $MALAI_LOCKDOWN_MODE
```

**4. Configuration Sync Testing:**
```bash
# Modify cluster config and verify sync
# Edit cluster-config.toml
# Observe logs in all connected nodes for config updates
```

### Cleanup
```bash
# Kill all background processes
pkill -f "malai ssh"

# Clean up test directories
rm -rf /tmp/malai-test/
```

## Getting Started

### Production Setup

**1. Initialize Cluster (on cluster manager machine):**
```bash
malai ssh init-cluster --alias company-cluster
# Outputs: "Cluster created with ID: <cluster-manager-id52>"
eval $(malai ssh agent -e)  # Start agent in background
```

**2. Initialize Machines:**
```bash
# On each machine that should join the cluster:
malai ssh init  # Generate identity for this machine
# Outputs: "Machine created with ID: <machine-id52>"

# Cluster admin manually adds to cluster manager's config:
# Edit $MALAI_HOME/ssh/cluster-config.toml:
# [machine.web01] 
# id52 = "<machine-id52>"
# accept_ssh = true        # If this should accept SSH connections
# allow_from = "*"
#
# Config automatically syncs to all machines via P2P
# Each machine's agent auto-detects its role and starts appropriate services
```

**3. Start Agents on All Machines:**
```bash
# On each machine (add to ~/.bashrc for automatic startup):
eval $(malai ssh agent -e)
# Agent automatically:
# - Receives config from cluster manager
# - Detects its role (cluster-manager/SSH server/client-only)
# - Starts appropriate services
```

**4. Use SSH:**
```bash
malai ssh web01.company-cluster "uptime"
curl admin.web01.company-cluster/status
```

### Development/Testing Setup

For development and testing, use `MALAI_HOME` to create isolated environments:

**1. Create Test Environment:**
```bash
export MALAI_HOME=/tmp/malai-dev
mkdir -p $MALAI_HOME
```

**2. Generate Test Identities:**
```bash
# Each component gets its own identity
malai keygen  # Creates identity in $MALAI_HOME
```

**3. Test Multi-Node Setup:**
```bash
# Terminal 1 - Create Cluster
export MALAI_HOME=/tmp/malai-cluster-manager
malai ssh create-cluster --alias test-cluster
# Note the cluster ID output: "Cluster created with ID: abc123..."
eval $(malai ssh agent -e)  # Auto-runs as cluster manager

# Terminal 2 - Create Server Machine
export MALAI_HOME=/tmp/malai-server1
malai ssh create-machine
# Note the machine ID output: "Machine created with ID: def456..."

# Terminal 1 - Add Server to Cluster Config
# Edit cluster config to add:
# [server.web01]
# id52 = "def456..."
# allow_from = "*"
# Config automatically syncs to Terminal 2

# Terminal 2 - Server Starts Automatically  
eval $(malai ssh agent -e)  # Agent detects role and starts SSH server

# Terminal 3 - Create and Add Client
export MALAI_HOME=/tmp/malai-client1
malai ssh create-machine
# Add this ID to cluster config as [device.laptop]
eval $(malai ssh agent -e)
malai ssh web01.test-cluster "echo 'Multi-node test successful!'"
```

This approach allows you to test complex multi-cluster scenarios, permission systems, and service configurations entirely on a single development machine.

## End-to-End Testing Strategy

The MALAI_HOME-based isolation enables comprehensive testing of the entire SSH system on a single machine without external dependencies.

### Test Architecture

**Single Machine Multi-Cluster Testing:**
- **Process isolation**: Each MALAI_HOME gets separate agent with lockfile protection
- **Network sharing**: All agents share the fastn-p2p network for communication
- **Config isolation**: Separate cluster-config.toml for each test instance
- **Identity separation**: Each instance has unique identity and role

### Test Scenarios

#### **Level 1: Basic Cluster Functionality**
```bash
#!/bin/bash
# Test basic cluster creation and SSH functionality

# Setup
export TEST_DIR=/tmp/malai-e2e-test
mkdir -p $TEST_DIR/{manager,server1,client1}

# 1. Create cluster
export MALAI_HOME=$TEST_DIR/manager
malai ssh create-cluster --alias test-cluster
CLUSTER_ID=$(malai ssh cluster-info | grep "Cluster ID" | cut -d: -f2)

# 2. Create SSH server
export MALAI_HOME=$TEST_DIR/server1  
malai keygen
SERVER_ID=$(malai keygen | grep "ID52" | cut -d: -f2)

# 3. Add server to cluster config
export MALAI_HOME=$TEST_DIR/manager
echo "[machine.web01]
id52 = \"$SERVER_ID\"
accept_ssh = true
allow_from = \"*\"" >> $MALAI_HOME/ssh/cluster-config.toml

# 4. Start agents
export MALAI_HOME=$TEST_DIR/manager && malai ssh agent &
export MALAI_HOME=$TEST_DIR/server1 && malai ssh agent &
sleep 2  # Wait for config sync

# 5. Test SSH execution
export MALAI_HOME=$TEST_DIR/client1
malai keygen
CLIENT_ID=$(malai keygen | grep "ID52" | cut -d: -f2)

# Add client to config
export MALAI_HOME=$TEST_DIR/manager  
echo "[machine.client1]
id52 = \"$CLIENT_ID\"" >> $MALAI_HOME/ssh/cluster-config.toml

# Wait for sync and test
export MALAI_HOME=$TEST_DIR/client1
eval $(malai ssh agent -e)
malai ssh web01.test-cluster "echo 'SSH test successful'"

# Verify output contains "SSH test successful"
```

#### **Level 2: Permission System Testing**
```bash
# Test command restrictions and access control
# Add restricted user to config:
# [machine.restricted]
# id52 = "restricted-id52"
# [machine.web01.command.ls]  
# allow_from = "restricted-id52"

# Test: restricted user can run ls but not other commands
malai ssh web01.test-cluster "ls"        # Should succeed
malai ssh web01.test-cluster "whoami"    # Should fail with permission denied
```

#### **Level 3: HTTP Service Testing**
```bash
# Test HTTP service proxying
# On server machine: python3 -m http.server 8080 &
# Add to config:
# [machine.web01.service.test-api]
# port = 8080  
# allow_from = "client1-id52"

# Test HTTP access through agent proxy
curl test-api.web01.test-cluster/
# Should return HTTP server content
```

#### **Level 4: Multi-Cluster Testing**
```bash
# Create two independent clusters
export MALAI_HOME=/tmp/test-company-cluster
malai ssh create-cluster --alias company

export MALAI_HOME=/tmp/test-dev-cluster  
malai ssh create-cluster --alias dev

# Create client with access to both clusters
export MALAI_HOME=/tmp/test-multi-client
# Copy both cluster configs or implement multi-cluster client support

# Test cross-cluster access isolation
malai ssh company-server.company "uptime"  # Should work
malai ssh dev-server.dev "uptime"          # Should work  
malai ssh company-server.dev "uptime"      # Should fail (wrong cluster)
```

#### **Level 5: Advanced Scenarios**
```bash
# Config sync testing
# 1. Start cluster with basic config
# 2. Add new machines to config while running
# 3. Verify all agents receive updates automatically
# 4. Test new machine functionality immediately

# Agent restart testing
# 1. Kill agent process
# 2. Restart agent 
# 3. Verify role detection and service restart

# Lockfile testing
# 1. Start agent with MALAI_HOME
# 2. Try starting second agent with same MALAI_HOME
# 3. Verify second agent exits gracefully
```

### Automated Test Suite

**Test Script Structure:**
```bash
#!/bin/bash
# run-e2e-tests.sh

set -e  # Exit on any failure

echo "🧪 Running malai SSH end-to-end tests"

# Level 1: Basic functionality
./tests/test-basic-cluster.sh

# Level 2: Permissions  
./tests/test-permissions.sh

# Level 3: HTTP services
./tests/test-http-services.sh

# Level 4: Multi-cluster
./tests/test-multi-cluster.sh

# Level 5: Advanced scenarios
./tests/test-config-sync.sh
./tests/test-agent-restart.sh
./tests/test-lockfiles.sh

echo "✅ All tests passed!"
```

**CI Integration:**
```yaml
# .github/workflows/ssh-e2e-tests.yml
name: SSH End-to-End Tests
on: [push, pull_request]
jobs:
  ssh-e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Build malai
        run: cargo build --release
      - name: Run SSH E2E Tests
        run: ./scripts/run-e2e-tests.sh
        env:
          RUST_LOG: malai=debug
```

### Benefits of This Approach

1. **No external dependencies** - pure single-machine testing
2. **Complete scenario coverage** - can test any cluster configuration
3. **Fast feedback loops** - no Docker/VM startup time
4. **CI friendly** - runs in standard GitHub Actions
5. **Reproducible** - same test environment every time
6. **Comprehensive** - tests real P2P communication over fastn network

The MALAI_HOME approach gives us everything we need for robust end-to-end testing!
