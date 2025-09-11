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
When joining a cluster, you use invite keys instead of exposing cluster manager ID52:

1. **Invite Key**: Use disposable invite key (recommended)
   ```bash
   malai machine init 789xyz123def456ghi company
   # Invite key resolves to actual cluster manager internally
   ```

2. **Direct ID52**: Use cluster manager's full ID52 (fallback)
   ```bash
   malai machine init abc123def456ghi789jkl012mno345pqr678stu901vwx234 company
   # Direct access to cluster manager (less secure - ID52 exposed)
   ```

## Cluster Security Architecture

### **Three-Tier Key System:**

#### **1. Cluster Root Key (Hidden)**
- **Purpose**: True cluster identity, never shared publicly
- **Visibility**: Hidden in MALAI_HOME config files, not shown in daily usage
- **Rotation**: `malai cluster rotate-key` generates new root key  
- **Cleanup**: `malai cluster delete-old-key` removes compromised keys

#### **2. Invite Keys (Public)**
- **Purpose**: Safe-to-share keys for joining cluster
- **Creation**: `malai cluster create-invite --alias "conference-2025"`
- **Revocation**: `malai cluster revoke-invite <invite-key>`
- **Security**: Compromised invite keys don't expose cluster root

#### **3. Multiple Identities → One Cluster**
- **Invite keys are aliases**: Multiple invite keys point to same cluster root
- **Post-join discovery**: After joining via invite, machine learns cluster root key
- **Day-to-day usage**: Users never see root key in normal operations
- **Root key obscurity**: Security through hiding cluster root from public view

### **Key Rotation Security:**
```bash
# Rotate cluster root key (security incident response):
malai cluster rotate-key
# 1. Generates new cluster root key
# 2. Contacts all machines with new key  
# 3. Machines update configs to use new root key
# 4. Old key still maintained for transitioning machines

# Delete compromised old key:
malai cluster delete-old-key <old-key>
# 1. Stop listening on old key
# 2. Continue using old key in client mode to update remaining machines  
# 3. Gradual migration to new key
```

### **Two-Level Alias System:**

#### **1. Cluster Aliases (per-cluster)**
Every cluster gets a local alias chosen during `machine init`:
- **Short aliases**: `ft` instead of `fifthtry.com` 
- **Personal naming**: Use whatever makes sense to you
- **Folder names**: Aliases become directory names in `$MALAI_HOME/clusters/`

#### **2. Global Machine Aliases (cross-cluster)**  
Edit `$MALAI_HOME/aliases.toml` for ultra-short machine access:
- **Super short**: `malai web top` instead of `malai web01.ft top`
- **Cross-cluster**: Mix machines from different clusters with unified names
- **Role-based**: `prod-web`, `staging-web`, `dev-web` for different environments
- **Service-based**: `db-primary`, `db-replica`, `monitoring` for service roles

#### **Alias Resolution Order:**
1. **Check global aliases**: `web` → `web01.ft`
2. **Check cluster.machine**: `web01.ft` → resolve cluster and machine
3. **Direct ID52**: `abc123...xyz789` → direct machine contact

### **Invite Key System (Recommended Security Model):**

#### **Invite Key Benefits:**
- **Security**: Real cluster manager ID52 stays private
- **Revocable**: Disable compromised invite keys without cluster disruption
- **Public sharing**: Safe to share at conferences, public forums
- **Multiple invites**: Different invite keys for different purposes

#### **Invite Key Management:**
```bash
# Cluster manager creates invite keys:
malai cluster create-invite company --alias "conference-2025"
# Outputs: 789xyz123def456ghi (safe to share publicly)

# Share invite key publicly:
"Join our cluster: malai machine init 789xyz123def456ghi company"

# Revoke invite key when needed:
malai cluster revoke-invite 789xyz123def456ghi

# List active invites:
malai cluster list-invites
```

#### **Implementation Requirements:**
- **invite.toml**: Maps invite keys to actual cluster manager ID52
- **Revocation system**: Disable invite keys in cluster config
- **P2P routing**: Invite keys forward to real cluster manager
- **Security audit**: Track invite key usage and revocation

#### **Development Estimate: 3-4 hours**
- Invite key generation and mapping: 2 hours
- P2P routing for invite keys: 1-2 hours  
- Revocation and management commands: 1 hour

## System Architecture

malai consists of **separate processes** that run independently on different machines:

### **1. Cluster Manager Process** (Separate machine/MALAI_HOME)
- **Purpose**: Configuration management and distribution
- **Runs on**: Dedicated machine (laptop, server, mobile device)
- **MALAI_HOME**: Contains cluster-config.toml for managed clusters
- **Role**: **P2P client only** - distributes config to machines
- **Functions**:
  - Monitor cluster-config.toml for changes
  - Generate personalized configs for each machine
  - Send configs via P2P to all cluster machines
  - Maintain state.json with per-machine sync status

### **2. Machine Process** (Separate machine/MALAI_HOME)  
- **Purpose**: Accept SSH commands and provide services
- **Runs on**: Server machines, laptops, any infrastructure
- **MALAI_HOME**: Contains machine-config.toml (received from cluster manager)
- **Role**: **P2P server** - accepts SSH requests and config updates
- **Functions**:
  - Listen for P2P config updates from cluster manager
  - Listen for P2P SSH requests from authorized machines
  - Execute commands with permission validation
  - **Follows ACL**: No special privileges, respects cluster permissions

### **3. Service Proxy** (Optional, per machine/MALAI_HOME)
- **Purpose**: Local TCP/HTTP forwarding for service access
- **Runs on**: Any machine that needs to access remote services  
- **MALAI_HOME**: Contains services.toml with forwarding configuration
- **Functions**:
  - **HTTP server**: Port 80, routes by `subdomain.localhost`
  - **TCP servers**: Forward configured ports via P2P
  - **Identity injection**: Add client ID52 headers to HTTP
  - **Connection pooling**: Efficient P2P connection reuse

### **Deployment Model - Separate Machines/MALAI_HOME:**

