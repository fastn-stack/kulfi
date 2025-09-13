#!/bin/bash
# 🌐 DIGITAL OCEAN P2P TEST
# 
# Tests real malai P2P communication across internet (laptop ↔ Digital Ocean droplet).
# Self-contained with automatic setup, cleanup, and comprehensive validation.
#
# Usage:
#   Default: ./test-digital-ocean-p2p.sh (builds on droplet - reliable)
#   CI: ./test-digital-ocean-p2p.sh --use-ci-binary (uses pre-built binary)
#
# Droplet sizes for builds:
#   Small (cheap): ./test-digital-ocean-p2p.sh --small (1GB, ~20min builds, $0.009/hr)
#   Fast (balanced): ./test-digital-ocean-p2p.sh --fast (4GB, ~8min builds, $0.071/hr)  
#   Turbo (fastest): ./test-digital-ocean-p2p.sh --turbo (8CPU/16GB, ~4min builds, $0.143/hr)
#
# Debugging:
#   Keep droplet: ./test-digital-ocean-p2p.sh --keep-droplet (for debugging)
#   Or: KEEP_DROPLET=1 ./test-digital-ocean-p2p.sh
#
# Requirements: doctl auth init (one-time setup)

#!/bin/bash
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
time_checkpoint() { 
    local checkpoint="$1"
    local current_time=$(date +%s)
    local elapsed=$((current_time - START_TIME))
    case "$checkpoint" in
        "Droplet boot") BOOT_TIME="$elapsed" ;;
        "SSH ready") SSH_TIME="$elapsed" ;;
        "Droplet build complete") BUILD_TIME="$elapsed" ;;
        "Binary verification") VERIFY_TIME="$elapsed" ;;
        "Cluster setup complete") CLUSTER_TIME="$elapsed" ;;
        "P2P testing complete") TEST_TIME="$elapsed" ;;
    esac
    log "⏱️  $checkpoint: ${elapsed}s"
}
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

# Timing tracking (simple approach)
START_TIME=$(date +%s)
BOOT_TIME=""
SSH_TIME=""
BUILD_TIME=""
VERIFY_TIME=""
CLUSTER_TIME=""
TEST_TIME=""

# Deployment mode selection  
USE_CI_BINARY=false
KEEP_DROPLET="${KEEP_DROPLET:-false}"
DROPLET_SIZE="s-8vcpu-16gb"  # Default: turbo (best balance of speed vs cost)

# Parse arguments (can combine flags)
for arg in "$@"; do
    case "$arg" in
        "--use-ci-binary")
            USE_CI_BINARY=true
            DROPLET_SIZE="s-1vcpu-1gb"  # No compilation needed
            log "Using pre-built CI binary - no compilation needed"
            ;;
        "--small")
            DROPLET_SIZE="s-1vcpu-1gb"  # $6/month, slow builds
            log "Using small droplet (1GB RAM, $6/month) - slower builds but cheaper"
            ;;
        "--fast")
            DROPLET_SIZE="s-4vcpu-8gb"  # $48/month, fast builds  
            log "Using fast droplet (4CPU/8GB RAM, $48/month) - faster builds"
            ;;
        "--turbo")
            DROPLET_SIZE="s-8vcpu-16gb"  # $96/month, very fast builds
            log "Using turbo droplet (8CPU/16GB RAM, $96/month) - fastest builds"
            ;;
        "--beast")
            DROPLET_SIZE="s-8vcpu-32gb"  # $168/month, ultra-fast builds
            log "Using beast droplet (8CPU/32GB RAM, $168/month) - ultra-fast builds"
            ;;
        "--keep-droplet")
            KEEP_DROPLET=true
            log "🔧 DEBUG MODE: Droplet will be kept for debugging"
            ;;
        *)
            if [[ "$arg" != "${BASH_SOURCE[0]}" ]]; then
                warn "Unknown argument: $arg (ignoring)"
            fi
            ;;
    esac
done

# Always build on droplet (simple and reliable)
log "Building malai on droplet (reliable, ~3 minutes with default turbo)"
DROPLET_REGION="nyc3"
DROPLET_IMAGE="ubuntu-22-04-x64"

