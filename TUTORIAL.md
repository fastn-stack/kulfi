# malai Tutorial: Complete Infrastructure Management Guide

This tutorial covers everything you need to know to use malai for production P2P infrastructure management.

## Table of Contents

- [Quick Start](#quick-start)
- [Daemon Management](#daemon-management)
- [Cluster Management](#cluster-management) 
- [Production Deployment](#production-deployment)
- [Troubleshooting](#troubleshooting)
- [Advanced Usage](#advanced-usage)

## Quick Start

Get malai running in under 5 minutes:

### Installation

```bash
# Install malai (macOS/Linux)
curl -fsSL https://malai.sh/install.sh | sh

# Or build from source
git clone https://github.com/fastn-stack/kulfi.git
cd kulfi
cargo build --bin malai
```

### Your First Cluster

```bash
# Create a cluster (this machine becomes cluster manager)
malai cluster init personal

# Start the daemon
malai daemon

# Check status
malai status
```

### Add Another Machine

On a second machine:

```bash
# Join the cluster using cluster manager ID52 (shown in malai status)
malai machine init <cluster-manager-id52> personal

# Start daemon to accept commands
malai daemon
```

On the cluster manager, add the new machine to the config and update:

```bash
# Edit cluster configuration (add machine section from init output)
$EDITOR $MALAI_HOME/clusters/personal/cluster.toml

# Update running daemon with new machine
malai rescan personal
```

### Execute Commands

```bash
# Run commands on remote machines
malai web01.personal ps aux
malai web01.personal whoami
malai web01.personal systemctl status nginx
```

## Daemon Management

The malai daemon is the core of your P2P infrastructure.

### Starting and Stopping

```bash
# Development mode (foreground, shows all output)
malai daemon --foreground

# Production mode (background)
malai daemon

# Check if daemon is running
malai status
```

### Daemon Status and Health

The `malai status` command provides comprehensive diagnostics:

```bash
$ malai status
📊 malai Status
═══════════════════════════════════════
📁 MALAI_HOME: /Users/admin/.malai
🔒 Daemon: RUNNING ✅
   📁 Lock: /Users/admin/.malai/malai.lock
   🔌 Socket: /Users/admin/.malai/malai.socket (CLI communication active)
🔍 Testing daemon responsiveness... ✅ RESPONSIVE

🏗️  Cluster Configurations:
   👑 company (Cluster Manager)
      📄 Config: /Users/admin/.malai/clusters/company/cluster.toml
      📊 Machines: 3

🖥️  Machine Configurations:
   💻 production (Machine)
      📄 Config: /Users/admin/.malai/clusters/production/machine.toml
```

**Status Indicators:**
- **RUNNING ✅**: Daemon healthy and responsive
- **STARTING ⚠️**: Daemon lock exists but socket not ready
- **CRASHED ❌**: Socket exists but no lock (stale socket)
- **NOT RUNNING 💤**: No daemon processes

### Configuration Management

Update daemon configuration without restarts:

```bash
# Create new cluster (automatically updates daemon)
malai cluster init staging

# Add new machine (automatically updates daemon)  
malai machine init <cluster-id52> production

# Manual rescan (selective - only affects specific cluster)
malai rescan staging

# Manual rescan (full - affects all clusters)
malai rescan

# Validate configurations
malai rescan --check staging  # Check specific cluster
malai rescan --check          # Check all clusters
```

**Key Features:**
- **Automatic Updates**: Init commands automatically update running daemon
- **Selective Rescans**: Target specific clusters to avoid disrupting stable ones
- **Zero Downtime**: Configuration changes don't require daemon restarts
- **Strict Error Handling**: Invalid configurations fail immediately

## Cluster Management

### Creating Clusters

```bash
# Initialize new cluster (this machine becomes cluster manager)
malai cluster init company

# What this creates:
# $MALAI_HOME/clusters/company/
# ├── cluster.toml          # Cluster configuration
# └── cluster.private-key   # Cluster manager identity (KEEP SECURE!)
```

### Adding Machines to Clusters

**Step 1: Initialize machine**
On the target machine:

```bash
malai machine init <cluster-manager-id52> company
```

This outputs machine details like:
```
Machine created with ID: abc123...xyz789
📋 Next steps:
1. Cluster admin should add this machine to cluster config:
   [machine.web01]
   id52 = "abc123...xyz789"
   allow_from = "*"
```

**Step 2: Add machine to cluster config**
On the cluster manager machine:

```bash
# Edit cluster configuration
$EDITOR $MALAI_HOME/clusters/company/cluster.toml

# Add the machine section (from step 1 output):
[machine.web01]
id52 = "abc123...xyz789"
allow_from = "*"

# Update running daemon
malai rescan company
```

**Step 3: Start daemon on target machine**
```bash
malai daemon
```

### Multi-Cluster Deployments

A single machine can participate in multiple clusters:

```bash
# Create personal cluster (as cluster manager)
malai cluster init personal

# Join work cluster (as machine)
malai machine init <work-cluster-id52> work

# Join client cluster (as machine)  
malai machine init <client-cluster-id52> client

# Single daemon handles all clusters
malai daemon

# Access different clusters
malai web01.personal ps aux
malai api.work systemctl status nginx  
malai db.client pg_dump mydb
```

### Security and Access Control

**Cryptographic Identity:**
- Each cluster has unique cluster manager identity
- Each machine has unique identity  
- Only machines in cluster config can connect
- No passwords or certificates required

**Access Control Examples:**
```toml
# Basic access (all commands allowed)
[machine.web01]
id52 = "machine-id52"
allow_from = "*"

# Restricted access (only specific groups)  
[machine.prod01]
id52 = "machine-id52"
allow_from = "admins,devops"

# Command-specific permissions
[machine.web01.command.restart-nginx]
command = "sudo systemctl restart nginx"
allow_from = "admins"
```

## Production Deployment

### System Requirements

**Minimum:**
- CPU: 1 core
- RAM: 512 MB  
- Disk: 100 MB + logs
- OS: Linux/macOS

**Production Recommended:**
- CPU: 2+ cores
- RAM: 2+ GB
- Disk: 10+ GB
- Network: Stable internet

### Production Setup

**1. Create dedicated user:**
```bash
sudo useradd -r -d /opt/malai -s /bin/false malai
sudo mkdir -p /opt/malai
sudo chown malai:malai /opt/malai
```

**2. Install malai:**
```bash
sudo curl -fsSL https://malai.sh/install.sh | sh
sudo mv ~/.malai/bin/malai /usr/local/bin/malai
```

**3. Initialize cluster:**
```bash
sudo -u malai env MALAI_HOME=/opt/malai malai cluster init production
```

**4. Create systemd service:**
```bash
sudo tee /etc/systemd/system/malai.service << 'EOF'
[Unit]
Description=malai P2P Infrastructure Daemon
After=network.target

[Service]
Type=simple
User=malai
Group=malai
Environment=MALAI_HOME=/opt/malai
Environment=RUST_LOG=malai=info
ExecStart=/usr/local/bin/malai daemon --foreground
Restart=always
RestartSec=5

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/malai

[Install]
WantedBy=multi-user.target
EOF
```

**5. Enable and start:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable malai
sudo systemctl start malai
sudo systemctl status malai
```

### Monitoring and Logging

**Health checks:**
```bash
# Regular status check
sudo -u malai env MALAI_HOME=/opt/malai malai status

# Monitor logs
sudo journalctl -u malai -f

# Health check script
echo '#!/bin/bash
sudo -u malai env MALAI_HOME=/opt/malai malai status | grep -q "RUNNING ✅"' | sudo tee /usr/local/bin/malai-healthcheck
sudo chmod +x /usr/local/bin/malai-healthcheck
```

## Troubleshooting

### Common Issues

**Daemon won't start:**
```bash
# Check status
malai status

# Validate configurations
malai rescan --check

# Remove stale lock
rm $MALAI_HOME/malai.lock

# Check for errors
malai daemon --foreground
```

**Commands hang or timeout:**
```bash
# Verify daemon running on target machine
malai status

# Test simple command first
malai web01.company echo "test"

# Check cluster configuration
cat $MALAI_HOME/clusters/company/cluster.toml
```

**Socket communication errors:**
```bash
# Test daemon responsiveness
malai status  # Should show "RESPONSIVE"

# Remove stale socket
rm $MALAI_HOME/malai.socket
malai daemon
```

**Configuration errors:**
```bash
# Check specific cluster
malai rescan --check company

# Check all clusters  
malai rescan --check

# Fix TOML syntax errors shown in output
```

### Debugging Tools

**Enable debug logging:**
```bash
export RUST_LOG=malai=debug
malai daemon --foreground
```

**Manual cluster testing:**
```bash
# Test without daemon (direct CLI mode)
malai web01.company echo "direct mode test"

# Compare with daemon mode
malai daemon &
malai web01.company echo "daemon mode test"
```

**File system debugging:**
```bash
# Verify MALAI_HOME structure
find $MALAI_HOME -type f -name "*.toml" -o -name "*.key"

# Check permissions
ls -la $MALAI_HOME/clusters/*/

# Verify daemon files
ls -la $MALAI_HOME/malai.*
```

## Advanced Usage

### Selective Cluster Management

```bash
# Only rescan specific cluster (safer for production)
malai rescan production

# Validate specific cluster without changes
malai rescan --check production

# Full rescan (affects all clusters)
malai rescan
```

### Multi-Environment Workflows

```bash
# Development machine participating in multiple environments
malai cluster init personal          # Personal projects (cluster manager)
malai machine init <prod-id52> prod  # Production access (machine)  
malai machine init <stage-id52> stage # Staging access (machine)

# Switch between environments seamlessly
malai web01.personal ps aux       # Personal cluster
malai api.prod systemctl status   # Production cluster  
malai db.stage pg_dump myapp      # Staging cluster
```

### Backup and Recovery

**Critical: Backup cluster manager keys**
```bash
# Backup all cluster identities (CRITICAL)
tar -czf malai-backup-$(date +%Y%m%d).tar.gz $MALAI_HOME/clusters/

# Configuration backup (for version control)
tar -czf malai-configs-$(date +%Y%m%d).tar.gz $MALAI_HOME/clusters/*/cluster.toml
```

**Disaster recovery:**
```bash
# Restore from backup
cd / && tar -xzf malai-backup-20241201.tar.gz

# Restart daemon
malai daemon

# Verify recovery
malai status
```

### Performance Optimization

**Use daemon mode for better performance:**
```bash
# Daemon mode (connection pooling)
malai daemon  

# Commands reuse connections = faster execution
malai web01.company ps aux  # Fast (reuses connection)
```

**Monitor daemon performance:**
```bash
# Check responsiveness
malai status  # Should show "RESPONSIVE"

# Test command speed
time malai web01.company echo "speed test"
```

## Security Best Practices

### Private Key Protection

**CRITICAL**: Always protect cluster manager private keys:

```bash
# Secure permissions
chmod 600 $MALAI_HOME/clusters/*/cluster.private-key
chmod 700 $MALAI_HOME/clusters/

# Regular encrypted backups
tar -czf /secure/backup/malai-keys-$(date +%Y%m%d).tar.gz $MALAI_HOME/clusters/*/cluster.private-key
```

### Network Security

- **No open ports**: malai uses P2P networking, no inbound firewall rules needed
- **Local communication**: Unix socket only accessible locally
- **Encrypted**: All cluster communication encrypted end-to-end
- **Identity-based**: Only authorized machines can join clusters

### Production Security

```bash
# Run as dedicated user
sudo useradd -r malai

# Restrict file permissions
sudo chown -R malai:malai /opt/malai
sudo chmod 700 /opt/malai

# Use systemd security features
# (see systemd service configuration above)
```

## Getting Help

If you encounter issues:

1. **Check malai status**: `malai status` provides comprehensive diagnostics
2. **Validate configs**: `malai rescan --check` shows configuration issues
3. **GitHub Issues**: [Report bugs](https://github.com/fastn-stack/kulfi/issues) 
4. **Discord Community**: [Join fastn Discord](https://discord.gg/nK4ZP8HpV7)
5. **Technical Design**: See [DESIGN.md](DESIGN.md) for architecture details

**When reporting issues, include:**
- Output of `malai status`
- Output of `malai rescan --check`
- Relevant daemon logs from `malai daemon --foreground`
- Your cluster configuration (remove private keys!)

---

**Built with [fastn-p2p](https://github.com/fastn-stack/fastn) • Cryptographic verification • Production ready**