#### **Cluster Manager Machine:**
```
Machine: laptop.local (or mobile device)
MALAI_HOME: /home/admin/.local/share/malai/
├── clusters/company/cluster-config.toml  # Master config with all machines
├── clusters/company/state.json          # Distribution tracking
└── malai.lock

Process: malai daemon (cluster manager only)
Role: Distributes configs, follows ACL like any other machine
```

#### **Server Machine 1:**
```  
Machine: web01.company.com
MALAI_HOME: /home/webuser/.local/share/malai/
├── clusters/company/cluster-info.toml   # Cluster manager verification
├── clusters/company/machine-config.toml # Received from cluster manager
├── clusters/company/identity.key        # Machine's identity for company cluster
└── malai.lock

Process: malai daemon (remote access daemon + optional service proxy)
Role: Accepts SSH commands, follows permissions in received config
```

#### **Server Machine 2:**
```
Machine: db01.company.com  
MALAI_HOME: /home/dbuser/.local/share/malai/
├── clusters/company/machine-config.toml # Different config than web01
├── clusters/company/identity.key        # Different identity than web01
└── malai.lock

Process: malai daemon (remote access daemon only)
Role: Database server, accepts SSH from authorized machines only
```

#### **Developer Laptop:**
```
Machine: dev-laptop.local
MALAI_HOME: /home/developer/.local/share/malai/
├── clusters/company/machine-config.toml # Client-only permissions
├── services.toml                        # Local forwarding: mysql, admin interface
└── malai.lock

Process: malai daemon (service proxy for CLI commands)
Role: Initiates SSH commands, accesses services via local forwarding
```

### **Cross-Machine Communication Flows:**

#### **Config Distribution Flow:**
1. **Admin edits**: cluster-config.toml on cluster manager machine
2. **CM detects change**: Hash comparison triggers distribution
3. **CM generates**: Personalized config for each machine ID52 in master config
4. **CM sends P2P**: fastn_p2p::call(machine_id52, personalized_config)
5. **Machine receives**: P2P listener accepts config, validates sender
6. **Machine saves**: machine-config.toml, triggers service restart

#### **SSH Execution Flow:**
1. **CLI command**: `malai web01.company ps aux` on developer laptop
2. **Local daemon**: Looks up web01.company in local clusters, finds target machine ID52
3. **P2P call**: fastn_p2p::call(target_machine_id52, ssh_request) 
4. **Target machine**: Receives request, validates permissions, executes command
5. **Response**: Command output returned via P2P to initiating machine

#### **Service Access Flow:**
1. **Service request**: `curl admin.company.localhost` on developer laptop  
2. **Local proxy**: Routes to admin service in company cluster
3. **P2P forwarding**: fastn_p2p::call to machine hosting admin service
4. **Service execution**: Remote machine proxies to local admin service
5. **Response streaming**: Service response returned via P2P tunnel

## Remote Cluster Configuration Management

### **Config Edit Authorization:**
The cluster manager can accept config edit commands from authorized machines:

```toml
[cluster-manager]
id52 = "cluster-manager-id52"
cluster_name = "company" 
config_editors = "admins,devops-leads,emergency-laptop-id52"
```

### **Version-Controlled Config Commands:**
```bash
# Download current cluster config with version hash
malai config download company
# Outputs: 
# - company-config.toml (config content)
# - .company-config.hash (version hash for upload)

# Edit config locally (any editor)  
vim company-config.toml

# Upload with version check (requires matching hash)
malai config upload company-config.toml
# Reads .company-config.hash, validates server hash matches before upload
# Fails if config changed remotely since download

# Atomic edit (download → edit → upload in single operation)
malai config edit company
# Downloads config + hash, opens $EDITOR, uploads on save
# Minimizes conflict window but still does hash validation

# View current config without downloading
malai config show company

# Validate config syntax before upload  
malai config validate company-config.toml

# Force upload (bypass hash check - dangerous)
malai config upload company-config.toml --force
```

### **Version-Controlled Config Management Flow:**
1. **Admin downloads**: `malai config download company` → P2P request to cluster manager
2. **Cluster manager**: Validates requester in `config_editors`, sends config + current hash
3. **Admin receives**: company-config.toml + .company-config.hash files created locally
4. **Admin edits**: Local file editing using standard tools (vim, nano, etc.)
5. **Admin uploads**: `malai config upload company-config.toml` → P2P request with original hash
6. **Hash validation**: Cluster manager compares original hash with current hash
   - **If match**: Accept upload, replace config atomically, trigger distribution
   - **If mismatch**: Reject upload, return error "Config changed remotely, please re-download"
7. **Distribution**: All machines receive updated personalized configs automatically

### **Three-Way Merge Conflict Resolution:**
When upload fails due to hash mismatch, malai provides intelligent conflict resolution:

```bash
# Upload attempt with conflicts:
malai config upload company-config.toml
❌ Upload failed: Config hash mismatch (config changed remotely)

# Automatic three-way merge using diffy crate:
🔄 Attempting automatic merge...
📋 Base version: <original config when downloaded>
📋 Your changes: <your edited config>
📋 Remote changes: <current remote config>

# Three possible outcomes:

# 1. AUTO-MERGE SUCCESS:
✅ Automatic merge successful
📄 Merged config saved to company-config.toml
💡 Review merged changes and re-upload: malai config upload company-config.toml

# 2. MERGE CONFLICTS:
❌ Automatic merge failed - conflicts require manual resolution
📄 Conflict markers added to company-config.toml:
<<<<<<< Your changes
[machine.web01]
allow_from = "admins,developers"
=======
[machine.web01]  
allow_from = "admins,security-team"
>>>>>>> Remote changes

💡 Resolve conflicts manually and re-upload

# 3. FORCE OVERRIDE:
malai config upload company-config.toml --force
⚠️  WARNING: This will overwrite remote changes
🔄 Uploading without hash validation...
```

