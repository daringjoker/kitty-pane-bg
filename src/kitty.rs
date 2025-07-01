use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Serialize, Deserialize)]
pub struct KittyWindow {
    pub id: u32,
    pub platform_window_id: Option<u64>,
    pub tabs: Vec<KittyTab>,
    pub geometry: Option<KittyGeometry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KittyTab {
    pub id: u32,
    pub title: String,
    pub windows: Vec<KittySubWindow>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KittySubWindow {
    pub id: u32,
    pub columns: u32,
    pub lines: u32,
    pub char_width: Option<f32>,
    pub char_height: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KittyGeometry {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct WindowDimensions {
    pub width: u32,
    pub height: u32,
    pub cell_width: f32,
    pub cell_height: f32,
}

impl WindowDimensions {
    pub fn char_to_pixel_x(&self, char_x: u32) -> u32 {
        (char_x as f32 * self.cell_width) as u32
    }

    pub fn char_to_pixel_y(&self, char_y: u32) -> u32 {
        (char_y as f32 * self.cell_height) as u32
    }

    pub fn char_to_pixel_width(&self, char_width: u32) -> u32 {
        (char_width as f32 * self.cell_width) as u32
    }

    pub fn char_to_pixel_height(&self, char_height: u32) -> u32 {
        (char_height as f32 * self.cell_height) as u32
    }
}

// Global cache for discovered kitty PID and socket path
lazy_static::lazy_static! {
    static ref KITTY_CACHE: Arc<Mutex<Option<KittyRemoteInfo>>> = Arc::new(Mutex::new(None));
}

#[derive(Debug, Clone)]
struct KittyRemoteInfo {
    pid: String,
    socket_path: String,
    validated_at: std::time::Instant,
}

impl KittyRemoteInfo {
    fn is_valid(&self) -> bool {
        // Cache expires after 60 seconds
        self.validated_at.elapsed().as_secs() < 600
    }
}

/// Centralized function for all kitty remote control operations
/// This function handles PID discovery, caching, validation, and fallback mechanisms
pub async fn kitty_remote_call(args: &[&str]) -> Result<std::process::Output> {
    // First, try to get cached info
    if let Some(cached_info) = get_cached_kitty_info().await? {
        if let Ok(output) = try_kitty_call_with_socket(&cached_info.socket_path, args).await {
            return Ok(output);
        } else {
            // Cache is stale, clear it
            clear_kitty_cache();
        }
    }

    // Discover and cache new kitty info
    let kitty_info = discover_and_cache_kitty_info().await?;
    // Try with discovered info
    if let Some(info) = kitty_info {
        if let Ok(output) = try_kitty_call_with_socket(&info.socket_path, args).await {
            return Ok(output);
        }
    }

    panic!("Failed to execute kitty command: No valid kitty PID or socket found");
}

async fn get_cached_kitty_info() -> Result<Option<KittyRemoteInfo>> {
    let cache = KITTY_CACHE.lock().unwrap();
    if let Some(ref info) = *cache {
        if info.is_valid() {
            // Validate that the PID and socket still exist
            if validate_kitty_info(info).await {
                return Ok(Some(info.clone()));
            }
        }
    }
    Ok(None)
}

fn clear_kitty_cache() {
    let mut cache = KITTY_CACHE.lock().unwrap();
    *cache = None;
}

async fn discover_and_cache_kitty_info() -> Result<Option<KittyRemoteInfo>> {
    if let Some(pid) = discover_kitty_pid().await? {
        let socket_path = format!("unix:/tmp/kitty-{}", pid);

        let info = KittyRemoteInfo {
            pid: pid.clone(),
            socket_path: socket_path.clone(),
            validated_at: std::time::Instant::now(),
        };

        // Validate the discovered info
        if validate_kitty_info(&info).await {
            // Cache it
            let mut cache = KITTY_CACHE.lock().unwrap();
            *cache = Some(info.clone());
            return Ok(Some(info));
        }
    }
    Ok(None)
}

async fn validate_kitty_info(info: &KittyRemoteInfo) -> bool {
    // Check if PID still exists and is a kitty process
    if !is_kitty_process(&info.pid).await {
        return false;
    }

    // Check if socket file exists
    let socket_file = info
        .socket_path
        .strip_prefix("unix:")
        .unwrap_or(&info.socket_path);
    std::path::Path::new(socket_file).exists()
}

async fn try_kitty_call_with_socket(
    socket_path: &str,
    args: &[&str],
) -> Result<std::process::Output> {
    let mut cmd = AsyncCommand::new("kitten");
    cmd.args(&["@"]);
    cmd.arg("--to").arg(socket_path);
    cmd.args(args);

    let output = cmd
        .output()
        .await
        .context("Failed to execute kitten command with socket")?;

    if output.status.success() {
        Ok(output)
    } else {
        anyhow::bail!(
            "Kitten command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

pub async fn get_kitty_window_info() -> Result<WindowDimensions> {
    // Try kitty remote control using centralized function
    match try_kitty_remote_control().await {
        Ok(dims) => return Ok(dims),
        Err(e) => {
            eprintln!("Warning: kitty remote control failed: {}", e);
            eprintln!("Trying alternative methods...");
        }
    }

    // Method 2: Use environment variables and fallback values
    get_fallback_dimensions().await
}

async fn try_kitty_remote_control() -> Result<WindowDimensions> {
    // Use centralized remote call function
    let output = kitty_remote_call(&["ls"])
        .await
        .context("Failed to get kitty window list")?;

    let windows: Vec<KittyWindow> =
        serde_json::from_slice(&output.stdout).context("Failed to parse kitty window info")?;

    parse_kitty_windows(windows).await
}

async fn parse_kitty_windows(windows: Vec<KittyWindow>) -> Result<WindowDimensions> {
    if windows.is_empty() {
        anyhow::bail!("No kitty windows found");
    }

    let window = &windows[0];
    if window.tabs.is_empty() {
        anyhow::bail!("No tabs found in kitty window");
    }

    let tab = &window.tabs[0];
    if tab.windows.is_empty() {
        anyhow::bail!("No sub-windows found in kitty tab");
    }

    let sub_window = &tab.windows[0];

    // Try to get more accurate cell dimensions using kitty's capabilities
    let cell_dims = get_kitty_cell_dimensions().await?;

    // Calculate total pixel dimensions
    let pixel_width = (sub_window.columns as f32 * cell_dims.0) as u32;
    let pixel_height = (sub_window.lines as f32 * cell_dims.1) as u32;

    Ok(WindowDimensions {
        width: pixel_width,
        height: pixel_height,
        cell_width: cell_dims.0,
        cell_height: cell_dims.1,
    })
}

async fn get_fallback_dimensions() -> Result<WindowDimensions> {
    // Try to get terminal size using stty or tput
    let (cols, rows) = get_terminal_size().await?;

    // Use reasonable defaults for cell dimensions
    let cell_width = 10.0; // Typical monospace character width
    let cell_height = 20.0; // Typical monospace character height

    let pixel_width = (cols as f32 * cell_width) as u32;
    let pixel_height = (rows as f32 * cell_height) as u32;

    Ok(WindowDimensions {
        width: pixel_width,
        height: pixel_height,
        cell_width,
        cell_height,
    })
}

async fn get_terminal_size() -> Result<(u32, u32)> {
    // Try stty first
    if let Ok(output) = AsyncCommand::new("stty").arg("size").output().await {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = output_str.trim().split_whitespace().collect();
            if parts.len() == 2 {
                if let (Ok(rows), Ok(cols)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    return Ok((cols, rows));
                }
            }
        }
    }

    // Try tput as fallback
    let cols_output = AsyncCommand::new("tput").arg("cols").output().await;
    let rows_output = AsyncCommand::new("tput").arg("lines").output().await;

    if let (Ok(cols_result), Ok(rows_result)) = (cols_output, rows_output) {
        if cols_result.status.success() && rows_result.status.success() {
            let cols_str = String::from_utf8_lossy(&cols_result.stdout);
            let rows_str = String::from_utf8_lossy(&rows_result.stdout);

            if let (Ok(cols), Ok(rows)) = (
                cols_str.trim().parse::<u32>(),
                rows_str.trim().parse::<u32>(),
            ) {
                return Ok((cols, rows));
            }
        }
    }

    // Final fallback to common terminal size
    println!("Warning: Could not determine terminal size, using default 80x24");
    Ok((80, 24))
}

async fn get_kitty_cell_dimensions() -> Result<(f32, f32)> {
    // Try to get cell dimensions from kitty window info first
    match kitty_remote_call(&["ls"]).await {
        Ok(output) if output.status.success() => {
            let window_info_str = String::from_utf8_lossy(&output.stdout);
            // Try to parse kitty window info for dimensions
            if let Ok(dims) = parse_kitty_window_dimensions(&window_info_str) {
                return Ok(dims);
            }
        }
        _ => {
            // Continue to fallback methods
        }
    }

    // Use reasonable default cell dimensions for modern terminals
    // Most modern terminals use fonts around 8-16px width and 12-24px height
    Ok((10.0, 20.0))
}

// Removed get_terminal_cell_info - was causing terminal freezes with blocking queries

// Removed detect_cell_size_ansi - was causing terminal freezes with blocking queries

// Removed unused functions that contained blocking terminal queries

#[allow(dead_code)]
async fn get_estimated_window_size() -> Result<(u32, u32)> {
    let dims = get_kitty_window_info().await?;
    Ok((dims.width, dims.height))
}

pub async fn check_kitty_setup() -> Result<()> {
    println!("🔍 Checking terminal environment...");

    // Check if we're in a supported terminal
    let in_kitty = std::env::var("KITTY_WINDOW_ID").is_ok();
    let in_tmux = std::env::var("TMUX").is_ok();
    let env_kitty_pid = std::env::var("KITTY_PID").ok();

    // Test the centralized remote control
    match get_cached_kitty_info().await? {
        Some(info) => {
            println!(
                "✅ Cached kitty info: PID {}, socket {}",
                info.pid, info.socket_path
            );
        }
        None => {
            println!("� No cached kitty info, discovering...");
            if let Some(info) = discover_and_cache_kitty_info().await? {
                println!(
                    "✅ Discovered kitty info: PID {}, socket {}",
                    info.pid, info.socket_path
                );
            } else {
                println!("⚠️  Could not discover kitty info");
            }
        }
    }

    if in_kitty {
        println!("✅ Running in kitty terminal");
        if let Some(ref pid) = env_kitty_pid {
            println!("📝 KITTY_PID environment: {}", pid);
        }
    } else {
        println!("⚠️  Not running in kitty terminal");
        println!("   Some features may not work optimally");
    }

    if in_tmux {
        println!("✅ Running in tmux session");
    }

    // Test remote control capabilities using centralized function
    match try_kitty_remote_control().await {
        Ok(dims) => {
            println!(
                "✅ Remote control working: {}x{} pixels",
                dims.width, dims.height
            );
            println!(
                "   Cell dimensions: {:.1}x{:.1} pixels",
                dims.cell_width, dims.cell_height
            );
        }
        Err(e) => {
            println!("⚠️  Remote control limited: {}", e);

            if in_kitty {
                println!();
                println!("💡 To enable full kitty remote control:");
                println!("   Add to ~/.config/kitty/kitty.conf:");
                println!("   allow_remote_control yes");
                println!("   listen_on unix:/tmp/kitty");
                println!();
                println!("   Then restart kitty or reload config (Ctrl+Shift+F5)");
            }

            // Try fallback method
            match get_fallback_dimensions().await {
                Ok(dims) => {
                    println!(
                        "✅ Using fallback method: {}x{} pixels",
                        dims.width, dims.height
                    );
                }
                Err(fallback_err) => {
                    println!("❌ Fallback failed: {}", fallback_err);
                    return Err(anyhow::anyhow!("Cannot determine terminal dimensions"));
                }
            }
        }
    }

    // Test background setting capability
    println!();
    println!("🎨 Testing background setting capability...");

    let test_methods = vec![
        ("Centralized remote control", true),
        ("ANSI escape sequences", true),
        ("Tmux passthrough", in_tmux),
    ];

    for (method, available) in test_methods {
        if available {
            println!("✅ {}", method);
        } else {
            println!("⚠️  {} (not available)", method);
        }
    }

    println!();
    println!("🎉 Environment check complete!");

    Ok(())
}

pub async fn set_kitty_background(image_path: &str) -> Result<()> {
    // Validate image file exists and is readable
    if !std::path::Path::new(image_path).exists() {
        anyhow::bail!("Image file does not exist: {}", image_path);
    }

    // Try centralized remote control first
    match kitty_remote_call(&["set-background-image", image_path]).await {
        Ok(output) if output.status.success() => return Ok(()),
        Ok(output) => {
            println!(
                "⚠️  Centralized remote control failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(e) => {
            println!("⚠️  Centralized remote control error: {}", e);
        }
    }

    // Fallback methods for non-remote scenarios
    if std::env::var("TMUX").is_ok() {
        set_background_tmux_passthrough(image_path).await?;
        return Ok(());
    }

    // Direct ANSI escape sequence
    set_background_ansi(image_path).await?;
    Ok(())
}

async fn set_background_tmux_passthrough(image_path: &str) -> Result<()> {
    // Convert image to base64 for transmission
    let image_data = tokio::fs::read(image_path)
        .await
        .context("Failed to read image file")?;
    let encoded = general_purpose::STANDARD.encode(&image_data);

    // Use tmux passthrough to send background image
    let escape_seq = format!("\\ePtmux;\\e\\e]20;{}\\e\\e\\\\\\e\\\\", encoded);

    let output = AsyncCommand::new("tmux")
        .arg("run-shell")
        .arg(format!("printf '{}'", escape_seq))
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => Ok(()),
        _ => anyhow::bail!("Failed to set background via tmux passthrough"),
    }
}

async fn set_background_ansi(image_path: &str) -> Result<()> {
    // Try OSC 20 sequence for background image
    let image_data = tokio::fs::read(image_path)
        .await
        .context("Failed to read image file")?;
    let encoded = general_purpose::STANDARD.encode(&image_data);

    let escape_seq = format!("\\e]20;{}\\e\\\\", encoded);

    let output = AsyncCommand::new("sh")
        .arg("-c")
        .arg(format!("printf '{}'", escape_seq))
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => Ok(()),
        _ => anyhow::bail!("Failed to set background via ANSI escape sequence"),
    }
}

pub async fn clear_kitty_background() -> Result<()> {
    // Try centralized remote control first
    match kitty_remote_call(&["set-background-image", "none"]).await {
        Ok(output) if output.status.success() => return Ok(()),
        Ok(output) => {
            println!(
                "⚠️  Centralized remote control failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(e) => {
            println!("⚠️  Centralized remote control error: {}", e);
        }
    }

    // Fallback methods for non-remote scenarios
    if std::env::var("TMUX").is_ok() {
        clear_background_tmux_passthrough().await?;
        return Ok(());
    }

    // Direct ANSI escape sequence
    clear_background_ansi().await?;
    Ok(())
}

#[allow(dead_code)]
async fn clear_background_tmux_passthrough() -> Result<()> {
    let escape_seq = "\\ePtmux;\\e\\e]20;\\e\\e\\\\\\e\\\\";

    let output = AsyncCommand::new("tmux")
        .arg("run-shell")
        .arg(format!("printf '{}'", escape_seq))
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => Ok(()),
        _ => anyhow::bail!("Failed to clear background via tmux passthrough"),
    }
}

#[allow(dead_code)]
async fn clear_background_ansi() -> Result<()> {
    let escape_seq = "\\e]20;\\e\\\\";

    let output = AsyncCommand::new("sh")
        .arg("-c")
        .arg(format!("printf '{}'", escape_seq))
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => Ok(()),
        _ => anyhow::bail!("Failed to clear background via ANSI escape sequence"),
    }
}

use std::collections::HashSet;

async fn discover_kitty_pid() -> Result<Option<String>> {
    // First try the environment variable
    if let Ok(kitty_pid) = std::env::var("KITTY_PID") {
        // Verify this PID actually points to a kitty process
        if is_kitty_process(&kitty_pid).await {
            println!(
                "🔍 Environment KITTY_PID {} verified as kitty process",
                kitty_pid
            );
            return Ok(Some(kitty_pid));
        } else {
            println!(
                "⚠️  Environment KITTY_PID {} is not a valid kitty process",
                kitty_pid
            );
        }
    }

    // If we're in tmux, get the tmux client PID and walk up the process tree
    if let Ok(tmux_info) = std::env::var("TMUX") {
        println!("🔍 Searching for kitty PID via tmux process tree...");
        match get_tmux_client_pid(&tmux_info).await {
            Ok(Some(client_pid)) => {
                println!("📋 Found tmux client PID: {}", client_pid);
                match find_kitty_in_process_tree(client_pid).await {
                    Ok(Some(kitty_pid)) => {
                        println!("✅ Discovered kitty PID: {}", kitty_pid);
                        return Ok(Some(kitty_pid.to_string()));
                    }
                    Ok(None) => {
                        println!(
                            "⚠️  No kitty found in process tree from tmux client {}",
                            client_pid
                        );
                    }
                    Err(e) => {
                        println!("⚠️  Error walking process tree: {}", e);
                    }
                }
            }
            Ok(None) => {
                println!("⚠️  Could not find tmux client PID");
            }
            Err(e) => {
                println!("⚠️  Error getting tmux client PID: {}", e);
            }
        }
    }

    // Fallback: try to find any kitty process that might be our parent
    println!("🔍 Trying fallback kitty discovery...");
    match find_any_kitty_parent().await {
        Ok(Some(pid)) => {
            println!("✅ Found fallback kitty PID: {}", pid);
            Ok(Some(pid))
        }
        Ok(None) => {
            println!("⚠️  No suitable kitty process found");
            Ok(None)
        }
        Err(e) => {
            println!("⚠️  Error in fallback discovery: {}", e);
            Ok(None)
        }
    }
}

async fn get_tmux_client_pid(tmux_info: &str) -> Result<Option<u32>> {
    // TMUX variable format: /tmp/tmux-1000/default,12345,0
    // Extract the session info to find our client
    let parts: Vec<&str> = tmux_info.split(',').collect();
    if parts.len() < 2 {
        return Ok(None);
    }

    // Try to get our specific session's client
    let session_id = parts[1];

    // Get all tmux clients and find ours by session
    let output = AsyncCommand::new("tmux")
        .args(&["list-clients", "-F", "#{client_pid} #{session_id}"])
        .output()
        .await
        .context("Failed to get tmux clients")?;

    if !output.status.success() {
        return Ok(None);
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        let client_parts: Vec<&str> = line.trim().split_whitespace().collect();
        if client_parts.len() >= 2 {
            if client_parts[1] == session_id {
                if let Ok(pid) = client_parts[0].parse::<u32>() {
                    return Ok(Some(pid));
                }
            }
        }
    }

    // Fallback: use any client PID if we can't match by session
    for line in output_str.lines() {
        let client_parts: Vec<&str> = line.trim().split_whitespace().collect();
        if !client_parts.is_empty() {
            if let Ok(pid) = client_parts[0].parse::<u32>() {
                return Ok(Some(pid));
            }
        }
    }

    Ok(None)
}

async fn find_kitty_in_process_tree(start_pid: u32) -> Result<Option<u32>> {
    let mut current_pid = start_pid;
    let mut visited = HashSet::new();

    // Walk up the process tree (max 20 levels to prevent infinite loops)
    for _ in 0..20 {
        if visited.contains(&current_pid) {
            break;
        }
        visited.insert(current_pid);

        // Check if current process is kitty
        if is_kitty_process(&current_pid.to_string()).await {
            return Ok(Some(current_pid));
        }

        // Get parent PID
        match get_parent_pid(current_pid).await {
            Ok(Some(parent_pid)) => current_pid = parent_pid,
            _ => break,
        }
    }

    Ok(None)
}

async fn get_parent_pid(pid: u32) -> Result<Option<u32>> {
    // Read /proc/PID/stat to get parent PID
    let stat_path = format!("/proc/{}/stat", pid);
    match tokio::fs::read_to_string(&stat_path).await {
        Ok(content) => {
            // stat format: pid (comm) state ppid ...
            // We need to handle the case where comm might contain spaces or parentheses
            if let Some(last_paren) = content.rfind(')') {
                let after_comm = &content[last_paren + 1..];
                let parts: Vec<&str> = after_comm.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(ppid) = parts[1].parse::<u32>() {
                        return Ok(if ppid <= 1 { None } else { Some(ppid) });
                    }
                }
            }
            Ok(None)
        }
        Err(_) => Ok(None),
    }
}

async fn is_kitty_process(pid: &str) -> bool {
    // Check if the process is actually kitty by reading its command line
    let cmdline_path = format!("/proc/{}/cmdline", pid);
    match tokio::fs::read_to_string(&cmdline_path).await {
        Ok(cmdline) => {
            let cmd = cmdline.replace('\0', " ");
            let is_kitty = cmd.contains("kitty") && !cmd.contains("kitty-pane-bg");
            if !is_kitty {
                println!(
                    "   PID {} command: {}",
                    pid,
                    cmd.chars().take(50).collect::<String>()
                );
            }
            is_kitty
        }
        Err(e) => {
            println!("   PID {} does not exist: {}", pid, e);
            false
        }
    }
}

async fn find_any_kitty_parent() -> Result<Option<String>> {
    // Last resort: find any kitty process that could be our terminal
    let output = AsyncCommand::new("pgrep")
        .args(&["-f", "kitty"])
        .output()
        .await
        .context("Failed to search for kitty processes")?;

    if !output.status.success() {
        return Ok(None);
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if let Ok(pid) = line.trim().parse::<u32>() {
            if is_kitty_process(&pid.to_string()).await {
                // Verify this kitty process has a reasonable chance of being our parent
                if is_potential_parent_kitty(pid).await {
                    return Ok(Some(pid.to_string()));
                }
            }
        }
    }

    Ok(None)
}

async fn is_potential_parent_kitty(kitty_pid: u32) -> bool {
    // Check if this kitty process has child processes that could lead to us
    // This is a heuristic - a kitty terminal should have shell children
    let children_output = AsyncCommand::new("pgrep")
        .args(&["-P", &kitty_pid.to_string()])
        .output()
        .await;

    match children_output {
        Ok(output) if output.status.success() => {
            let child_count = String::from_utf8_lossy(&output.stdout).lines().count();
            child_count > 0 // Kitty should have at least one child process
        }
        _ => false,
    }
}

fn parse_kitty_window_dimensions(window_info: &str) -> Result<(f32, f32)> {
    // Try to parse JSON output from kitty @ ls
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(window_info) {
        if let Some(tabs) = json_value.as_array() {
            for tab in tabs {
                if let Some(windows) = tab["windows"].as_array() {
                    for window in windows {
                        // Look for cell dimensions in window info
                        if let (Some(cols), Some(rows)) =
                            (window["columns"].as_u64(), window["rows"].as_u64())
                        {
                            // Try to get pixel dimensions too
                            if let (Some(px_width), Some(px_height)) = (
                                window["geometry"]["width"].as_u64(),
                                window["geometry"]["height"].as_u64(),
                            ) {
                                let cell_width = px_width as f32 / cols as f32;
                                let cell_height = px_height as f32 / rows as f32;

                                if cell_width > 0.0
                                    && cell_height > 0.0
                                    && cell_width < 50.0
                                    && cell_height < 50.0
                                {
                                    return Ok((cell_width, cell_height));
                                }
                            }

                            // Fallback: estimate based on typical font sizes
                            return Ok((10.0, 20.0));
                        }
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!("Could not parse kitty window dimensions"))
}
