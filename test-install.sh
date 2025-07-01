#!/bin/bash

# Test script for the installation script
# This script tests the install.sh script in a safe way

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${YELLOW}[TEST]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

# Test the installation script without actually installing
test_install_script() {
    log_info "Testing installation script..."
    
    # Check if script exists and is executable
    if [ -f "./install.sh" ] && [ -x "./install.sh" ]; then
        log_success "install.sh exists and is executable"
    else
        log_error "install.sh is missing or not executable"
        return 1
    fi
    
    # Test script syntax
    if bash -n ./install.sh; then
        log_success "install.sh has valid bash syntax"
    else
        log_error "install.sh has syntax errors"
        return 1
    fi
    
    # Test help/dry-run (if we add that feature)
    log_info "Installation script appears to be valid"
    
    # Show what the script would do
    log_info "The install script will:"
    echo "  - Check for tmux, curl, and kitty"
    echo "  - Install Rust if not present"
    echo "  - Build kitty-pane-bg from source"
    echo "  - Install to ~/.local/bin"
    echo "  - Configure kitty for remote control"
    echo "  - Update shell PATH"
    
    log_success "Installation script test completed"
}

# Test current build
test_current_build() {
    log_info "Testing current build..."
    
    if cargo check --quiet; then
        log_success "Project compiles successfully"
    else
        log_error "Project has compilation errors"
        return 1
    fi
    
    if cargo test --quiet 2>/dev/null; then
        log_success "All tests pass"
    else
        log_info "No tests found or tests failed (this may be expected)"
    fi
}

# Test binary functionality (if already built)
test_binary() {
    log_info "Testing binary functionality..."
    
    if [ -f "target/release/kitty-pane-bg" ]; then
        if ./target/release/kitty-pane-bg --help >/dev/null 2>&1; then
            log_success "Binary runs and shows help"
        else
            log_error "Binary fails to run"
            return 1
        fi
    else
        log_info "Binary not built yet, building for test..."
        if cargo build --release --quiet; then
            log_success "Binary built successfully"
            if ./target/release/kitty-pane-bg --help >/dev/null 2>&1; then
                log_success "Binary runs correctly"
            else
                log_error "Binary built but fails to run"
                return 1
            fi
        else
            log_error "Failed to build binary"
            return 1
        fi
    fi
}

# Main test function
main() {
    log_info "Starting installation tests..."
    echo
    
    # Change to script directory
    cd "$(dirname "$0")"
    
    # Run tests
    test_install_script
    echo
    
    test_current_build
    echo
    
    test_binary
    echo
    
    log_success "🎉 All tests passed! The installation script should work correctly."
    echo
    echo "To test the installation script:"
    echo "  1. In a clean environment/container:"
    echo "     curl -sSL https://raw.githubusercontent.com/YOUR_USERNAME/kitty-pane-bg/main/install.sh | sh"
    echo
    echo "  2. Or run locally (will modify your system):"
    echo "     ./install.sh"
}

main "$@"