### **Three-Way Merge Algorithm (diffy crate):**
```rust
use diffy::{merge, PatchOptions};

// Three-way merge using diffy
let merge_result = merge(
    &original_config,    // Base version (when downloaded)
    &your_config,        // Your edited version
    &remote_config       // Current remote version
);

match merge_result {
    Ok(merged) => {
        // Clean merge - save and offer to upload
        save_merged_config(&merged);
    }
    Err(conflicts) => {
        // Add conflict markers for manual resolution
        add_conflict_markers(&conflicts);
    }
}
```

### **Atomic Edit Safety:**
```bash  
malai config edit company
# 1. Downloads config + hash
# 2. Opens $EDITOR with config  
# 3. On save, validates hash before upload
# 4. If hash mismatch:
#    a. Downloads latest remote version
#    b. Attempts three-way merge using diffy
#    c. Shows merge result or conflicts for manual resolution
#    d. Asks user to review and confirm upload
```

### **Security Benefits:**
- **Authorized editing**: Only specified machines can modify cluster config
- **No SSH needed**: No need to SSH to cluster manager machine
- **Atomic updates**: Config changes applied atomically across cluster
- **Audit trail**: All config changes logged with sender identity
- **Group support**: Use groups for team-based config editing permissions

### **Mobile Admin Support:**
- **Edit from mobile**: Admin can download, edit, and upload config from mobile device
- **Emergency management**: Emergency config changes from any authorized device
- **Offline editing**: Download config, edit offline, upload when convenient

### **Missing Design Elements - TO ADDRESS:**

#### **Multi-Cluster Resolution:**
- **Gap**: How does `malai web01.company ps aux` know which local cluster "company" refers to?
- **Solution needed**: Cluster alias → cluster ID52 resolution from local MALAI_HOME
- **Implementation**: Check all clusters in MALAI_HOME/clusters/ for matching alias

#### **Cluster Manager Discovery:**
- **Gap**: How do machines find cluster manager to receive configs?  
- **Current**: cluster-info.toml stores cluster manager ID52 after `malai machine init`
- **Missing**: DNS TXT record resolution for domain-based cluster discovery

#### **Config Authentication:**
- **Gap**: How machines verify config comes from authorized cluster manager
- **Current**: fastn_p2p provides sender ID52 verification automatically
- **Missing**: Cross-reference sender ID52 with cluster-info.toml verification

#### **Service Discovery:**
- **Gap**: How service proxy finds which machine hosts each service
- **Current**: Services defined in cluster configs with hosting machine specified
- **Missing**: Service-to-machine resolution in service proxy implementation

#### **CLI Process Resolution:**
- **Gap**: How CLI commands route to appropriate local daemon  
- **Current**: Unix socket to local daemon assumed
- **Missing**: Multi-cluster CLI routing when machine participates in multiple clusters

#### **Command Aliases:**
- **Feature**: Global command aliases in $MALAI_HOME/aliases.toml
- **Usage**: `malai web` → runs `malai web01.company ps aux`
- **Benefits**: Ultra-short commands for frequently used operations
- **Format**: `alias = "service.server.cluster command args"`
- **Validation**: Aliases conflicting with subcommands (daemon, cluster, etc.) MUST fail validation
- **Reserved names**: daemon, cluster, machine, info, status, service, identity, rescan

### CLI Execution Modes

#### **Direct Mode (MVP - Primary Mode):**
- **CLI commands work independently**: No daemon required for basic functionality
- **Fresh P2P connections**: Each command creates new fastn_p2p connection to target
- **MALAI_HOME-based**: CLI reads cluster configs and identities directly from filesystem
- **Machine auto-selection**: Automatically picks local machine identity for target cluster
- **Self-command optimization**: Local execution when targeting same identity

**Implementation Details:**
```rust
// Machine auto-selection logic:
1. Parse: malai web01.company ps aux → machine="web01", cluster="company"
2. Find cluster dir: $MALAI_HOME/clusters/company/
3. Auto-select identity:
   - If cluster.private-key exists → use cluster manager identity
   - If machine.private-key exists → use machine identity  
   - If both exist → configuration error (crash)
   - If neither → no identity error
4. Read config: cluster.toml OR machine.toml (whichever exists)
5. Find target: Look up web01 machine ID52 in config
6. Execute: Self-command (local) OR P2P call (remote)
```

**Resilience Benefits:**
- Works without daemon dependency (survives daemon crashes)
- No socket configuration needed
- No connection pooling complexity
- Simple troubleshooting (direct file/network operations)

#### **Daemon Mode (Post-MVP - Performance Optimization):**
- **Optional daemon**: `malai daemon` provides connection pooling for better performance  
- **CLI → daemon socket**: Commands sent to daemon via Unix socket when available
- **Pooled connections**: Daemon maintains P2P connections, CLI reuses them
- **Fallback**: Falls back to direct mode when daemon not running

#### **CLI Communication Protocol:**
```
CLI Command: malai web01.company ps aux
    ↓
1. CLI connects to $MALAI_HOME/malai.sock
2. CLI sends: {"type": "ssh_exec", "machine": "web01.company", "command": "ps", "args": ["aux"]}
3. malai daemon receives, validates permissions, forwards via existing P2P connection
4. malai daemon sends response: {"stdout": "...", "stderr": "...", "exit_code": 0}
5. CLI displays output and exits

No P2P overhead per command - all connections pooled in malai daemon process.
```

#### **Fallback Behavior:**
- **malai daemon running**: CLI uses socket communication (fast)
- **malai daemon not running**: CLI creates direct P2P connection (slower, but works)

## Explicit Configuration Management

### **No File System Watchers:**
File system watchers cause issues:
- **Partial file writes**: Triggers on incomplete/invalid config during editing
- **Race conditions**: Multiple rapid changes cause unnecessary reloads
- **Unpredictable timing**: Admin loses control over when changes take effect

### **Explicit Rescanning Commands:**
```bash
# Check config validity without applying changes
malai rescan --check
# Output: Reports config syntax errors, permission issues, invalid references

# Apply config changes atomically  
malai rescan
# Sends reload signal to running malai daemon via Unix socket
# 1. Daemon loads and validates all configs in MALAI_HOME
# 2. If valid: replace running config, trigger service updates
# 3. If invalid: keep existing config, report errors  
# 4. Atomic operation: either all configs update or none do
```

