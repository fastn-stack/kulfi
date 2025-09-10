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

## Core Features

### 🔐 **P2P Security**
- **Cryptographic identity**: Each machine has unique ID52 identifier
- **Closed network**: Only cluster members can connect
- **Direct verification**: Uses cryptographic verification instead of passwords
- **No certificate authorities**: Direct public key verification

### 🌐 **Multi-Cluster Support**  
- **Multiple clusters**: Connect to different infrastructure clusters
- **Role flexibility**: Can manage some clusters, participate in others
- **Independent operation**: Machines work when cluster manager offline

### 📡 **Remote Access**
- **Command execution**: `malai web01.company ps aux` 
- **Permission system**: Basic access control with cluster configuration
- **Real execution**: Commands run on target machines via P2P

### ⚡ **Simple Interface**
- **Familiar syntax**: Similar to SSH for ease of use
- **Configuration files**: TOML-based cluster configuration
- **Role detection**: Automatic detection of cluster manager vs machine roles

## Architecture

malai operates as three integrated services:

1. **Cluster Manager**: Configuration distribution and cluster coordination
2. **SSH Daemon**: Accept remote commands on authorized machines
3. **Client Agent**: Local TCP/HTTP proxy for transparent service access

A single `malai daemon` command auto-detects roles and starts appropriate services.

## Real-World Examples

### DevOps Engineer
```bash
# Morning routine:
malai daemon  # Auto-daemonizes  # Starts all services for all clusters

# Server management:
malai web ps aux                    # Check web server processes
malai db backup                     # Run database backup
malai cache restart                 # Restart Redis cache

# Monitoring access:
open http://grafana.company.localhost    # Grafana dashboard  
open http://logs.company.localhost       # Log analysis tools
```

### Mobile Infrastructure Management
```bash
# Initialize from mobile device (iPhone/Android malai app):
malai cluster init company
# Edit cluster config in mobile app UI
malai daemon  # Distribute config to all servers

# Daily infrastructure management from mobile:
malai web01.company systemctl status nginx
malai db01.company backup-database
open http://grafana.company.localhost  # Mobile browser → server monitoring

# Servers continue operating when mobile cluster manager goes offline
# Configuration changes sync when mobile comes back online
```

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

```bash
curl -fsSL https://malai.sh/install.sh | sh
```

## Documentation

- **[DESIGN.md](DESIGN.md)**: Complete technical design and architecture
- **[malai.sh](https://malai.sh)**: Website with tutorials and examples
- **[CONTRIBUTING.md](CONTRIBUTING.md)**: How to contribute to the project

---

**Built on [fastn-p2p](https://github.com/fastn-stack/fastn) • Secured by cryptographic verification • No central servers required**

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