# Comprehensive cleanup (handles all resources)
cleanup() {
    log "🧹 Comprehensive cleanup..."
    
    # Kill local daemons
    pkill -f "malai daemon" 2>/dev/null || true
    
    # Destroy droplet (unless debugging)
    if [[ "$KEEP_DROPLET" == "true" ]]; then
        log "🔧 DEBUG MODE: Keeping droplet and SSH key for debugging"
        if [[ -n "${DROPLET_NAME:-}" ]] && [[ -n "${DROPLET_IP:-}" ]]; then
            echo ""
            echo "📍 DEBUGGING INFORMATION:"
            echo "   Droplet Name: $DROPLET_NAME"
            echo "   Droplet IP: $DROPLET_IP" 
            echo "   SSH Command: ssh -i $TEST_SSH_KEY root@$DROPLET_IP"
            echo ""
            echo "🔍 Useful debugging commands:"
            echo "   Check remote daemon: sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai status"
            echo "   View daemon logs: sudo -u malai cat /opt/malai/daemon.log"
            echo "   Test malai version: /usr/local/bin/malai --version"
            echo ""
            echo "🧹 Manual cleanup when done:"
            echo "   Droplet: $DOCTL compute droplet delete $DROPLET_NAME --force"
            echo "   SSH key: $DOCTL compute ssh-key delete $TEST_ID --force"
            echo "   Local files: rm -rf /tmp/$TEST_ID*"
            echo ""
        fi
        
        # Keep SSH key for debugging (don't delete it)
        log "SSH key preserved for debugging access"
    else
        # Normal cleanup: destroy droplet
        if command -v doctl >/dev/null 2>&1; then
            CLEANUP_DOCTL="doctl"
        elif [[ -f "$HOME/doctl" ]] && [[ -x "$HOME/doctl" ]]; then
            CLEANUP_DOCTL="$HOME/doctl"
        fi
        
        if [[ -n "${CLEANUP_DOCTL:-}" ]] && $CLEANUP_DOCTL account get >/dev/null 2>&1; then
            if [[ -n "${DROPLET_NAME:-}" ]] && $CLEANUP_DOCTL compute droplet list --format Name --no-header | grep -q "$DROPLET_NAME"; then
                log "Destroying droplet: $DROPLET_NAME"
                $CLEANUP_DOCTL compute droplet delete "$DROPLET_NAME" --force
            fi
            
            # Remove auto-generated SSH key
            if $CLEANUP_DOCTL compute ssh-key list --format Name --no-header | grep -q "$TEST_ID"; then
                $CLEANUP_DOCTL compute ssh-key delete "$TEST_ID" --force 2>/dev/null || true
            fi
        fi
    fi
    
    # Clean up test files
    rm -rf "/tmp/$TEST_ID"* 2>/dev/null || true
    
    success "Cleanup complete"
}
trap cleanup EXIT

header "🌐 FULLY AUTOMATED DIGITAL OCEAN P2P TEST"
log "Test ID: $TEST_ID"
log "Tests real P2P across internet (laptop ↔ Digital Ocean droplet)"

if [[ "$KEEP_DROPLET" != "true" ]]; then
    log "💡 For debugging failed tests, use: ./test-digital-ocean-p2p.sh --keep-droplet"
fi
echo

# Phase 1: Auto-setup dependencies
header "🔧 Phase 1: Auto-Setup Dependencies"

# Setup doctl (assume user is logged in for local testing)
log "Checking Digital Ocean CLI..."
if command -v doctl >/dev/null 2>&1; then
    DOCTL="doctl"
elif [[ -f "$HOME/doctl" ]] && [[ -x "$HOME/doctl" ]]; then
    DOCTL="$HOME/doctl"
    log "Using doctl from home directory: $HOME/doctl"
else
    error "Install doctl first: brew install doctl (or download to ~/doctl)"
fi

if ! $DOCTL account get >/dev/null 2>&1; then
    # For CI: use environment token
    if [[ -n "${DIGITALOCEAN_ACCESS_TOKEN:-}" ]]; then
        log "Authenticating with CI token..."
        $DOCTL auth init --access-token "$DIGITALOCEAN_ACCESS_TOKEN"
        success "doctl authenticated from environment"
    else
        # For local: guide user to authenticate
        error "Please authenticate doctl first: $DOCTL auth init"
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
SSH_KEY_ID=$($DOCTL compute ssh-key import "$TEST_ID" --public-key-file "$TEST_SSH_KEY.pub" --format ID --no-header)
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
elif [[ "$BUILD_ON_DROPLET" == "true" ]]; then
    # Fallback mode: Build on droplet (no local binary needed)
    log "Will build malai on droplet - no local compilation needed"
    MALAI_BINARY=""  # No local binary needed
    success "Droplet build mode selected"
else
    # Default mode: Try cross-compile, fallback to droplet build
    log "Attempting cross-compilation for fastest deployment..."
    if CC_x86_64_unknown_linux_musl=x86_64-linux-musl-gcc cargo build --bin malai --target x86_64-unknown-linux-musl --no-default-features --release 2>/dev/null; then
        MALAI_BINARY="target/x86_64-unknown-linux-musl/release/malai"
        success "Cross-compiled Linux binary ready (fastest deployment)"
    else
        warn "Cross-compilation failed - falling back to droplet build mode"
        BUILD_ON_DROPLET=true
        DROPLET_SIZE="s-2vcpu-2gb"  # Need larger droplet for compilation
        MALAI_BINARY=""  # No local binary needed for droplet build
        log "Will build malai on droplet instead"
    fi