### **Atomic Config Update Process:**
1. **Load new configs**: Parse all cluster-config.toml and machine-config.toml files
2. **Validate configs**: Check syntax, references, permissions
3. **Test compatibility**: Ensure new configs work with current cluster state  
4. **Apply atomically**: Replace running config only if validation passes
5. **Trigger sync**: Start distributing new config to machines (cluster managers only)
6. **Rollback on error**: Keep existing config if any validation fails

### **Admin Workflow:**
```bash
# 1. Edit config files as needed (vi, nano, etc.)
# 2. Check changes before applying:
malai rescan --check

# 3. Apply if valid:
malai rescan  

# 4. Monitor distribution:
malai info  # Shows sync status per cluster
```

This gives admins full control over when configuration changes take effect.

### Service Integration  
- **Remote commands**: CLI → malai daemon → P2P execution
- **TCP services**: `mysql -h localhost:3306` → malai daemon forwards via P2P
- **HTTP services**: `http://admin.localhost` → malai daemon routes via subdomain  
- **Unified operation**: Single `malai daemon` handles all P2P connections and service forwarding

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
malai daemon                         # Distribute config to all servers

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
config_editors = "admins,devops-leads,emergency-laptop-id52"  # Who can remotely edit cluster config

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
3. **Agent default**: Same user that runs `malai daemon`

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
- **Cluster Manager Machine**: Admin manually edits `$MALAI_HOME/cluster-config.toml`
  - Use any editor: vim, nano, cp, etc.
  - Agent watches for changes and auto-distributes to all cluster machines
- **All Other Machines**: 
  - Receive config from cluster manager via P2P sync
  - Agent automatically overwrites `$MALAI_HOME/cluster-config.toml`
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
# Creates: $MALAI_HOME/clusters/company/ with cluster manager config

# Join existing cluster as machine (contacts cluster manager)
malai machine init <cluster-id52-or-domain> <local-alias>
# Examples:
malai machine init abc123def456ghi789... company     # Using cluster manager ID52
malai machine init fifthtry.com ft                  # Using domain (if DNS configured)
# Creates: $MALAI_HOME/clusters/ft/ with machine config and registration
```

### Unified Service Management
```bash
# Start all SSH services (auto-detects roles across all clusters)
malai daemon
# Scans $MALAI_HOME/clusters/ and starts:
# - Cluster manager for clusters where this machine is manager
# - remote access daemon for clusters where this machine accepts SSH
# - Client agent for connection pooling across all clusters
# Environment: malai daemon -e

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
malai daemon

# Get environment setup commands for shell integration
malai daemon -e

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
- **Logging**: All logs stored in `LOGDIR[malai]/malai/` (stdout/stderr reserved for command output)

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
eval $(malai daemon -e)

# With specific options
eval $(malai daemon -e --lockdown --http)

# Disable HTTP proxy
eval $(malai daemon -e --http=false)
```

### Persistent Setup
Add to your shell profile (`.bashrc`, `.zshrc`, etc.):
```bash
# Enable malai daemon on shell startup
eval $(malai daemon -e)
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
eval $(malai daemon -e)  # Agent auto-detects role and starts appropriate services
```

**Complete MALAI_HOME Structure (Multi-Role, Multi-Cluster):**
```
$MALAI_HOME/
├── clusters/
│   ├── company/                     # Cluster 1 - This device is cluster manager
│   │   ├── cluster.toml            # → Cluster manager role (master config)
│   │   ├── cluster.private-key     # → Cluster root private key (or keyring if available)
│   │   ├── invites/                # → Invite keys for secure cluster joining
│   │   │   ├── 789xyz123def.private-key # → Individual invite key  
│   │   │   └── 456abc789ghi.private-key # → Another invite key
│   │   ├── invites.toml            # → Invite key metadata and aliases
│   │   ├── old-keys/               # → Rotated keys during transition
│   │   │   └── cluster.private-key.1   # → Previous root key (migration)
│   │   └── state.json              # → Config distribution tracking
│   ├── personal/                    # Cluster 2 - This device is machine only
│   │   ├── machine.toml            # → Machine role (received from CM)
│   │   └── machine.private-key     # → Private key for machine role
│   └── client-work/                 # Cluster 3 - This device is cluster manager only  
│       ├── cluster.toml            # → Cluster manager role
│       ├── cluster.private-key     # → Private key for cluster manager role
│       └── state.json              # → Config distribution tracking
├── malai.toml                       # Local services: aliases, TCP/HTTP forwarding
├── malai.sock                       # CLI communication socket
├── malai.lock                       # Process lockfile
└── malai.log                        # Unified log file (all clusters)
```

**Role Detection Rules:**
- `cluster.toml` exists, `machine.toml` missing → **Cluster Manager** (reads cluster.toml for ACL)
- `machine.toml` exists, `cluster.toml` missing → **Machine** (received config from CM)  
- **Both exist → CONFIGURATION ERROR** (daemon must crash with clear error)
- Neither exists → **Waiting** (machine not yet configured)

**Configuration Sources:**
- **Cluster Manager**: Reads `cluster.toml` directly for ACL (acts as machine using cluster config)
- **Remote Machine**: Reads `machine.toml` (received via P2P from cluster manager)
- **Invalid State**: Both files present → daemon crashes with error message

## Single malai Daemon Architecture

### **One Daemon, Multiple Identities:**
- **Single malai daemon process** handles all clusters simultaneously
- **One P2P listener per cluster identity** (one fastn_p2p::listen! per identity.key)
- **Role detection per cluster**: Each cluster directory defines roles independently
- **Unified protocol handling**: Same protocols (ConfigUpdate, ExecuteCommand) for all identities

### **Cluster Manager Self-Operation:**
When cluster manager acts as machine (common single-cluster scenario):
- **No config sync**: Reads `cluster.toml` directly for ACL validation
- **Self-command optimization**: Direct file access instead of P2P
- **Same interface**: `malai web01.company ps aux` works whether self or remote
- **Unified ACL**: Same permission checking code for self and remote operations

### **File Name Conventions:**
- `cluster.toml`: Master cluster config (cluster manager role)
- `cluster.private-key`: Cluster root private key (filesystem fallback - prefer keyring)
- `invites/`: Directory containing individual invite private keys
- `invites.toml`: Invite key metadata, aliases, and expiration tracking
- `machine.toml`: Machine-specific config (machine role, received via P2P)
- `machine.private-key`: Private key for machine role (filesystem fallback)
- `old-keys/`: Directory for rotated keys during transition
- `state.json`: Config distribution tracking (cluster manager only)

**Key Storage Preference:**
1. **Keyring (preferred)**: Store private keys in system keyring when available
2. **Filesystem (fallback)**: Store in .private-key files when keyring unavailable
3. **No .public-key files needed**: Public keys derived from private keys

**invites.toml Structure:**
```toml
# Active invite keys for cluster joining
[invite."invite789xyz123def456"]
alias = "conference-2025"
created = "2025-01-15T10:30:00Z"
expires = "2025-02-15T10:30:00Z"  # Optional expiration
created_by = "admin-laptop-id52"

