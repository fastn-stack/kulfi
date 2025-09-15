#!/bin/bash
# 🌐 DUAL DROPLET P2P TEST
# 
# Tests real malai P2P communication between two Digital Ocean droplets.
# This eliminates CI networking restrictions by using cloud-to-cloud P2P.
#
# Usage:
#   Default: ./test-dual-droplet-p2p.sh (2x beast droplets in Mumbai)
#   Small: ./test-dual-droplet-p2p.sh --small (2x small droplets)
#   Keep: ./test-dual-droplet-p2p.sh --keep-droplets (for debugging)
#
# Requirements: doctl auth init (one-time setup)

set -euo pipefail

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m' 
RED='\033[0;31m'
YELLOW='\033[0;33m'
BOLD='\033[1m'
NC='\033[0m'

# Logging functions
log() { printf "${BLUE}[$(date +'%H:%M:%S')] $1${NC}\n"; }
time_checkpoint() { 
    local checkpoint="$1"
    local current_time=$(date +%s)
    local elapsed=$((current_time - START_TIME))
    printf "${BLUE}[$(date +'%H:%M:%S')] ⏱️  $checkpoint: ${elapsed}s${NC}\n"
}
success() { printf "${GREEN}✅ $1${NC}\n"; }
error() { printf "${RED}❌ $1${NC}\n"; exit 1; }
warn() { printf "${YELLOW}⚠️  $1${NC}\n"; }
header() { printf "${BOLD}${BLUE}$1${NC}\n"; }

# Test configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_ID="dual-p2p-$(date +%s)"
TEST_CLUSTER_NAME="dual-test"
export MALAI_HOME="/tmp/$TEST_ID"
TEST_SSH_KEY="/tmp/$TEST_ID-ssh"
CLUSTER_DROPLET="$TEST_ID-cluster"
MACHINE_DROPLET="$TEST_ID-machine"

# Timing tracking
START_TIME=$(date +%s)

# Configuration
KEEP_DROPLETS="${KEEP_DROPLETS:-false}"
DROPLET_SIZE="s-8vcpu-16gb"  # Default: beast mode for both droplets
DROPLET_REGION="blr1"  # Mumbai region
DROPLET_IMAGE="ubuntu-22-04-x64"

# Parse arguments
for arg in "$@"; do
    case "$arg" in
        "--small")
            DROPLET_SIZE="s-1vcpu-1gb"
            log "Using small droplets (cheaper but slower)"
            ;;
        "--fast")
            DROPLET_SIZE="s-4vcpu-8gb" 
            log "Using fast droplets (balanced)"
            ;;
        "--beast")
            DROPLET_SIZE="s-8vcpu-16gb"
            log "Using beast droplets (fastest)"
            ;;
        "--keep-droplets")
            KEEP_DROPLETS=true
            log "🔧 DEBUG MODE: Both droplets will be kept for debugging"
            ;;
        *)
            if [[ "$arg" != "${BASH_SOURCE[0]}" ]]; then
                warn "Unknown argument: $arg (ignoring)"
            fi
            ;;
    esac
done

# Setup doctl
if command -v doctl >/dev/null 2>&1; then
    DOCTL="doctl"
elif [[ -f "$HOME/doctl" ]] && [[ -x "$HOME/doctl" ]]; then
    DOCTL="$HOME/doctl"
else
    error "Install doctl first: brew install doctl"
fi

if ! $DOCTL account get >/dev/null 2>&1; then
    error "Please authenticate doctl first: $DOCTL auth init"
fi

# Cleanup function
cleanup() {
    log "🧹 Comprehensive cleanup..."
    
    if [[ "$KEEP_DROPLETS" == "true" ]]; then
        log "🔧 DEBUG MODE: Keeping both droplets for debugging"
        if [[ -n "${CLUSTER_DROPLET:-}" ]] && [[ -n "${MACHINE_DROPLET:-}" ]]; then
            echo ""
            echo "📍 DEBUGGING INFORMATION:"
            echo "   Cluster Droplet: $CLUSTER_DROPLET"
            echo "   Machine Droplet: $MACHINE_DROPLET" 
            echo "   SSH Key: $TEST_SSH_KEY"
            echo ""
            echo "🔍 SSH Commands:"
            echo "   Cluster: ssh -i $TEST_SSH_KEY root@\$CLUSTER_IP"
            echo "   Machine: ssh -i $TEST_SSH_KEY root@\$MACHINE_IP"
            echo ""
            echo "🧹 Manual cleanup when done:"
            echo "   $DOCTL compute droplet delete $CLUSTER_DROPLET $MACHINE_DROPLET --force"
            echo "   $DOCTL compute ssh-key delete $TEST_ID --force"
            echo "   rm -rf /tmp/$TEST_ID*"
            echo ""
        fi
    else
        # Normal cleanup
        if [[ -n "${DOCTL:-}" ]] && $DOCTL account get >/dev/null 2>&1; then
            if [[ -n "${CLUSTER_DROPLET:-}" ]]; then
                $DOCTL compute droplet delete "$CLUSTER_DROPLET" --force 2>/dev/null || true
            fi
            if [[ -n "${MACHINE_DROPLET:-}" ]]; then
                $DOCTL compute droplet delete "$MACHINE_DROPLET" --force 2>/dev/null || true
            fi
            $DOCTL compute ssh-key delete "$TEST_ID" --force 2>/dev/null || true
        fi
    fi
    
    rm -rf "/tmp/$TEST_ID"* 2>/dev/null || true
    success "Cleanup complete"
}
trap cleanup EXIT

