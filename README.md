# malai: Secure P2P Infrastructure Platform

**Enterprise-grade remote access and service mesh with zero-configuration security.**

malai provides secure remote access to your machines and services using peer-to-peer networking. No central servers, no certificate authorities, no complex configuration - just cryptographically secure access to your infrastructure.

---

## Quick Start

### Personal Infrastructure Setup

```bash
# On your laptop (cluster manager):
malai cluster init personal
# Outputs: "Cluster created with ID: abc123def456ghi789..."
malai daemon &

# On your home server (copy the cluster manager ID52 from above):  
malai machine init abc123def456ghi789... personal  # Cluster manager ID52 + local alias
# Outputs: "Machine created with ID: xyz789abc123def456..."

# Back on laptop, add machine to cluster config:
# Edit ~/.local/share/malai/ssh/clusters/personal/cluster-config.toml
# Add: [machine.home-server] id52 = "xyz789abc123def456..." allow_from = "*"

# On home server, start services:
malai daemon &

# Now enjoy natural remote access from laptop:
malai home-server.personal htop
malai home-server.personal docker ps
open http://admin.personal.localhost  # Direct browser access to admin service in personal cluster
```

### Enterprise Cluster Setup

```bash
# On ops machine (cluster manager):
malai cluster init company
malai daemon &

# On each server:
malai machine init company.example.com corp  # Join via domain
malai daemon &

# Developers get instant access:
malai web01.corp systemctl status nginx
malai db01.corp backup-database
mysql -h localhost:3306  # Direct database access via forwarding
```

## Core Features

### 🔐 **Zero-Configuration Security**
- **Cryptographic identity**: Each machine has unique ID52 (stronger than SSH keys)
- **Closed network**: Only cluster members can connect (unknown machines rejected)
- **No passwords**: Cryptographic verification replaces password authentication
- **No certificate authorities**: Direct public key verification

### 🌐 **Multi-Cluster Management**  
- **Multiple clusters**: Personal, work, client clusters from single device
- **Mobile cluster manager**: Manage infrastructure from iPhone/Android malai app
- **Role flexibility**: Cluster manager of one, machine in others
- **Offline tolerance**: Servers operate independently when cluster manager offline
- **Local aliases**: `malai web top` instead of `malai web01.fifthtry.com top`

### 📡 **Identity-Aware Service Mesh**
- **Transparent TCP forwarding**: `mysql -h localhost:3306` → remote database
- **Browser-native HTTP**: `http://admin.localhost` → remote admin interface  
- **Automatic identity injection**: HTTP services receive client ID52 headers
- **Protocol agnostic**: HTTP, TCP, or any protocol

### ⚡ **Natural Remote Access**
- **SSH-like syntax**: `malai web01.company ps aux` (no quotes needed)
- **Interactive shells**: `malai web01.company` for full shell access
- **Command aliases**: `restart-nginx` → `sudo systemctl restart nginx`
- **Permission system**: Hierarchical groups with fine-grained access control

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
malai daemon &  # Starts all services for all clusters

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