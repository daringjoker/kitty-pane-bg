#!/bin/bash

# Demo script for kitty-pane-bg with color caching
# This script demonstrates the color persistence and opacity features

set -e

echo "🎨 Kitty Pane Background Generator Demo (with Bright Pastel Colors)"
echo "=================================================================="

# Check if we're in the right environment
if [ -z "$KITTY_WINDOW_ID" ]; then
    echo "❌ Please run this demo inside Kitty terminal"
    exit 1
fi

if [ -z "$TMUX" ]; then
    echo "❌ Please run this demo inside a tmux session"
    echo "   Start tmux with: tmux"
    exit 1
fi

echo "✅ Environment check passed"
echo ""

# Build the project if needed
if [ ! -f "target/release/kitty-pane-bg" ]; then
    echo "🔨 Building the project..."
    cargo build --release
    echo "✅ Build complete"
    echo ""
fi

BINARY="./target/release/kitty-pane-bg"

# Clear any existing cache to start fresh
echo "🧹 Clearing existing color cache..."
$BINARY cache clear
echo ""

# Check environment
echo "1️⃣  Checking environment..."
$BINARY check
echo ""

# Show empty cache
echo "2️⃣  Showing empty color cache..."
$BINARY cache show
echo ""

# Generate initial background
echo "3️⃣  Generating initial background with bright pastel colors (15% opacity)..."
$BINARY generate --opacity 0.15 --output demo_bg_1.png
echo "Generated: demo_bg_1.png"
echo ""

# Show cache after first generation
echo "4️⃣  Showing color cache after first generation..."
$BINARY cache show
echo ""

# Split the pane horizontally
echo "5️⃣  Splitting pane horizontally (new pane gets maximally distinct color)..."
tmux split-window -h -c "#{pane_current_path}"
sleep 1

$BINARY generate --opacity 0.2 --output demo_bg_2.png
echo "Generated: demo_bg_2.png (2 panes with optimally distinct colors)"
echo ""

# Show cache with 2 panes
echo "6️⃣  Cache now contains 2 pane colors..."
$BINARY cache show
echo ""

# Split the right pane vertically
echo "7️⃣  Splitting right pane vertically..."
tmux split-window -v -c "#{pane_current_path}"
sleep 1

$BINARY generate --opacity 0.25 --output demo_bg_3.png
echo "Generated: demo_bg_3.png (3 panes with bright pastels)"
echo ""

# Create one more split
echo "8️⃣  Creating another split..."
tmux select-pane -t 0
tmux split-window -v -c "#{pane_current_path}"
sleep 1

$BINARY generate --opacity 0.3 --output demo_bg_4.png
echo "Generated: demo_bg_4.png (4 panes with maximum color distinctness)"
echo ""

# Show final cache state
echo "9️⃣  Final cache state with 4 panes..."
$BINARY cache show
echo ""

# Demonstrate color persistence by killing a pane and recreating
echo "� Testing color persistence..."
echo "   Killing one pane and recreating - notice color differences!"

# Kill the last pane
tmux select-pane -t 3
KILLED_PANE_ID=$(tmux display-message -p "#{pane_id}")
tmux kill-pane

sleep 1
$BINARY generate --opacity 0.15 --output demo_bg_after_kill.png
echo "Generated: demo_bg_after_kill.png (after killing pane $KILLED_PANE_ID)"
echo ""

# Create a new pane - it should get a different color
tmux split-window -v -c "#{pane_current_path}"
sleep 1
$BINARY generate --opacity 0.25 --output demo_bg_new_pane.png
echo "Generated: demo_bg_new_pane.png (new pane with maximally distinct color)"
echo ""

# Show the cache after cleanup
echo "🧽 Cache after automatic cleanup (removed killed pane)..."
$BINARY cache show
echo ""

# Install hooks for automatic generation
echo "🔧 Installing tmux hooks for automatic generation..."
$BINARY install-hooks
echo ""

echo "⚡ Testing automatic generation..."
echo "   (Creating and closing a pane - background will auto-generate)"
tmux split-window -h -c "#{pane_current_path}" "sleep 2"
sleep 3

if [ -f "/tmp/kitty-pane-bg.png" ]; then
    echo "✅ Automatic generation working! Check /tmp/kitty-pane-bg.png"
    echo "   (This image uses bright pastel colors with 10% opacity)"
else
    echo "⚠️  Automatic generation may not be working"
fi

echo ""
echo "🎉 Demo complete!"
echo ""
echo "📁 Generated files:"
echo "   - demo_bg_1.png (1 pane, first color assigned)"
echo "   - demo_bg_2.png (2 panes, colors cached)" 
echo "   - demo_bg_3.png (3 panes)"
echo "   - demo_bg_4.png (4 panes)"
echo "   - demo_bg_after_kill.png (after killing a pane)"
echo "   - demo_bg_new_pane.png (new pane gets different color)"
echo "   - /tmp/kitty-pane-bg.png (auto-generated with hooks)"
echo ""
echo "🔍 Key features demonstrated:"
echo "   ✅ 10% opacity for subtle background effect"
echo "   ✅ Color persistence across pane operations"
echo "   ✅ Automatic cache cleanup when panes are closed"
echo "   ✅ Different colors for new panes even with same dimensions"
echo "   ✅ Deterministic color generation based on startup seed"
echo ""
echo "💡 Try resizing panes or creating new splits - the background"
echo "   will automatically update thanks to the installed hooks!"
echo ""
echo "🧹 To clean up demo files: rm demo_bg*.png"
echo "🗑️  To clear color cache: $BINARY cache clear"
