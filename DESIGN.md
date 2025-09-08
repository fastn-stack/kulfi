# malai: Technical Design

malai provides a secure, P2P infrastructure platform for managing clusters of machines and services over the fastn network.

## Overview

malai enables:
- Creating and managing machine clusters
- Secure remote command execution
- Protocol-agnostic service proxying over P2P connections
- Centralized configuration and access control
- Identity-aware service mesh capabilities

## Clusters

malai organizes machines into clusters. Each cluster has:

- **Cluster Manager**: A designated server that manages cluster configuration and coordinates member communication
- **Unique Identity**: Each cluster is identified by the cluster manager's id52
- **Domain Aliases**: Optional domain-based aliases for easier identification
- **DNS Integration**: Cluster manager id52 can be stored in DNS TXT records for domain-based discovery

Machines can belong to multiple clusters simultaneously, each with their own id52 keypair.

## Cluster Identification and Aliases

### **Cluster Contact Methods:**
When joining a cluster, you need to contact the cluster manager. Two methods:

1. **Direct ID52**: Use cluster manager's full ID52 (always works)
   ```bash
   malai machine init abc123def456ghi789jkl012mno345pqr678stu901vwx234 ft
   ```

2. **Domain Name**: Use domain with DNS TXT record (if configured)
   ```bash
   malai machine init fifthtry.com ft
   # DNS lookup: TXT record for fifthtry.com contains cluster manager ID52
   ```

### **Two-Level Alias System:**

#### **1. Cluster Aliases (per-cluster)**
Every cluster gets a local alias chosen during `machine init`:
- **Short aliases**: `ft` instead of `fifthtry.com` 
- **Personal naming**: Use whatever makes sense to you
- **Folder names**: Aliases become directory names in `$MALAI_HOME/ssh/clusters/`

#### **2. Global Machine Aliases (cross-cluster)**  
Edit `$MALAI_HOME/ssh/aliases.toml` for ultra-short machine access:
- **Super short**: `malai web top` instead of `malai web01.ft top`
- **Cross-cluster**: Mix machines from different clusters with unified names
- **Role-based**: `prod-web`, `staging-web`, `dev-web` for different environments
- **Service-based**: `db-primary`, `db-replica`, `monitoring` for service roles

#### **Alias Resolution Order:**
1. **Check global aliases**: `web` → `web01.ft`
2. **Check cluster.machine**: `web01.ft` → resolve cluster and machine
3. **Direct ID52**: `abc123...xyz789` → direct machine contact

### **DNS Integration (Optional):**
For domains with DNS access:
```bash
# Set TXT record: fifthtry.com TXT "malai-ssh=abc123def456..."
# Then machines can join via domain:
malai machine init fifthtry.com ft  # Resolves to cluster manager ID52
```

## System Architecture

malai runs as a **single unified process** that provides all functionality:

### **Unified malai start Process:**
- **Purpose**: All infrastructure functionality in one process
- **Runs on**: Any machine (servers, laptops, mobile devices)  
- **Auto-detects roles**: Scans MALAI_HOME for clusters and starts appropriate services
- **Integrated services**: No separate agent processes needed

### **Services Within Single Process:**

#### **1. Cluster Manager(s)** (0 or more per MALAI_HOME)
- **Triggered by**: Finding cluster-config.toml files
- **Functions**:
  - Monitor cluster-config.toml for changes
  - Distribute config updates via P2P to cluster machines
  - Maintain state.json tracking per-machine sync status
- **Mobile friendly**: Can run on iOS/Android, offline tolerance built-in

#### **2. SSH Daemon** (0 or 1 per MALAI_HOME)
- **Triggered by**: Finding machine-config.toml with SSH permissions
- **Functions**:
  - Listen for P2P SSH requests from cluster machines
  - Execute authorized commands with permission validation
  - Provide secure remote shell access

#### **3. Service Proxy** (Always runs)
- **Always active**: Handles local service access
- **Functions**:
  - **HTTP server**: Port 80, routes by `subdomain.localhost` to remote services
  - **TCP servers**: Listen on configured ports (3306, 6379), forward via P2P
  - **Identity injection**: Add client ID52 headers to HTTP requests
  - **Connection pooling**: Shared P2P connections across all clusters

### CLI to Service Communication

#### **Connection Pooling Architecture:**
- **CLI commands**: `malai web01.company ps aux` → connect to local `malai start` via Unix socket
- **Shared P2P connections**: `malai start` maintains fastn-p2p connections, CLI reuses them  
- **Performance optimization**: No new iroh connection per CLI invocation
- **Local protocol**: Custom protocol over Unix socket for CLI ↔ malai start communication

#### **CLI Communication Protocol:**
```
CLI Command: malai web01.company ps aux
    ↓
1. CLI connects to $MALAI_HOME/ssh/malai.sock
2. CLI sends: {"type": "ssh_exec", "machine": "web01.company", "command": "ps", "args": ["aux"]}
3. malai start receives, validates permissions, forwards via existing P2P connection
4. malai start sends response: {"stdout": "...", "stderr": "...", "exit_code": 0}
5. CLI displays output and exits

No P2P overhead per command - all connections pooled in malai start process.
```

#### **Fallback Behavior:**
- **malai start running**: CLI uses socket communication (fast)
- **malai start not running**: CLI creates direct P2P connection (slower, but works)

### Service Integration  
- **Remote commands**: CLI → malai start → P2P execution
- **TCP services**: `mysql -h localhost:3306` → malai start forwards via P2P
- **HTTP services**: `http://admin.localhost` → malai start routes via subdomain  
- **Unified operation**: Single `malai start` handles all P2P connections and service forwarding

## Mobile Cluster Manager

### **Mobile Infrastructure Management:**
The cluster manager can run on mobile devices (iOS/Android), enabling infrastructure management from anywhere:

#### **Mobile App Architecture:**
- **Terminal + Networking**: Single app provides both terminal interface and malai networking
- **Background execution**: App stays active when providing terminal interface
- **P2P networking**: Full fastn-p2p support for config distribution
- **iOS/Android native**: Platform-specific apps with terminal emulation