[invite."invite456abc789def"]  
alias = "partners-q1"
created = "2025-01-10T09:00:00Z"
# No expiration = permanent until revoked
```

## Real-World Deployment Scenarios

### **Scenario 1: Personal Single-Cluster Setup**
```
Device: Personal laptop
$MALAI_HOME/clusters/personal/cluster.toml     # Cluster manager role
$MALAI_HOME/clusters/personal/identity.key     # One identity for personal cluster

Behavior: 
- Cluster manager: Manages personal cluster config
- Machine: Executes commands (reads ACL from cluster.toml directly)  
- No config sync needed: Same file serves both roles
```

### **Scenario 2: Multi-Cluster Power User**  
```
Device: Developer laptop  
$MALAI_HOME/clusters/personal/cluster.toml     # CM of personal cluster
$MALAI_HOME/clusters/personal/identity.key     # Identity 1
$MALAI_HOME/clusters/company/machine.toml      # Machine in company cluster  
$MALAI_HOME/clusters/company/identity.key      # Identity 2
$MALAI_HOME/clusters/client/machine.toml       # Machine in client cluster
$MALAI_HOME/clusters/client/identity.key       # Identity 3

Behavior:
- Three P2P listeners (one per identity)
- Cluster manager of personal, machine in company+client
- Same daemon handles all roles simultaneously
```

### **Scenario 3: Dedicated Server**
```
Device: Production server
$MALAI_HOME/clusters/company/machine.toml      # Machine only
$MALAI_HOME/clusters/company/identity.key      # One identity

Behavior:
- One P2P listener for company cluster
- Machine role only (receives config from remote cluster manager)
- Executes commands based on received machine.toml permissions
```

**Logging Strategy:**
- **Single log file**: `$MALAI_HOME/malai.log` for all clusters and services
- **Structured logging**: Include cluster_alias and service_type in log entries
- **Log rotation**: Standard log rotation via system tools or internal rotation
- **Debug levels**: Different verbosity for development vs production

**Log Entry Format:**
```
2025-01-15T10:30:45Z [INFO] [company] [cluster-manager] Config distributed to 3 machines
2025-01-15T10:30:46Z [INFO] [ft] [ssh-daemon] Executed command 'ps aux' for client abc123...
2025-01-15T10:30:47Z [INFO] [service-proxy] [http] Routed admin.company.localhost to web01.company
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

**malai.toml - Unified Local Configuration:**
```toml
# Command aliases for convenient access  
[aliases]
web = "web01.ft ps aux"             # malai web → malai web01.ft ps aux
db = "db01.ft backup"               # malai db → malai db01.ft backup
logs = "web01.ft tail -f /var/log/nginx/access.log"
deploy = "web01.ft deploy latest"

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
  "master_config_hash": "abc123def456",
  "last_distribution": "2025-01-15T10:30:00Z",
  "machine_states": {
    "web01-machine-id52": {
      "machine_alias": "web01",
      "personalized_config": "...machine-specific TOML content...",
      "personalized_config_hash": "machine789hash123",
      "last_sync": "2025-01-15T10:30:00Z", 
      "sync_status": "success"
    },
    "db01-machine-id52": {
      "machine_alias": "db01",
      "personalized_config": "...machine-specific TOML content...",
      "personalized_config_hash": "machine456hash789", 
      "last_sync": "2025-01-15T09:45:00Z",
      "sync_status": "pending"
    }
  }
}
```

### **Machine-Specific Config Generation:**
1. **Extract machine section**: From master config, extract `[machine.web01]` and related sections
2. **Include cluster info**: Add cluster manager ID52, cluster name for verification
3. **Include dependencies**: Add referenced groups, services, commands for this machine
4. **Generate hash**: Hash the personalized config content  
5. **Store in state**: Cache personalized config and hash for each machine
6. **Sync when changed**: Send personalized config when hash differs

### **Config Distribution Algorithm:**
1. **Monitor master config**: Watch cluster-config.toml for file changes
2. **Generate personalized configs**: Create machine-specific config for each machine
3. **Calculate hashes**: Hash each machine's personalized config content
4. **Compare states**: Check which machines have outdated personalized config hash  
5. **Distribute updates**: Send personalized config to machines with old hash via P2P
6. **Update state**: Store personalized config and hash for each machine

### **Personalized Config Example:**
Master config contains all machines, but each machine receives only relevant sections:

**Master config** (cluster manager):
```toml
[cluster_manager]
id52 = "cluster123"
cluster_name = "company"

[machine.web01] 
id52 = "web01-id52"
allow_from = "admins"

[machine.db01]
id52 = "db01-id52" 
allow_from = "admins,devs"

[group.admins]
members = "laptop-id52"
```

