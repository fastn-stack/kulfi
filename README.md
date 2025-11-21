# malai: P2P Infrastructure Platform

malai provides remote access to your machines and services using peer-to-peer networking. It aims to simplify infrastructure management by eliminating central servers and certificate authorities.

---

## Quick Start

### Personal Infrastructure Setup

```bash
# On your laptop (cluster manager):
malai cluster init personal
malai daemon --foreground  # Start daemon

# Currently, machine joining requires manual setup:
# 1. Generate machine identity on target machine
# 2. Add machine ID to cluster configuration
# 3. Start daemon on target machine

# Remote command execution:
malai web01.personal ps aux    # Execute command on remote machine
malai web01.personal whoami    # Self-commands work via local optimization
```

### Enterprise Cluster Setup

```bash
# On ops machine (cluster manager):
malai cluster init company
malai daemon  # Auto-daemonizes

# On each server:
malai machine init company.example.com corp  # Join via domain
malai daemon  # Auto-daemonizes

# Developers get instant access:
malai web01.corp systemctl status nginx
malai db01.corp backup-database
mysql -h localhost:3306  # Direct database access via forwarding
```

## Current Features (Working Now)

### 🔐 **P2P Security**
- **Cryptographic identity**: Each machine has unique ID52 identifier
- **Closed network**: Only cluster members can connect
- **Direct verification**: Uses cryptographic verification instead of passwords

### 📡 **Remote Command Execution**
- **Command execution**: `malai web01.company ps aux` works via P2P
- **Self-commands**: Cluster manager commands execute locally (optimized)
- **Basic permissions**: Access control via cluster configuration
- **Real execution**: Commands actually run on target machines

### 🌐 **Multi-Cluster Foundation**  
- **Multiple clusters**: Architecture supports different clusters per device
- **Role detection**: Automatic cluster manager vs machine role detection
- **Configuration**: TOML-based cluster configuration with validation

## Planned Features (Future Releases)

### 🔑 **Invite Keys & Key Rotation**
- **Invite key system**: Safe cluster joining without exposing root keys
- **Key rotation**: Automated cluster root key rotation for security incidents
- **Remote configuration**: Download/edit/upload cluster configs
- **Command aliases**: `malai web` shortcuts for common operations

### 📡 **Service Mesh**
- **HTTP/TCP forwarding**: Access remote services transparently
- **Identity injection**: Services receive client identity headers

### 🌐 **Always-On HTTP Proxy**
- **Dynamic proxy routing**: Control all devices' internet routing via CLI
- **Privacy chains**: P2P encrypted proxy tunnels
- **One-time setup**: Configure devices once, control via malai commands

### 🔄 **On-Demand Process Management**
- **Dynamic startup**: Start services when first request arrives
- **Idle shutdown**: Stop services when no longer needed  
- **Resource efficiency**: Run Django, nginx, etc. only when actively used
- **Health monitoring**: Auto-restart crashed services on next request

## Architecture

### **Two Usage Modes**

**Direct Mode** (Default):
- CLI commands work without running daemon
- Creates fresh P2P connection for each command
- Reads cluster config and identity directly from MALAI_HOME

**Daemon Mode** (Optional optimization):  
- `malai daemon` provides connection reuse for better performance
- CLI commands use pooled connections when daemon available
- Falls back to direct mode when daemon not running

## Current Usage Examples

### Basic Cluster Setup
```bash
# Initialize cluster
malai cluster init company
malai daemon --foreground  # Start daemon

# Execute commands (works now)
malai web01.company echo "Hello remote infrastructure"
malai web01.company whoami
malai web01.company ps aux
```

### Status and Management
```bash
# Check cluster status
malai scan-roles              # Show detected roles
malai status                  # Detailed daemon and cluster info
malai rescan --check         # Validate all configurations
```

**See [DESIGN.md](DESIGN.md) for complete architecture and feature specifications.**


## Daemon Usage

### Personal Setup
```bash
# Add to ~/.bashrc or ~/.zshrc for automatic startup:
malai daemon  # Auto-starts on shell login, runs in background

# Or start manually when needed:
malai d  # Short alias, daemonizes automatically
```

### Server/Production Setup  
```bash
# systemd service (foreground mode):
malai daemon --foreground

# Docker/supervisor (foreground mode):  
malai daemon --foreground

# Manual daemon:
malai daemon  # Detaches from terminal, survives shell close
```

## Installation

Currently available for development and testing:

```bash
git clone https://github.com/fastn-stack/kulfi.git
cd kulfi
cargo build --bin malai
```

## Documentation

- **[DESIGN.md](DESIGN.md)**: Technical design and architecture
- **[test-e2e.sh](test-e2e.sh)**: End-to-end testing script

---

**Built on [fastn-p2p](https://github.com/fastn-stack/fastn) • Uses cryptographic verification • Early development stage**

## Legacy Single-Service Mode

malai still supports simple single-service exposure for backwards compatibility:

```bash
malai http 8080 --public           # Expose single HTTP service
malai tcp 3306 --public            # Expose single TCP service  
malai folder /path --public        # Expose folder via HTTP
```

These commands work without cluster setup for simple use cases.

---

This project is backed by [FifthTry](https://fifthtry.com/) and licensed under the [UPL](LICENSE) license.