#### **Operational Model - CM Offline Tolerance:**
- **Config distribution only**: Cluster manager only needed when updating configuration
- **Machine-to-machine direct**: Operational SSH/services work without cluster manager
- **Cached configs**: Machines operate independently with last synced configuration
- **Sync when convenient**: Mobile CM comes online, syncs config changes, goes offline

#### **Mobile Use Cases:**
```bash
# On mobile (cluster manager):
malai cluster init company          # Initialize company infrastructure cluster
# Edit config in mobile app to add servers
malai start                         # Distribute config to all servers

# Daily server management from mobile:
malai web01.company systemctl status nginx
malai db01.company backup
malai web01.company deploy latest

# Infrastructure monitoring via mobile:
open http://grafana.company.localhost  # Mobile browser → remote Grafana
open http://logs.company.localhost     # Mobile browser → log analysis
```

#### **Reliability Benefits:**
- **Decentralized operations**: Servers continue operating when mobile CM offline
- **Admin flexibility**: Manage infrastructure from anywhere with mobile device
- **No single point of failure**: CM offline doesn't break machine-to-machine communication
- **Sync-when-ready**: Config changes applied when convenient, not immediately required

### **Mobile App Requirements:**
#### **Terminal Integration:**
- **Combined app**: Terminal emulator + malai networking in single iOS/Android app
- **Background persistence**: App stays active when terminal interface is active
- **Avoids backgrounding**: Prevents iOS/Android from killing networking services
- **Native platform support**: iOS and Android specific implementations

#### **Operational Advantages:**
- **Always-available terminal**: Mobile terminal ensures malai commands always work
- **No background restrictions**: App doesn't need kernel drivers or special permissions
- **Infrastructure on-the-go**: Manage servers from anywhere with mobile connectivity
- **Emergency management**: Critical infrastructure fixes possible from mobile device

#### **Implementation Considerations:**
- **Platform-specific builds**: iOS app store and Android app distributions
- **Terminal emulation**: Full bash/shell support within mobile app
- **P2P networking**: Complete fastn-p2p implementation for mobile platforms
- **Config editing**: Mobile-friendly UI for cluster configuration management

## Addressing and Aliases

### Machine Addressing
Each machine has multiple addressing options:

- **Domain-based**: `machine-alias.cluster-domain.com` (when domain is available)
- **ID-based**: `machine-alias.cluster-id52` (always works)
- **Full ID**: `machine-id52.cluster-id52` (direct addressing)

### Service Addressing

#### **Cluster-Global Unique Service Names:**
Services have cluster-global unique names (no machine prefix needed):

- **HTTP**: `admin.company` → routes to admin service in company cluster
- **TCP**: `mysql.company:3306` → routes to mysql service in company cluster
- **Service-only**: `grafana.ft` → routes to grafana service in ft cluster

#### **Full localhost URL Structure:**
For HTTP services, the complete URL format is:
`http://<service>.<cluster>.localhost[:<port>]`

**Examples:**
- `http://admin.company.localhost` → admin service in company cluster
- `http://grafana.ft.localhost` → grafana service in ft cluster  
- `http://api.personal.localhost` → api service in personal cluster
- `http://admin.abc123def456.localhost` → admin service in cluster with ID52 abc123def456

#### **URL Parsing Rules:**
Agent parses `subdomain.localhost` requests as:
1. **Simple**: `admin.company.localhost` → service=admin, cluster=company
2. **Domain cluster**: `api.mycompany.com.localhost` → service=api, cluster=mycompany.com  
3. **ID52 cluster**: `grafana.abc123def456.localhost` → service=grafana, cluster=abc123def456
4. **Complex domain**: `api.aws.mycompany.com.localhost` → service=api, cluster=aws.mycompany.com

#### **Service Resolution:**
1. Parse subdomain: extract service name and cluster identifier
2. Lookup cluster: resolve cluster ID52 from cluster identifier  
3. Find service: locate service in cluster config
4. Route request: forward to `service.machine-running-service` via P2P

## Protocol-Agnostic Service Proxying

Machines can expose services through the malai network using any protocol:

### **TCP Services** (protocol-agnostic)
- **Database access**: MySQL, PostgreSQL, Redis via TCP tunneling
- **Raw protocols**: Any TCP service can be proxied
- **Port forwarding**: Direct TCP connection between authorized machines

### **HTTP Services** (enhanced features)
- **Header injection**: Automatic `malai-client-id52` header for application-level ACL (default: enabled)
- **Transparent proxying**: Services appear as if running locally to authorized clients  
- **HTTPS support**: Optional secure flag for encrypted HTTP services
- **Application integration**: Local services can implement ACL using injected client ID52
- **Disable injection**: Set `inject_headers = false` for public APIs that shouldn't receive identity

**HTTP Header Injection Example:**
```toml
[machine.api-server.http.admin]
port = 8080
# inject_headers = true (default)
allow_from = "admins,developers"

[machine.api-server.http.public]
port = 3000
inject_headers = false               # Disable for public API
allow_from = "*"
```

**Application receives request with:**
```http
GET /admin/users HTTP/1.1
Host: admin.api-server.company
malai-client-id52: abc123def456ghi789...
malai-client-machine: laptop
malai-client-cluster: company
Authorization: Bearer original-token
```

**Application-level ACL:**
```python
# Your application can now do sophisticated ACL
client_id52 = request.headers.get('malai-client-id52')
client_machine = request.headers.get('malai-client-machine')

if client_id52 in allowed_admin_ids:
    return admin_data()
else:
    return forbidden()
```

### **Service Benefits:**
- **Protocol flexibility**: HTTP, TCP, or any other protocol
- **Access control**: Per-service permissions with group support
- **Port conflict resolution**: No port management needed across cluster
- **Enhanced HTTP**: Client identity headers for sophisticated app-level authorization

## Config File Format

