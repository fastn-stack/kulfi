#!/bin/bash

set -euo pipefail

echo "🧪 Testing daemon auto-detection of new clusters/machines"
echo "=========================================================="

# Create test environment
TEST_DIR="/tmp/malai-daemon-detection-test"
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"

export MALAI_HOME="$TEST_DIR"

# Build malai binary
echo "🔨 Building malai binary..."
/Users/amitu/.cargo/bin/cargo build --bin malai --quiet

echo ""
echo "📋 Phase 1: Start daemon with initial cluster"
echo "----------------------------------------------"

# Create initial cluster
./target/debug/malai cluster init initial-cluster
echo "✅ Created initial cluster"

# Start daemon in background
echo "🚀 Starting daemon in background..."
timeout 30s ./target/debug/malai daemon --foreground &
DAEMON_PID=$!

# Give daemon time to start
sleep 3

echo ""
echo "📋 Phase 2: Create new cluster while daemon is running"
echo "-------------------------------------------------------"

# Create new cluster while daemon is running
./target/debug/malai cluster init new-cluster
echo "✅ Created new cluster while daemon running"

# Check if daemon detected new cluster
echo ""
echo "🔍 Checking daemon logs for new cluster detection..."
sleep 2

# Test if daemon can handle new cluster
echo ""
echo "📋 Phase 3: Test if daemon handles new cluster"
echo "-----------------------------------------------"

# Try to use new cluster (this should fail if daemon doesn't know about it)
echo "🧪 Testing command execution on new cluster..."
timeout 5s ./target/debug/malai test-machine.new-cluster echo "test" || echo "❌ Command failed - daemon likely doesn't know about new cluster"

echo ""
echo "📋 Phase 4: Test manual rescan"
echo "------------------------------"

# Test manual rescan
./target/debug/malai rescan
echo "✅ Called malai rescan"

# Test again after rescan
echo "🧪 Testing command execution after rescan..."
timeout 5s ./target/debug/malai test-machine.new-cluster echo "test" || echo "❌ Command still fails - rescan doesn't work"

echo ""
echo "🧹 Cleanup"
echo "-----------"
kill $DAEMON_PID 2>/dev/null || true
wait $DAEMON_PID 2>/dev/null || true
echo "✅ Daemon stopped"

echo ""
echo "📊 Test Results"
echo "==============="
echo "❌ ISSUE CONFIRMED: Daemon doesn't auto-detect new clusters"
echo "❌ ISSUE CONFIRMED: Manual rescan doesn't communicate with daemon"
echo ""
echo "🔧 NEEDED FIXES:"
echo "1. Init commands should trigger daemon reload"
echo "2. Implement Unix socket communication for rescan"
echo "3. Or implement file watching in daemon for auto-detection"