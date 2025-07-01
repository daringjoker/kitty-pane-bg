# Kitty Pane Background Generator

A high-performance Rust CLI tool that generates beautiful pastel-colored background images matching your tmux/kitty pane layout. Each pane is rendered with a distinct color, and the tool automatically sets the generated image as your kitty terminal background.

## Features

- 🎨 **Pastel Color Generation**: Automatically generates distinct pastel colors for each pane
- 🔄 **Persistent Colors**: Colors are cached and persist across sessions
- 🖼️ **Automatic Background Setting**: Directly integrates with kitty's remote control protocol
- 🔧 **Robust Fallbacks**: Uses ANSI escape sequences or tmux passthrough when remote control isn't available
- ⚡ **High Performance**: Optimized with parallel processing for large images
- 🛡️ **Defensive Programming**: Comprehensive error handling and input validation
- 🎯 **tmux Integration**: Automatic hooks for real-time background updates
- 🌈 **Smart Color Distribution**: Ensures maximum visual distinctness between panes

- 🎨 **Visual Pane Mapping**: Each tmux pane gets a unique color in the generated image
- 🔄 **Automatic Updates**: Uses tmux hooks to regenerate images when panes are split, resized, or closed
- 🖥️ **Accurate Dimensions**: Queries kitty terminal for precise window and cell dimensions
- 🎯 **Active Pane Highlighting**: Shows the currently active pane with a white border
- 🌈 **Smart Color Generation**: Creates bright pastel colors with optimal distinctness
- 💾 **Color Persistence**: Colors are cached and persist across sessions - each pane keeps its color
- 🎭 **Configurable Opacity**: Set opacity from 0% to 100% via command line
- 🔑 **Deterministic Colors**: Uses startup seed for consistent color generation
- 🎨 **Optimal Color Distribution**: Ensures maximum visual distinctness between pane colors
- ✨ **Bright Pastel Palette**: 70-80% saturation, 75-85% lightness for appealing colors

## Prerequisites

- **Kitty Terminal**: Must be running in kitty terminal with remote control enabled
- **tmux**: Required for pane management and hooks
- **Rust**: For building the project (cargo)
- **FFmpeg**: (Optional) The original concept mentioned ffmpeg, but this implementation uses Rust's image crate for better integration

## Installation

### Quick Installation (Recommended)

Install with a single command using our installation script:

```bash
curl -sSL https://raw.githubusercontent.com/USERNAME/kitty-pane-bg/main/install.sh | sh
```

This script will:
- ✅ Check system requirements (tmux, kitty, curl)
- ⚙️ Install Rust if not already present
- 🔄 Try to download a pre-built binary for your platform
- 🔨 Build from source if no pre-built binary is available
- 📁 Install the binary to `~/.local/bin`
- 🛠️ Update your PATH automatically
- ⚙️ Configure kitty for remote control
- ✅ Run post-installation checks

After installation, restart your shell or run:
```bash
source ~/.bashrc  # or ~/.zshrc depending on your shell
```

### Manual Installation

If you prefer to install manually:

#### Prerequisites
- **Kitty Terminal**: Must be running in kitty with remote control enabled
- **tmux**: Required for pane management
- **Rust**: For building (installed automatically by quick installer)

#### Steps

1. **Clone the repository**:
   ```bash
   git clone https://github.com/USERNAME/kitty-pane-bg.git
   cd kitty-pane-bg
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Install the binary**:
   ```bash
   # Copy to a directory in your PATH
   cp target/release/kitty-pane-bg ~/.local/bin/
   
   # Or create a symlink
   ln -s "$(pwd)/target/release/kitty-pane-bg" ~/.local/bin/kitty-pane-bg
   ```

4. **Configure kitty** (add to `~/.config/kitty/kitty.conf`):
   ```
   allow_remote_control yes
   listen_on unix:/tmp/kitty
   ```

5. **Restart kitty** for the configuration to take effect

### Verify Installation

```bash
# Check if everything is working
kitty-pane-bg check

# Test basic functionality
kitty-pane-bg generate --output /tmp/test.png
```

## Usage

### Basic Commands

```bash
# Check if your environment is properly configured
kitty-pane-bg check

# Generate a background image for current tmux window (default 10% opacity)
kitty-pane-bg generate

# Generate with custom opacity (0.0 to 1.0)
kitty-pane-bg generate --opacity 0.3

# Generate with custom output path
kitty-pane-bg generate --output /path/to/background.png --opacity 0.2

# Include all panes across all tmux sessions
kitty-pane-bg generate --all-panes --opacity 0.15

# Generate and automatically set as kitty background
kitty-pane-bg set-background

# Quick auto mode (alias for set-background)
kitty-pane-bg auto

# Set background with custom opacity and keep file
kitty-pane-bg set-background --opacity 0.2 --keep-file

# Auto mode with all panes
kitty-pane-bg auto --all-panes --opacity 0.15