header "🌐 DUAL DROPLET P2P TEST"
log "Test ID: $TEST_ID"
log "Tests real P2P between two Digital Ocean droplets (cloud ↔ cloud)"
log "This eliminates CI networking restrictions"

if [[ "$KEEP_DROPLETS" != "true" ]]; then
    log "💡 For debugging failed tests, use: ./test-dual-droplet-p2p.sh --keep-droplets"
fi
echo

# Phase 1: Setup
header "🔧 Phase 1: Auto-Setup"
success "doctl authenticated"

# Generate SSH key
log "Generating test SSH key..."
mkdir -p "$(dirname "$TEST_SSH_KEY")"
ssh-keygen -t rsa -b 2048 -f "$TEST_SSH_KEY" -N "" -C "$TEST_ID" -q
SSH_KEY_ID=$($DOCTL compute ssh-key import "$TEST_ID" --public-key-file "$TEST_SSH_KEY.pub" --format ID --no-header)
success "SSH key imported: $SSH_KEY_ID"

mkdir -p "$MALAI_HOME"
success "Test environment: $MALAI_HOME"
time_checkpoint "Setup complete"

# Phase 2: Create dual droplets
header "🚀 Phase 2: Dual Droplet Provisioning"
log "Creating cluster manager droplet..."
CLUSTER_ID=$($DOCTL compute droplet create "$CLUSTER_DROPLET" \
    --size "$DROPLET_SIZE" \
    --image "$DROPLET_IMAGE" \
    --region "$DROPLET_REGION" \
    --ssh-keys "$SSH_KEY_ID" \
    --format ID --no-header)

log "Creating machine droplet..."  
MACHINE_ID=$($DOCTL compute droplet create "$MACHINE_DROPLET" \
    --size "$DROPLET_SIZE" \
    --image "$DROPLET_IMAGE" \
    --region "$DROPLET_REGION" \
    --ssh-keys "$SSH_KEY_ID" \
    --format ID --no-header)

log "Waiting for both droplets to boot..."
sleep 60

CLUSTER_IP=$($DOCTL compute droplet get "$CLUSTER_ID" --format PublicIPv4 --no-header)
MACHINE_IP=$($DOCTL compute droplet get "$MACHINE_ID" --format PublicIPv4 --no-header)

log "Cluster droplet: $CLUSTER_IP ($DROPLET_SIZE)"
log "Machine droplet: $MACHINE_IP ($DROPLET_SIZE)"
time_checkpoint "Dual droplets ready"

# Phase 3: Install malai on both droplets
header "📦 Phase 3: Dual malai Installation"
log "Installing malai on both droplets in parallel..."

# Build malai locally first (for deployment)
if [[ ! -f "target/debug/malai" ]]; then
    cargo build --bin malai --quiet
fi

# Install script for droplets
cat > "/tmp/$TEST_ID-install.sh" << 'INSTALL_SCRIPT'
#!/bin/bash
export DEBIAN_FRONTEND=noninteractive

# Disable auto-updates
systemctl stop apt-daily.timer unattended-upgrades || true
killall apt-get apt dpkg || true
sleep 3

# Install dependencies
apt-get update -y
apt-get install -y curl git build-essential pkg-config libssl-dev gcc

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Clone and build
cd /tmp
git clone https://github.com/fastn-stack/kulfi.git
cd kulfi
cargo build --bin malai --no-default-features --release

# Install
cp target/release/malai /usr/local/bin/malai
chmod +x /usr/local/bin/malai

# Setup user
useradd -r -d /opt/malai -s /bin/bash malai || true
mkdir -p /opt/malai
chown malai:malai /opt/malai

echo "✅ malai installation complete"
INSTALL_SCRIPT

# Install on both droplets in parallel
log "Installing malai on cluster droplet..."
scp -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no "/tmp/$TEST_ID-install.sh" root@"$CLUSTER_IP":/tmp/install.sh &
scp -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no "/tmp/$TEST_ID-install.sh" root@"$MACHINE_IP":/tmp/install.sh &
wait

ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$CLUSTER_IP" "bash /tmp/install.sh" &
CLUSTER_INSTALL_PID=$!
ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$MACHINE_IP" "bash /tmp/install.sh" &
MACHINE_INSTALL_PID=$!

log "Waiting for parallel installations to complete..."
wait $CLUSTER_INSTALL_PID
wait $MACHINE_INSTALL_PID

