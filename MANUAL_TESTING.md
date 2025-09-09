# Manual Testing Guide

## Current Implementation Status

### ✅ What's Working:
- **Config distribution**: Cluster manager → machine via P2P ✅
- **Remote command execution**: `malai machine.cluster command` via P2P ✅ 
- **Separate processes**: Independent daemons with file locking ✅
- **MALAI_HOME isolation**: Each process completely isolated ✅

### ❌ What's NOT Working:
- **ACL enforcement**: Remote commands execute without permission checking ❌
- **Real command execution**: Currently simulated (not actual process execution) ❌
- **CLI → daemon socket**: Direct P2P calls, no connection pooling ❌
- **Environment variables**: `malai daemon -e` not integrated with shell ❌

## Manual Two-Cluster Test

### Setup Test Environment:
```bash
# Clean slate
rm -rf /tmp/malai-manual-test
mkdir -p /tmp/malai-manual-test/{cluster-mgr,machine1,machine2}

# Build malai (if not installed)
cargo build --bin malai
MALAI_BIN="./target/debug/malai"  # or "malai" if installed
```

### Test 1: Single Cluster - Config Distribution
```bash
# Terminal 1 - Cluster Manager
export MALAI_HOME="/tmp/malai-manual-test/cluster-mgr"
$MALAI_BIN cluster init company
# Edit cluster config to add machines:
echo '
[machine.web01]
id52 = "web01-id52-placeholder" 
allow_from = "*"

[machine.db01]
id52 = "db01-id52-placeholder"
allow_from = "admins"' >> $MALAI_HOME/clusters/company/cluster-config.toml

$MALAI_BIN daemon --foreground  # Should show config distribution

# Terminal 2 - Machine 1
export MALAI_HOME="/tmp/malai-manual-test/machine1"
# Manually create cluster registration (machine init P2P not implemented):
mkdir -p $MALAI_HOME/clusters/company
$MALAI_BIN keygen --file $MALAI_HOME/clusters/company/identity.key
echo 'cluster_alias = "company"
cluster_id52 = "7e0mv60nt1d0irqj5q2lsru98vigbehos74p1k0o4ddp5jk6i4mg"
machine_id52 = "mk2bhr3h2prb4bm69pi2ums5fmkki47jgfbclg7gu7rfg5t9g5o0"' > $MALAI_HOME/clusters/company/cluster-info.toml

$MALAI_BIN daemon --foreground  # Should receive config from cluster manager
```

### Expected Results:
- **Cluster manager**: "Config distributed to all machines"
- **Machine**: "Config received and saved successfully" 
- **Machine config**: `machine-config.toml` created with personalized config

### Test 2: Remote Command Execution
```bash
# With both daemons running from Test 1:

# Terminal 3 - Remote Access Test
export MALAI_HOME="/tmp/malai-manual-test/cluster-mgr"
$MALAI_BIN web01.company echo "Hello from remote!"

# Expected output:
# 🧪 Executing remote command...
# 📍 Target: web01.company  
# 🎯 Target machine ID52: mk2bhr3h2prb4bm69pi2ums5fmkki47jgfbclg7gu7rfg5t9g5o0
# 📡 Sending remote access command via fastn_p2p::call...
# Executed on company: echo ["Hello from remote!"]
# ✅ Remote command executed successfully
```

### Test 3: ACL Enforcement (Currently BROKEN)
```bash
# This should test permission denial but currently doesn't work:
$MALAI_BIN web01.company restricted-command

# Currently: Executes without checking permissions ❌
# Should: Check machine-config.toml allow_from and deny unauthorized access ✅
```

## What You Can Test Right Now:

### Working Tests:
1. **Config distribution**: Two separate processes, config gets distributed
2. **Remote execution**: P2P command execution with simulated output
3. **Daemon startup**: File locking, config validation, service orchestration
4. **Status reporting**: `malai status` shows comprehensive cluster info

### Broken/Missing Tests:
1. **ACL enforcement**: No permission checking (security issue!)
2. **Real commands**: Simulated execution, not actual process execution  
3. **Environment integration**: No shell environment variable support
4. **Connection pooling**: Each command creates fresh P2P connection

## Critical Issues to Fix:

### 🚨 Security Issue - No ACL Enforcement:
The `handle_execute_command_protocol` function currently:
```rust
// TODO: Validate permissions using machine_config  ← NOT IMPLEMENTED
// For now, simulate successful execution            ← BYPASSES SECURITY
```

This means **any machine can run any command** on any other machine, regardless of the `allow_from` field in the config!

### 🔧 Missing Real Execution:
Commands are simulated, not actually executed:
```rust
let output = format!("Executed on {}: {} {:?}", ...)  ← SIMULATION
// Should use: tokio::process::Command::new(command).args(args).spawn()
```

## Recommended Next Steps:
1. **Fix ACL enforcement**: Implement permission validation before command execution
2. **Implement real execution**: Replace simulation with actual process execution
3. **Add environment variables**: Complete `malai daemon -e` integration
4. **Test security**: Verify unauthorized commands are properly denied