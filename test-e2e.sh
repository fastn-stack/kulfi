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
    header "🏗️  CRITICAL TEST: malai Business Logic E2E" 
    log "Test: Complete malai with proper file structure and role detection"
    log "Mode: Design-compliant cluster.toml, machine.toml, multi-role support"
    echo
    
    # Phase 1: Role Detection with Proper File Structure
    log "📋 Phase 1: Testing role detection and file structure"
    
    # Setup cluster manager with design-compliant structure
    CLUSTER_DIR="$TEST_DIR/cluster-manager/clusters/company"
    mkdir -p "$CLUSTER_DIR"
    
    # Generate identity for cluster manager
    export MALAI_HOME="$TEST_DIR/cluster-manager"
    if ! $MALAI_BIN keygen --file "$CLUSTER_DIR/identity.key" > "$TEST_DIR/cm-keygen.log" 2>&1; then
        error "Cluster manager keygen failed"
    fi
    
    CM_ID52=$(grep "Generated Public Key (ID52):" "$TEST_DIR/cm-keygen.log" | cut -d: -f2 | tr -d ' ')
    log "✅ Cluster Manager ID52: $CM_ID52"
    
    # Create design-compliant cluster.toml (not cluster-config.toml)
    cat > "$CLUSTER_DIR/cluster.toml" << EOF
[cluster_manager]
id52 = "$CM_ID52"
cluster_name = "company"

[machine.web01]
id52 = "$CM_ID52"
allow_from = "*"

[machine.server1]  
id52 = "remote-machine-id52"
allow_from = "*"
EOF
    
    # Test role detection
    if ! $MALAI_BIN scan-roles > "$TEST_DIR/role-scan.log" 2>&1; then
        cat "$TEST_DIR/role-scan.log"
        error "Role detection failed"
    fi
    
    assert_contains "$TEST_DIR/role-scan.log" "Cluster Manager role detected"
    assert_contains "$TEST_DIR/role-scan.log" "ClusterManager"
    success "Role detection working with proper file structure"
    
    # Phase 2: Configuration Validation
    log "📝 Phase 2: Testing configuration validation"
    
    if ! $MALAI_BIN rescan --check > "$TEST_DIR/config-check.log" 2>&1; then
        cat "$TEST_DIR/config-check.log"
        error "Config validation failed"
    fi
    
    assert_contains "$TEST_DIR/config-check.log" "All configurations valid"
    success "Configuration validation working"
    
    # Phase 3: Basic P2P Infrastructure Test  
    log "📡 Phase 3: Testing P2P infrastructure"
    
    if ! $MALAI_BIN test-simple > "$TEST_DIR/simple-p2p.log" 2>&1; then
        cat "$TEST_DIR/simple-p2p.log"
        error "Basic P2P test failed"
    fi
    assert_contains "$TEST_DIR/simple-p2p.log" "Echo: Hello from simple test"
    success "P2P infrastructure working"
    
    # Phase 4: Complete Infrastructure Test
    log "🚀 Phase 4: Testing complete malai functionality"
    
    if ! $MALAI_BIN test-real > "$TEST_DIR/complete-test.log" 2>&1; then
        cat "$TEST_DIR/complete-test.log"
        error "Complete infrastructure test failed"
    fi
    
    assert_contains "$TEST_DIR/complete-test.log" "Config distribution successful"
    assert_contains "$TEST_DIR/complete-test.log" "Complete malai infrastructure working!"
    success "Complete infrastructure working"
    
    # Phase 5: File Structure Validation
    log "📁 Phase 5: Validating created file structure"
    
    # Check that config file was created with proper structure
    if [[ -f "machine-config.toml" ]]; then
        assert_contains "machine-config.toml" "cluster_manager"
        success "Config file structure correct"
        rm -f "machine-config.toml"
    fi
    
    # Verify our test setup matches design
    assert_file_exists "$CLUSTER_DIR/cluster.toml"
    assert_file_exists "$CLUSTER_DIR/identity.key" 
    assert_contains "$CLUSTER_DIR/cluster.toml" "cluster_manager"
    assert_contains "$CLUSTER_DIR/cluster.toml" "machine.web01"
    success "Test setup follows design specification"

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