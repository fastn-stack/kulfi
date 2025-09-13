#!/bin/bash
# 🎯 MANUAL MALAI SETUP
# Creates droplet and provides SSH access for manual malai testing

set -euo pipefail

DROPLET_NAME="malai-manual-$(date +%s)"
DROPLET_SIZE="s-1vcpu-1gb"
DROPLET_REGION="nyc3" 
DROPLET_IMAGE="ubuntu-22-04-x64"

BLUE='\033[0;34m'
GREEN='\033[0;32m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date +'%H:%M:%S')] $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }

log "🎯 Creating droplet for manual malai testing"

# Get SSH key
SSH_KEY_ID=$(~/doctl compute ssh-key list --format ID,Name --no-header | grep "malai-test-key" | awk '{print $1}')

# Create droplet
log "Creating droplet: $DROPLET_NAME"
DROPLET_ID=$(~/doctl compute droplet create "$DROPLET_NAME" \
    --size "$DROPLET_SIZE" \
    --image "$DROPLET_IMAGE" \
    --region "$DROPLET_REGION" \
    --ssh-keys "$SSH_KEY_ID" \
    --format ID \
    --no-header)

sleep 60
DROPLET_IP=$(~/doctl compute droplet get "$DROPLET_ID" --format PublicIPv4 --no-header)

# Wait for SSH
for i in {1..20}; do
    if ssh -i ~/.ssh/malai-test-key -o ConnectTimeout=5 -o StrictHostKeyChecking=no root@"$DROPLET_IP" echo "ready" >/dev/null 2>&1; then
        break
    fi
    sleep 5
done

success "Droplet ready for manual testing"
echo ""
echo "🔌 SSH Command:"
echo "ssh -i ~/.ssh/malai-test-key root@$DROPLET_IP"
echo ""
echo "📋 Manual Setup Steps:"
echo "1. SSH to droplet: ssh -i ~/.ssh/malai-test-key root@$DROPLET_IP"
echo "2. Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
echo "3. Install deps: apt-get update && apt-get install -y git build-essential pkg-config libssl-dev"
echo "4. Clone repo: git clone https://github.com/fastn-stack/kulfi.git && cd kulfi"
echo "5. Build malai: source ~/.cargo/env && cargo build --bin malai"
echo "6. Install: cp target/debug/malai /usr/local/bin/ && chmod +x /usr/local/bin/malai"
echo "7. Setup user: useradd -r -d /opt/malai malai && mkdir -p /opt/malai && chown malai:malai /opt/malai"
echo "8. Initialize: sudo -u malai env MALAI_HOME=/opt/malai malai machine init <cluster-id52> test"
echo ""
echo "💡 Cleanup when done: ~/doctl compute droplet delete $DROPLET_NAME --force"
echo ""
echo "🎯 Droplet Info:"
echo "   ID: $DROPLET_ID"
echo "   IP: $DROPLET_IP"
echo "   Name: $DROPLET_NAME"