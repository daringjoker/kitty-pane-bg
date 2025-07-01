#!/bin/sh

# kitty-pane-bg Installation Script
# Usage: curl -sSL https://raw.githubusercontent.com/daringjoker/kitty-pane-bg/main/install.sh | shin/bash

# kitty-pane-bg Installation Script
# Usage: curl -sSL https://raw.githubusercontent.com/USERNAME/kitty-pane-bg/main/install.sh | sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_URL="https://github.com/daringjoker/kitty-pane-bg"
BINARY_NAME="kitty-pane-bg"
INSTALL_DIR="$HOME/.local/bin"
TEMP_DIR="/tmp/kitty-pane-bg-install"

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check glibc version and binary compatibility
check_glibc_compatibility() {
    binary_path="$1"
    
    # Only check on Linux systems
    case "$(uname -s)" in
        Linux*) ;;
        *) return 0 ;;  # Not Linux, skip check
    esac
    
    log_info "Checking binary compatibility..."
    
    # Try to run the binary with --help to test compatibility
    if "$binary_path" --help >/dev/null 2>&1; then
        log_success "Binary is compatible with system"
        return 0
    fi
    
    # Check if it's a glibc version issue
    if ldd "$binary_path" 2>&1 | grep -q "version.*not found"; then
        log_warning "glibc version mismatch detected"
        
        # Get system glibc version
        system_glibc=""
        if command_exists ldd; then
            system_glibc=$(ldd --version 2>&1 | head -n1 | grep -o '[0-9]\+\.[0-9]\+' | head -n1)
        fi
        
        # Get required glibc version from binary
        required_glibc=""
        if command_exists objdump; then
            required_glibc=$(objdump -T "$binary_path" 2>/dev/null | grep "GLIBC_" | sed 's/.*GLIBC_//' | sort -V | tail -n1)
        fi
        
        log_warning "System glibc: ${system_glibc:-unknown}"
        log_warning "Required glibc: ${required_glibc:-unknown}"
        log_warning "Pre-built binary is not compatible with your system's glibc version"
        log_info "Will build from source instead..."
        
        return 1
    fi
    
    # Other compatibility issue
    log_warning "Binary compatibility issue detected (not glibc related)"
    log_info "Will build from source instead..."
    return 1
}

# Check if we can upgrade glibc (for advanced users)
suggest_glibc_upgrade() {
    log_info "Alternative solutions:"
    echo "  1. Build from source (recommended - will be done automatically)"
    echo "  2. Use a newer Linux distribution with updated glibc"
    echo "  3. Use static binary (if available in future releases)"
    echo ""
    echo "Common glibc versions by distribution:"
    echo "  • Ubuntu 18.04: glibc 2.27"
    echo "  • Ubuntu 20.04: glibc 2.31"
    echo "  • Ubuntu 22.04: glibc 2.35"
    echo "  • CentOS 7: glibc 2.17"
    echo "  • CentOS 8: glibc 2.28"
    echo "  • Debian 10: glibc 2.28"
    echo "  • Debian 11: glibc 2.31"
    echo ""
}

# Detect OS and architecture
detect_platform() {
    os=""
    arch=""
    
    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="macos" ;;
        CYGWIN*|MINGW*|MSYS*) os="windows" ;;
        *)          log_error "Unsupported operating system: $(uname -s)" ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64" ;;
        arm64|aarch64)  arch="aarch64" ;;
        armv7l)         arch="armv7" ;;
        *)              log_error "Unsupported architecture: $(uname -m)" ;;
    esac
    
    echo "${os}-${arch}"
}

# Check system requirements
check_requirements() {
    log_info "Checking system requirements..."
    
    # Check for required commands
    missing_deps=""
    
    if ! command_exists curl; then
        missing_deps="curl"
    fi
    
    if ! command_exists tmux; then
        if [ -z "$missing_deps" ]; then
            missing_deps="tmux"
        else
            missing_deps="$missing_deps tmux"
        fi
    fi
    
    if ! command_exists kitty; then
        log_warning "kitty terminal not found. This tool requires kitty terminal to work."
        log_info "Install kitty from: https://sw.kovidgoyal.net/kitty/binary/"
    fi
    
    if [ -n "$missing_deps" ]; then
        log_error "Missing required dependencies: $missing_deps"
        log_info "Please install them using your system package manager:"
        log_info "  Ubuntu/Debian: sudo apt install $missing_deps"
        log_info "  RHEL/CentOS/Fedora: sudo dnf install $missing_deps"
        log_info "  macOS: brew install $missing_deps"
        exit 1
    fi
    
    log_success "All required dependencies found"
}

