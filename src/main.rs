mod color_cache;
mod image_gen;
mod kitty;
mod tmux;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use color_cache::ColorCache;
use image_gen::{generate_pane_image, generate_unique_filename};
use kitty::{
    check_kitty_setup, clear_kitty_background, get_kitty_window_info, set_kitty_background,
};
use tmux::{check_tmux_session, get_current_window_panes, install_tmux_hooks};

#[derive(Parser)]
#[command(name = "kitty-pane-bg")]
#[command(about = "Generate pane background images using kitty and tmux")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate background image based on current pane layout
    Generate {
        /// Output image path
        #[arg(short, long, default_value = "pane_bg.png")]
        output: String,
        /// Use all panes across sessions (default: current window only)
        #[arg(short, long)]
        all_panes: bool,
    },
    /// Generate and automatically set as kitty background
    SetBackground {
        /// Use all panes across sessions (default: current window only)
        #[arg(short, long)]
        all_panes: bool,
        /// Keep the generated image file (default: delete after setting)
        #[arg(long)]
        keep_file: bool,
    },
    /// Alias for set-background - quickly generate and set as kitty background
    Auto {
        /// Use all panes across sessions (default: current window only)
        #[arg(short, long)]
        all_panes: bool,
        /// Keep the generated image file (default: delete after setting)
        #[arg(long)]
        keep_file: bool,
    },
    /// Install tmux hooks
    InstallHooks,
    /// Check if running in tmux and kitty
    Check,
    /// Clear kitty background
    Clear,
    /// Manage color cache
    Cache {
        #[command(subcommand)]
        action: CacheCommands,
    },
}

