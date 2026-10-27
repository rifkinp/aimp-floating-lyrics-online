use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppConfig {
    pub click_through: bool,
    pub display_mode: String, // "two-line", "single-line", "karaoke"
    pub font_size: u32,
    pub font_family: String,
    pub active_color: String,
    pub secondary_color: String,
    pub stroke_color: String,
    pub stroke_width: f64,
    pub text_align: String, // "center", "left", "right"
    pub save_to_audio_folder: bool,
    pub close_with_aimp: bool,
    pub lrclib_app_id: Option<String>,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub show_dual_lyrics: bool,
    pub dual_lyrics_type: String, // "translation" | "romaji"
    pub background_opacity: f64,  // 0.0 - 1.0
    pub enable_glassmorphism: bool,
    pub dock_position: String,    // "none" | "bottom-taskbar" | "top-screen"
    pub target_monitor: Option<usize>,
    pub enabled_sources: Vec<String>,
    pub source_priority: Vec<String>,
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
            stroke_width: 2.0,
            text_align: "center".to_string(),
            save_to_audio_folder: true,
            close_with_aimp: false,
            lrclib_app_id: None,
            window_x: None,
            window_y: None,
            window_width: Some(840),
            window_height: Some(180),
            show_dual_lyrics: true,
            dual_lyrics_type: "translation".to_string(),
            background_opacity: 0.15,
            enable_glassmorphism: true,
            dock_position: "none".to_string(),
            target_monitor: None,
            enabled_sources: vec![
                "local".to_string(),
                "lrclib".to_string(),
                "netease".to_string(),
                "lrcred".to_string(),
                "cache".to_string(),
            ],
            source_priority: vec![
                "local".to_string(),
                "lrclib".to_string(),
                "netease".to_string(),
                "lrcred".to_string(),
                "cache".to_string(),
            ],
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

pub fn get_lrclib_app_id() -> Option<String> {
    // 1. Check environment variable
    if let Ok(id) = std::env::var("LRCLIB_APP_ID") {
        let trimmed = id.trim().trim_matches('"').trim_matches('\'').to_string();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }

    // 2. Check .env in exe parent dir, current working dir, and appdata dir
    let mut possible_paths = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            possible_paths.push(parent.join(".env"));
        }
    }
    possible_paths.push(PathBuf::from(".env"));
    possible_paths.push(get_app_dir().join(".env"));

    for p in possible_paths {
        if p.exists() {
            if let Ok(content) = fs::read_to_string(&p) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("LRCLIB_APP_ID=") {
                        let val = trimmed.trim_start_matches("LRCLIB_APP_ID=").trim().trim_matches('"').trim_matches('\'');
                        if !val.is_empty() {
                            return Some(val.to_string());
                        }
                    }
                }
            }
        }
    }

    // 3. Check config.json
    let cfg = load_config();
    if let Some(id) = cfg.lrclib_app_id {
        let trimmed = id.trim().to_string();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }

    None
}