# Install Rust if not present
install_rust() {
    if command_exists rustc && command_exists cargo; then
        log_info "Rust already installed: $(rustc --version)"
        return 0
    fi
    
    log_info "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    
    # Source the cargo environment
    if [ -f "$HOME/.cargo/env" ]; then
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    else
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
    
    if command_exists rustc && command_exists cargo; then
        log_success "Rust installed successfully: $(rustc --version)"
    else
        log_error "Failed to install Rust"
    fi
}

# Try to download pre-built binary
download_binary() {
    platform=$(detect_platform)
    
    log_info "Attempting to download pre-built binary for $platform..."
    
    # Determine the correct binary name and extension
    binary_name="${BINARY_NAME}-${platform}"
    download_name="${BINARY_NAME}"
    
    # Add .exe extension for Windows
    case "$platform" in
        *windows*)
            binary_name="${binary_name}.exe"
            download_name="${download_name}.exe"
            ;;
    esac
    
    # Try to get the latest release
    latest_url="${REPO_URL}/releases/latest/download/${binary_name}"
    
    if curl -sSLf "$latest_url" -o "${TEMP_DIR}/${download_name}" 2>/dev/null; then
        chmod +x "${TEMP_DIR}/${download_name}"
        log_success "Downloaded pre-built binary"
        
        # Check glibc compatibility on Linux
        if check_glibc_compatibility "${TEMP_DIR}/${download_name}"; then
            return 0
        else
            log_warning "Downloaded binary is not compatible with this system"
            suggest_glibc_upgrade
            return 1
        fi
    else
        log_warning "Pre-built binary not available for $platform"
        return 1
    fi
}

# Build from source
build_from_source() {
    log_info "Building from source..."
    
    # Clean up any existing temp directory
    rm -rf "$TEMP_DIR"
    mkdir -p "$TEMP_DIR"
    
    # Clone the repository
    log_info "Cloning repository..."
    if command_exists git; then
        git clone "$REPO_URL.git" "$TEMP_DIR"
    else
        # Fallback to downloading archive
        archive_url="${REPO_URL}/archive/refs/heads/main.tar.gz"
        curl -sSL "$archive_url" | tar -xz -C "$TEMP_DIR" --strip-components=1
    fi
    
    # Build the project
    cd "$TEMP_DIR"
    log_info "Building with cargo (this may take a few minutes)..."
    cargo build --release
    
    # Copy the binary with correct name/extension
    source_binary="target/release/${BINARY_NAME}"
    dest_binary="${BINARY_NAME}"
    
    # Add .exe extension for Windows
    case "$(uname -s)" in
        CYGWIN*|MINGW*|MSYS*)
            source_binary="${source_binary}.exe"
            dest_binary="${dest_binary}.exe"
            ;;
    esac
    
    cp "$source_binary" "${TEMP_DIR}/${dest_binary}"
    log_success "Built from source successfully"
}

# Install the binary
install_binary() {
    # Determine the correct binary name
    source_binary="${BINARY_NAME}"
    dest_binary="${BINARY_NAME}"
    
    # Add .exe extension for Windows if needed
    case "$(uname -s)" in
        CYGWIN*|MINGW*|MSYS*)
            source_binary="${source_binary}.exe"
            # But keep dest_binary without .exe for Unix-like PATH
            ;;
    esac
    
    log_info "Installing ${dest_binary} to ${INSTALL_DIR}..."
    
    # Create install directory
    mkdir -p "$INSTALL_DIR"
    
    # Copy binary
    cp "${TEMP_DIR}/${source_binary}" "${INSTALL_DIR}/${dest_binary}"
    chmod +x "${INSTALL_DIR}/${dest_binary}"
    
    log_success "Installed ${dest_binary} to ${INSTALL_DIR}/${dest_binary}"
}

