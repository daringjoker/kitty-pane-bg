use anyhow::{Context, Result};
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Clone)]
pub struct TmuxPane {
    pub id: String,
    pub window_id: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    #[allow(dead_code)]
    pub active: bool,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct TmuxSession {
    pub name: String,
    pub windows: Vec<TmuxWindow>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct TmuxWindow {
    pub id: String,
    pub name: String,
    pub panes: Vec<TmuxPane>,
}

pub async fn get_tmux_panes() -> Result<Vec<TmuxPane>> {
    let output = AsyncCommand::new("tmux")
        .args(&[
            "list-panes",
            "-a",
            "-F",
            "#{pane_id} #{window_id} #{pane_left} #{pane_top} #{pane_width} #{pane_height} #{pane_active}"
        ])
        .output()
        .await
        .context("Failed to execute tmux list-panes")?;

    if !output.status.success() {
        anyhow::bail!(
            "tmux list-panes failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output_str =
        String::from_utf8(output.stdout).context("Failed to parse tmux output as UTF-8")?;

    let mut panes = Vec::new();
    for line in output_str.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 7 {
            continue;
        }

        let pane = TmuxPane {
            id: parts[0].to_string(),
            window_id: parts[1].to_string(),
            x: parts[2]
                .parse()
                .context("Failed to parse pane x position")?,
            y: parts[3]
                .parse()
                .context("Failed to parse pane y position")?,
            width: parts[4].parse().context("Failed to parse pane width")?,
            height: parts[5].parse().context("Failed to parse pane height")?,
            active: parts[6] == "1",
        };

        panes.push(pane);
    }

    Ok(panes)
}

pub async fn get_current_window_panes() -> Result<Vec<TmuxPane>> {
    let output = AsyncCommand::new("tmux")
        .args(&[
            "list-panes",
            "-F",
            "#{pane_id} #{window_id} #{pane_left} #{pane_top} #{pane_width} #{pane_height} #{pane_active}"
        ])
        .output()
        .await
        .context("Failed to execute tmux list-panes for current window")?;

    if !output.status.success() {
        anyhow::bail!(
            "tmux list-panes failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output_str =
        String::from_utf8(output.stdout).context("Failed to parse tmux output as UTF-8")?;

    let mut panes = Vec::new();
    for line in output_str.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 7 {
            continue;
        }

        let pane = TmuxPane {
            id: parts[0].to_string(),
            window_id: parts[1].to_string(),
            x: parts[2]
                .parse()
                .context("Failed to parse pane x position")?,
            y: parts[3]
                .parse()
                .context("Failed to parse pane y position")?,
            width: parts[4].parse().context("Failed to parse pane width")?,
            height: parts[5].parse().context("Failed to parse pane height")?,
            active: parts[6] == "1",
        };

        panes.push(pane);
    }

    Ok(panes)
}

pub async fn install_tmux_hooks(program_path: &str) -> Result<()> {
    let hook_command = format!(
        "run-shell '{} set-background >/dev/null 2>&1'",
        program_path
    );

    // Install hooks for pane events
    let hooks = [
        // Pane lifecycle events
        ("after-split-window", &hook_command),
        ("pane-exited", &hook_command),
        ("after-resize-pane", &hook_command),
        // Layout and window events
        ("window-layout-changed", &hook_command),
        ("after-select-window", &hook_command),
        ("after-new-window", &hook_command),
        ("after-kill-window", &hook_command),
        // Session events
        ("after-new-session", &hook_command),
        ("session-window-changed", &hook_command),
        ("client-session-changed", &hook_command),
        // Pane focus events
        ("after-select-pane", &hook_command),
    ];

    let mut installed_count = 0;
    let mut failed_count = 0;

    for (hook_name, command) in hooks {
        let output = AsyncCommand::new("tmux")
            .args(&["set-hook", "-g", hook_name, command])
            .output()
            .await
            .context(format!("Failed to set tmux hook: {}", hook_name))?;

        if !output.status.success() {
            eprintln!(
                "Warning: Failed to set hook {}: {}",
                hook_name,
                String::from_utf8_lossy(&output.stderr)
            );
            failed_count += 1;
        } else {
            println!("✅ Installed tmux hook: {}", hook_name);
            installed_count += 1;
        }
    }

    println!();
    println!("🎉 Tmux hooks installation complete!");
    println!("   ✅ Successfully installed: {}", installed_count);
    if failed_count > 0 {
        println!("   ⚠️  Failed to install: {}", failed_count);
    }
    println!();
    println!("Background images will be automatically generated when:");
    println!("  🔲 Panes are split, killed, or resized");
    println!("  🪟 Windows are created, switched, or closed");
    println!("  📋 Sessions are created or switched");
    println!("  🎯 Panes are focused");
    println!();

    println!("🎨 Auto-background mode: Images will be automatically set as kitty background");
    println!("   Requires kitty remote control to be enabled");

    println!("💾 Colors will be cached and persist across operations");
    Ok(())
}

pub async fn check_tmux_session() -> Result<bool> {
    let output = AsyncCommand::new("tmux")
        .args(&["display-message", "-p", "#{session_name}"])
        .output()
        .await;

    match output {
        Ok(result) => Ok(result.status.success()),
        Err(_) => Ok(false),
    }
}
