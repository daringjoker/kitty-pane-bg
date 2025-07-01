#!/bin/bash

# Kitty configuration helper for kitty-pane-bg

echo "🐱 Kitty Configuration Helper for kitty-pane-bg"
echo "=============================================="

# Check if we're in kitty
if [ -z "$KITTY_WINDOW_ID" ]; then
    echo "❌ This script must be run inside Kitty terminal"
    exit 1
fi

echo "✅ Running in Kitty terminal"

# Check if kitty.conf exists
KITTY_CONFIG_DIR="$HOME/.config/kitty"
KITTY_CONFIG="$KITTY_CONFIG_DIR/kitty.conf"

if [ ! -d "$KITTY_CONFIG_DIR" ]; then
    echo "📁 Creating kitty config directory: $KITTY_CONFIG_DIR"
    mkdir -p "$KITTY_CONFIG_DIR"
fi

if [ ! -f "$KITTY_CONFIG" ]; then
    echo "📝 Creating new kitty.conf file"
    touch "$KITTY_CONFIG"
fi

# Check if remote control is already enabled
if grep -q "allow_remote_control" "$KITTY_CONFIG" 2>/dev/null; then
    echo "ℹ️  Remote control setting already exists in kitty.conf"
    if grep -q "^allow_remote_control yes" "$KITTY_CONFIG"; then
        echo "✅ Remote control is enabled"
    else
        echo "⚠️  Remote control may be disabled or commented out"
    fi
else
    echo "➕ Adding remote control settings to kitty.conf"
    cat >> "$KITTY_CONFIG" << 'EOF'

# Settings for kitty-pane-bg
allow_remote_control yes
listen_on unix:/tmp/kitty

EOF
    echo "✅ Remote control settings added"
fi

# Check if socket settings exist
if ! grep -q "listen_on" "$KITTY_CONFIG" 2>/dev/null; then
    echo "➕ Adding socket settings to kitty.conf"
    echo "listen_on unix:/tmp/kitty" >> "$KITTY_CONFIG"
    echo "✅ Socket settings added"
fi

echo ""
echo "📋 Current remote control related settings in kitty.conf:"
grep -E "(allow_remote_control|listen_on)" "$KITTY_CONFIG" || echo "   (none found)"

echo ""
echo "🔄 To apply these changes:"
echo "   1. Press Ctrl+Shift+F5 to reload kitty config, OR"
echo "   2. Restart kitty"

echo ""
echo "🧪 Testing remote control..."
if kitty @ ls >/dev/null 2>&1; then
    echo "✅ Remote control is working!"
else
    echo "❌ Remote control is not working yet"
    echo "   Please reload kitty config or restart kitty"
fi

echo ""
echo "✨ Configuration complete!"
echo "   You can now use kitty-pane-bg with full functionality"