**Personalized config for web01** (sent to web01 machine):
```toml
[cluster_manager]
id52 = "cluster123"
cluster_name = "company"

[machine.web01]  # Only this machine's section
id52 = "web01-id52"
allow_from = "admins"

[group.admins]   # Referenced groups included
members = "laptop-id52"
```

### **Multi-Cluster State:**
Each cluster directory has its own state.json:
- `$MALAI_HOME/clusters/company/state.json` 
- `$MALAI_HOME/clusters/ft/state.json`
- `$MALAI_HOME/clusters/personal/state.json`

### **malai daemon Architecture:**
Self-daemonizing process that provides all infrastructure services:

#### **Daemon Behavior:**
- **Default**: `malai daemon` or `malai d` → automatically daemonizes and runs in background
- **Survives shell close**: Process detaches from terminal, continues running
- **Auto-restart friendly**: Add to `.bashrc` or `.zshrc` for automatic startup
- **Foreground mode**: `malai daemon --foreground` for systemd/supervisor integration
- **Single instance**: **MUST use exclusive file locking** to prevent multiple daemon instances
- **Graceful shutdown**: Ctrl+C triggers graceful shutdown with service cleanup

#### **Process Management (REQUIRED):**
- **Exclusive file locking**: `$MALAI_HOME/malai.lock` with `try_lock()` - fail if another daemon running
- **Lock lifetime**: Hold file lock for entire daemon lifetime, automatically released on process exit
- **Graceful shutdown**: Use fastn-p2p graceful shutdown pattern for clean service termination
- **Service cleanup**: Allow current requests to complete before shutdown (configurable timeout)

#### **Daemon Startup Process (REQUIRED):**
1. **Config validation**: Load and validate ALL configs before daemonizing
2. **Fail fast**: Exit immediately if any config has syntax errors or invalid references  
3. **Atomic validation**: Either all configs are valid or daemon refuses to start
4. **Only then daemonize**: Self-daemonize only after successful config validation
5. **Start services**: Begin cluster managers + remote access daemon + service proxy

#### **Config Validation Requirements:**
- **TOML syntax**: All .toml files must parse correctly
- **Reference validation**: All machine IDs, group references must be valid
- **Permission consistency**: No circular group references or invalid permissions
- **Identity verification**: All identity.key files must be valid and accessible
- **Startup failure**: Log specific config errors and exit with non-zero status

#### **Daemon Responsibilities (After Validation):**
1. **Multi-role operation**: Runs cluster managers + remote access daemon + service proxy as needed
2. **CLI communication**: Provides Unix socket for CLI commands (connection pooling)  
3. **Config management**: Responds to `malai rescan` for atomic config reloading
4. **Service orchestration**: Coordinates all P2P services in single process

#### **Implementation Sequence:**
```rust
// 1. Acquire exclusive lock first
let lock_file = std::fs::OpenOptions::new()
    .create(true)
    .write(true) 
    .open(&lock_path)?;
lock_file.try_lock().map_err(|_| "Another daemon running")?;
let _lock_guard = lock_file;

// 2. Load and validate ALL configs (MUST succeed before daemonizing)
let validated_configs = load_and_validate_all_configs(&malai_home)?;
// Exit immediately if any config invalid

// 3. Only then daemonize (if not --foreground)
if !foreground {
    daemonize()?; // Fork to background
}

// 4. Start services using validated configs
start_services_from_configs(validated_configs);

// 5. Wait for graceful shutdown
fastn_p2p::cancelled().await;
```

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
- **Automatic Scanning**: Agent discovers all clusters in `DATADIR[malai]/malai/clusters/`
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
eval $(malai daemon -e)  # Start agent (automatically runs as cluster manager)
```

**3. Initialize Server Machine (Terminal 2):**
```bash
export MALAI_HOME=/tmp/malai-test/server1
malai init  # Generate machine identity (NO config yet)
# Outputs: "Machine created with ID: def456..."
```

**4. Update Cluster Config (Terminal 1 - Cluster Manager):**
```bash
# Edit $MALAI_HOME/cluster-config.toml to add:
# [machine.web01]
# id52 = "def456..."  # The ID from step 3
# accept_ssh = true
# allow_from = "*"
# 
# Config automatically syncs to Terminal 2's machine
```

**5. Start Server Agent (Terminal 2):**
```bash
eval $(malai daemon -e)  # Agent receives config and auto-detects SSH server role
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
eval $(malai daemon -e)  # Start agent (automatically runs as client)
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
eval $(malai daemon -e)  # Runs as cluster manager automatically
```

**Test Cluster:**
```bash
export MALAI_HOME=/tmp/malai-test/test-cluster
malai init-cluster --alias test-cluster
eval $(malai daemon -e)  # Runs as different cluster manager
```

**Client with Access to Both:**
```bash
export MALAI_HOME=/tmp/malai-test/multi-client
eval $(malai daemon -e)
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
eval $(malai daemon -e --lockdown --http)
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
eval $(malai daemon -e)  # Start agent in background
```

**2. Initialize Machines:**
```bash
# On each machine that should join the cluster:
malai init  # Generate identity for this machine
# Outputs: "Machine created with ID: <machine-id52>"

# Cluster admin manually adds to cluster manager's config:
# Edit $MALAI_HOME/cluster-config.toml:
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
eval $(malai daemon -e)
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
eval $(malai daemon -e)  # Auto-runs as cluster manager

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
eval $(malai daemon -e)  # Agent detects role and starts SSH server

# Terminal 3 - Create and Add Client
export MALAI_HOME=/tmp/malai-client1
malai create-machine
# Add this ID to cluster config as [device.laptop]
eval $(malai daemon -e)
malai web01.test-cluster "echo 'Multi-node test successful!'"
```

This approach allows you to test complex multi-cluster scenarios, permission systems, and service configurations entirely on a single development machine.

## Real-World Usage Examples

### Example 1: Personal Infrastructure Cluster

**Setup (one-time):**
```bash
# On my laptop (cluster manager):
malai cluster init personal
# Edit $MALAI_HOME/clusters/personal/cluster-config.toml to add machines
malai daemon &  # Starts cluster manager + client agent

