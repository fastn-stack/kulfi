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

## Node Types

### Servers
- Run `malai ssh server` to accept incoming connections
- Can execute remote commands from authorized cluster members
- Can expose HTTP services with access control
- Must be reachable by other cluster members

### Devices
- Client-only nodes that cannot accept incoming connections
- Can initiate SSH connections to servers in the cluster
- Suitable for laptops, mobile devices, or firewalled machines

## Addressing and Aliases

### Node Addressing
Each node has multiple addressing options:

- **Domain-based**: `node-alias.cluster-domain.com` (when domain is available)
- **ID-based**: `node-alias.cluster-id52` (always works)
- **Full ID**: `node-id52.cluster-id52` (direct addressing)

### Service Addressing
HTTP services on nodes can be addressed as:

- **Domain-based**: `service-alias.node-alias.cluster-domain.com`
- **ID-based**: `service-alias.node-alias.cluster-id52`
- **Full ID**: `service-alias.node-id52.cluster-id52`

Example: A Django admin service on server `web01` in cluster `company.com` could be reached at `admin.web01.company.com`.

## HTTP Service Proxying

Servers can expose HTTP services through the malai network:

- **Service Registration**: Each HTTP service gets a unique alias
- **Access Control**: Configure which cluster members can access each service
- **Transparent Proxying**: Services appear as if running locally to authorized clients
- **Port Flexibility**: No need to manage port conflicts across the cluster

## Config File Format

```toml
# Cluster manager configuration
[cluster-manager]
id52 = "cluster-manager-id52-here"

# Key management (choose one)
use-keyring = true              # Default: use system keyring
private-key-file = "path/to/key"  # Alternative: key file
private-key = "base64-key"        # Alternative: inline key

# Server definitions
[server.web01]
id52 = "web01-id52-here"
allow-from = "device1-id52,device2-id52"  # Full SSH access

# Command-specific access control
[server.web01.ls]
allow-from = "readonly-device-id52"  # Can only run 'ls'

# HTTP service exposure
[server.web01.service.admin]
http = 8080
allow-from = "admin-device-id52,manager-device-id52"

[server.web01.service.api]
http = 3000
allow-from = "*"  # Public access within cluster

# Device definitions (client-only nodes)
[device.laptop]
id52 = "laptop-id52-here"

# Groups for easier management
[group.web-servers]
members = "web01,web02,web03"

[group.admins]
members = "laptop,admin-desktop"
```

## Configuration Management

### Automatic Sync
The cluster manager automatically distributes configuration updates:

1. **Change Detection**: Monitors config file hash changes
2. **Selective Distribution**: Each node receives only relevant configuration
3. **Security Filtering**: Private keys and sensitive data are never shared
4. **Incremental Updates**: Only changed configurations are synchronized

### Node-Specific Configs
Each node receives tailored configuration containing:

- **Servers**: Services they expose and who can access them
- **Devices**: Servers they can connect to and available services
- **Access Rules**: Permissions for commands and services
- **Cluster Metadata**: Node aliases, group memberships 

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

### Server Commands
```bash
# Start SSH server (accepts connections)
malai ssh server

# Start SSH server with specific config
malai ssh server --config /path/to/cluster-config.toml
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
# Start agent in background
malai ssh agent

# Get environment setup commands
malai ssh agent -e

# Start with lockdown mode
malai ssh agent --lockdown

# Start without HTTP proxy
malai ssh agent --http=false
```

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

## Getting Started

### 1. Generate Identity
```bash
# Generate keypair for this device
malai keygen
```

### 2. Set up Cluster Manager
Create a cluster configuration file and start the cluster manager:
```bash
# Edit cluster-config.toml with your cluster setup
malai ssh server --config cluster-config.toml
```

### 3. Join Devices
On client devices:
```bash
# Generate device identity
malai keygen

# Connect to cluster (cluster manager distributes config)
malai ssh cluster-manager.company.com
```

### 4. Start Agent (Recommended)
```bash
# Set up environment with agent
eval $(malai ssh agent -e)

# Now you can use SSH and HTTP services transparently
malai ssh web01.company.com "uptime"
curl admin.web01.company.com/status
```
