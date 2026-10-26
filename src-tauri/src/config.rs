use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub click_through: bool,
    pub display_mode: String, // "two-line", "single-line", "karaoke"
    pub font_size: u32,
    pub font_family: String,
    pub active_color: String,
    pub secondary_color: String,
    pub stroke_color: String,
    pub stroke_width: u32,
    pub text_align: String, // "center", "left", "right"
    pub save_to_audio_folder: bool,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            click_through: false,
            display_mode: "two-line".to_string(),
            font_size: 28,
            font_family: "Inter, system-ui, sans-serif".to_string(),
            active_color: "#38BDF8".to_string(),    // Sky 400
            secondary_color: "rgba(255, 255, 255, 0.65)".to_string(),
            stroke_color: "#000000".to_string(),
            stroke_width: 2,
            text_align: "center".to_string(),
            save_to_audio_folder: true,
            window_x: None,
            window_y: None,
            window_width: Some(840),
            window_height: Some(180),
        }
    }
}

pub fn get_app_dir() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let app_dir = base.join("aimp-floating-lyrics");
    let _ = fs::create_dir_all(&app_dir);
    app_dir
}

pub fn get_cache_dir() -> PathBuf {
    let cache_dir = get_app_dir().join("cache");
    let _ = fs::create_dir_all(&cache_dir);
    cache_dir
}

pub fn get_config_path() -> PathBuf {
    get_app_dir().join("config.json")
}

pub fn load_config() -> AppConfig {
    let path = get_config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                return cfg;
            }
        }
    }
    let default_cfg = AppConfig::default();
    let _ = save_config(&default_cfg);
    default_cfg
}

pub fn save_config(cfg: &AppConfig) -> Result<(), String> {
    let path = get_config_path();
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}
