#!/bin/bash
# 🌐 QUICK DIGITAL OCEAN P2P TEST
# Test our working P2P implementation across real internet infrastructure

set -euo pipefail

# Configuration
DROPLET_NAME="malai-quick-$(date +%s)"
DROPLET_SIZE="s-1vcpu-1gb"
DROPLET_REGION="nyc3"
DROPLET_IMAGE="ubuntu-22-04-x64"
CLUSTER_NAME="quick-p2p-test"

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m' 
RED='\033[0;31m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date +'%H:%M:%S')] $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; exit 1; }

# Cleanup function
cleanup() {
    log "🧹 Cleaning up..."
    if ~/doctl compute droplet list --format Name | grep -q "$DROPLET_NAME"; then
        ~/doctl compute droplet delete "$DROPLET_NAME" --force
    fi
    pkill -f "malai daemon" 2>/dev/null || true
}
trap cleanup EXIT

log "🌐 Quick malai P2P test across internet"

# Prerequisites check
if [[ -z "${MALAI_HOME:-}" ]]; then
    error "Set MALAI_HOME first: export MALAI_HOME=/tmp/malai-do-test"
fi

if ! ~/doctl account get >/dev/null 2>&1; then
    error "doctl not authenticated. Run: doctl auth init"
fi

# Get SSH key ID
SSH_KEY_ID=$(~/doctl compute ssh-key list --format ID,Name --no-header | grep "malai-test-key" | awk '{print $1}')
if [[ -z "$SSH_KEY_ID" ]]; then
    SSH_KEY_ID=$(~/doctl compute ssh-key list --format ID --no-header | head -1)
fi

if [[ -z "$SSH_KEY_ID" ]]; then
    error "No SSH keys found in Digital Ocean account"
fi

# Create droplet
log "Creating droplet: $DROPLET_NAME"
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

# Copy malai binary directly (skip compilation)
log "Copying malai binary to droplet..."
scp -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no ./target/debug/malai root@"$DROPLET_IP":/usr/local/bin/malai
ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "chmod +x /usr/local/bin/malai"

# Test binary works
if ! ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "/usr/local/bin/malai --version" >/dev/null 2>&1; then
    error "malai binary not working on droplet"
fi
success "malai binary working on droplet"

# Setup cluster locally (laptop as cluster manager)
log "Setting up P2P cluster..."
rm -rf "$MALAI_HOME" 2>/dev/null || true
mkdir -p "$MALAI_HOME"
./target/debug/malai cluster init "$CLUSTER_NAME"

# Get cluster manager ID52
CLUSTER_MANAGER_ID52=$(./target/debug/malai scan-roles | grep "Identity:" | head -1 | cut -d: -f2 | tr -d ' ')
log "Cluster manager ID52: $CLUSTER_MANAGER_ID52"

# Initialize machine on droplet
log "Initializing machine on droplet..."
ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "
useradd -r -d /opt/malai -s /bin/bash malai || true
mkdir -p /opt/malai
chown malai:malai /opt/malai
sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai machine init $CLUSTER_MANAGER_ID52 $CLUSTER_NAME
"

# Get machine ID52 from droplet
MACHINE_ID52=$(ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai scan-roles | grep 'Identity:' | cut -d: -f2 | tr -d ' '")
log "Machine ID52: $MACHINE_ID52"

# Add machine to cluster config locally
cat >> "$MALAI_HOME/clusters/$CLUSTER_NAME/cluster.toml" << EOF

[machine.web01]
id52 = "$MACHINE_ID52"
allow_from = "*"
EOF

success "Cluster configured with real different machine IDs"

# Start daemons
log "Starting daemons..."
./target/debug/malai daemon --foreground &
LOCAL_PID=$!
sleep 3

ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai nohup /usr/local/bin/malai daemon --foreground > /opt/malai/daemon.log 2>&1 &"
sleep 5

# THE CRITICAL TEST: Real P2P communication across internet!
log "🧪 TESTING REAL CROSS-INTERNET P2P COMMUNICATION..."
log "This is the test that was failing before - different machine IDs, real network"

if ./target/debug/malai web01."$CLUSTER_NAME" echo "Hello real cross-internet P2P!" > /tmp/do-p2p-result.log 2>&1; then
    if grep -q "Hello real cross-internet P2P!" /tmp/do-p2p-result.log; then
        success "🎉 REAL CROSS-INTERNET P2P COMMUNICATION WORKING!"
        echo "✅ Command executed on Digital Ocean droplet via real P2P"
        echo "✅ Response received back through internet P2P connection"
        echo "🌐 malai P2P infrastructure VERIFIED across real internet!"
        cat /tmp/do-p2p-result.log
    else
        log "❌ P2P command output not received correctly"
        cat /tmp/do-p2p-result.log
        error "P2P communication failed"
    fi
else
    log "❌ P2P command execution failed" 
    cat /tmp/do-p2p-result.log
    error "Real cross-internet P2P failed"
fi

kill $LOCAL_PID 2>/dev/null || true

success "🎯 REAL CROSS-INTERNET P2P TEST COMPLETE!"
echo ""
echo "🌐 FINAL RESULTS:"
echo "✅ Digital Ocean droplet provisioned"
echo "✅ malai installed on remote Ubuntu server"  
echo "✅ Real cluster with different machine IDs"
echo "✅ P2P daemons running on laptop and cloud"
echo "✅ REAL COMMAND EXECUTION ACROSS INTERNET P2P!"
echo ""
echo "🚀 malai P2P infrastructure VERIFIED end-to-end across internet!"