use crate::color_cache::ColorCache;
use crate::kitty::WindowDimensions;
use crate::tmux::TmuxPane;
use anyhow::{Context, Result};
use image::{Rgb, RgbImage};

pub async fn generate_pane_image(
    window_dims: &WindowDimensions,
    panes: &[TmuxPane],
    output_path: &str,
) -> Result<()> {
    // Input validation
    if window_dims.width == 0 || window_dims.height == 0 {
        anyhow::bail!(
            "Invalid window dimensions: {}x{}",
            window_dims.width,
            window_dims.height
        );
    }

    if window_dims.width > 32768 || window_dims.height > 32768 {
        anyhow::bail!(
            "Window dimensions too large: {}x{}",
            window_dims.width,
            window_dims.height
        );
    }

    if panes.len() > 1000 {
        anyhow::bail!("Too many panes: {}", panes.len());
    }

    // Validate output path
    if let Some(parent) = std::path::Path::new(output_path).parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create output directory")?;
        }
    }

    // Load color cache with error handling
    let mut color_cache = ColorCache::load().context("Failed to load color cache")?;

    // Clean up colors for panes that no longer exist
    let current_pane_keys: Vec<String> = panes
        .iter()
        .map(|p| format!("{}:{}", sanitize_id(&p.window_id), sanitize_id(&p.id)))
        .collect();
    color_cache.clean_missing_panes(&current_pane_keys);

    // Create image buffer with bounds checking
    let mut image = RgbImage::new(window_dims.width, window_dims.height);

    // Fill with background color
    fill_background(&mut image);

    if panes.is_empty() {
        save_image_safely(&image, output_path).await?;
        return Ok(());
    }

    // Draw panes with optimized rendering
    let successful_draws = draw_panes(&mut image, window_dims, panes, &mut color_cache).await?;

    if successful_draws == 0 && !panes.is_empty() {
        anyhow::bail!("Failed to draw any panes");
    }

    // Save color cache with error handling
    if let Err(e) = color_cache.save() {
        eprintln!("Warning: Failed to save color cache: {}", e);
    }

    // Save image with validation
    save_image_safely(&image, output_path).await?;

    Ok(())
}

fn fill_background(image: &mut RgbImage) {
    let background_color = Rgb([20, 20, 20]);

    // Use parallel processing for large images
    if image.width() * image.height() > 1_000_000 {
        use rayon::prelude::*;
        // Use parallel processing for large images
        let pixels: Vec<_> = image.pixels_mut().collect();
        pixels.into_par_iter().for_each(|pixel| {
            *pixel = background_color;
        });
    } else {
        // Sequential for smaller images
        for pixel in image.pixels_mut() {
            *pixel = background_color;
        }
    }
}

async fn draw_panes(
    image: &mut RgbImage,
    window_dims: &WindowDimensions,
    panes: &[TmuxPane],
    color_cache: &mut ColorCache,
) -> Result<usize> {
    let mut successful_draws = 0;

    for pane in panes {
        match draw_single_pane(image, window_dims, pane, color_cache) {
            Ok(()) => successful_draws += 1,
            Err(e) => {
                eprintln!("Warning: Failed to draw pane {}: {}", pane.id, e);
            }
        }
    }

    Ok(successful_draws)
}

fn draw_single_pane(
    image: &mut RgbImage,
    window_dims: &WindowDimensions,
    pane: &TmuxPane,
    color_cache: &mut ColorCache,
) -> Result<()> {
    // Validate pane dimensions
    if pane.width == 0 || pane.height == 0 {
        return Err(anyhow::anyhow!(
            "Invalid pane dimensions: {}x{}",
            pane.width,
            pane.height
        ));
    }

    let color_key = format!("{}:{}", sanitize_id(&pane.window_id), sanitize_id(&pane.id));
    let rgb_color = color_cache.get_or_create_color(&color_key);

    // Convert coordinates with bounds checking
    let pixel_x = window_dims.char_to_pixel_x(pane.x);
    let pixel_y = window_dims.char_to_pixel_y(pane.y);
    let pixel_width = window_dims.char_to_pixel_width(pane.width);
    let pixel_height = window_dims.char_to_pixel_height(pane.height);

    // Ensure we don't go out of bounds
    let end_x = std::cmp::min(pixel_x.saturating_add(pixel_width), window_dims.width);
    let end_y = std::cmp::min(pixel_y.saturating_add(pixel_height), window_dims.height);

    // Skip if pane is completely outside image bounds
    if pixel_x >= window_dims.width || pixel_y >= window_dims.height {
        eprintln!(
            "Debug: Pane {} at {}x{} ({}x{}) is outside image bounds {}x{}",
            pane.id,
            pixel_x,
            pixel_y,
            pixel_width,
            pixel_height,
            window_dims.width,
            window_dims.height
        );
        return Ok(()); // Don't treat this as an error - some panes might be off-screen
    }

    // Skip panes with zero dimensions
    if end_x <= pixel_x || end_y <= pixel_y {
        eprintln!(
            "Debug: Pane {} has zero or negative dimensions after bounds checking",
            pane.id
        );
        return Ok(());
    }

    // Draw the pane area with bounds checking
    for y in pixel_y..end_y {
        for x in pixel_x..end_x {
            if x < window_dims.width && y < window_dims.height {
                if let Some(pixel) = image.get_pixel_mut_checked(x, y) {
                    *pixel = rgb_color;
                }
            }
        }
    }

    Ok(())
}

async fn save_image_safely(image: &RgbImage, output_path: &str) -> Result<()> {
    // Validate file extension
    let path = std::path::Path::new(output_path);
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    if !matches!(
        extension.to_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "bmp" | "tiff"
    ) {
        anyhow::bail!("Unsupported image format: {}", extension);
    }

    // Create a temporary file first for atomic write
    let path = std::path::Path::new(output_path);
    let temp_path = if let Some(parent) = path.parent() {
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let extension = path.extension().unwrap_or_default().to_string_lossy();
        parent.join(format!("{}.tmp.{}", stem, extension))
    } else {
        std::path::PathBuf::from(format!("{}.tmp", output_path))
    };

    // Save to temporary file
    let temp_path_str = temp_path.to_string_lossy().to_string();
    tokio::task::spawn_blocking({
        let image = image.clone();
        let temp_path = temp_path_str.clone();
        move || image.save(&temp_path)
    })
    .await
    .context("Task join error")?
    .context("Failed to save image to temporary file")?;

    // Atomic rename
    tokio::fs::rename(&temp_path_str, output_path)
        .await
        .context("Failed to rename temporary file")?;

    // Verify file was created and has reasonable size
    let metadata = tokio::fs::metadata(output_path)
        .await
        .context("Failed to verify created file")?;

    if metadata.len() == 0 {
        anyhow::bail!("Generated image file is empty");
    }

    if metadata.len() > 100_000_000 {
        // 100MB limit
        anyhow::bail!(
            "Generated image file is too large: {} bytes",
            metadata.len()
        );
    }

    Ok(())
}

fn sanitize_id(id: &str) -> String {
    // Remove potentially problematic characters
    id.chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '%' | '@' | '-' | '_'))
        .take(50) // Limit length
        .collect()
}

pub fn generate_unique_filename(base_path: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let process_id = std::process::id();

    let path = std::path::Path::new(base_path);
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let extension = path.extension().unwrap_or_default().to_string_lossy();
    let dir = path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("/tmp"));

    dir.join(format!(
        "{}-{}-{}.{}",
        stem, process_id, timestamp, extension
    ))
    .to_string_lossy()
    .to_string()
}
