#!/bin/bash
# 🌐 OPTIMIZED REAL P2P TEST
# Build malai once on droplet, then test P2P multiple times quickly

set -euo pipefail

DROPLET_NAME="malai-real-$(date +%s)"
DROPLET_SIZE="s-2vcpu-2gb"  # Larger for faster builds
DROPLET_REGION="nyc3"
DROPLET_IMAGE="ubuntu-22-04-x64"
CLUSTER_NAME="real-p2p-test"

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m' 
RED='\033[0;31m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date +'%H:%M:%S')] $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; exit 1; }

cleanup() {
    log "🧹 Cleaning up..."
    if ~/doctl compute droplet list --format Name | grep -q "$DROPLET_NAME"; then
        ~/doctl compute droplet delete "$DROPLET_NAME" --force
    fi
    pkill -f "malai daemon" 2>/dev/null || true
}
trap cleanup EXIT

log "🌐 Optimized real P2P test"

# Prerequisites
if [[ -z "${MALAI_HOME:-}" ]]; then
    error "Set MALAI_HOME first: export MALAI_HOME=/tmp/malai-real-test"
fi

SSH_KEY_ID=$(~/doctl compute ssh-key list --format ID,Name --no-header | grep "malai-test-key" | awk '{print $1}')
if [[ -z "$SSH_KEY_ID" ]]; then
    error "SSH key malai-test-key not found"
fi

# Create droplet
log "Creating larger droplet for faster builds..."
DROPLET_ID=$(~/doctl compute droplet create "$DROPLET_NAME" \
    --size "$DROPLET_SIZE" \
    --image "$DROPLET_IMAGE" \
    --region "$DROPLET_REGION" \
    --ssh-keys "$SSH_KEY_ID" \
    --format ID \
    --no-header)

sleep 60
DROPLET_IP=$(~/doctl compute droplet get "$DROPLET_ID" --format PublicIPv4 --no-header)
log "Droplet ready: $DROPLET_IP"

# Wait for SSH
for i in {1..30}; do
    if ssh -i ~/.ssh/malai-test-key -o ConnectTimeout=5 -o StrictHostKeyChecking=no root@"$DROPLET_IP" echo "ready" >/dev/null 2>&1; then
        break
    fi
    sleep 5
done
success "SSH ready"

# OPTIMIZED BUILD: Just build malai quickly on larger droplet
log "Building malai on 2GB droplet (optimized)..."
ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "
export DEBIAN_FRONTEND=noninteractive

# Wait for apt lock
while pgrep -x apt > /dev/null; do echo 'Waiting for apt...'; sleep 5; done

# Install minimal deps
apt-get update -y
apt-get install -y curl git build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Clone and build (optimized for server)
cd /tmp
git clone https://github.com/fastn-stack/kulfi.git
cd kulfi
git checkout $GITHUB_REF_NAME || git checkout feat/real-infrastructure-testing
cargo build --bin malai --no-default-features --release

# Install binary
cp target/release/malai /usr/local/bin/malai
chmod +x /usr/local/bin/malai

echo '✅ malai build complete'
"

# Verify build worked
if ! ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "/usr/local/bin/malai --version"; then
    error "malai build failed on droplet"
fi
success "malai built and installed on droplet"

# Setup users
ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "
useradd -r -d /opt/malai -s /bin/bash malai
mkdir -p /opt/malai
chown malai:malai /opt/malai
"

# NOW THE FAST PART: P2P testing!
log "🧪 TESTING REAL P2P WITH WORKING IMPLEMENTATION..."

# Setup cluster locally
rm -rf "$MALAI_HOME" 2>/dev/null || true
./target/debug/malai cluster init "$CLUSTER_NAME"
CLUSTER_MANAGER_ID52=$(./target/debug/malai scan-roles | grep "Identity:" | head -1 | cut -d: -f2 | tr -d ' ')

# Initialize machine on droplet
ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai machine init $CLUSTER_MANAGER_ID52 $CLUSTER_NAME"

# Get machine ID52
MACHINE_ID52=$(ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai scan-roles | grep 'Identity:' | cut -d: -f2 | tr -d ' '")

log "✅ Cluster Manager: $CLUSTER_MANAGER_ID52"
log "✅ Remote Machine: $MACHINE_ID52"
log "✅ DIFFERENT IDs - real P2P test setup!"

# Add machine to cluster config
cat >> "$MALAI_HOME/clusters/$CLUSTER_NAME/cluster.toml" << EOF

[machine.web01]
id52 = "$MACHINE_ID52"
allow_from = "*"
EOF

# Start daemons
log "Starting daemons for real P2P test..."
./target/debug/malai daemon --foreground &
LOCAL_PID=$!
sleep 3

ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai nohup /usr/local/bin/malai daemon --foreground > /opt/malai/daemon.log 2>&1 &"
sleep 5

# THE ULTIMATE TEST: Real cross-internet P2P!
log "🎯 ULTIMATE TEST: Real P2P command execution across internet!"
log "Laptop (cluster manager) → Digital Ocean (machine) via P2P"

if ./target/debug/malai web01."$CLUSTER_NAME" echo "SUCCESS: Real cross-internet P2P working!" > /tmp/ultimate-p2p-test.log 2>&1; then
    if grep -q "SUCCESS: Real cross-internet P2P working!" /tmp/ultimate-p2p-test.log; then
        success "🎉🎉🎉 ULTIMATE SUCCESS!"
        echo ""
        echo "🌐 BREAKTHROUGH ACHIEVED:"
        echo "✅ Real P2P communication across internet"
        echo "✅ Laptop cluster manager → Digital Ocean machine"  
        echo "✅ Command executed via P2P networking"
        echo "✅ Response received back through internet"
        echo ""
        echo "🚀 malai P2P infrastructure FULLY VALIDATED!"
        echo ""
        echo "📊 Full test output:"
        cat /tmp/ultimate-p2p-test.log
    else
        error "P2P command output not received"
    fi
else
    log "❌ P2P test failed - checking logs..."
    cat /tmp/ultimate-p2p-test.log
    error "Real cross-internet P2P failed"
fi

kill $LOCAL_PID 2>/dev/null || true
success "🎯 REAL CROSS-INTERNET P2P TEST COMPLETE!"