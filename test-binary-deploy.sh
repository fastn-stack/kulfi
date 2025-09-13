#!/bin/bash
# 🚀 QUICK BINARY DEPLOYMENT TEST
# Test if we can deploy local binary to droplet quickly

set -euo pipefail

DROPLET_NAME="malai-binary-test-$(date +%s)"
DROPLET_SIZE="s-1vcpu-1gb"
DROPLET_REGION="nyc3"
DROPLET_IMAGE="ubuntu-22-04-x64"

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

log "🚀 Testing binary deployment to Digital Ocean"

# Get SSH key
SSH_KEY_ID=$(~/doctl compute ssh-key list --format ID,Name --no-header | grep "malai-test-key" | awk '{print $1}')
if [[ -z "$SSH_KEY_ID" ]]; then
    error "SSH key malai-test-key not found"
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

sleep 60
DROPLET_IP=$(~/doctl compute droplet get "$DROPLET_ID" --format PublicIPv4 --no-header)
log "Droplet ready: $DROPLET_IP"

# Wait for SSH
for i in {1..20}; do
    if ssh -i ~/.ssh/malai-test-key -o ConnectTimeout=5 -o StrictHostKeyChecking=no root@"$DROPLET_IP" echo "ready" >/dev/null 2>&1; then
        break
    fi
    sleep 5
done

success "SSH ready"

# Test 1: Copy Mac binary and see what happens (should fail gracefully)
log "Testing Mac ARM64 binary on Linux x86_64..."
scp -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no ./target/debug/malai root@"$DROPLET_IP":/tmp/malai-mac
ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "chmod +x /tmp/malai-mac"

if ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "file /tmp/malai-mac" 2>&1; then
    log "Binary file type check completed"
fi

if ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "/tmp/malai-mac --version" 2>&1; then
    success "🎉 UNEXPECTED: Mac binary works on Linux! No cross-compilation needed!"
else
    log "Expected: Mac ARM64 binary doesn't work on Linux x86_64"
    log "Next step: Set up cross-compilation or build on droplet"
fi

success "Binary deployment test complete"
log "Droplet IP: $DROPLET_IP (will be cleaned up automatically)"