# On home server:
malai machine init personal  # Contacts cluster, registers
# Laptop admin adds machine to personal cluster config
malai daemon &  # Starts remote access daemon + client agent

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
# Edit $MALAI_HOME/clusters/ft/cluster-config.toml
malai daemon  # Starts cluster manager

# On each fastn server:
malai machine init fifthtry.com ft  # Join via domain, use short alias
# fastn-ops adds machine to cluster config
malai daemon  # Starts remote access daemon

# On developer laptops:
malai machine init <cluster-manager-id52> ft  # Join via ID52, short alias
malai daemon  # Starts client agent for connection pooling
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
malai daemon  # Automatically starts:
                 # - Cluster manager for 'personal'
                 # - remote access daemon for 'company' and 'fastn-cloud'  
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
2. `malai daemon` → auto-starts all appropriate services

**Multi-cluster management** (unified):
- Single `malai daemon` handles all cluster roles
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
allow_from = \"*\"" >> $MALAI_HOME/cluster-config.toml

# 4. Start agents
export MALAI_HOME=$TEST_DIR/manager && malai daemon &
export MALAI_HOME=$TEST_DIR/server1 && malai daemon &
sleep 2  # Wait for config sync

# 5. Test SSH execution
export MALAI_HOME=$TEST_DIR/client1
malai identity create
CLIENT_ID=$(malai identity create | grep "ID52" | cut -d: -f2)

# Add client to config
export MALAI_HOME=$TEST_DIR/manager  
echo "[machine.client1]
id52 = \"$CLIENT_ID\"" >> $MALAI_HOME/cluster-config.toml

# Wait for sync and test
export MALAI_HOME=$TEST_DIR/client1
eval $(malai daemon -e)
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
- ✅ **Cryptographically secure foundation**: fastn-p2p provides authentication
- 🟡 **Application-level security needed**: Command validation and input sanitization required
- 🎯 **Security model**: Stronger than OpenSSH (no certificate authorities needed, direct cryptographic verification)

## Security Implementation Notes

### **fastn-p2p Security Foundation:**
The malai SSH system builds on fastn-p2p's cryptographic foundation:

- **Automatic Identity Verification**: Every P2P call cryptographically verifies both sender and receiver
- **End-to-End Encryption**: All communication channels encrypted by default
- **No Certificate Authorities**: Direct public key verification
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

The foundation uses cryptographic verification - application-level input validation is still needed.

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
malai daemon  
malai web01.company ps aux
```

**Should become top-level:**
```bash
malai cluster init company          # Promote to top-level
malai machine init company corp     # Promote to top-level  
malai daemon                         # Replaces both 'malai run' and 'malai daemon'
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
- Keep `malai/src/malai/` module name (avoid massive reorganization)
- Promote SSH functions to top-level malai API  
- Update CLI command structure to reflect core status
- Maintain backwards compatibility with existing commands

### **Documentation Strategy:**
- Current `malai/README.md` contains complete design (preserve all content)
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
4. **Unified start**: `malai daemon` (starts cluster manager + remote access daemons + agent)
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

## P2P Entity Architecture (CRITICAL)

### **One Entity, One Listener, Multi-Protocol:**
Fundamental principle: Each entity has exactly one P2P listener handling all protocols.

#### **Cluster Manager Entity:**
- **Identity**: One fastn_id52 (cluster manager identity)
- **Listener**: One fastn_p2p::server::listen per cluster manager
- **Protocols**: ConfigDownload, ConfigUpload, ExecuteCommand  
- **Startup**: Single listener spawned on daemon start

#### **Machine Entity:**
- **Identity**: One fastn_id52 (machine identity)  
- **Listener**: One fastn_p2p::server::listen per machine
- **Protocols**: ConfigUpdate, ExecuteCommand
- **Startup**: Single listener spawned on daemon start

### **Service Architecture:**
```rust
// CORRECT: One listener per entity
malai daemon startup:
  if cluster_manager_role:
    spawn cluster_manager_listener(cm_identity, [ConfigDownload, ConfigUpload, ExecuteCommand])
  if machine_role:  
    spawn machine_listener(machine_identity, [ConfigUpdate, ExecuteCommand])
  spawn service_proxy() // Local TCP/HTTP forwarding

// Each listener handles ALL protocols for that entity
fn handle_request(request):
  match request.protocol():
    ConfigUpdate -> handle_config_update()
    ExecuteCommand -> handle_execute_command() 
    ConfigDownload -> handle_config_download()
    ConfigUpload -> handle_config_upload()
```

### **Critical Fix Needed:**
Current implementation has **multiple listeners per entity** (config listener + remote access daemon).
Must be **single listener per entity** with protocol dispatch.

This fixes connection timeouts and simplifies service lifecycle.

## MVP Release Plan

### **🎯 MVP Features (Release Blockers)**

#### **✅ IMPLEMENTED (Ready)**
1. **P2P Infrastructure**: ConfigUpdate + ExecuteCommand protocols working end-to-end
2. **Role Detection**: cluster.toml vs machine.toml with configuration error handling
3. **File Structure**: Design-compliant clusters/ directory with identity per cluster
4. **Config Validation**: `malai rescan --check` comprehensive TOML validation
5. **Basic Command Execution**: Real remote command execution via P2P
6. **E2E Testing**: Comprehensive business logic testing with proper file structure

#### **✅ IMPLEMENTED (MVP Ready)**
1. **Real malai daemon**: Single daemon with multi-identity P2P listeners ✅
2. **Multi-cluster daemon startup**: One daemon handles all cluster identities simultaneously ✅  
3. **Basic ACL system**: Group expansion and permission validation (simple implementation) ✅
4. **Direct CLI mode**: Commands work without daemon dependency ✅

### **❌ NOT IMPLEMENTED (Moved to Post-MVP for Security)**
1. **DNS TXT support**: Removed due to security concerns (exposes cluster root ID52 publicly)
2. **Invite key system**: Secure alternative to DNS (Release 2 priority)

### **🚀 Post-MVP Features (Next Releases)**