# Update PATH if needed
update_path() {
    # Check if install directory is in PATH
    case ":$PATH:" in
        *":$INSTALL_DIR:"*)
            # Already in PATH
            ;;
        *)
            log_info "Adding ${INSTALL_DIR} to PATH..."
            
            # Determine shell config file
            shell_config=""
            case "$SHELL" in
                */bash)
                    shell_config="$HOME/.bashrc"
                    [ -f "$HOME/.bash_profile" ] && shell_config="$HOME/.bash_profile"
                    ;;
                */zsh)
                    shell_config="$HOME/.zshrc"
                    ;;
                */fish)
                    shell_config="$HOME/.config/fish/config.fish"
                    ;;
                *)
                    shell_config="$HOME/.profile"
                    ;;
            esac
            
            # Add to PATH
            if [ -f "$shell_config" ]; then
                echo "" >> "$shell_config"
                echo "# Added by kitty-pane-bg installer" >> "$shell_config"
                echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$shell_config"
                log_success "Added to PATH in $shell_config"
                log_info "Please run 'source $shell_config' or restart your shell"
            else
                log_warning "Could not automatically update PATH"
                log_info "Please manually add $INSTALL_DIR to your PATH"
            fi
            ;;
    esac
}

# Setup kitty configuration
setup_kitty() {
    log_info "Setting up kitty configuration..."
    
    local kitty_config="$HOME/.config/kitty/kitty.conf"
    
    # Create kitty config directory if it doesn't exist
    mkdir -p "$(dirname "$kitty_config")"
    
    # Check if remote control is already enabled
    if [ -f "$kitty_config" ] && grep -q "allow_remote_control" "$kitty_config"; then
        log_info "Kitty remote control already configured"
    else
        log_info "Enabling kitty remote control..."
        
        # Backup existing config
        if [ -f "$kitty_config" ]; then
            cp "$kitty_config" "${kitty_config}.backup"
            log_info "Backed up existing kitty.conf to ${kitty_config}.backup"
        fi
        
        # Add remote control configuration
        cat >> "$kitty_config" << EOF

# Added by kitty-pane-bg installer
allow_remote_control yes
listen_on unix:/tmp/kitty
EOF
        
        log_success "Updated kitty configuration"
        log_warning "Please restart kitty for changes to take effect"
    fi
}

# Run post-installation checks
post_install_check() {
    log_info "Running post-installation checks..."
    
    # Test binary
    log_info "Running ${INSTALL_DIR}/${BINARY_NAME} > /dev/null 2>&1"
    if ! "${INSTALL_DIR}/${BINARY_NAME}" --help >/dev/null 2>&1; then
        log_error "Binary installation failed - ${BINARY_NAME} not working"
    fi
    
    # Check if binary is in PATH
    if command_exists "$BINARY_NAME"; then
        log_success "✅ ${BINARY_NAME} is available in PATH"
    else
        log_warning "⚠️  ${BINARY_NAME} not in PATH - you may need to restart your shell"
    fi
    
    log_success "Installation completed successfully!"
}

# Show usage instructions
show_usage() {
    cat << EOF

🎉 kitty-pane-bg has been installed successfully!

Next steps:
1. Restart kitty terminal (or reload kitty config)
2. Start a tmux session: tmux
3. Try the tool:
   ${BINARY_NAME} check              # Check if everything is working
   ${BINARY_NAME} generate           # Generate a background image
   ${BINARY_NAME} set-background     # Generate and set as kitty background
   ${BINARY_NAME} install-hooks      # Install automatic tmux hooks

For more information, run: ${BINARY_NAME} --help

EOF
}

# Cleanup
cleanup() {
    if [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR"
    fi
}

# Main installation function
main() {
    log_info "Starting kitty-pane-bg installation..."
    
    # Set up cleanup trap
    trap cleanup EXIT
    
    # Create temp directory
    mkdir -p "$TEMP_DIR"
    
    # Run installation steps
    check_requirements
    
    # Try to download binary, fallback to building from source
    if ! download_binary; then
        install_rust
        build_from_source
    fi
    
    install_binary
    update_path
    setup_kitty
    post_install_check
    show_usage
    
    log_success "🚀 Installation complete!"
}

# Run main function
main "$@"
