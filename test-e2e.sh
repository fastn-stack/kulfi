#!/bin/bash
# 🎯 MALAI CRITICAL INFRASTRUCTURE TESTS
#
# This script runs the most important test in malai - complete P2P infrastructure.
# If this test passes, the entire malai system is operational.
#
# Usage:
#   ./test-e2e.sh            # Run bash test (default, fastest)
#   ./test-e2e.sh --rust     # Run Rust integration test (future)
#   ./test-e2e.sh --both     # Run both tests (future)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m' 
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
BOLD='\033[1m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date +'%H:%M:%S')] $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; exit 1; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
header() { echo -e "${BOLD}${BLUE}$1${NC}"; }

# Parse command line arguments
RUN_RUST=false
RUN_BASH=true

case "${1:-}" in
    --rust)
        RUN_BASH=false
        RUN_RUST=true
        log "Running only Rust test (not yet implemented)"
        ;;
    --both)
        RUN_RUST=true
        RUN_BASH=true
        log "Running both malai tests"
        ;;
    --help)
        echo "malai Critical Infrastructure Tests"
        echo "Usage: $0 [--rust|--both|--help]"
        echo "  (default)  Run bash test only"
        echo "  --rust     Run Rust integration test (future)"
        echo "  --both     Run both tests (future)"
        exit 0
        ;;
    "")
        log "Running bash test (use --rust for Rust test when available)"
        ;;
    *)
        error "Unknown argument: $1 (use --help for usage)"
        ;;
esac

# Test configuration
TEST_DIR="/tmp/malai-e2e-$$"
CLUSTER_NAME="company"
MALAI_BIN="./target/debug/malai"

cleanup() {
    log "Cleaning up test environment..."
    pkill -f "malai daemon" 2>/dev/null || true
    rm -rf "$TEST_DIR" 2>/dev/null || true
}

trap cleanup EXIT

log "🎯 Starting malai end-to-end test"
log "📁 Test directory: $TEST_DIR"

# Setup test environment
mkdir -p "$TEST_DIR"/{cluster-manager,machine1}

header "🔨 Building malai binary"
log "Building malai for infrastructure testing..."
if ! /Users/amitu/.cargo/bin/cargo build --bin malai --quiet; then
    error "Failed to build malai binary"
fi
success "malai binary built"

# Track test results
BASH_RESULT=""
TESTS_RUN=0
TESTS_PASSED=0

# Helper functions for test
assert_contains() {
    if ! grep -q "$2" "$1" 2>/dev/null; then
        error "Assertion failed: '$1' does not contain '$2'"
    fi
}
assert_file_exists() {
    if [[ ! -f "$1" ]]; then
        error "Assertion failed: File '$1' does not exist"
    fi
}

