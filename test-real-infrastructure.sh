#!/bin/bash
# 🌐 REAL INFRASTRUCTURE TESTING
#
# Automated end-to-end testing with real machines:
# - Local laptop (cluster manager)  
# - Digital Ocean droplet (remote machine)
# - Real P2P communication across internet
#
# Prerequisites:
# - doctl installed and authenticated: doctl auth init
# - SSH key added to DO account
# - MALAI_HOME set for local testing

set -euo pipefail

# Configuration
DROPLET_NAME="malai-test-$(date +%s)"
DROPLET_SIZE="s-2vcpu-2gb"  # Reliable for Rust builds
DROPLET_REGION="nyc3"       # Close to US East Coast
DROPLET_IMAGE="ubuntu-22-04-x64"
LOCAL_CLUSTER_NAME="test-real-infra"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m' 
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date +'%H:%M:%S')] $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; exit 1; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }

# Cleanup function
cleanup() {
    log "🧹 Cleaning up test infrastructure..."
    
    # Destroy droplet if it exists
    if ~/doctl compute droplet list --format Name | grep -q "$DROPLET_NAME"; then
        log "Destroying droplet: $DROPLET_NAME"
        ~/doctl compute droplet delete "$DROPLET_NAME" --force
        success "Droplet destroyed"
    fi
    
    # Clean up local test environment
    if [[ -d "$MALAI_HOME/clusters/$LOCAL_CLUSTER_NAME" ]]; then
        log "Cleaning up local cluster: $LOCAL_CLUSTER_NAME"
        rm -rf "$MALAI_HOME/clusters/$LOCAL_CLUSTER_NAME"
        success "Local cluster cleaned up"
    fi
}

trap cleanup EXIT

log "🌐 Starting malai real infrastructure test"
log "📁 Test cluster: $LOCAL_CLUSTER_NAME"
log "🖥️  Remote droplet: $DROPLET_NAME"

# Prerequisites check
log "🔍 Checking prerequisites..."

# Check doctl
if ! ~/doctl account get >/dev/null 2>&1; then
    error "doctl not authenticated. Run: doctl auth init"
fi
success "Digital Ocean CLI authenticated"

# Check SSH key exists (prefer test key)
if [[ -f ~/.ssh/malai-test-key ]]; then
    success "SSH test key found at ~/.ssh/malai-test-key"
elif [[ -f ~/.ssh/ssh-key ]]; then
    success "SSH key found at ~/.ssh/ssh-key"
else
    error "No SSH key found. Generate one: ssh-keygen -t rsa -f ~/.ssh/malai-test-key"
fi

# Check MALAI_HOME
if [[ -z "${MALAI_HOME:-}" ]]; then
    error "MALAI_HOME not set. Set it to your test directory."
fi
success "MALAI_HOME: $MALAI_HOME"

# Check malai binary
if [[ ! -f "./target/debug/malai" ]]; then
    log "Building malai binary..."
    cargo build --bin malai --quiet
fi
success "malai binary available"

# Get SSH key ID (prefer "malai-test-key" for testing)
if ~/doctl compute ssh-key list --format Name --no-header | grep -q "malai-test-key"; then
    SSH_KEY_ID=$(~/doctl compute ssh-key list --format ID,Name --no-header | grep "malai-test-key" | awk '{print $1}')
    SSH_KEY_NAME="malai-test-key"
    SSH_KEY_FILE="~/.ssh/malai-test-key"
else
    SSH_KEY_ID=$(~/doctl compute ssh-key list --format ID --no-header | head -1)
    SSH_KEY_NAME=$(~/doctl compute ssh-key list --format Name --no-header | head -1)
    SSH_KEY_FILE="~/.ssh/ssh-key"
fi

if [[ -z "$SSH_KEY_ID" ]]; then
    error "No SSH keys found in Digital Ocean account. Add one first: doctl compute ssh-key import"
fi
log "Using SSH key: $SSH_KEY_NAME (ID: $SSH_KEY_ID, file: $SSH_KEY_FILE)"

# Phase 1: Create and configure droplet
log "🚀 Phase 1: Creating Digital Ocean droplet"

# Create droplet
log "Creating droplet: $DROPLET_NAME"
DROPLET_ID=$(~/doctl compute droplet create "$DROPLET_NAME" \
    --size "$DROPLET_SIZE" \
    --image "$DROPLET_IMAGE" \
    --region "$DROPLET_REGION" \
    --ssh-keys "$SSH_KEY_ID" \
    --format ID \
    --no-header)

if [[ -z "$DROPLET_ID" ]]; then
    error "Failed to create droplet"
fi

log "Droplet created with ID: $DROPLET_ID"

# Wait for droplet to be ready
log "Waiting for droplet to boot..."
sleep 60  # Give DO droplets more time to fully boot

# Get droplet IP
DROPLET_IP=$(~/doctl compute droplet get "$DROPLET_ID" --format PublicIPv4 --no-header)
if [[ -z "$DROPLET_IP" ]]; then
    error "Failed to get droplet IP"
fi

log "Droplet ready at IP: $DROPLET_IP"
success "Droplet provisioned successfully"