```toml
# Cluster manager configuration
[cluster-manager]
id52 = "cluster-manager-id52-here"
cluster_name = "company"

# Machine definitions
[machine.web01]
id52 = "web01-id52-here"
allow_from = "admins,devs"           # Who can run commands on this machine
allow_shell = "admins"               # Who can start interactive shells (default: same as allow_from)
username = "webservice"              # Run commands as this user (default: same as agent user)

# Command-specific access control
[machine.web01.command.sudo]
allow_from = "admins"                # Only admin group can run sudo
username = "root"                    # Run as root user

[machine.web01.command.restart-nginx]
allow_from = "admins,on-call-devs"   # Custom command with alias
command = sudo systemctl restart nginx  # Actual command to execute
username = "nginx"                   # Run as nginx user

[machine.web01.command.top]
allow_from = "devs"                  # Simple command (uses command name as-is)
# username not specified = inherits from machine.username or agent user

# Machine configuration (no services defined here)
[machine.web01]
id52 = "web01-id52-here"
allow_from = "admins,devs"           # Who can run commands on this machine
allow_shell = "admins"               # Who can start interactive shells
username = "webservice"              # Default user for commands

# Cluster-global unique services (specify which machine runs them)
[service.mysql]
machine = "db01"                     # Which machine runs this service
protocol = "tcp"
port = 3306
allow_from = "backend-services,admins"

[service.admin]
machine = "web01" 
protocol = "http"
port = 8080
allow_from = "admins"
# inject_headers = true (default for HTTP)

[service.api]
machine = "web01"
protocol = "http"  
port = 3000
allow_from = "*"
inject_headers = false               # Disable for public API

[service.redis]
machine = "cache01"
protocol = "tcp"
port = 6379
allow_from = "backend-services"

# Client-only machine (no accept_ssh = true)
[machine.laptop]
id52 = "laptop-id52-here"

# Hierarchical Group System
[group.admins]
members = "laptop-id52,admin-desktop-id52"

[group.devs]  
members = "dev1-id52,dev2-id52,junior-devs"  # Can include other groups

[group.junior-devs]
members = "intern1-id52,intern2-id52"

[group.web-servers]
members = "web01,web02,web03"               # Machine aliases

[group.all-staff]
members = "admins,devs,web-servers"         # Group hierarchies
```

## Access Control System

### **Access Control Levels:**
SSH access is controlled at multiple levels for fine-grained security:

1. **Command Execution**: `allow_from` - Who can run specific commands
2. **Interactive Shell**: `allow_shell` - Who can start full shell sessions (defaults to same as allow_from if not specified)  
3. **Machine Inclusion**: Any machine in config accepts SSH connections (no accept_ssh flag needed)
4. **Username Control**: `username` field specifies execution user (hierarchical inheritance)

### **User Execution Context:**
Commands can run as different users based on a hierarchy of username settings:

**Username Resolution Order:**
1. **Command-level**: `[machine.X.command.Y] username = "specific-user"`
2. **Machine-level**: `[machine.X] username = "machine-user"`  
3. **Agent default**: Same user that runs `malai start`

**Examples:**
- `malai web01 restart-nginx` → runs as `nginx` user (command-level override)
- `malai web01 top` → runs as `webservice` user (machine-level default)  
- `malai database restart-db` → runs as `postgres` user (command-level override)

**Security Benefits:**
- **Privilege separation**: Different commands can run as appropriate service users
- **Least privilege**: Commands only get the permissions they need
- **Service account usage**: Integrate with existing system user management

```toml
[machine.production-db]
id52 = "db-machine-id52"
allow_from = "admins,devops"        # Can run commands
allow_shell = "senior-admins"       # Only senior admins get shell access  
username = "postgres"               # All commands run as postgres user

[machine.web01]
id52 = "web01-id52"  
allow_from = "*"                    # Everyone can run commands
# allow_shell defaults to same as allow_from ("*")
# username not specified = runs as same user as agent

[machine.restricted]
id52 = "restricted-id52"
# No allow_from = no SSH access to this machine
```

### **allow_from Field Syntax:**
The `allow_from` field supports flexible access control with individual IDs, groups, and wildcards:

- **Individual machine IDs**: `"machine1-id52,machine2-id52"`  
- **Group names**: `"admins,devs"`
- **Mixed syntax**: `"admins,machine1-id52,contractors"`
- **Wildcard**: `"*"` (all cluster machines)

### **Hierarchical Group System:**
Groups can contain both individual machine IDs and other groups, enabling flexible organizational structures:

```toml
# Leaf groups (contain only machine IDs)
[group.senior-devs]
members = "alice-id52,bob-id52"

[group.junior-devs] 
members = "charlie-id52,diana-id52"

# Parent groups (contain other groups)
[group.all-devs]
members = "senior-devs,junior-devs"

# Department groups (mix of individuals and groups)
[group.engineering]
members = "all-devs,lead-architect-id52"

# Company-wide groups
[group.everyone]
members = "engineering,marketing,sales"
```

### **Group Resolution:**
When processing `allow_from`, the system recursively expands groups:
1. **Direct IDs**: `machine1-id52` → match immediately
2. **Group expansion**: `admins` → expand to all members recursively
3. **Nested groups**: `all-staff` → `admins,devs` → individual IDs
4. **Wildcard**: `*` → all machines in cluster

### **Access Control Examples:**
```toml
# SSH access for admin tasks
[machine.production-server]
allow_from = "admins,on-call-devs"

# Cluster-global services (unique names, specify hosting machine)
[service.postgres]
machine = "database"  
protocol = "tcp"
port = 5432
allow_from = "backend-services,admins"

[service.internal-api]
machine = "web01"
protocol = "http"
port = 5000
allow_from = "backend-services,monitoring-id52"
secure = true                        # HTTPS endpoint
# inject_headers = true (default)

[service.redis]
machine = "cache01"
protocol = "tcp"
port = 6379
allow_from = "backend-services"

# Command aliases and restrictions
[machine.database.command.restart-db]
allow_from = "senior-admins"         # Only senior admins can run this
command = "sudo systemctl restart postgresql"  # Actual command executed
username = "postgres"                # Run as postgres user

[machine.web01.command.deploy]
allow_from = "devs,ci-cd-id52"
command = "/opt/deploy/deploy.sh production"  # Custom deployment script
username = "deploy"                  # Run as deploy user (safer than root)

[machine.web01.command.logs]
allow_from = "devs,support"
command = tail -f /var/log/nginx/access.log
# username not specified = inherits from machine.username
```