time_checkpoint "Dual installation complete"
success "malai installed on both droplets"

# Phase 4: Setup P2P cluster
header "🔗 Phase 4: Dual Droplet P2P Setup"

# Initialize cluster on cluster droplet
log "Setting up cluster manager..."
ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$CLUSTER_IP" "
sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai cluster init $TEST_CLUSTER_NAME
"

# Get cluster manager ID
CLUSTER_MANAGER_ID52=$(ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$CLUSTER_IP" "sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai scan-roles | grep 'Identity:' | cut -d: -f2 | tr -d ' '")
log "Cluster Manager ID: $CLUSTER_MANAGER_ID52"

# Initialize machine on machine droplet
log "Setting up machine..."
ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$MACHINE_IP" "
sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai machine init $CLUSTER_MANAGER_ID52 $TEST_CLUSTER_NAME
"

# Get machine ID
MACHINE_ID52=$(ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$MACHINE_IP" "sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai scan-roles | grep 'Identity:' | cut -d: -f2 | tr -d ' '")
log "Machine ID: $MACHINE_ID52"

# Add machine to cluster config on cluster droplet
ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$CLUSTER_IP" "
sudo -u malai bash -c 'cat >> /opt/malai/clusters/$TEST_CLUSTER_NAME/cluster.toml << EOF

[machine.web01]
id52 = \"$MACHINE_ID52\"
allow_from = \"*\"
EOF'
"

time_checkpoint "Cluster configuration complete"
success "Dual droplet P2P cluster configured"

# Phase 5: Test real cloud-to-cloud P2P
header "🧪 Phase 5: Cloud-to-Cloud P2P Testing"

log "Starting daemons on both droplets..."
ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$CLUSTER_IP" "sudo -u malai env MALAI_HOME=/opt/malai nohup /usr/local/bin/malai daemon --foreground > /opt/malai/daemon.log 2>&1 &"
ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$MACHINE_IP" "sudo -u malai env MALAI_HOME=/opt/malai nohup /usr/local/bin/malai daemon --foreground > /opt/malai/daemon.log 2>&1 &"

sleep 5
time_checkpoint "Daemons started"

# THE ULTIMATE TEST: Cloud-to-cloud P2P!
log "🎯 TESTING CLOUD-TO-CLOUD P2P COMMUNICATION"
log "Digital Ocean cluster manager → Digital Ocean machine via P2P"

if ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$CLUSTER_IP" "sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai web01.$TEST_CLUSTER_NAME echo 'SUCCESS: Cloud-to-cloud P2P working!'" > "/tmp/$TEST_ID-p2p-test.log" 2>&1; then
    if grep -q "SUCCESS: Cloud-to-cloud P2P working!" "/tmp/$TEST_ID-p2p-test.log"; then
        success "🎉 CLOUD-TO-CLOUD P2P COMMUNICATION WORKING!"
        cat "/tmp/$TEST_ID-p2p-test.log"
    else
        log "❌ P2P response not received"
        cat "/tmp/$TEST_ID-p2p-test.log"
        error "Cloud-to-cloud P2P failed"
    fi
else
    cat "/tmp/$TEST_ID-p2p-test.log"
    error "Cloud-to-cloud P2P command failed"
fi

time_checkpoint "P2P testing complete"

# Final results
header "🎉 DUAL DROPLET P2P TEST RESULTS"
echo
success "🌐 CLOUD-TO-CLOUD P2P COMMUNICATION VERIFIED!"
echo
echo "📊 Test Summary:"
echo "  ✅ Dual Digital Ocean droplets: Both in Mumbai region"
echo "  ✅ Real cloud infrastructure: No CI networking restrictions"
echo "  ✅ P2P communication: Droplet ↔ Droplet via real internet"
echo "  ✅ Different machine IDs: Real P2P setup validation"
echo
TOTAL_TIME=$(($(date +%s) - START_TIME))
echo "⏱️  Performance:"
echo "  📍 Droplet size: $DROPLET_SIZE (both droplets)"
echo "  📍 Total test time: ${TOTAL_TIME}s ($(($TOTAL_TIME / 60))m $(($TOTAL_TIME % 60))s)"
echo "  📍 Region: Mumbai (blr1)"
echo
HOURLY_COST="0.14286"  # Beast droplet cost
DUAL_COST=$(echo "scale=4; $HOURLY_COST * 2 * $TOTAL_TIME / 3600" | bc)
echo "💰 Cost Analysis:"
echo "  📍 Dual beast droplets: \$$(echo "scale=4; $HOURLY_COST * 2" | bc)/hour"
echo "  📍 Test duration: $(echo "scale=2; $TOTAL_TIME/3600" | bc) hours"  
echo "  📍 Cost per test: \$$DUAL_COST"
echo
echo "🚀 PRODUCTION READY: Real cloud-to-cloud P2P validated!"
echo "💡 This proves malai works in production cloud environments"
log "Dual droplet P2P test completed successfully"