# Wait for SSH to be ready
log "Waiting for SSH to be ready..."
for i in {1..60}; do  # Increased attempts for better reliability
    log "SSH attempt $i/60..."
    if ssh -i ~/.ssh/malai-test-key -o ConnectTimeout=10 -o StrictHostKeyChecking=no root@"$DROPLET_IP" echo "SSH ready" >/dev/null 2>&1; then
        log "SSH connection established!"
        break
    fi
    sleep 10
done

# Verify SSH works
if ! ssh -i ~/.ssh/malai-test-key -o ConnectTimeout=5 -o StrictHostKeyChecking=no root@"$DROPLET_IP" echo "SSH test" >/dev/null 2>&1; then
    error "SSH connection failed to $DROPLET_IP"
fi
success "SSH connection to droplet working"

# Phase 2: Install malai on remote machine
log "📦 Phase 2: Installing malai on remote machine"

# Create simpler installation script with better error handling
cat > /tmp/install-malai-remote.sh << 'REMOTE_SCRIPT'
#!/bin/bash
set -euo pipefail

echo "🔨 Installing malai on remote machine..."

# Install dependencies first
echo "📦 Installing system dependencies..."
export DEBIAN_FRONTEND=noninteractive

# Wait for automatic apt processes to complete (Ubuntu does this on first boot)
echo "⏳ Waiting for automatic apt processes to complete..."
while pgrep -x apt-get > /dev/null || pgrep -x apt > /dev/null || pgrep -x dpkg > /dev/null; do
    echo "   Waiting for apt lock to be released..."
    sleep 5
done
echo "✅ apt lock available"

apt-get update -y
apt-get install -y curl git build-essential pkg-config libssl-dev

# Install Rust (required for building malai)
echo "📦 Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
source ~/.cargo/env

# Verify Rust installation
echo "✅ Rust version: $(rustc --version)"
echo "✅ Cargo version: $(cargo --version)"

# Clone kulfi repository to tmp first
echo "📂 Cloning kulfi repository..."
cd /tmp
rm -rf kulfi 2>/dev/null || true
git clone https://github.com/fastn-stack/kulfi.git kulfi
cd kulfi

# Build malai optimized for server (exclude UI dependencies)
echo "🔨 Building malai server binary (optimized, faster)..."
~/.cargo/bin/cargo build --bin malai --no-default-features --release

# Verify binary was created
if [[ ! -f target/release/malai ]]; then
    echo "❌ malai binary not created"
    exit 1
fi

echo "✅ malai binary built successfully"

# Create malai user and directory
echo "👤 Setting up malai user..."
useradd -r -d /opt/malai -s /bin/bash malai || echo "User may already exist"
mkdir -p /opt/malai
chown malai:malai /opt/malai

# Copy binary
echo "📋 Installing malai binary..."
cp target/release/malai /usr/local/bin/malai
chmod +x /usr/local/bin/malai

# Test binary works
echo "🧪 Testing malai binary..."
/usr/local/bin/malai --version

echo "✅ malai installation complete!"
echo "📍 Binary location: /usr/local/bin/malai"
echo "📁 Data directory: /opt/malai"
echo "👤 User: malai"
REMOTE_SCRIPT

# Copy and execute installation script
log "Copying installation script to droplet..."
scp -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no /tmp/install-malai-remote.sh root@"$DROPLET_IP":/tmp/
success "Installation script copied"

log "Executing malai installation on droplet..."
if ! ssh -i ~/.ssh/malai-test-key -o StrictHostKeyChecking=no root@"$DROPLET_IP" "bash /tmp/install-malai-remote.sh" 2>&1 | tee /tmp/remote-install.log; then
    echo ""
    log "❌ malai installation failed on droplet"
    log "📋 Installation output:"
    cat /tmp/remote-install.log || echo "No installation log available"
    log "🔍 Droplet IP: $DROPLET_IP (keeping alive for debugging)"
    log "🔌 SSH command: ssh -i ~/.ssh/malai-test-key root@$DROPLET_IP"
    log "💡 Manual cleanup: ~/doctl compute droplet delete $DROPLET_NAME --force"
    exit 1
fi
success "malai installed successfully on droplet"

# Verify malai works on remote
if ! ssh -o StrictHostKeyChecking=no root@"$DROPLET_IP" "/usr/local/bin/malai --version" >/dev/null 2>&1; then
    error "malai binary not working on droplet"
fi
success "malai binary verified working on droplet"

# Phase 3: Set up real P2P cluster
log "🔗 Phase 3: Setting up real P2P infrastructure"

# Create cluster locally (laptop as cluster manager)
log "Creating cluster on laptop (cluster manager)..."
if [[ -d "$MALAI_HOME/clusters/$LOCAL_CLUSTER_NAME" ]]; then
    rm -rf "$MALAI_HOME/clusters/$LOCAL_CLUSTER_NAME"
fi

./target/debug/malai cluster init "$LOCAL_CLUSTER_NAME"
CLUSTER_MANAGER_ID52=$(./target/debug/malai scan-roles | grep "Identity:" | head -1 | cut -d: -f2 | tr -d ' ')