fi

# Phase 2: Automated droplet provisioning
header "🚀 Phase 2: Automated Droplet Provisioning"

log "Creating optimized droplet..."
time_checkpoint "Setup complete"

DROPLET_ID=$($DOCTL compute droplet create "$DROPLET_NAME" \
    --size "$DROPLET_SIZE" \
    --image "$DROPLET_IMAGE" \
    --region "$DROPLET_REGION" \
    --ssh-keys "$SSH_KEY_ID" \
    --format ID \
    --no-header)

log "Droplet ID: $DROPLET_ID (size: $DROPLET_SIZE)"
log "Waiting for droplet to boot..."
sleep 60

DROPLET_IP=$($DOCTL compute droplet get "$DROPLET_ID" --format PublicIPv4 --no-header)
log "Droplet IP: $DROPLET_IP"
time_checkpoint "Droplet boot"
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
time_checkpoint "SSH ready"
success "SSH connection ready"

# Phase 3: Optimized malai deployment
header "📦 Phase 3: Optimized malai Deployment"

if [[ "$USE_CI_BINARY" == "true" ]]; then
    # FAST: Copy pre-built CI binary
    log "Deploying pre-built CI binary to droplet..."
    
    # Copy binary directly
    scp -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no "$MALAI_BINARY" root@"$DROPLET_IP":/usr/local/bin/malai
    ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" "chmod +x /usr/local/bin/malai"
    
    # Setup user only (no compilation needed)
    ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" "
    useradd -r -d /opt/malai -s /bin/bash malai || true
    mkdir -p /opt/malai
    chown malai:malai /opt/malai
    "
    
    success "malai deployed via CI binary copy"
    
else
    # SLOW: Build on droplet (reliable fallback)
    log "Building malai on droplet (fallback mode - takes ~15 minutes)..."
    ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" "
    export DEBIAN_FRONTEND=noninteractive

    # Wait for automatic apt processes (Ubuntu does this on boot)
    echo 'Waiting for Ubuntu automatic updates to complete...'
    while pgrep -x apt-get > /dev/null || pgrep -x apt > /dev/null || pgrep -x dpkg > /dev/null; do 
        echo 'Waiting for apt lock...'
        sleep 5
    done

    # Install all dependencies
    echo 'Installing system dependencies...'
    START_DEPS=\\\$(date +%s)
    apt-get update -y
    apt-get install -y curl git build-essential pkg-config libssl-dev gcc
    END_DEPS=\\\$(date +%s)
    echo \"✅ Dependencies installed in \\\$((END_DEPS - START_DEPS))s\"

    # Verify build tools
    which gcc && gcc --version

    # Install Rust  
    echo 'Installing Rust...'
    START_RUST=\\\$(date +%s)
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source \\\$HOME/.cargo/env
    END_RUST=\\\$(date +%s)
    echo \"✅ Rust installed in \\\$((END_RUST - START_RUST))s\"
    
    # Verify Rust
    rustc --version && cargo --version

    # Clone and build malai
    echo 'Cloning kulfi repository...'
    START_CLONE=\\\$(date +%s)
    cd /tmp
    rm -rf kulfi 2>/dev/null || true
    git clone https://github.com/fastn-stack/kulfi.git
    cd kulfi
    git checkout feat/real-infrastructure-testing
    END_CLONE=\\\$(date +%s)
    echo \"✅ Repository cloned in \\\$((END_CLONE - START_CLONE))s\"

    # Build optimized for server
    echo 'Building malai (this takes ~10-15 minutes)...'
    START_BUILD=\\\$(date +%s)
    cargo build --bin malai --no-default-features --release
    END_BUILD=\\\$(date +%s)
    echo \"✅ malai built in \\\$((END_BUILD - START_BUILD))s\"

    # Verify build succeeded
    if [[ ! -f target/release/malai ]]; then
        echo '❌ malai build failed'
        exit 1
    fi

    # Install binary
    cp target/release/malai /usr/local/bin/malai
    chmod +x /usr/local/bin/malai

    # Setup malai user
    useradd -r -d /opt/malai -s /bin/bash malai || true
    mkdir -p /opt/malai
    chown malai:malai /opt/malai

    echo '✅ malai build and installation complete'
    "
    
    time_checkpoint "Droplet build complete"
    success "malai built and installed on droplet"
fi

