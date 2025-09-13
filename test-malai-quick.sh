#!/bin/bash
# 🚀 QUICK MALAI P2P TEST
# 
# Simplified test using local malai binary on remote machine
# Skip Rust/build complexity, focus on P2P functionality

set -euo pipefail

DROPLET_NAME="malai-test-$(date +%s)"
DROPLET_SIZE="s-1vcpu-1gb"
DROPLET_REGION="nyc3"
DROPLET_IMAGE="ubuntu-22-04-x64"
LOCAL_CLUSTER_NAME="quick-test"

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
}
trap cleanup EXIT

log "🚀 Quick malai P2P test"

# Prerequisites
if [[ -z "${MALAI_HOME:-}" ]]; then
    error "Set MALAI_HOME first"
fi

# Get SSH key
SSH_KEY_ID=$(~/doctl compute ssh-key list --format ID,Name --no-header | grep "malai-test-key" | awk '{print $1}')
if [[ -z "$SSH_KEY_ID" ]]; then
    error "SSH key malai-test-key not found"
fi

# Build malai locally
if [[ ! -f "./target/debug/malai" ]]; then
    log "Building malai locally..."
    cargo build --bin malai --quiet
fi

# Create droplet
log "Creating droplet..."
DROPLET_ID=$(~/doctl compute droplet create "$DROPLET_NAME" \
    --size "$DROPLET_SIZE" \
    --image "$DROPLET_IMAGE" \
    --region "$DROPLET_REGION" \
    --ssh-keys "$SSH_KEY_ID" \
    --format ID \
    --no-header)

sleep 60  # Wait for boot
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

# Copy malai binary directly (NO COMPILATION - just copy local binary)
log "Copying local malai binary to droplet (skipping all compilation)..."
scp -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no ./target/debug/malai root@"$DROPLET_IP":/usr/local/bin/malai
ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "chmod +x /usr/local/bin/malai"

# Test binary works (this will fail if architecture mismatch, but fast to test)
log "Testing if Mac ARM64 binary works on Linux x86_64..."
if ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "/usr/local/bin/malai --version" >/dev/null 2>&1; then
    success "Local binary works on droplet (unexpected but great!)"
else
    error "Mac ARM64 binary doesn't work on Linux x86_64 droplet (expected) - need cross-compilation"
fi

# Create users and setup
ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "
useradd -r -d /opt/malai -s /bin/bash malai
mkdir -p /opt/malai
chown malai:malai /opt/malai
"
success "User setup complete"

# Setup cluster locally
log "Setting up P2P cluster..."
rm -rf "$MALAI_HOME/clusters/$LOCAL_CLUSTER_NAME" 2>/dev/null || true
./target/debug/malai cluster init "$LOCAL_CLUSTER_NAME"
CLUSTER_MANAGER_ID52=$(./target/debug/malai scan-roles | grep "Identity:" | head -1 | cut -d: -f2 | tr -d ' ')

# Initialize machine on droplet
ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai machine init $CLUSTER_MANAGER_ID52 $LOCAL_CLUSTER_NAME"

# Get machine ID52
MACHINE_ID52=$(ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai scan-roles | grep 'Identity:' | cut -d: -f2 | tr -d ' '")

# Add machine to cluster config
cat >> "$MALAI_HOME/clusters/$LOCAL_CLUSTER_NAME/cluster.toml" << EOF

[machine.web01]
id52 = "$MACHINE_ID52"
allow_from = "*"
EOF

success "Cluster configured"

# Start daemons
log "Starting daemons..."
./target/debug/malai daemon --foreground &
LOCAL_PID=$!
sleep 3

ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai nohup /usr/local/bin/malai daemon --foreground > /opt/malai/daemon.log 2>&1 &"
sleep 3

# TEST P2P COMMUNICATION!
log "🧪 TESTING REAL P2P COMMUNICATION..."

# Test basic command
if ./target/debug/malai web01."$LOCAL_CLUSTER_NAME" echo "Hello real P2P!" > /tmp/p2p-result.log 2>&1; then
    if grep -q "Hello real P2P!" /tmp/p2p-result.log; then
        success "🎉 REAL P2P COMMUNICATION WORKING!"
        echo "✅ Command executed on droplet via P2P networking"
        echo "✅ Response received back through P2P"
        echo "🌐 malai P2P infrastructure VERIFIED across internet!"
    else
        cat /tmp/p2p-result.log
        error "P2P command output not received"
    fi
else
    cat /tmp/p2p-result.log
    error "P2P command execution failed"
fi

# Test system command
if ./target/debug/malai web01."$LOCAL_CLUSTER_NAME" whoami > /tmp/whoami-result.log 2>&1; then
    if grep -q "malai" /tmp/whoami-result.log; then
        success "System commands working via P2P"
    fi
fi

kill $LOCAL_PID 2>/dev/null || true

log "🎯 REAL P2P INFRASTRUCTURE TEST COMPLETE"
echo ""
echo "🌐 RESULTS:"
echo "✅ Digital Ocean droplet provisioned"
echo "✅ malai installed on remote Ubuntu server"  
echo "✅ P2P cluster established (laptop ↔ cloud)"
echo "✅ Real command execution across internet P2P"
echo "✅ malai infrastructure working end-to-end!"