if [[ -z "$CLUSTER_MANAGER_ID52" ]]; then
    error "Failed to get cluster manager ID52"
fi

log "Cluster manager ID52: $CLUSTER_MANAGER_ID52"
success "Local cluster created"

# Initialize machine on droplet
log "Initializing machine on droplet..."
if ! ssh -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai machine init $CLUSTER_MANAGER_ID52 $LOCAL_CLUSTER_NAME"; then
    error "Machine initialization failed on droplet"
fi

# Get machine ID52 from droplet
MACHINE_ID52=$(ssh -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai scan-roles | grep 'Identity:' | cut -d: -f2 | tr -d ' '")

if [[ -z "$MACHINE_ID52" ]]; then
    error "Failed to get machine ID52 from droplet"
fi

log "Machine ID52: $MACHINE_ID52"
success "Machine initialized on droplet"

# Add machine to cluster config locally
log "Adding machine to cluster configuration..."
cat >> "$MALAI_HOME/clusters/$LOCAL_CLUSTER_NAME/cluster.toml" << EOF

[machine.web01]
id52 = "$MACHINE_ID52"
allow_from = "*"
EOF
success "Machine added to cluster configuration"

# Phase 4: Start daemons and test P2P communication
log "🔥 Phase 4: Testing real P2P communication"

# Start daemon locally
log "Starting daemon on laptop..."
./target/debug/malai daemon --foreground &
LOCAL_DAEMON_PID=$!
sleep 5

# Verify local daemon started
if ! kill -0 "$LOCAL_DAEMON_PID" 2>/dev/null; then
    error "Local daemon failed to start"
fi
success "Local daemon running"

# Start daemon on droplet
log "Starting daemon on droplet..."
ssh -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai nohup /usr/local/bin/malai daemon --foreground > /opt/malai/daemon.log 2>&1 &"
sleep 5

# Verify remote daemon started
if ! ssh -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai status | grep -q 'RUNNING'"; then
    error "Remote daemon failed to start"
fi
success "Remote daemon running"

# Phase 5: Real P2P command execution tests
log "🧪 Phase 5: Testing real P2P command execution"

# Test basic command execution
log "Testing basic command execution..."
if ! timeout 30s ./target/debug/malai web01."$LOCAL_CLUSTER_NAME" echo "Hello from real P2P!" > /tmp/p2p-test.log 2>&1; then
    cat /tmp/p2p-test.log
    error "Basic P2P command execution failed"
fi

if ! grep -q "Hello from real P2P!" /tmp/p2p-test.log; then
    cat /tmp/p2p-test.log
    error "P2P command output not received"
fi
success "Basic P2P command execution working"

# Test system commands
log "Testing system command execution..."
if ! timeout 30s ./target/debug/malai web01."$LOCAL_CLUSTER_NAME" whoami > /tmp/whoami-test.log 2>&1; then
    cat /tmp/whoami-test.log
    error "System command execution failed"
fi

if ! grep -q "malai" /tmp/whoami-test.log; then
    cat /tmp/whoami-test.log
    error "Unexpected whoami output"
fi
success "System command execution working"

# Test daemon status on both machines
log "Testing status commands..."
./target/debug/malai status > /tmp/local-status.log
ssh -o StrictHostKeyChecking=no root@"$DROPLET_IP" "sudo -u malai env MALAI_HOME=/opt/malai /usr/local/bin/malai status" > /tmp/remote-status.log

if ! grep -q "RUNNING" /tmp/local-status.log; then
    cat /tmp/local-status.log
    error "Local daemon status check failed"
fi

if ! grep -q "RUNNING" /tmp/remote-status.log; then
    cat /tmp/remote-status.log  
    error "Remote daemon status check failed"
fi
success "Status commands working on both machines"

# Phase 6: Test configuration management
log "🔄 Phase 6: Testing configuration management"

# Test selective rescan
log "Testing selective rescan..."
if ! ./target/debug/malai rescan "$LOCAL_CLUSTER_NAME" > /tmp/rescan-test.log 2>&1; then
    cat /tmp/rescan-test.log
    error "Selective rescan failed"
fi

if ! grep -q "Daemon rescan request completed" /tmp/rescan-test.log; then
    cat /tmp/rescan-test.log
    error "Rescan didn't complete successfully"
fi
success "Selective rescan working"

# Cleanup daemon
kill "$LOCAL_DAEMON_PID" 2>/dev/null || true
wait "$LOCAL_DAEMON_PID" 2>/dev/null || true

# Final results
log "🎉 Real infrastructure test complete!"
echo ""
echo "📊 Test Results:"
echo "✅ Digital Ocean droplet provisioned and configured"
echo "✅ malai installed and running on remote machine" 
echo "✅ Real P2P cluster communication working"
echo "✅ Remote command execution via P2P"
echo "✅ Configuration management working"
echo "✅ Status monitoring on both machines"
echo ""
echo "🚀 malai real-world P2P infrastructure VERIFIED!"
echo ""
log "Droplet will be destroyed in cleanup..."