## Command System

### **Command Execution Syntax:**
```bash
# Direct commands (natural SSH-like syntax)
malai web01.cluster top
malai web01.cluster ps aux

# Command aliases (defined in config)
malai web01.cluster restart-nginx   # Executes: sudo systemctl restart nginx  
malai database.cluster restart-db   # Executes: sudo systemctl restart postgresql

# Interactive shell (requires allow_shell permission)
malai web01.cluster                 # Starts interactive shell session

# Alternative explicit syntax also supported
malai exec web01.cluster "top"
malai shell web01.cluster
```

### **Command Configuration:**
- **Simple commands**: Use command name as-is (e.g., `top`, `ps`, `ls`)
- **Command aliases**: Map friendly name to actual command
- **Security benefit**: Hide complex commands behind simple aliases
- **Access control**: Each command/alias has separate `allow_from` permissions

### **Command vs Alias Resolution:**
1. **Check alias first**: If `[machine.X.command.CMD]` exists → use `command = "..."` 
2. **Fallback to direct**: If no alias → execute `CMD` directly
3. **Permission check**: Verify client in `allow_from` for that specific command/alias

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
malai web01.company.com "ps aux"

# Interactive SSH session
malai web01.company.com

# Using ID-based addressing
malai web01.cluster-id52 systemctl status nginx
```

### Single Cluster Per MALAI_HOME
- Each MALAI_HOME directory represents one machine in one cluster
- Multi-cluster support via multiple MALAI_HOME environments
- Clear separation: one cluster identity per MALAI_HOME instance
- No complex multi-cluster management needed

### Cluster Registration Security
- Machines store verified cluster manager ID52 in `cluster-info.toml`
- All config updates must come from verified cluster manager
- DNS TXT record integration for automatic cluster manager discovery
- Cryptographic proof required for cluster manager verification

## Command Reference

### Initialization Commands
```bash
# Initialize a new cluster (generates cluster manager identity)
malai cluster init <cluster-alias>
# Example: malai cluster init company
# Creates: $MALAI_HOME/ssh/clusters/company/ with cluster manager config

# Join existing cluster as machine (contacts cluster manager)
malai machine init <cluster-id52-or-domain> <local-alias>
# Examples:
malai machine init abc123def456ghi789... company     # Using cluster manager ID52
malai machine init fifthtry.com ft                  # Using domain (if DNS configured)
# Creates: $MALAI_HOME/ssh/clusters/ft/ with machine config and registration
```

### Unified Service Management
```bash
# Start all SSH services (auto-detects roles across all clusters)
malai start
# Scans $MALAI_HOME/ssh/clusters/ and starts:
# - Cluster manager for clusters where this machine is manager
# - SSH daemon for clusters where this machine accepts SSH
# - Client agent for connection pooling across all clusters
# Environment: malai start -e

# Show information for all clusters
malai info
# Shows role and status for each cluster this machine participates in

# Local service management
malai service add ssh web web01.ft                    # Add SSH alias  
malai service add tcp mysql 3306 mysql.db01.ft:3306   # Add TCP forwarding
malai service add http admin admin.web01.ft           # Add HTTP subdomain route
malai service remove mysql                            # Remove service
malai service list                                    # List all configured services
```

### SSH Execution Commands
```bash
# Execute command on remote machine (natural SSH syntax)
malai <machine-address> <command>
# Examples:
malai web01.company systemctl status nginx
malai web01.cluster-id52 ps aux

# Interactive shell session
malai <machine-address>
# Example: malai web01.company

# Alternative explicit syntax
malai exec web01.company uptime
malai shell web01.company
```

### Agent Commands
```bash
# Start agent in background (handles all SSH functionality automatically)
# Requires MALAI_HOME to be set or uses default location
malai start

# Get environment setup commands for shell integration
malai start -e

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
malai curl admin.web01.company.com/api

# Equivalent to:
# HTTP_PROXY=<agent-proxy> curl admin.web01.company.com/api
```

## Agent Environment Setup

### Shell Integration
The agent outputs environment variables in `ENV=value` format for shell evaluation:

```bash
# Start agent and configure environment
eval $(malai start -e)

# With specific options
eval $(malai start -e --lockdown --http)

# Disable HTTP proxy
eval $(malai start -e --http=false)
```

### Persistent Setup
Add to your shell profile (`.bashrc`, `.zshrc`, etc.):
```bash
# Enable malai start on shell startup
eval $(malai start -e)
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
eval $(malai start -e)  # Agent auto-detects role and starts appropriate services
```

**Multi-Cluster Directory Structure:**
```
$MALAI_HOME/
├── ssh/
│   ├── clusters/
│   │   ├── company/                 # Local alias for cluster
│   │   │   ├── cluster-config.toml  # Full cluster config (if cluster manager)
│   │   │   ├── machine-config.toml  # Machine-specific config (if regular machine)
│   │   │   ├── cluster-info.toml    # Cluster details and registration
│   │   │   ├── identity.key         # This machine's identity for this cluster
│   │   │   └── state.json          # Config distribution state (cluster manager only)
│   │   ├── ft/                      # Local alias for fifthtry.com cluster  
│   │   │   ├── cluster-info.toml    # Contains cluster_id52, domain, role
│   │   │   ├── machine-config.toml  # Received from cluster manager
│   │   │   └── identity.key         # Machine identity for this cluster
│   │   └── personal/                # Personal cluster alias
│   │       └── ...
│   ├── services.toml                # Local services: aliases + port forwarding
│   ├── malai.sock                   # CLI communication socket (malai commands → malai start)
│   └── malai.lock                   # Process lockfile
└── keys/
    └── default-identity.key         # Default identity for new clusters

