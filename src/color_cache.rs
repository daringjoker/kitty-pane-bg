use anyhow::{Result, Context};
use image::Rgb;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachedColor {
    pub rgb: [u8; 3],
    pub hue: f32, // Store hue for better distinctness
    pub created_at: u64, // timestamp
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColorCache {
    pub colors: HashMap<String, CachedColor>,
    pub startup_seed: u64,
    pub used_hues: Vec<f32>, // Track used hues for better distribution
}

impl ColorCache {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            colors: HashMap::new(),
            startup_seed: rng.gen(),
            used_hues: Vec::new(),
        }
    }

    pub fn get_cache_path() -> PathBuf {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("kitty-pane-bg");
        
        if !cache_dir.exists() {
            let _ = fs::create_dir_all(&cache_dir);
        }
        
        cache_dir.join("pane_colors.json")
    }

    pub fn load() -> Result<Self> {
        let cache_path = Self::get_cache_path();
        
        if !cache_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&cache_path)
            .context("Failed to read color cache file")?;
        
        let mut cache: Self = serde_json::from_str(&content)
            .context("Failed to parse color cache")?;
        
        // Rebuild used_hues from existing colors
        cache.used_hues = cache.colors.values().map(|c| c.hue).collect();
        
        Ok(cache)
    }

    pub fn save(&self) -> Result<()> {
        let cache_path = Self::get_cache_path();
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize color cache")?;
        
        fs::write(&cache_path, content)
            .context("Failed to write color cache file")?;
        
        Ok(())
    }

    pub fn get_or_create_color(&mut self, color_key: &str) -> Rgb<u8> {
        if let Some(cached_color) = self.colors.get(color_key) {
            return Rgb(cached_color.rgb);
        }

        // Generate new color with maximum distinctness
        let (color, hue) = self.generate_distinct_color(color_key);
        let cached_color = CachedColor {
            rgb: color.0,
            hue,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        self.colors.insert(color_key.to_string(), cached_color);
        self.used_hues.push(hue);
        color
    }

    pub fn remove_pane(&mut self, pane_id: &str) -> bool {
        if let Some(cached_color) = self.colors.remove(pane_id) {
            // Remove the hue from used_hues
            self.used_hues.retain(|&h| (h - cached_color.hue).abs() > 1.0);
            true
        } else {
            false
        }
    }

    pub fn clean_missing_panes(&mut self, existing_color_keys: &[String]) {
        let existing_set: std::collections::HashSet<_> = existing_color_keys.iter().collect();
        let removed_colors: Vec<_> = self.colors.iter()
            .filter(|(color_key, _)| !existing_set.contains(color_key))
            .map(|(_, color)| color.hue)
            .collect();
        
        self.colors.retain(|color_key, _| existing_set.contains(color_key));
        
        // Clean up used_hues
        for hue in removed_colors {
            self.used_hues.retain(|&h| (h - hue).abs() > 1.0);
        }
    }

    fn generate_distinct_color(&self, pane_id: &str) -> (Rgb<u8>, f32) {
        // Use startup seed and pane ID for base randomness
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        
        self.startup_seed.hash(&mut hasher);
        pane_id.hash(&mut hasher);
        
        let hash_value = hasher.finish();
        
        // Find the most distinct hue
        let candidate_hue = (hash_value % 360) as f32;
        let hue = self.find_most_distinct_hue(candidate_hue);
        
        // Bright pastel colors: high lightness (75-85%), medium-high saturation (70-80%)
        let saturation = 0.70 + ((hash_value >> 8) % 11) as f32 / 100.0; // 70-80%
        let lightness = 0.75 + ((hash_value >> 16) % 11) as f32 / 100.0; // 75-85%
        
        (hsv_to_rgb(hue, saturation, lightness), hue)
    }

    fn find_most_distinct_hue(&self, preferred_hue: f32) -> f32 {
        if self.used_hues.is_empty() {
            return preferred_hue;
        }

        // Try the preferred hue first
        if self.is_hue_distinct(preferred_hue) {
            return preferred_hue;
        }

        // Find the hue with maximum minimum distance to all used hues
        let mut best_hue = preferred_hue;
        let mut best_min_distance = 0.0f32;

        // Check hues in steps around the color wheel
        for step in 0..36 {
            let test_hue = (preferred_hue + step as f32 * 10.0) % 360.0;
            let min_distance = self.used_hues.iter()
                .map(|&used_hue| hue_distance(test_hue, used_hue))
                .fold(360.0f32, f32::min);

            if min_distance > best_min_distance {
                best_min_distance = min_distance;
                best_hue = test_hue;
            }
        }

        best_hue
    }

    fn is_hue_distinct(&self, hue: f32) -> bool {
        const MIN_HUE_DISTANCE: f32 = 30.0; // Minimum 30 degrees apart
        
        self.used_hues.iter().all(|&used_hue| {
            hue_distance(hue, used_hue) >= MIN_HUE_DISTANCE
        })
    }

    #[allow(dead_code)]
    pub fn get_base_colors() -> Vec<Rgb<u8>> {
        vec![
            Rgb([255, 99, 71]),   // Tomato
            Rgb([60, 179, 113]),  // Medium Sea Green
            Rgb([100, 149, 237]), // Cornflower Blue
            Rgb([255, 165, 0]),   // Orange
            Rgb([147, 112, 219]), // Medium Purple
            Rgb([255, 20, 147]),  // Deep Pink
            Rgb([0, 206, 209]),   // Dark Turquoise
            Rgb([255, 215, 0]),   // Gold
        ]
    }

    #[allow(dead_code)]
    pub fn list_cached_panes(&self) -> Vec<String> {
        self.colors.keys().cloned().collect()
    }
}

// Calculate the minimum distance between two hues on the color wheel
fn hue_distance(hue1: f32, hue2: f32) -> f32 {
    let diff = (hue1 - hue2).abs();
    diff.min(360.0 - diff)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Rgb<u8> {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    Rgb([
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    ])
}
