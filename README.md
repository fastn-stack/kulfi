# malai: Secure P2P Infrastructure Platform

**Enterprise-grade remote access and service mesh with zero-configuration security.**

malai provides secure remote access to your machines and services using peer-to-peer networking. No central servers, no certificate authorities, no complex configuration - just cryptographically secure access to your infrastructure.

---

## Quick Start

### Personal Infrastructure Setup

```bash
# On your laptop (cluster manager):
malai cluster init personal
malai start &

# On your home server:  
malai machine init personal
# Add machine to cluster config on laptop
malai start &

# Now enjoy natural remote access:
malai home-server.personal htop
malai home-server.personal docker ps
open http://admin.localhost  # Direct browser access to remote services
```

### Enterprise Cluster Setup

```bash
# On ops machine (cluster manager):
malai cluster init company
malai start &

# On each server:
malai machine init company.example.com corp  # Join via domain
malai start &

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
- **Multiple clusters**: Personal, work, client clusters from single machine
- **Role flexibility**: Cluster manager of one, machine in others
- **Unified access**: Single agent handles all clusters
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

A single `malai start` command auto-detects roles and starts appropriate services.

## Real-World Examples

### DevOps Engineer
```bash
# Morning routine:
malai start &  # Starts all services for all clusters

# Server management:
malai web ps aux                    # Check web server processes
malai db backup                     # Run database backup
malai cache restart                 # Restart Redis cache

# Monitoring access:
open http://grafana.localhost       # Grafana dashboard
open http://logs.localhost          # Log analysis tools
```

### Multi-Cloud Setup
```bash
# Connect to multiple clouds:
malai machine init aws.mycompany.com aws
malai machine init gcp.mycompany.com gcp  
malai machine init personal.mylab.com lab

# Single agent handles all:
malai start &

# Cross-cloud access:
malai web01.aws systemctl status nginx
malai db01.gcp pg_stat_activity
malai home.lab htop
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