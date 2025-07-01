# kitty-pane-bg

A Rust CLI tool that generates background images based on your tmux pane layout for the kitty terminal. Each pane is rendered with a unique color, creating a visual representation of your workspace.

## Features

- Generates background images matching tmux pane layouts
- Persistent color assignments for each pane
- Automatic background setting via kitty's remote control
- Real-time updates through tmux hooks
- High-performance image generation

## Requirements

- **kitty terminal** with remote control enabled
- **tmux** for pane management
- **Rust** (for building from source)

## Installation

### Quick Install

```bash
curl -sSL https://raw.githubusercontent.com/daringjoker/kitty-pane-bg/main/install.sh | sh
```

### Manual Installation

1. **Clone and build**:
   ```bash
   git clone https://github.com/daringjoker/kitty-pane-bg.git
   cd kitty-pane-bg
   cargo build --release
   ```

2. **Install binary**:
   ```bash
   cp target/release/kitty-pane-bg ~/.local/bin/
   ```

3. **Configure kitty** (add to `~/.config/kitty/kitty.conf`):
   ```
   allow_remote_control yes
   listen_on unix:/tmp/kitty
   ```

4. **Restart kitty** for configuration to take effect

## Usage

```bash
# Check environment setup
kitty-pane-bg check

# Generate background image
kitty-pane-bg generate

# Generate and set as kitty background
kitty-pane-bg set-background

# Install automatic tmux hooks
kitty-pane-bg install-hooks

# Manage color cache
kitty-pane-bg cache show
kitty-pane-bg cache clear
```

## Building from Source

```bash
# Clone repository
git clone https://github.com/daringjoker/kitty-pane-bg.git
cd kitty-pane-bg

# Build release binary
cargo build --release

# Run tests
cargo test
```

### Dependencies

- `serde` - JSON parsing
- `tokio` - Async runtime
- `clap` - CLI parsing
- `anyhow` - Error handling
- `image` - Image generation
- `rand` - Color generation

## License

Open source - see LICENSE file for details.