# Logs stored in standard system log directories per cluster:
# - ~/.local/state/malai/ssh/logs/company/
# - ~/.local/state/malai/ssh/logs/ft/
# - ~/.local/state/malai/ssh/logs/personal/
```

**cluster-info.toml Example:**
```toml
# Cluster registration information
cluster_alias = "ft"                               # Local alias
cluster_id52 = "abc123def456ghi789..."            # Cluster manager ID52
domain = "fifthtry.com"                           # Original domain (if used)
role = "machine"                                   # cluster-manager, machine, or client-only
machine_alias = "dev-laptop-001"                  # This machine's alias in cluster
```

**services.toml - Unified Local Services Configuration:**
```toml
# SSH aliases for convenient access across all clusters
[ssh]
web = "web01.ft"                    # malai web top
db = "db01.ft"                      # malai db pg_stat_activity  
home = "home-server.personal"       # malai home htop
monitoring = "grafana.ft"           # malai monitoring restart

# TCP port forwarding (agent listens on local ports, forwards via P2P)
[tcp]
mysql = { local_port = 3306, remote = "mysql.db01.ft:3306" }
redis = { local_port = 6379, remote = "redis.cache01.ft:6379" }
postgres = { local_port = 5432, remote = "postgres.db01.company:5432" }

# HTTP subdomain routing (agent listens on port 80/8080, routes by Host header)
[http]
# Agent listens on localhost:80 and routes based on subdomain
port = 80                                    # Agent HTTP port (80 or 8080)
routes = {
    "admin" = "admin.web01.ft",              # admin.localhost → admin.web01.ft
    "api" = "api.web01.ft",                  # api.localhost → api.web01.ft  
    "db-admin" = "admin.db01.company",       # db-admin.localhost → admin.db01.company
    "grafana" = "grafana.monitoring.ft",     # grafana.localhost → grafana.monitoring.ft
}
inject_headers = true                        # Default: add client ID52 headers
public_routes = ["api"]                      # These routes don't get identity headers
```

**Usage after agent starts:**
```bash
# SSH with aliases:
malai web systemctl status nginx

# Direct TCP connections:
mysql -h localhost:3306              # → mysql.db01.ft:3306 via P2P
redis-cli -p 6379                    # → redis.cache01.ft:6379 via P2P
psql -h localhost -p 5432            # → postgres.db01.company:5432 via P2P

# HTTP via subdomain routing (browser-friendly):
curl http://admin.localhost/users    # → admin.web01.ft (gets client ID52 header)
curl http://api.localhost/metrics    # → api.web01.ft (gets client ID52 header)
curl http://grafana.localhost/dash   # → grafana.monitoring.ft (gets client ID52 header)

# Browser access (works in any browser):
http://admin.localhost               # Direct browser access to remote admin interface
http://grafana.localhost             # Direct browser access to remote Grafana
```

**Agent Service Forwarding:**
- **TCP port binding**: Agent listens on configured local ports (3306, 6379, etc.)
- **HTTP subdomain routing**: Agent listens on port 80, routes by `Host: subdomain.localhost` header
- **P2P forwarding**: All connections forwarded to remote services via encrypted P2P
- **Browser compatibility**: `http://admin.localhost` works directly in any browser
- **Identity injection**: HTTP requests automatically get client ID52 headers
- **Service discovery**: Automatic connection routing based on services.toml configuration
- **Multi-cluster access**: Single agent can forward to services across all clusters

**Multi-Cluster Benefits:**
- **Single agent**: Manages all SSH connections and service forwarding across clusters
- **Unified proxy**: Access services from any cluster via localhost ports
- **Role flexibility**: Can be cluster manager of one, machine in another
- **Isolated configs**: Each cluster has separate configuration and identity

## Config Distribution State Management

### **state.json Structure (Cluster Manager Only):**
```json
{
  "cluster_alias": "company",
  "cluster_config_hash": "abc123def456",
  "last_distribution": "2025-01-15T10:30:00Z",
  "machine_states": {
    "web01-machine-id52": {
      "machine_alias": "web01",
      "last_config_hash": "abc123def456",
      "last_sync": "2025-01-15T10:30:00Z",
      "sync_status": "success"
    },
    "db01-machine-id52": {
      "machine_alias": "db01", 
      "last_config_hash": "old456def789",
      "last_sync": "2025-01-15T09:45:00Z",
      "sync_status": "pending"
    }
  }
}
```

### **Config Distribution Algorithm:**
1. **Monitor config**: Watch cluster-config.toml for file changes
2. **Calculate hash**: Hash current config content
3. **Compare states**: Check which machines have outdated config hash
4. **Distribute updates**: Send new config to machines with old hash via P2P
5. **Update state**: Record successful distribution and new hash per machine

### **Multi-Cluster State:**
Each cluster directory has its own state.json:
- `$MALAI_HOME/ssh/clusters/company/state.json` 
- `$MALAI_HOME/ssh/clusters/ft/state.json`
- `$MALAI_HOME/ssh/clusters/personal/state.json`

### **Unified malai start Architecture:**
Single process handles everything:

1. **Scan MALAI_HOME**: Find cluster configs and machine configs in all cluster directories
2. **Start cluster managers**: For each cluster-config.toml found (0 or more per MALAI_HOME)
3. **Start SSH daemon**: If any machine-config.toml indicates SSH acceptance (0 or 1 per MALAI_HOME)  
4. **Start service proxy**: Always runs - handles TCP ports + HTTP subdomain routing
5. **Integrated operation**: No separate malai agent process needed

### **Service Integration in Single Process:**
- **HTTP server**: Listen on port 80, route by `subdomain.localhost` to remote services
- **TCP servers**: Listen on configured ports (3306, 6379, etc.), forward to remote services
- **Cluster manager poller**: Monitor config changes, distribute via P2P  
- **SSH P2P listener**: Accept remote commands via fastn-p2p
- **All services**: Run in same process with shared connection pool and identity management

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
malai init-cluster --alias test-cluster
# Outputs: "Cluster created with ID: abc123..."
eval $(malai start -e)  # Start agent (automatically runs as cluster manager)
```

**3. Initialize Server Machine (Terminal 2):**
```bash
export MALAI_HOME=/tmp/malai-test/server1
malai init  # Generate machine identity (NO config yet)
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
eval $(malai start -e)  # Agent receives config and auto-detects SSH server role
```

**6. Create Client Machine (Terminal 3):**
```bash
export MALAI_HOME=/tmp/malai-test/client1
malai identity create  # Generate client identity
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
eval $(malai start -e)  # Start agent (automatically runs as client)
malai web01.test-cluster "echo 'Hello from remote server!'"
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
malai init-cluster --alias company-cluster
eval $(malai start -e)  # Runs as cluster manager automatically
```

**Test Cluster:**
```bash
export MALAI_HOME=/tmp/malai-test/test-cluster
malai init-cluster --alias test-cluster
eval $(malai start -e)  # Runs as different cluster manager
```

**Client with Access to Both:**
```bash
export MALAI_HOME=/tmp/malai-test/multi-client
eval $(malai start -e)
malai web01.company.com "uptime"
malai test-server.test.local "ps aux"
```

### Test Scenarios

**1. Permission Testing:**
```bash
# Test command restrictions
malai restricted-server.cluster.local "ls"  # Should work
malai restricted-server.cluster.local "rm file"  # Should fail
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
eval $(malai start -e --lockdown --http)
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
pkill -f "malai"