# Function to run bash infrastructure test
run_bash_test() {
    header "🏗️  CRITICAL TEST: Bash P2P Infrastructure" 
    log "Test: Complete malai infrastructure with real P2P"
    log "Mode: Cluster manager → machine config → remote execution"
    echo
    
    # Phase 1: Cluster Manager Setup
    log "👑 Phase 1: Setting up cluster manager"
    export MALAI_HOME="$TEST_DIR/cluster-manager"

# Initialize cluster
log "Creating cluster..."
if ! $MALAI_BIN cluster init $CLUSTER_NAME > "$TEST_DIR/cluster-init.log" 2>&1; then
    cat "$TEST_DIR/cluster-init.log"
    error "Cluster initialization failed"
fi

# Extract cluster manager ID52 from output
CLUSTER_MANAGER_ID52=$(grep "Cluster created with ID:" "$TEST_DIR/cluster-init.log" | cut -d: -f2 | tr -d ' ')
if [[ -z "$CLUSTER_MANAGER_ID52" ]]; then
    error "Could not extract cluster manager ID52"
fi

log "✅ Cluster manager ID52: $CLUSTER_MANAGER_ID52"

# Verify cluster config was created
CLUSTER_CONFIG="$MALAI_HOME/clusters/$CLUSTER_NAME/cluster-config.toml"
assert_file_exists "$CLUSTER_CONFIG"
assert_contains "$CLUSTER_CONFIG" "\[cluster_manager\]"
assert_contains "$CLUSTER_CONFIG" "$CLUSTER_MANAGER_ID52"
success "Cluster configuration created correctly"

# Phase 2: Machine Setup  
log "🖥️  Phase 2: Setting up machine"
export MALAI_HOME="$TEST_DIR/machine1"

# Generate machine identity
log "Generating machine identity..."
if ! $MALAI_BIN keygen --file "$TEST_DIR/machine-identity.key" > "$TEST_DIR/machine-keygen.log" 2>&1; then
    error "Machine keygen failed"
fi

# Extract machine ID52 
MACHINE_ID52=$(grep "Generated Public Key (ID52):" "$TEST_DIR/machine-keygen.log" | cut -d: -f2 | tr -d ' ')
if [[ -z "$MACHINE_ID52" ]]; then
    error "Could not extract machine ID52"
fi

log "✅ Machine ID52: $MACHINE_ID52"

# Create machine cluster directory and copy identity
mkdir -p "$MALAI_HOME/clusters/$CLUSTER_NAME"
cp "$TEST_DIR/machine-identity.key" "$MALAI_HOME/clusters/$CLUSTER_NAME/identity.key"

# Create cluster-info.toml for machine
cat > "$MALAI_HOME/clusters/$CLUSTER_NAME/cluster-info.toml" << EOF
cluster_alias = "$CLUSTER_NAME"
cluster_id52 = "$CLUSTER_MANAGER_ID52"
machine_id52 = "$MACHINE_ID52"
EOF

success "Machine registration created"

# Phase 3: Add Machine to Cluster Config
log "📝 Phase 3: Adding machine to cluster config"
export MALAI_HOME="$TEST_DIR/cluster-manager"

# Add machine to cluster config
cat >> "$CLUSTER_CONFIG" << EOF

[machine.web01]
id52 = "$MACHINE_ID52"
allow_from = "*"

[machine.restricted] 
id52 = "fake-restricted-machine-id52"
allow_from = "admins-only"
EOF

log "Machine added to cluster config"

# Phase 4: Start Daemons and Test Config Distribution
log "🚀 Phase 4: Starting daemons and testing config distribution"

# Start machine daemon (should wait for config)
log "Starting machine daemon..."
export MALAI_HOME="$TEST_DIR/machine1"
$MALAI_BIN daemon --foreground > "$TEST_DIR/machine-daemon.log" 2>&1 &
MACHINE_PID=$!
sleep 2

# Verify machine daemon started
if ! kill -0 $MACHINE_PID 2>/dev/null; then
    cat "$TEST_DIR/machine-daemon.log"
    error "Machine daemon failed to start"
fi
success "Machine daemon started"

# Start cluster manager daemon (should distribute config)
log "Starting cluster manager daemon..."
export MALAI_HOME="$TEST_DIR/cluster-manager"
$MALAI_BIN daemon --foreground > "$TEST_DIR/cluster-daemon.log" 2>&1 &
CLUSTER_PID=$!
sleep 3

# Verify cluster manager started
if ! kill -0 $CLUSTER_PID 2>/dev/null; then
    cat "$TEST_DIR/cluster-daemon.log"
    error "Cluster manager daemon failed to start"
fi
success "Cluster manager daemon started"

# Check config distribution happened
log "Checking config distribution..."
if ! grep -q "Config sent successfully" "$TEST_DIR/cluster-daemon.log"; then
    cat "$TEST_DIR/cluster-daemon.log"
    error "Config distribution did not complete"
fi
success "Config distribution successful"

# Verify machine received config
MACHINE_CONFIG="$TEST_DIR/machine1/clusters/$CLUSTER_NAME/machine-config.toml"
sleep 2  # Wait for config to be written
assert_file_exists "$MACHINE_CONFIG"
assert_contains "$MACHINE_CONFIG" "$MACHINE_ID52"
assert_contains "$MACHINE_CONFIG" "allow_from"
success "Machine received personalized config"

# Phase 5: Test Remote Command Execution
log "💻 Phase 5: Testing remote command execution"

# Test successful command execution
log "Testing authorized command execution..."
if ! $MALAI_BIN web01.$CLUSTER_NAME echo "E2E test successful" > "$TEST_DIR/command-success.log" 2>&1; then
    cat "$TEST_DIR/command-success.log"
    error "Authorized command execution failed"
fi

# Verify command output
assert_contains "$TEST_DIR/command-success.log" "E2E test successful"
assert_contains "$TEST_DIR/command-success.log" "Remote command executed successfully"
success "Authorized command execution working"

# Test real command execution with actual output
log "Testing real command execution..."
export MALAI_HOME="$TEST_DIR/cluster-manager"
if ! $MALAI_BIN web01.$CLUSTER_NAME whoami > "$TEST_DIR/whoami.log" 2>&1; then
    cat "$TEST_DIR/whoami.log"
    error "Real command execution failed"
fi

# Verify we got actual command output (username)
if ! grep -q "amitu\|$(whoami)" "$TEST_DIR/whoami.log"; then
    cat "$TEST_DIR/whoami.log"  
    error "Did not receive real command output"
fi
success "Real command execution verified"

# Phase 6: Test Permission Denial (TODO when ACL fully implemented)
log "⛔ Phase 6: Testing permission denial"
log "⚠️  ACL denial testing not yet implemented (permissions currently allow all)"

# Phase 7: Verify Status Command
log "📊 Phase 7: Testing status command"
if ! $MALAI_BIN status > "$TEST_DIR/status.log" 2>&1; then
    cat "$TEST_DIR/status.log"
    error "Status command failed"
fi

assert_contains "$TEST_DIR/status.log" "Cluster Manager"
assert_contains "$TEST_DIR/status.log" "Machines: 2"
assert_contains "$TEST_DIR/status.log" "Config Sync Status"
success "Status command working with sync information"

# Cleanup
log "🧹 Cleaning up test processes..."
kill $MACHINE_PID $CLUSTER_PID 2>/dev/null || true
wait $MACHINE_PID $CLUSTER_PID 2>/dev/null || true

    success "Bash P2P infrastructure test PASSED"
    BASH_RESULT="✅ PASSED" 
    TESTS_PASSED=$((TESTS_PASSED + 1))
    TESTS_RUN=$((TESTS_RUN + 1))
    echo
}

# Run Rust test (future)
run_rust_test() {
    header "🦀 CRITICAL TEST: Rust Integration (Future)"
    log "Rust integration tests not yet implemented"
    warn "Will implement comprehensive Rust tests in next phase"
    echo
}

# Main execution following fastn-me pattern
header "🎯 MALAI CRITICAL INFRASTRUCTURE TESTS"
echo
log "This is the most important test in malai"
log "If this passes, the entire infrastructure system is operational"
echo

# Run selected tests
if $RUN_BASH; then
    run_bash_test
fi

if $RUN_RUST; then
    run_rust_test
fi

# Final results
header "📊 Final Test Results"
echo "Bash P2P Infrastructure: ${BASH_RESULT:-Not run}"
echo "Tests passed: $TESTS_PASSED/$TESTS_RUN"
echo

if [ "$TESTS_PASSED" -eq "$TESTS_RUN" ] && [ "$TESTS_RUN" -gt 0 ]; then
    success "All malai tests PASSED!"
    log "🚀 malai infrastructure is working!"
else
    error "Some tests failed - infrastructure needs fixes"
fi