#[derive(Subcommand)]
enum CacheCommands {
    /// Show current color cache
    Show,
    /// Clear all cached colors
    Clear,
    /// Remove specific pane color
    Remove { pane_id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { output, all_panes } => {
            // Check if we're in a tmux session
            if !check_tmux_session().await? {
                anyhow::bail!("Not running in a tmux session. Please start tmux first.");
            }

            println!("Getting kitty window information...");
            let window_dims = get_kitty_window_info().await?;
            println!(
                "Window dimensions: {}x{} (cell: {:.1}x{:.1})",
                window_dims.width,
                window_dims.height,
                window_dims.cell_width,
                window_dims.cell_height
            );

            println!("Getting tmux pane information...");
            let panes = if all_panes {
                tmux::get_tmux_panes().await?
            } else {
                get_current_window_panes().await?
            };

            println!("Found {} panes", panes.len());

            if panes.is_empty() {
                println!("No tmux panes found. Creating a solid color background.");
            }

            generate_pane_image(&window_dims, &panes, &output).await?;
        }
        Commands::SetBackground {
            all_panes,
            keep_file,
        } => {
            // Check if we're in a tmux session
            if !check_tmux_session().await? {
                anyhow::bail!("Not running in a tmux session. Please start tmux first.");
            }

            println!("🖼️  Generating background and setting as kitty background...");
            let temp_output = generate_unique_filename("/tmp/kitty-pane-bg-temp.png");

            println!("Getting kitty window information...");
            let window_dims = get_kitty_window_info().await?;
            println!(
                "Window dimensions: {}x{} (cell: {:.1}x{:.1})",
                window_dims.width,
                window_dims.height,
                window_dims.cell_width,
                window_dims.cell_height
            );

            println!("Getting tmux pane information...");
            let panes = if all_panes {
                tmux::get_tmux_panes().await?
            } else {
                get_current_window_panes().await?
            };

            println!("Found {} panes", panes.len());

            if panes.is_empty() {
                println!("No tmux panes found. Creating a solid background.");
            }

            generate_pane_image(&window_dims, &panes, &temp_output).await?;

            // Set as kitty background
            match set_kitty_background(&temp_output).await {
                Ok(()) => {
                    println!("🎨 Successfully set pane layout as kitty background!");
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to set kitty background: {}", e);
                    eprintln!("   The image was still generated at: {}", temp_output);
                    eprintln!("   You can manually set it or check kitty remote control setup.");
                }
            }

            // Clean up temp file unless requested to keep it
            if !keep_file {
                if let Err(e) = std::fs::remove_file(&temp_output) {
                    eprintln!("Warning: Failed to remove temp file {}: {}", temp_output, e);
                }
            } else {
                println!("📁 Keeping generated file: {}", temp_output);
            }
        }
        Commands::Auto {
            all_panes,
            keep_file,
        } => {
            // Check if we're in a tmux session
            if !check_tmux_session().await? {
                anyhow::bail!("Not running in a tmux session. Please start tmux first.");
            }

            println!("🚀 Auto mode: Generating background and setting as kitty background...");
            let temp_output = generate_unique_filename("/tmp/kitty-pane-bg-auto.png");

            println!("Getting kitty window information...");
            let window_dims = get_kitty_window_info().await?;
            println!(
                "Window dimensions: {}x{} (cell: {:.1}x{:.1})",
                window_dims.width,
                window_dims.height,
                window_dims.cell_width,
                window_dims.cell_height
            );

            println!("Getting tmux pane information...");
            let panes = if all_panes {
                tmux::get_tmux_panes().await?
            } else {
                get_current_window_panes().await?
            };

            println!("Found {} panes", panes.len());

            if panes.is_empty() {
                println!("No tmux panes found. Creating a solid background.");
            }

            generate_pane_image(&window_dims, &panes, &temp_output).await?;

            // Set as kitty background
            match set_kitty_background(&temp_output).await {
                Ok(()) => {
                    println!("🎨 Successfully set pane layout as kitty background!");
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to set kitty background: {}", e);
                    eprintln!("   The image was still generated at: {}", temp_output);
                    eprintln!("   You can manually set it or check kitty remote control setup.");
                }
            }

            // Clean up temp file unless requested to keep it
            if !keep_file {
                if let Err(e) = std::fs::remove_file(&temp_output) {
                    eprintln!("Warning: Failed to remove temp file {}: {}", temp_output, e);
                }
            } else {
                println!("📁 Keeping generated file: {}", temp_output);
            }
        }
        Commands::InstallHooks => {
            let program_path = std::env::current_exe()
                .context("Failed to get current executable path")?
                .display()
                .to_string();

            install_tmux_hooks(&program_path).await?;
        }
        Commands::Check => {
            println!("🔍 Checking environment...");
            println!();

            // Check tmux
            let in_tmux = check_tmux_session().await?;
            println!(
                "📋 Tmux session: {}",
                if in_tmux {
                    "✅ Found"
                } else {
                    "❌ Not found"
                }
            );

            // Check kitty with detailed setup info
            check_kitty_setup().await?;

            // Check if in tmux and can get panes
            if in_tmux {
                match get_current_window_panes().await {
                    Ok(panes) => println!("🔲 Tmux panes: ✅ Found {} panes", panes.len()),
                    Err(e) => println!("🔲 Tmux panes: ❌ Error ({})", e),
                }
            }

            println!();
            if in_tmux {
                println!("🎉 Environment is ready!");
                println!("📝 Next steps:");
                println!("   • Run 'kitty-pane-bg generate' to create a background image");
                println!("   • Run 'kitty-pane-bg set-background' to set it as kitty background");
                println!("   • Run 'kitty-pane-bg install-hooks' for automatic generation");
            } else {
                println!("⚠️  Please start a tmux session first:");
                println!("   tmux");
            }
        }
        Commands::Clear => {
            println!("🧹 Clearing kitty background...");

            match clear_kitty_background().await {
                Ok(()) => {
                    println!("✅ Successfully cleared kitty background");
                }
                Err(e) => {
                    eprintln!("❌ Failed to clear kitty background: {}", e);
                    eprintln!("   You may need to check kitty remote control setup");
                    std::process::exit(1);
                }
            }
        }
        Commands::Cache { action } => {
            match action {
                CacheCommands::Show => {
                    let cache = ColorCache::load().context("Failed to load color cache")?;
                    println!("Color Cache Information:");
                    println!("Cache file: {:?}", ColorCache::get_cache_path());
                    println!("Startup seed: {}", cache.startup_seed);
                    println!("Cached panes: {}", cache.colors.len());
                    // Remove opacity printout
                    if cache.colors.is_empty() {
                        println!("No colors cached yet.");
                    } else {
                        println!("\nCached pane colors:");
                        for (pane_id, cached_color) in &cache.colors {
                            println!(
                                "  {} -> RGB({}, {}, {}) Hue:{:.1}° (created: {})",
                                pane_id,
                                cached_color.rgb[0],
                                cached_color.rgb[1],
                                cached_color.rgb[2],
                                cached_color.hue,
                                cached_color.created_at
                            );
                        }
                    }
                }
                CacheCommands::Clear => {
                    let cache_path = ColorCache::get_cache_path();
                    if cache_path.exists() {
                        std::fs::remove_file(&cache_path).context("Failed to remove cache file")?;
                        println!("✅ Color cache cleared!");
                    } else {
                        println!("Color cache was already empty.");
                    }
                }
                CacheCommands::Remove { pane_id } => {
                    let mut cache = ColorCache::load().context("Failed to load color cache")?;
                    if cache.remove_pane(&pane_id) {
                        cache.save().context("Failed to save color cache")?;
                        println!("✅ Removed color for pane: {}", pane_id);
                    } else {
                        println!("Pane {} not found in cache.", pane_id);
                    }
                }
            }
        }
    }

    Ok(())
}