# Clean up test directories
rm -rf /tmp/malai-test/
```

## Getting Started

### Production Setup

**1. Initialize Cluster (on cluster manager machine):**
```bash
malai init-cluster --alias company-cluster
# Outputs: "Cluster created with ID: <cluster-manager-id52>"
eval $(malai start -e)  # Start agent in background
```

**2. Initialize Machines:**
```bash
# On each machine that should join the cluster:
malai init  # Generate identity for this machine
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
eval $(malai start -e)
# Agent automatically:
# - Receives config from cluster manager
# - Detects its role (cluster-manager/SSH server/client-only)
# - Starts appropriate services
```

**4. Use SSH:**
```bash
malai web01.company-cluster "uptime"
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
malai identity create  # Creates identity in $MALAI_HOME
```

**3. Test Multi-Node Setup:**
```bash
# Terminal 1 - Create Cluster
export MALAI_HOME=/tmp/malai-cluster-manager
malai create-cluster --alias test-cluster
# Note the cluster ID output: "Cluster created with ID: abc123..."
eval $(malai start -e)  # Auto-runs as cluster manager

# Terminal 2 - Create Server Machine
export MALAI_HOME=/tmp/malai-server1
malai create-machine
# Note the machine ID output: "Machine created with ID: def456..."

# Terminal 1 - Add Server to Cluster Config
# Edit cluster config to add:
# [server.web01]
# id52 = "def456..."
# allow_from = "*"
# Config automatically syncs to Terminal 2

# Terminal 2 - Server Starts Automatically  
eval $(malai start -e)  # Agent detects role and starts SSH server

# Terminal 3 - Create and Add Client
export MALAI_HOME=/tmp/malai-client1
malai create-machine
# Add this ID to cluster config as [device.laptop]
eval $(malai start -e)
malai web01.test-cluster "echo 'Multi-node test successful!'"
```

This approach allows you to test complex multi-cluster scenarios, permission systems, and service configurations entirely on a single development machine.

## Real-World Usage Examples

### Example 1: Personal Infrastructure Cluster

**Setup (one-time):**
```bash
# On my laptop (cluster manager):
malai cluster init personal
# Edit $MALAI_HOME/ssh/clusters/personal/cluster-config.toml to add machines
malai start &  # Starts cluster manager + client agent

# On home server:
malai machine init personal  # Contacts cluster, registers
# Laptop admin adds machine to personal cluster config
malai start &  # Starts SSH daemon + client agent

# Both machines now participate in 'personal' cluster
```

**Daily usage:**
```bash
# Direct SSH commands (natural syntax):
malai home-server.personal htop
malai home-server.personal docker ps  
malai home-server.personal sudo systemctl restart nginx

# HTTP services:
curl admin.home-server.personal/api
```

### Example 2: Fastn Cloud Cluster

**Setup:**
```bash
# On fastn-ops machine (cluster manager):
malai cluster init ft
# Edit $MALAI_HOME/ssh/clusters/ft/cluster-config.toml
malai start  # Starts cluster manager

# On each fastn server:
malai machine init fifthtry.com ft  # Join via domain, use short alias
# fastn-ops adds machine to cluster config
malai start  # Starts SSH daemon

# On developer laptops:
malai machine init <cluster-manager-id52> ft  # Join via ID52, short alias
malai start  # Starts client agent for connection pooling
```

**Daily operations:**
```bash
# Server management (using short alias):
malai web01.ft systemctl status nginx
malai db01.ft restart-postgres  # Command alias

# Monitoring:
malai web01.ft tail -f /var/log/nginx/access.log

# HTTP services (using short alias):
curl api.web01.ft/health
curl grafana.monitoring.ft/dashboard
```

### Example 3: Multi-Cluster Power User

**Setup (same machine in multiple clusters):**
```bash
# Initialize participation in multiple clusters:
malai cluster init personal                           # Create personal cluster (cluster manager)
malai machine init company.example.com company       # Join company cluster (via domain)
malai machine init abc123def456ghi789... ft          # Join fifthtry cluster (via ID52, alias "ft")

# Single unified start:
malai start  # Automatically starts:
                 # - Cluster manager for 'personal'
                 # - SSH daemon for 'company' and 'fastn-cloud'  
                 # - Client agent for all three clusters
```

**Multi-cluster daily usage:**
```bash
# Ultra-short commands using global aliases:
malai home htop                    # home = home-server.personal
malai web systemctl status nginx  # web = web01.company
malai db pg_stat_activity         # db = db01.ft

# Or use cluster.machine format:
malai home-server.personal htop
malai web01.company systemctl status nginx  
malai db01.ft pg_stat_activity

# Cross-cluster services via agent forwarding:
curl http://admin.personal.localhost/dashboard  # → admin service in personal cluster (+ client ID52 header)
curl http://api.company.localhost/metrics       # → api service in company cluster (+ client ID52 header)
mysql -h localhost:3306                         # → mysql service via TCP forwarding
redis-cli -p 6379                              # → redis service via TCP forwarding

