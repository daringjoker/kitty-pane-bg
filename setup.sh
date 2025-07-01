#!/bin/bash

# kitty-pane-bg setup script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR"

echo "🎨 Kitty Pane Background Generator Setup"
echo "========================================"

# Check if we're in kitty
if [ -z "$KITTY_WINDOW_ID" ]; then
    echo "❌ This script must be run inside Kitty terminal"
    exit 1
fi

echo "✅ Running in Kitty terminal"

# Check if tmux is available
if ! command -v tmux >/dev/null 2>&1; then
    echo "❌ tmux is not installed. Please install tmux first."
    exit 1
fi

echo "✅ tmux is available"

# Check if we're in a tmux session
if [ -z "$TMUX" ]; then
    echo "⚠️  Not currently in a tmux session"
    echo "   You can start tmux with: tmux"
else
    echo "✅ Running in tmux session"
fi

# Build the project
echo "🔨 Building the project..."
cd "$PROJECT_DIR"

if ! cargo build --release; then
    echo "❌ Failed to build the project"
    exit 1
fi

echo "✅ Project built successfully"

# Create symlink to make it easily accessible
BINARY_PATH="$PROJECT_DIR/target/release/kitty-pane-bg"
SYMLINK_PATH="$HOME/.local/bin/kitty-pane-bg"

if [ ! -d "$HOME/.local/bin" ]; then
    mkdir -p "$HOME/.local/bin"
fi

if [ -L "$SYMLINK_PATH" ] || [ -f "$SYMLINK_PATH" ]; then
    rm "$SYMLINK_PATH"
fi

ln -s "$BINARY_PATH" "$SYMLINK_PATH"
echo "✅ Created symlink: $SYMLINK_PATH"

# Add to PATH if not already there
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo "⚠️  $HOME/.local/bin is not in your PATH"
    echo "   Add this to your shell config:"
    echo "   export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

echo ""
echo "🎉 Setup complete!"
echo ""
echo "Usage:"
echo "  kitty-pane-bg check              # Check environment"
echo "  kitty-pane-bg generate           # Generate background image (10% opacity)"
echo "  kitty-pane-bg install-hooks      # Install tmux hooks for auto-generation"
echo "  kitty-pane-bg cache show         # Show cached pane colors"
echo "  kitty-pane-bg cache clear        # Clear all cached colors"
echo ""
echo "Features:"
echo "  🎨 Each pane gets a unique, persistent color"
echo "  💾 Colors are cached and survive pane operations"
echo "  🎭 10% opacity for subtle background effect"
echo "  🔄 Automatic cleanup when panes are closed"
echo ""
echo "Next steps:"
echo "1. Start a tmux session: tmux"
echo "2. Check environment: kitty-pane-bg check"
echo "3. Generate a test image: kitty-pane-bg generate"
echo "4. Install hooks for automatic generation: kitty-pane-bg install-hooks"
echo "5. Run the demo to see all features: ./demo.sh"