#### **Release 2: Secure Cluster Management**
1. **Invite key system**: Secure cluster joining without exposing root keys
2. **Key rotation**: Cluster root key rotation and migration management  
3. **Remote config editing**: Download/upload/edit with hash validation and three-way merge
4. **Command aliases**: Global aliases in malai.toml for convenient access

#### **Release 3: Service Mesh**
1. **TCP forwarding**: `mysql -h localhost:3306` → remote MySQL via P2P
2. **HTTP forwarding**: `curl admin.company.localhost` → remote admin interface

#### **Release 3: Always-On HTTP Proxy**  
1. **Dynamic proxy routing**: CLI control of all devices' internet routing
2. **Privacy chains**: P2P encrypted proxy tunnels for IP masking
3. **Device management**: One-time setup, permanent CLI control

#### **Release 4: On-Demand Process Management**
1. **Dynamic service startup**: Start services when first request arrives
2. **Idle shutdown**: Stop services when no longer needed
3. **Process lifecycle**: Full process management (start, stop, restart, monitor)
4. **Resource optimization**: Run services only when actively used

### **On-Demand Process Management Design:**

Since malai controls all incoming P2P connections, it can manage service processes dynamically:

#### **Dynamic Service Lifecycle:**
```bash
# Service configuration in cluster.toml:
[machine.web01.http.admin]
port = 8080
command = "python manage.py runserver 8080"
idle_timeout = "300s"    # Stop after 5 minutes of inactivity
startup_time = "10s"     # Expected startup time

# malai behavior:
1. HTTP request arrives for admin.web01.company
2. Check if admin service process running
3. If not running: Start "python manage.py runserver 8080"  
4. Wait for service to be ready (up to 10s)
5. Forward request to localhost:8080
6. Track activity - stop after 5 minutes idle
```

#### **Process Management Features:**
- **Lazy startup**: Services start only when first request arrives
- **Health monitoring**: Check if processes are responsive  
- **Graceful shutdown**: Stop services cleanly when idle
- **Resource efficiency**: No idle processes consuming resources
- **Auto-restart**: Restart crashed services on next request

#### **Configuration Example:**
```toml
[machine.database.tcp.postgres]
port = 5432
command = "docker run -p 5432:5432 postgres:15"
idle_timeout = "30m"     # Database can idle longer
health_check = "pg_isready -p 5432"
restart_policy = "on-failure"
```

### **Always-On HTTP Proxy Design (Release 3):**

malai provides seamless privacy through an always-on proxy server with dynamic routing:

#### **Always-On Proxy Server:**
```bash
# malai daemon automatically starts proxy server on fixed port
malai daemon  # Starts proxy on localhost:8080 (fixed port)

# One-time device configuration (TV, mobile, laptop, etc.):
# Set HTTP proxy: 192.168.1.100:8080  # Your malai machine IP
# Never change device settings again!
```

#### **Dynamic Proxy Routing (CLI Control):**
```bash
# Check current proxy status:
malai status
# Shows: 
# 📡 HTTP Proxy: localhost:8080 → direct (no upstream)
# 📋 Configure devices: HTTP proxy 192.168.1.100:8080

# Route through specific machine for privacy:
malai proxy-via proxy-server.company
# All devices now use proxy-server.company transparently

# Check updated status:
malai status  
# Shows: 📡 HTTP Proxy: localhost:8080 → proxy-server.company

# Switch to different proxy server:
malai proxy-via vpn-exit.datacenter
# All devices instantly use new proxy (no device reconfiguration)

# Go back to direct connections:
malai proxy-via direct
# All devices back to direct internet
```

#### **User Experience Benefits:**
- **Configure once**: Set proxy on all devices to fixed malai port (one-time)
- **Dynamic routing**: Change proxy destination via CLI without touching devices
- **Seamless switching**: TV, mobile, laptop all switch proxy instantly
- **No device pain**: Never change proxy settings on individual devices again

#### **Technical Implementation:**
```toml
# Always-on proxy configuration
[daemon.proxy]
port = 8080                    # Fixed port for device configuration
mode = "direct"                # Default: no upstream proxy
# mode = "upstream"            # Route via upstream machine when configured

[proxy]
upstream_machine = ""          # Empty = direct mode
# upstream_machine = "proxy-server.company"  # Set via malai proxy-via command
```

#### **Privacy Workflow:**
1. **Install malai**: Proxy server starts automatically on port 8080
2. **Configure devices once**: Point all devices to malai proxy port  
3. **Control via CLI**: `malai proxy-via <machine>` changes routing for all devices
4. **Seamless switching**: Change proxy destination without device reconfiguration
5. **Privacy on-demand**: Enable/disable proxy routing as needed

This solves the real-world problem of painful proxy configuration on multiple devices.

#### **Release 4: Performance & Advanced Features**  
1. **CLI → daemon socket communication**: Connection pooling optimization
2. **Self-command optimization**: Cluster manager bypass P2P for self-operations
3. **Advanced ACL**: Complex group hierarchies and command-specific permissions
4. **Identity management**: Rich identity commands replacing keygen

### **🎯 MVP Success Criteria**

**User can:**
1. **Setup cluster**: `malai cluster init company` → working cluster manager  
2. **Start daemon**: `malai daemon` handles all clusters and identities
3. **Join machines**: Machine gets config via P2P, accepts commands
4. **Execute commands**: `malai web01.company ps aux` works remotely with basic ACL
5. **Multi-cluster**: Same device participates in multiple clusters with different identities

**Technical requirements:**
- Single `malai daemon` handles all clusters and identities
- Real P2P communication between devices (proven working)
- Basic security with ACL validation  
- Clean, maintainable code organization

### **🚫 Explicitly NOT in MVP**
- **CLI → daemon socket communication** (performance optimization for Release 4)
- **Service mesh** (TCP/HTTP forwarding - Release 3)  
- **Advanced ACL features** (complex group hierarchies - Release 4)
- **Remote config management** (Release 2)
- **Command aliases** (Release 2)

**MVP Focus**: Direct CLI mode - commands work without daemon requirement for resilience and simplicity.