# Browser access (explicit cluster.service.localhost):
open http://grafana.ft.localhost               # Grafana service in ft cluster
open http://admin.company.localhost           # Admin service in company cluster
open http://mysql-admin.personal.localhost    # MySQL admin interface in personal cluster
```

### Example 4: Power User Alias Setup

**After joining multiple clusters, set up personal services:**
```bash
# Set up SSH aliases and service forwarding:
malai service add ssh web web01.ft
malai service add ssh db db01.ft
malai service add tcp mysql 3306 mysql.db01.ft:3306
malai service add http admin admin.web01.ft
malai service add http grafana grafana.monitoring.ft

# Now ultra-convenient access:
malai web systemctl status nginx    # SSH via alias
malai db backup                     # SSH via alias
mysql -h localhost:3306                 # Direct MySQL access
open http://admin.localhost             # Browser access to admin interface
open http://grafana.localhost           # Browser access to monitoring
```

**Workflow benefits:**
- **Instant access**: 3-4 characters instead of full machine.cluster names
- **Personal choice**: Aliases match your workflow and preferences
- **Cross-cluster**: Mix machines from different clusters with unified naming
- **Future-proof**: Change underlying machines without changing aliases

### User Experience Summary

**Onboarding a new machine** (2 commands):
1. `malai machine init company` → register with cluster
2. `malai start` → auto-starts all appropriate services

**Multi-cluster management** (unified):
- Single `malai start` handles all cluster roles
- Cross-cluster SSH access with cluster.machine addressing
- Unified HTTP proxy across all clusters

**Daily SSH usage** (ultra-convenient):
- `malai web ps aux` (global alias) or `malai web01.company ps aux` (full form)
- No quotes needed for commands (like real SSH)
- Personal aliases: `malai db backup` much better than `malai db01.fifthtry.com backup`
- Single agent optimizes connections across all clusters

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
malai create-cluster --alias test-cluster
CLUSTER_ID=$(malai info | grep "Cluster ID" | cut -d: -f2)

# 2. Create SSH server
export MALAI_HOME=$TEST_DIR/server1  
malai identity create
SERVER_ID=$(malai identity create | grep "ID52" | cut -d: -f2)

# 3. Add server to cluster config
export MALAI_HOME=$TEST_DIR/manager
echo "[machine.web01]
id52 = \"$SERVER_ID\"
accept_ssh = true
allow_from = \"*\"" >> $MALAI_HOME/ssh/cluster-config.toml

# 4. Start agents
export MALAI_HOME=$TEST_DIR/manager && malai start &
export MALAI_HOME=$TEST_DIR/server1 && malai start &
sleep 2  # Wait for config sync

# 5. Test SSH execution
export MALAI_HOME=$TEST_DIR/client1
malai identity create
CLIENT_ID=$(malai identity create | grep "ID52" | cut -d: -f2)

# Add client to config
export MALAI_HOME=$TEST_DIR/manager  
echo "[machine.client1]
id52 = \"$CLIENT_ID\"" >> $MALAI_HOME/ssh/cluster-config.toml

# Wait for sync and test
export MALAI_HOME=$TEST_DIR/client1
eval $(malai start -e)
malai web01.test-cluster "echo 'SSH test successful'"

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
malai web01.test-cluster "ls"        # Should succeed
malai web01.test-cluster "whoami"    # Should fail with permission denied
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
malai create-cluster --alias company

export MALAI_HOME=/tmp/test-dev-cluster  
malai create-cluster --alias dev

# Create client with access to both clusters
export MALAI_HOME=/tmp/test-multi-client
# Copy both cluster configs or implement multi-cluster client support

# Test cross-cluster access isolation
malai company-server.company "uptime"  # Should work
malai dev-server.dev "uptime"          # Should work  
malai company-server.dev "uptime"      # Should fail (wrong cluster)
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

## Security Model

### **Threat Model and Mitigations**

#### **1. Identity and Authentication**
- **Machine Identity**: Each machine has unique ID52 cryptographic identity
- **Closed Network**: Only cluster members can connect (unknown machines rejected at P2P level)
- **P2P Cryptographic Verification**: fastn-p2p automatically verifies both sender and receiver using public keys
- **No Brute Force Possible**: Unknown attackers cannot even establish connections

#### **2. Configuration Security**  
- **Sender Verification**: Machines automatically verify config sender ID52 via fastn-p2p
- **Authenticated Channel**: Config distribution uses cryptographically verified P2P channels
- **Machine Authorization**: Machines only process config sections containing their own verified ID52

#### **3. Command Execution Security**
- **Authenticated Requests**: All SSH requests cryptographically verified via fastn-p2p
- **Permission Enforcement**: Multi-level access control (machine → command → group)
- **Safe Execution**: Direct process execution without shell interpretation
- **User Context**: Commands run as specified username with proper privilege separation

#### **4. Access Control**
- **Hierarchical Groups**: Recursive group expansion with loop detection
- **Principle of Least Privilege**: Granular permissions per command/service
- **Shell vs Command Access**: Separate permissions for interactive shells vs command execution

### **Security Implementation Checklist:**

**CRYPTOGRAPHICALLY SECURE (via fastn-p2p):**
- ✅ **Authentication**: fastn-p2p verifies both parties using ID52 public keys
- ✅ **Config authenticity**: Sender identity verified automatically  
- ✅ **Transport security**: End-to-end encryption provided by P2P layer
- ✅ **No replay attacks**: fastn-p2p handles session security

**STILL REQUIRED:**
- [ ] **Command injection protection**: Safe argument parsing and execution
- [ ] **Username validation**: Prevent privilege escalation via username field
- [ ] **Group loop detection**: Prevent infinite recursion in group expansion
- [ ] **Config content validation**: Validate TOML structure and permissions
- [ ] **DNS TXT integration**: Automatic cluster manager discovery

**MEDIUM:**
- [ ] **Rate limiting**: Prevent SSH command flooding attacks
- [ ] **Audit logging**: Security event logging for compliance
- [ ] **Session timeouts**: Automatic session expiration
- [ ] **Failed authentication handling**: Lockout after failed attempts

### **Security Implementation Status:**
- ✅ **Cryptographically secure foundation**: fastn-p2p provides enterprise-grade authentication
- 🟡 **Application-level security needed**: Command validation and input sanitization required
- 🎯 **Security model**: Stronger than OpenSSH (no certificate authorities needed, direct cryptographic verification)

## Security Implementation Notes

### **fastn-p2p Security Foundation:**
The malai SSH system builds on fastn-p2p's cryptographic foundation:

- **Automatic Identity Verification**: Every P2P call cryptographically verifies both sender and receiver
- **End-to-End Encryption**: All communication channels encrypted by default
- **No Certificate Authorities**: Direct public key verification (stronger than traditional CA model)
- **Session Security**: fastn-p2p handles connection security and prevents replay attacks

### **Application Security Requirements:**
While fastn-p2p handles transport security, malai SSH must implement:

1. **Safe Command Execution**: Direct process execution without shell interpretation
2. **Input Validation**: Validate usernames, command arguments, and config content
3. **Permission Enforcement**: Hierarchical group resolution with loop detection
4. **Config Authorization**: Only accept config containing machine's own verified ID52

### **Security Advantages over Traditional SSH:**
- **No host key management**: P2P identities replace SSH host keys
- **No certificate authorities**: Direct cryptographic verification
- **Closed network model**: Only cluster members can connect (fastn-p2p rejects unknown machine ID52s at transport level)
- **No brute force attacks**: Only known machine ID52s can even establish connections
- **No password attacks**: Cryptographic identity required for any communication
- **Automatic key rotation**: P2P layer can handle key updates
- **Perfect forward secrecy**: Each session uses fresh cryptographic material

The foundation is cryptographically stronger than OpenSSH - we just need application-level input validation.

## Strategic Design Insight: malai SSH IS Complete malai

### **Design Revelation:**
What we've built as "malai" actually fulfills the complete malai vision:

**malai 0.3 planned features:**
- Multiple services in single process ✅
- User-controlled configuration ✅  
- Identity management ✅
- Service orchestration ✅

**Our "malai" provides all this PLUS:**
- Secure remote access (SSH functionality)
- Multi-cluster enterprise capabilities
- Identity-aware service mesh
- Cryptographic security model
- Natural command syntax

### **Command Structure Evolution:**
**Current nested structure:**
```bash
malai cluster init company
malai machine init company corp
malai start  
malai web01.company ps aux
```

**Should become top-level:**
```bash
malai cluster init company          # Promote to top-level
malai machine init company corp     # Promote to top-level  
malai start                         # Replaces both 'malai run' and 'malai start'
malai web01.company ps aux          # Direct SSH execution (no 'ssh' prefix)

