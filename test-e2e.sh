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
    log "Test: Complete malai infrastructure using clean malai_server.rs"
    log "Mode: Working P2P protocols with one listener per identity"
    echo
    
    # Simple test using working malai_server.rs
    log "🧪 Testing working malai infrastructure"
    
    # Test basic P2P functionality
    log "Testing simple P2P..."
    if ! $MALAI_BIN test-simple > "$TEST_DIR/simple-test.log" 2>&1; then
        cat "$TEST_DIR/simple-test.log"
        error "Simple P2P test failed"
    fi
    assert_contains "$TEST_DIR/simple-test.log" "Echo: Hello from simple test"
    success "Simple P2P working"
    
    # Test complete infrastructure  
    log "Testing complete infrastructure..."
    if ! $MALAI_BIN test-real > "$TEST_DIR/real-test.log" 2>&1; then
        cat "$TEST_DIR/real-test.log"
        error "Real infrastructure test failed"  
    fi
    
    # Verify config distribution worked
    assert_contains "$TEST_DIR/real-test.log" "Config distribution successful"
    assert_contains "$TEST_DIR/real-test.log" "Config saved to: machine-config.toml"
    success "Config distribution working"
    
    # Verify command execution worked
    assert_contains "$TEST_DIR/real-test.log" "Complete malai infrastructure working!"
    assert_contains "$TEST_DIR/real-test.log" "Command completed: exit_code=0"
    success "Command execution working"
    
    # Verify config file was created
    if [[ -f "machine-config.toml" ]]; then
        assert_contains "machine-config.toml" "cluster_manager"
        assert_contains "machine-config.toml" "machine.server1"
        success "Config file created correctly"
        rm -f "machine-config.toml"  # Cleanup
    else
        error "Config file not created"
    fi

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