# Install tmux hooks for automatic generation
kitty-pane-bg install-hooks
```

### Color Cache Management

```bash
# Show current color cache
kitty-pane-bg cache show

# Clear all cached colors (forces new colors for all panes)
kitty-pane-bg cache clear

# Remove color for a specific pane
kitty-pane-bg cache remove %1
```

### Tmux Hooks

The program can install tmux hooks that automatically regenerate the background image when:
- **Pane events**: split, kill, exit, resize, focus
- **Window events**: create, switch, close, layout changes  
- **Session events**: create, switch between sessions
- **Focus events**: when switching between panes

Install hooks with:
```bash
kitty-pane-bg install-hooks
```

After installation, background images will be automatically generated at `/tmp/kitty-pane-bg.png` whenever the layout changes.

## How It Works

### 1. Window Dimension Detection
The program uses kitty's remote control protocol to query:
- Terminal window dimensions in characters
- Character cell dimensions (width/height in pixels)
- Current terminal geometry

```rust
// Example kitty query
kitty @ ls  // Gets window and tab information
```

### 2. Tmux Pane Information
Queries tmux for pane layout information:
- Pane positions (x, y coordinates in characters)
- Pane dimensions (width, height in characters)
- Active pane status

```bash
# Example tmux query
tmux list-panes -F "#{pane_id} #{pane_left} #{pane_top} #{pane_width} #{pane_height} #{pane_active}"
```

### 4. Color Cache System
- Generates a startup seed for deterministic color generation
- Caches colors per pane ID in `~/.cache/kitty-pane-bg/pane_colors.json`
- Stores opacity settings in the cache file
- Tracks hue values to ensure maximum color distinctness
- Automatically cleans up colors for closed panes
- Ensures new panes get maximally distinct colors

### 5. Image Generation
- Converts character coordinates to pixel coordinates
- Loads cached colors or generates new ones with optimal distinctness
- Creates bright pastel colors (70-80% saturation, 75-85% lightness)
- Renders each pane area with configurable opacity
- Highlights the active pane with a white border
- Saves the result as a PNG image with RGBA support

## Configuration

### Kitty Setup
For full functionality, enable kitty remote control. Run the setup script:
```bash
./setup-kitty.sh
```

Or manually add to your `kitty.conf`:
```
allow_remote_control yes
listen_on unix:/tmp/kitty
```

Then reload kitty config with `Ctrl+Shift+F5` or restart kitty.

**Note**: The program will work without remote control using fallback terminal size detection, but you'll get more accurate dimensions with remote control enabled.

### Color Customization
The program uses a predefined color palette for the first 8 panes, then generates random colors:
- Tomato Red
- Medium Sea Green  
- Cornflower Blue
- Orange
- Medium Purple
- Deep Pink
- Dark Turquoise
- Gold

## File Structure

```
kitty-pane-bg/
├── src/
│   ├── main.rs          # CLI interface and main logic
│   ├── kitty.rs         # Kitty remote control integration
│   ├── tmux.rs          # Tmux pane querying and hooks
│   ├── image_gen.rs     # Image generation with opacity support
│   └── color_cache.rs   # Color persistence and cache management
├── Cargo.toml           # Rust dependencies
├── setup.sh             # Installation script
└── README.md            # This file
```

## Dependencies

- **serde/serde_json**: For parsing kitty's JSON output and cache files
- **tokio**: Async runtime for process execution
- **clap**: Command-line argument parsing
- **anyhow**: Error handling
- **image**: Image creation and manipulation with RGBA support
- **rand**: Random color generation with deterministic seeding
- **dirs**: Cross-platform cache directory detection

## Troubleshooting

### "Not running in a tmux session"
Start tmux first:
```bash
tmux
```

### "kitty @ ls failed"
Ensure kitty remote control is enabled:
```bash
# Check if remote control is working
kitty @ ls
```

### "Failed to get kitty window info"
- Verify you're running in kitty terminal
- Check that remote control is enabled in kitty.conf
- Try restarting kitty

### Hooks not working
- Verify hooks are installed: `tmux show-hooks`
- Check that the binary path is correct
- Ensure the binary is executable

## Advanced Usage

### Custom Hook Commands
You can customize what happens when hooks trigger by modifying the tmux hooks:

```bash
# Example: Set background image as kitty background
tmux set-hook -g after-split-window "run-shell 'kitty-pane-bg generate --output /tmp/bg.png && kitty @ set-background-image /tmp/bg.png'"
```

### Integration with Window Managers
The generated images can be used with various tools:
- Set as desktop wallpaper
- Use with kitty's background image feature
- Integrate with tiling window managers

## Contributing

Feel free to submit issues, feature requests, or pull requests. Some ideas for improvements:

- Support for other terminals (alacritty, wezterm)
- Different color schemes and themes
- Animation support for smooth transitions
- Integration with other multiplexers (screen, zellij)

## License

This project is open source. Please check the license file for details.