# Keep legacy single-service mode:
malai http 8080 --public            # Backwards compatibility
malai tcp 3306 --public             # Backwards compatibility
```

### **Identity Management Integration:**
Replace `malai identity create` with richer identity system from 0.3 plan:
```bash
malai identity create [name]         # Replace keygen  
malai identity list                  # List all identities
malai identity export name           # Export identity for sharing
malai identity import file           # Import identity
malai identity delete name           # Remove identity
```

### **Module Organization Decision:**
- Keep `malai/src/ssh/` module name (avoid massive reorganization)
- Promote SSH functions to top-level malai API  
- Update CLI command structure to reflect core status
- Maintain backwards compatibility with existing commands

### **Documentation Strategy:**
- Current `ssh/README.md` contains complete design (preserve all content)
- Main README.md should become user-focused overview  
- Consider moving design to malai.sh website for public access
- DESIGN.md for technical contributors

This positioning makes malai much more compelling - it's not just another tool, but a complete secure infrastructure platform.

## Latest Design Insights Captured

### **HTTP Subdomain Routing Architecture:**
Agent listens on port 80/8080 and routes HTTP requests by subdomain:
- `http://admin.localhost` → `admin.web01.ft` (automatic routing)
- `http://grafana.localhost` → `grafana.monitoring.ft`  
- Browser-native access without proxy configuration
- Automatic client ID52 header injection for app-level ACL

### **Unified services.toml Configuration:**
```toml
# SSH aliases for convenient access
[ssh]
web = "web01.ft"
db = "db01.ft"

# TCP port forwarding  
[tcp]
mysql = { local_port = 3306, remote = "mysql.db01.ft:3306" }
redis = { local_port = 6379, remote = "redis.cache01.ft:6379" }

# HTTP subdomain routing (agent listens on port 80)
[http]
port = 80
# Routes map localhost subdomains to cluster-global services
# Format: "service.cluster.localhost" → service in cluster
routes = {
    "admin.company" = "admin",           # admin.company.localhost → admin service in company cluster
    "api.company" = "api",               # api.company.localhost → api service in company cluster
    "grafana.ft" = "grafana",            # grafana.ft.localhost → grafana service in ft cluster
    "mysql-admin.personal" = "mysql-admin"  # mysql-admin.personal.localhost → mysql-admin service
}
inject_headers = true                    # Default: add client ID52 headers
public_services = ["api"]               # These services don't get identity headers
```

### **Multi-Cluster Power User Workflow:**
1. **Cluster manager**: `malai cluster init personal` (manage personal cluster)
2. **Join company**: `malai machine init company.example.com corp` (work cluster)  
3. **Join fifthtry**: `malai machine init abc123...xyz789 ft` (client cluster)
4. **Unified start**: `malai start` (starts cluster manager + SSH daemons + agent)
5. **Cross-cluster access**: `malai web01.ft systemctl status nginx`

### ** Capabilities:**
- **Identity-aware service mesh**: HTTP services receive client identity automatically
- **Protocol-agnostic**: TCP for databases, HTTP for web services  
- **Browser integration**: Direct browser access to remote services
- **Multi-cluster**: Single agent handles services across all clusters
- **Zero-configuration security**: Closed network model prevents attacks
- **Enterprise-grade**: Multi-tenant with hierarchical access control

### **Implementation Priority:**
1. **Restructure CLI**: Promote SSH commands to top-level
2. **Update identity management**: Replace keygen with identity commands
3. **Implement P2P protocols**: Config distribution and service forwarding
4. **Complete service mesh**: TCP + HTTP forwarding with identity injection

The design is now complete and captures the full vision of malai as a secure infrastructure platform.
