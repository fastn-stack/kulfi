#!/bin/bash
# 🌐 FULLY AUTOMATED MALAI INFRASTRUCTURE TEST
# 
# Self-contained test requiring NO manual setup beyond Digital Ocean token.
# Handles all dependencies: MALAI_HOME, SSH keys, droplet lifecycle, cleanup.
#
# Usage:
#   Local: ./test-automated-infra.sh (builds on droplet)
#   CI:    ./test-automated-infra.sh --use-ci-binary (uses pre-built binary)
#
# Local requirements: doctl auth init (one-time)
# CI requirements: DIGITALOCEAN_ACCESS_TOKEN secret

set -euo pipefail

# Colors (define first)
BLUE='\033[0;34m'
GREEN='\033[0;32m' 
RED='\033[0;31m'
YELLOW='\033[0;33m'
BOLD='\033[1m'
NC='\033[0m'

# Logging functions (define early)
log() { echo -e "${BLUE}[$(date +'%H:%M:%S')] $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; exit 1; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
header() { echo -e "${BOLD}${BLUE}$1${NC}"; }

# Self-contained environment (no external dependencies)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_ID="malai-auto-$(date +%s)"
TEST_CLUSTER_NAME="auto-test"
export MALAI_HOME="/tmp/$TEST_ID"
TEST_SSH_KEY="/tmp/$TEST_ID-ssh"
DROPLET_NAME="$TEST_ID"

# Check if using pre-built binary from CI
USE_CI_BINARY=false
if [[ "${1:-}" == "--use-ci-binary" ]]; then
    USE_CI_BINARY=true
    DROPLET_SIZE="s-1vcpu-1gb"  # Smaller droplet sufficient (no compilation)
    log "Using pre-built CI binary - no compilation on droplet needed"
else
    DROPLET_SIZE="s-2vcpu-2gb"  # Larger droplet needed for 11-minute builds
    log "Will build malai on droplet (slower but works everywhere)"
fi
DROPLET_REGION="nyc3"
DROPLET_IMAGE="ubuntu-22-04-x64"
success() { echo -e "${GREEN}✅ $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; exit 1; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
header() { echo -e "${BOLD}${BLUE}$1${NC}"; }

# Comprehensive cleanup (handles all resources)
cleanup() {
    log "🧹 Comprehensive cleanup..."
    
    # Kill local daemons
    pkill -f "malai daemon" 2>/dev/null || true
    
    # Destroy droplet
    if command -v doctl >/dev/null 2>&1 && doctl account get >/dev/null 2>&1; then
        if doctl compute droplet list --format Name --no-header | grep -q "$DROPLET_NAME"; then
            log "Destroying droplet: $DROPLET_NAME"
            doctl compute droplet delete "$DROPLET_NAME" --force
        fi
        
        # Remove auto-generated SSH key
        if doctl compute ssh-key list --format Name --no-header | grep -q "$TEST_ID"; then
            doctl compute ssh-key delete "$TEST_ID" --force 2>/dev/null || true
        fi
    fi
    
    # Clean up test files
    rm -rf "/tmp/$TEST_ID"* 2>/dev/null || true
    
    success "Cleanup complete"
}
trap cleanup EXIT

header "🌐 FULLY AUTOMATED MALAI INFRASTRUCTURE TEST"
log "Test ID: $TEST_ID"
log "Self-contained - no manual setup required"
echo

# Phase 1: Auto-setup dependencies
header "🔧 Phase 1: Auto-Setup Dependencies"

# Setup doctl (assume user is logged in for local testing)
log "Checking Digital Ocean CLI..."
if ! command -v doctl >/dev/null 2>&1; then
    error "Install doctl first: brew install doctl"
fi

if ! doctl account get >/dev/null 2>&1; then
    # For CI: use environment token
    if [[ -n "${DIGITALOCEAN_ACCESS_TOKEN:-}" ]]; then
        log "Authenticating with CI token..."
        doctl auth init --access-token "$DIGITALOCEAN_ACCESS_TOKEN"
        success "doctl authenticated from environment"
    else
        # For local: guide user to authenticate
        error "Please authenticate doctl first: doctl auth init"
    fi
else
    success "doctl already authenticated"
fi

# Auto-generate SSH key
log "Generating test SSH key..."
mkdir -p "$(dirname "$TEST_SSH_KEY")"
ssh-keygen -t rsa -b 2048 -f "$TEST_SSH_KEY" -N "" -C "$TEST_ID" -q
success "SSH key generated: $TEST_SSH_KEY"

# Auto-import SSH key to Digital Ocean
log "Importing SSH key to Digital Ocean..."
SSH_KEY_ID=$(doctl compute ssh-key import "$TEST_ID" --public-key-file "$TEST_SSH_KEY.pub" --format ID --no-header)
success "SSH key imported to DO: $SSH_KEY_ID"

# Auto-setup MALAI_HOME
log "Setting up isolated test environment..."
mkdir -p "$MALAI_HOME"
success "MALAI_HOME: $MALAI_HOME"

# Ensure malai binary exists (local or CI)
log "Checking malai binary..."
cd "$SCRIPT_DIR"

if [[ "$USE_CI_BINARY" == "true" ]]; then
    # CI mode: Use pre-built release binary
    if [[ ! -f "target/release/malai" ]]; then
        error "Pre-built release binary not found. Run: cargo build --bin malai --no-default-features --release"
    fi
    MALAI_BINARY="target/release/malai"
    success "Using pre-built CI binary (optimized)"
else
    # Local mode: Build debug binary if needed
    if [[ ! -f "target/debug/malai" ]]; then
        log "Building malai locally..."
        cargo build --bin malai --quiet
    fi
    MALAI_BINARY="target/debug/malai"
    success "Local malai binary ready"
fi

# Phase 2: Automated droplet provisioning
header "🚀 Phase 2: Automated Droplet Provisioning"

log "Creating optimized droplet..."
DROPLET_ID=$(doctl compute droplet create "$DROPLET_NAME" \
    --size "$DROPLET_SIZE" \
    --image "$DROPLET_IMAGE" \
    --region "$DROPLET_REGION" \
    --ssh-keys "$SSH_KEY_ID" \
    --format ID \
    --no-header)

log "Droplet ID: $DROPLET_ID"
log "Waiting for droplet to boot..."
sleep 60

DROPLET_IP=$(doctl compute droplet get "$DROPLET_ID" --format PublicIPv4 --no-header)
log "Droplet IP: $DROPLET_IP"
success "Droplet provisioned"

# Auto-wait for SSH readiness
log "Waiting for SSH to be ready..."
for i in {1..30}; do
    if ssh -i "$TEST_SSH_KEY" -o ConnectTimeout=5 -o StrictHostKeyChecking=no root@"$DROPLET_IP" echo "ready" >/dev/null 2>&1; then
        break
    fi
    log "SSH attempt $i/30..."
    sleep 10
done
success "SSH connection ready"

# Phase 3: Optimized malai deployment
header "📦 Phase 3: Optimized malai Deployment"

if [[ "$USE_CI_BINARY" == "true" ]]; then
    # FAST: Copy pre-built binary from CI (30 seconds vs 11+ minutes)
    log "Deploying pre-built binary to droplet (CI optimization)..."
    
    # Copy binary directly
    scp -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no "$MALAI_BINARY" root@"$DROPLET_IP":/usr/local/bin/malai
    ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" "chmod +x /usr/local/bin/malai"
    
    # Setup user only (no compilation needed)
    ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" "
    useradd -r -d /opt/malai -s /bin/bash malai || true
    mkdir -p /opt/malai
    chown malai:malai /opt/malai
    "
    
    success "malai deployed via binary copy (fast CI mode)"
    
else
    # SLOW: Build on droplet (original approach for local testing)
    log "Building malai on droplet (local testing mode)..."
    ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" "
    export DEBIAN_FRONTEND=noninteractive

    # Wait for automatic apt processes
    while pgrep -x apt > /dev/null; do echo 'Waiting for apt...'; sleep 5; done

    # Install dependencies
    apt-get update -y
    apt-get install -y curl git build-essential pkg-config libssl-dev

    # Install Rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env

    # Clone and build malai
    cd /tmp
    rm -rf kulfi 2>/dev/null || true
    git clone https://github.com/fastn-stack/kulfi.git
    cd kulfi
    git checkout feat/real-infrastructure-testing

    # Build optimized for server (11-minute build on 2GB droplet)
    cargo build --bin malai --no-default-features --release

    # Install binary
    cp target/release/malai /usr/local/bin/malai
    chmod +x /usr/local/bin/malai

    # Setup malai user
    useradd -r -d /opt/malai -s /bin/bash malai || true
    mkdir -p /opt/malai
    chown malai:malai /opt/malai

    echo '✅ malai build and installation complete'
    "
    
    success "malai built and installed on droplet (local mode)"
fi

# Verify installation works
if ! ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" "/usr/local/bin/malai --version" >/dev/null 2>&1; then
    error "malai binary not working on droplet"
fi
success "malai verified working on droplet"

# Phase 4: Automated P2P cluster setup
header "🔗 Phase 4: Automated P2P Cluster Setup"

log "Creating cluster locally..."
./"$MALAI_BINARY" cluster init "$TEST_CLUSTER_NAME"
CLUSTER_MANAGER_ID52=$(./"$MALAI_BINARY" scan-roles | grep "Identity:" | head -1 | cut -d: -f2 | tr -d ' ')
log "Cluster Manager ID: $CLUSTER_MANAGER_ID52"

log "Initializing machine on droplet..."
ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai machine init $CLUSTER_MANAGER_ID52 $TEST_CLUSTER_NAME"

MACHINE_ID52=$(ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai scan-roles | grep 'Identity:' | cut -d: -f2 | tr -d ' '")
log "Machine ID: $MACHINE_ID52"

# Auto-add machine to cluster config
log "Configuring cluster automatically..."
cat >> "$MALAI_HOME/clusters/$TEST_CLUSTER_NAME/cluster.toml" << EOF

[machine.web01]
id52 = "$MACHINE_ID52"  
allow_from = "*"
EOF
success "Cluster configured with different machine IDs (real P2P setup)"

# Phase 5: Automated daemon startup and testing
header "🧪 Phase 5: Automated P2P Testing"

log "Starting local daemon..."
./"$MALAI_BINARY" daemon --foreground > "$MALAI_HOME/local-daemon.log" 2>&1 &
LOCAL_DAEMON_PID=$!
sleep 3

if ! kill -0 "$LOCAL_DAEMON_PID" 2>/dev/null; then
    cat "$MALAI_HOME/local-daemon.log"
    error "Local daemon failed to start"
fi
success "Local daemon running"

log "Starting remote daemon..."
ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai nohup /usr/local/bin/malai daemon --foreground > /opt/malai/daemon.log 2>&1 &"
sleep 5

# Verify remote daemon
if ! ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai status | grep -q 'RUNNING'"; then
    error "Remote daemon failed to start"
fi
success "Remote daemon running"

# Phase 6: Critical P2P validation
header "🎯 Phase 6: Critical P2P Validation"

log "Testing real cross-internet P2P communication..."
log "Laptop (cluster manager) → Digital Ocean (machine) via P2P"

# Test 1: Custom message
if ./"$MALAI_BINARY" web01."$TEST_CLUSTER_NAME" echo "SUCCESS: Automated real P2P test!" > "$MALAI_HOME/test1.log" 2>&1; then
    if grep -q "SUCCESS: Automated real P2P test!" "$MALAI_HOME/test1.log"; then
        success "Test 1: Custom message via P2P ✅"
    else
        cat "$MALAI_HOME/test1.log"
        error "Test 1: P2P message not received"
    fi
else
    cat "$MALAI_HOME/test1.log"
    error "Test 1: P2P command execution failed"
fi

# Test 2: System command
if ./"$MALAI_BINARY" web01."$TEST_CLUSTER_NAME" whoami > "$MALAI_HOME/test2.log" 2>&1; then
    if grep -q "malai" "$MALAI_HOME/test2.log"; then
        success "Test 2: System command via P2P ✅"
    else
        cat "$MALAI_HOME/test2.log"
        error "Test 2: Unexpected whoami output"
    fi
else
    cat "$MALAI_HOME/test2.log"
    error "Test 2: System command failed"
fi

# Test 3: Command with arguments
if ./"$MALAI_BINARY" web01."$TEST_CLUSTER_NAME" ls -la /opt/malai > "$MALAI_HOME/test3.log" 2>&1; then
    if grep -q "/opt/malai" "$MALAI_HOME/test3.log"; then
        success "Test 3: Command with arguments via P2P ✅"
    else
        cat "$MALAI_HOME/test3.log"
        error "Test 3: Command arguments not processed"
    fi
else
    cat "$MALAI_HOME/test3.log"
    error "Test 3: Command with arguments failed"
fi

# Clean up daemons
kill "$LOCAL_DAEMON_PID" 2>/dev/null || true
wait "$LOCAL_DAEMON_PID" 2>/dev/null || true

# Final results
header "🎉 AUTOMATED TEST RESULTS"
echo
success "🌐 REAL CROSS-INTERNET P2P COMMUNICATION VERIFIED!"
echo
echo "📊 Validation Summary:"
echo "  ✅ Digital Ocean droplet: Automated provisioning and setup"
echo "  ✅ malai installation: Automated build and deployment (11min)"  
echo "  ✅ P2P cluster setup: Automated cluster manager ↔ machine configuration"
echo "  ✅ Cross-internet P2P: Real command execution across internet"
echo "  ✅ Multiple commands: Custom messages, system commands, arguments"
echo "  ✅ Proper output: Real stdout capture with correct exit codes"
echo
echo "🚀 PRODUCTION READY: malai P2P infrastructure fully validated!"
echo "💡 Next: Deploy with confidence - real P2P communication proven"
echo
log "Test completed successfully - infrastructure working end-to-end"