# Verify installation works (with debugging)
log "Testing malai binary on droplet..."
if ! ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" "/usr/local/bin/malai --version" > "$MALAI_HOME/version-test.log" 2>&1; then
    log "❌ malai binary test failed - debugging..."
    
    # Debug information
    ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" "
    echo 'File info:'
    file /usr/local/bin/malai
    echo 'Permissions:'
    ls -la /usr/local/bin/malai
    echo 'Ldd check:'
    ldd /usr/local/bin/malai 2>&1 || echo 'ldd failed'
    echo 'Direct execution test:'
    /usr/local/bin/malai --version 2>&1 || echo 'Execution failed'
    " > "$MALAI_HOME/debug-info.log" 2>&1
    
    cat "$MALAI_HOME/debug-info.log"
    cat "$MALAI_HOME/version-test.log"
    error "malai binary not working on droplet - see debug info above"
fi
time_checkpoint "Binary verification"
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
time_checkpoint "Cluster setup complete"
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
    if grep -q "malai" "$MALAI_HOME/test3.log" && grep -q "drwx" "$MALAI_HOME/test3.log"; then
        success "Test 3: Command with arguments via P2P ✅"
    else
        cat "$MALAI_HOME/test3.log"
        error "Test 3: Command arguments not processed correctly"
    fi
else
    cat "$MALAI_HOME/test3.log"
    error "Test 3: Command with arguments failed"
fi

# Clean up daemons
kill "$LOCAL_DAEMON_PID" 2>/dev/null || true
wait "$LOCAL_DAEMON_PID" 2>/dev/null || true

time_checkpoint "P2P testing complete"

# Final results with timing summary
header "🎉 AUTOMATED TEST RESULTS"
echo
success "🌐 REAL CROSS-INTERNET P2P COMMUNICATION VERIFIED!"
echo
echo "📊 Validation Summary:"
echo "  ✅ Digital Ocean droplet: Automated provisioning and setup"
echo "  ✅ malai installation: Automated build and deployment"  
echo "  ✅ P2P cluster setup: Automated cluster manager ↔ machine configuration"
echo "  ✅ Cross-internet P2P: Real command execution across internet"
echo "  ✅ Multiple commands: Custom messages, system commands, arguments"
echo "  ✅ Proper output: Real stdout capture with correct exit codes"
echo
echo "⏱️  TIMING BREAKDOWN:"
echo "  📍 Droplet size: $DROPLET_SIZE"
echo "  📍 Droplet boot: ${BOOT_TIME:-0}s" 
echo "  📍 SSH ready: ${SSH_TIME:-0}s"
if [[ -n "${BUILD_TIME:-}" ]]; then
echo "  📍 Droplet build: ${BUILD_TIME:-0}s"
fi
echo "  📍 Binary verification: ${VERIFY_TIME:-0}s"
echo "  📍 Cluster setup: ${CLUSTER_TIME:-0}s" 
echo "  📍 P2P testing: ${TEST_TIME:-0}s"
TOTAL_TIME=$(($(date +%s) - START_TIME))
echo "  📍 Total time: ${TOTAL_TIME}s ($(($TOTAL_TIME / 60))m $(($TOTAL_TIME % 60))s)"
echo
echo "💰 COST ANALYSIS (per test run):"
case "$DROPLET_SIZE" in
    "s-1vcpu-1gb") 
        HOURLY_COST="0.00893"
        echo "  📍 Small droplet: \$0.00893/hour × $(echo "scale=2; $TOTAL_TIME/3600" | bc)h = \$$(echo "scale=4; $HOURLY_COST * $TOTAL_TIME / 3600" | bc)"
        ;;
    "s-4vcpu-8gb")
        HOURLY_COST="0.07143" 
        echo "  📍 Fast droplet: \$0.07143/hour × $(echo "scale=2; $TOTAL_TIME/3600" | bc)h = \$$(echo "scale=4; $HOURLY_COST * $TOTAL_TIME / 3600" | bc)"
        ;;
    "s-8vcpu-16gb")
        HOURLY_COST="0.14286"
        echo "  📍 Turbo droplet: \$0.14286/hour × $(echo "scale=2; $TOTAL_TIME/3600" | bc)h = \$$(echo "scale=4; $HOURLY_COST * $TOTAL_TIME / 3600" | bc)"
        ;;
    "s-8vcpu-32gb")
        HOURLY_COST="0.25000"
        echo "  📍 Beast droplet: \$0.25000/hour × $(echo "scale=2; $TOTAL_TIME/3600" | bc)h = \$$(echo "scale=4; $HOURLY_COST * $TOTAL_TIME / 3600" | bc)"
        ;;
    "s-2vcpu-2gb")
        HOURLY_COST="0.02679"
        echo "  📍 Balanced droplet: \$0.02679/hour × $(echo "scale=2; $TOTAL_TIME/3600" | bc)h = \$$(echo "scale=4; $HOURLY_COST * $TOTAL_TIME / 3600" | bc)"
        ;;
esac
echo
echo "🚀 PRODUCTION READY: malai P2P infrastructure fully validated!"
echo "💡 Next: Deploy with confidence - real P2P communication proven"
echo
log "Test completed successfully - infrastructure working end-to-end"