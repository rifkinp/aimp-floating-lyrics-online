pub mod aimp;
pub mod config;
pub mod lrclib;
pub mod parser;

use aimp::{start_aimp_polling_task, AimpReader, AimpTrackInfo};
use config::{load_config, save_config, AppConfig};
use lrclib::{LrcManager, LrclibRecord};
use parser::{parse_lrc, update_lrc_offset, ParsedLyrics};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Listener, Manager, State};

struct AppState {
    lrc_manager: LrcManager,
    _running: Arc<AtomicBool>,
}

#[tauri::command]
fn get_current_track() -> Option<AimpTrackInfo> {
    AimpReader::get_current_track()
}

#[tauri::command]
fn get_app_config() -> AppConfig {
    load_config()
}

#[tauri::command]
fn save_app_config(app: AppHandle, mut config: AppConfig) -> Result<(), String> {
    let existing = load_config();
    if config.window_x.is_none() {
        config.window_x = existing.window_x;
    }
    if config.window_y.is_none() {
        config.window_y = existing.window_y;
    }
    if config.window_width.is_none() {
        config.window_width = existing.window_width;
    }
    if config.window_height.is_none() {
        config.window_height = existing.window_height;
    }

    save_config(&config)?;
    let _ = app.emit("config://updated", &config);
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.emit("config://updated", &config);
    }
    Ok(())
}

#[tauri::command]
fn open_settings_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
    Ok(())
}

#[tauri::command]
fn open_search_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("search") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
    Ok(())
}

#[tauri::command]
fn open_editor_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("editor") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
    Ok(())
}

#[tauri::command]
fn clear_cache() -> Result<usize, String> {
    let cache_dir = config::get_cache_dir();
    let mut count = 0;
    if cache_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if std::fs::remove_file(path).is_ok() {
                        count += 1;
                    }
                }
            }
        }
    }
    Ok(count)
}

#[tauri::command]
fn hide_window(app: AppHandle, label: Option<String>) -> Result<(), String> {
    let lbl = label.as_deref().unwrap_or("main");
    if let Some(window) = app.get_webview_window(lbl) {
        window.hide().map_err(|e| e.to_string())?;
        if lbl == "main" {
            if let Some(u) = app.get_webview_window("unlock") {
                let _ = u.hide();
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn minimize_window(app: AppHandle, label: Option<String>) -> Result<(), String> {
    let lbl = label.as_deref().unwrap_or("main");
    if let Some(window) = app.get_webview_window(lbl) {
        window.minimize().map_err(|e| e.to_string())?;
        if lbl == "main" {
            if let Some(u) = app.get_webview_window("unlock") {
                let _ = u.hide();
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_always_on_top(true);
        let _ = window.set_focus();

        let cfg = load_config();
        if cfg.click_through {
            if let Some(unlock_win) = app.get_webview_window("unlock") {
                if let (Ok(main_pos), Ok(main_size)) = (window.outer_position(), window.outer_size()) {
                    let badge_x = main_pos.x + main_size.width as i32 - 44;
                    let badge_y = main_pos.y + 8;
                    let _ = unlock_win.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: badge_x, y: badge_y }));
                    let _ = unlock_win.show();
                    let _ = unlock_win.set_always_on_top(true);
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn toggle_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
            if let Some(unlock_win) = app.get_webview_window("unlock") {
                let _ = unlock_win.hide();
            }
        } else {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_always_on_top(true);
            let _ = window.set_focus();

            let cfg = load_config();
            if cfg.click_through {
                if let Some(unlock_win) = app.get_webview_window("unlock") {
                    if let (Ok(main_pos), Ok(main_size)) = (window.outer_position(), window.outer_size()) {
                        let badge_x = main_pos.x + main_size.width as i32 - 44;
                        let badge_y = main_pos.y + 8;
                        let _ = unlock_win.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: badge_x, y: badge_y }));
                        let _ = unlock_win.show();
                        let _ = unlock_win.set_always_on_top(true);
                    }
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn set_click_through(app: AppHandle, locked: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_ignore_cursor_events(locked)
            .map_err(|e| e.to_string())?;

        let mut cfg = load_config();
        cfg.click_through = locked;
        let _ = save_config(&cfg);

        let _ = app.emit("lyrics://lock-changed", locked);

        // Manage floating unlock badge window
        if let Some(unlock_win) = app.get_webview_window("unlock") {
            if locked {
                if let (Ok(main_pos), Ok(main_size)) = (window.outer_position(), window.outer_size()) {
                    let badge_x = main_pos.x + main_size.width as i32 - 44;
                    let badge_y = main_pos.y + 8;
                    let _ = unlock_win.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: badge_x, y: badge_y }));
                    let _ = unlock_win.show();
                    let _ = unlock_win.set_always_on_top(true);
                }
            } else {
                let _ = unlock_win.hide();
            }
        }
    }
    Ok(())
}

#[tauri::command]
async fn resolve_lyrics(
    state: State<'_, AppState>,
    audio_path: String,
    artist: String,
    title: String,
    album: String,
    duration_ms: u32,
) -> Result<ParsedLyrics, String> {
    state
        .lrc_manager
        .resolve_lyrics(&audio_path, &artist, &title, &album, duration_ms)
        .await
}

#[tauri::command]
async fn search_lyrics_online(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<LrclibRecord>, String> {
    state.lrc_manager.manual_search_query(&query).await
}

#[tauri::command]
fn apply_lyrics(
    app: AppHandle,
    audio_path: String,
    artist: String,
    title: String,
    raw_lrc: String,
) -> Result<ParsedLyrics, String> {
    let parsed = parse_lrc(&raw_lrc);
    let _ = LrcManager::save_custom_lrc(&audio_path, &artist, &title, &raw_lrc);
    let _ = app.emit("lyrics://loaded", &parsed);
    Ok(parsed)
}

#[tauri::command]
fn adjust_offset(
    audio_path: String,
    artist: String,
    title: String,
    current_raw_lrc: String,
    delta_ms: i64,
) -> Result<ParsedLyrics, String> {
    let (updated_lrc, _new_offset) = update_lrc_offset(&current_raw_lrc, delta_ms);
    let _ = LrcManager::save_custom_lrc(&audio_path, &artist, &title, &updated_lrc);
    let parsed = parse_lrc(&updated_lrc);
    Ok(parsed)
}

fn to_base64(bytes: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARSET[(n >> 18) & 63] as char);
        out.push(CHARSET[(n >> 12) & 63] as char);
        if chunk.len() > 1 {
            out.push(CHARSET[(n >> 6) & 63] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARSET[n & 63] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[tauri::command]
fn get_album_art(file_path: String) -> Option<String> {
    if file_path.trim().is_empty() {
        return None;
    }
    let p = std::path::Path::new(&file_path);
    let parent = p.parent()?;
    let stem = p.file_stem()?.to_string_lossy().to_string();

    let candidates = [
        format!("{}.jpg", stem),
        format!("{}.png", stem),
        "cover.jpg".to_string(),
        "cover.png".to_string(),
        "cover.jpeg".to_string(),
        "folder.jpg".to_string(),
        "folder.png".to_string(),
        "albumart.jpg".to_string(),
        "front.jpg".to_string(),
        "front.png".to_string(),
    ];

    for name in &candidates {
        let candidate_path = parent.join(name);
        if candidate_path.exists() && candidate_path.is_file() {
            if let Ok(bytes) = std::fs::read(&candidate_path) {
                if bytes.len() <= 10 * 1024 * 1024 {
                    let mime = if name.ends_with(".png") {
                        "image/png"
                    } else if name.ends_with(".webp") {
                        "image/webp"
                    } else {
                        "image/jpeg"
                    };
                    return Some(format!("data:{};base64,{}", mime, to_base64(&bytes)));
                }
            }
        }
    }
    None
}

#[tauri::command]
fn control_playback(command: String) -> Result<(), String> {
    aimp::send_aimp_command(&command)
}

#[tauri::command]
fn set_aimp_volume(volume: u32) -> Result<(), String> {
    aimp::set_aimp_volume(volume)
}

#[tauri::command]
fn save_edited_lyrics(app: AppHandle, audio_path: String, raw_lrc: String) -> Result<ParsedLyrics, String> {
    let parsed = parse_lrc(&raw_lrc);

    if !audio_path.trim().is_empty() {
        let p = std::path::Path::new(&audio_path);
        if let Some(parent) = p.parent() {
            if let Some(stem) = p.file_stem() {
                let local_lrc_path = parent.join(format!("{}.lrc", stem.to_string_lossy()));
                let _ = std::fs::write(&local_lrc_path, &raw_lrc);
            }
        }
    }

    let cache_dir = crate::config::get_cache_dir();
    let sanitized_name = format!(
        "{}_{}.lrc",
        parsed.artist.as_deref().unwrap_or("unknown").replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_"),
        parsed.title.as_deref().unwrap_or("track").replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
    );
    let _ = std::fs::write(cache_dir.join(sanitized_name), &raw_lrc);

    let _ = app.emit("lyrics://loaded", &parsed);
    Ok(parsed)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
    pub is_primary: bool,
}

#[tauri::command]
fn get_available_monitors(app: AppHandle) -> Vec<MonitorInfo> {
    let mut list = Vec::new();
    if let Ok(monitors) = app.available_monitors() {
        for (i, m) in monitors.into_iter().enumerate() {
            let size = m.size();
            let name = m.name().cloned().unwrap_or_else(|| format!("Display {}", i + 1));
            list.push(MonitorInfo {
                index: i,
                name,
                width: size.width,
                height: size.height,
                scale_factor: m.scale_factor(),
                is_primary: i == 0,
            });
        }
    }
    list
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn seek_track(position_ms: u32) -> Result<(), String> {
    aimp::seek_track(position_ms)
}

#[tauri::command]
fn dock_window(app: AppHandle, position: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let cfg_current = load_config();
        let target_mon_opt = if let Some(idx) = cfg_current.target_monitor {
            app.available_monitors().ok().and_then(|m_list| m_list.into_iter().nth(idx))
        } else {
            None
        };

        let monitor = target_mon_opt
            .or_else(|| window.current_monitor().ok().flatten())
            .or_else(|| app.primary_monitor().ok().flatten());

        if let Some(mon) = monitor {
            let monitor_size = mon.size();
            let mon_pos = mon.position();
            let scale_factor = mon.scale_factor();
            let screen_w = monitor_size.width as f64;
            let screen_h = monitor_size.height as f64;

            let cur_size = window.outer_size().map_err(|e| e.to_string())?;
            let win_w = cur_size.width as f64;
            let win_h = cur_size.height as f64;

            let rel_x = ((screen_w - win_w) / 2.0).round() as i32;
            let rel_y = match position.as_str() {
                "bottom-taskbar" => {
                    let taskbar_h = 56.0 * scale_factor;
                    let pad = 12.0 * scale_factor;
                    ((screen_h - taskbar_h - win_h - pad).max(0.0)).round() as i32
                }
                "top-screen" => (24.0 * scale_factor).round() as i32,
                "bottom-screen" => ((screen_h - win_h - 16.0 * scale_factor).max(0.0)).round() as i32,
                _ => return Ok(()),
            };

            let new_x = mon_pos.x + rel_x;
            let new_y = mon_pos.y + rel_y;

            let pos = tauri::Position::Physical(tauri::PhysicalPosition { x: new_x, y: new_y });
            window.set_position(pos).map_err(|e| e.to_string())?;

            let mut cfg = load_config();
            cfg.dock_position = position;
            cfg.window_x = Some(new_x);
            cfg.window_y = Some(new_y);
            let _ = save_config(&cfg);
            let _ = app.emit("config://updated", &cfg);

            if cfg.click_through {
                if let Some(unlock_win) = app.get_webview_window("unlock") {
                    let badge_x = new_x + win_w as i32 - 44;
                    let badge_y = new_y + 8;
                    let _ = unlock_win.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: badge_x, y: badge_y }));
                }
            }
        }
    }
    Ok(())
}

fn start_global_hotkey_listener(app: AppHandle) {
    std::thread::Builder::new()
        .name("global-hotkey".into())
        .spawn(move || unsafe {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::Input::KeyboardAndMouse::{
                RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL,
            };
            use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

            const HOTKEY_ID: i32 = 0x51A0;
            // 0x4C is 'L'
            let registered = RegisterHotKey(
                HWND::default(),
                HOTKEY_ID,
                HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0),
                0x4C,
            );

            if registered.is_ok() {
                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                    if msg.message == WM_HOTKEY && msg.wParam.0 as i32 == HOTKEY_ID {
                        let cfg = load_config();
                        let new_locked = !cfg.click_through;
                        let _ = set_click_through(app.clone(), new_locked);
                    }
                }
                let _ = UnregisterHotKey(HWND::default(), HOTKEY_ID);
            }
        })
        .expect("failed to spawn global hotkey thread");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            lrc_manager: LrcManager::new(),
            _running: running.clone(),
        })
        .setup(move |app| {
            let handle = app.handle().clone();

            // Setup System Tray
            let toggle_show_item = MenuItem::with_id(
                app,
                "toggle_show",
                "Show / Hide Floating Lyrics",
                true,
                None::<&str>,
            )?;
            let lock_item = MenuItem::with_id(
                app,
                "lock_toggle",
                "Lock / Click-Through (Toggle)",
                true,
                None::<&str>,
            )?;
            let search_item = MenuItem::with_id(
                app,
                "manual_search",
                "Search Lyrics Manually",
                true,
                None::<&str>,
            )?;
            let settings_item =
                MenuItem::with_id(app, "settings", "Open Settings", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[
                    &toggle_show_item,
                    &PredefinedMenuItem::separator(app)?,
                    &lock_item,
                    &search_item,
                    &settings_item,
                    &PredefinedMenuItem::separator(app)?,
                    &quit_item,
                ],
            )?;

            let tray = TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "toggle_show" => {
                        let _ = toggle_main_window(app.clone());
                    }
                    "lock_toggle" => {
                        let cfg = load_config();
                        let new_state = !cfg.click_through;
                        let _ = set_click_through(app.clone(), new_state);
                    }
                    "manual_search" => {
                        let _ = open_search_window(app.clone());
                    }
                    "settings" => {
                        let _ = open_settings_window(app.clone());
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    match event {
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            ..
                        }
                        | TrayIconEvent::DoubleClick {
                            button: MouseButton::Left,
                            ..
                        } => {
                            let app = tray.app_handle();
                            let _ = show_main_window(app.clone());
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            let _ = tray;

            // Start AIMP background polling task
            start_aimp_polling_task(handle.clone(), running_clone);

            // Start Win32 Global Hotkey listener (Ctrl + Alt + L) for lock/unlock
            start_global_hotkey_listener(handle.clone());

            // Restore saved window position & size for "main" overlay
            if let Some(main_window) = app.get_webview_window("main") {
                let cfg = load_config();
                if let (Some(x), Some(y)) = (cfg.window_x, cfg.window_y) {
                    let _ = main_window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
                }
                if let (Some(w), Some(h)) = (cfg.window_width, cfg.window_height) {
                    let _ = main_window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: w, height: h }));
                }

                if cfg.click_through {
                    let _ = main_window.set_ignore_cursor_events(true);
                    if let Some(unlock_win) = app.get_webview_window("unlock") {
                        let cur_pos = main_window.outer_position().unwrap_or_default();
                        let cur_size = main_window.outer_size().unwrap_or_default();
                        let badge_x = cur_pos.x + cur_size.width as i32 - 44;
                        let badge_y = cur_pos.y + 8;
                        let _ = unlock_win.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: badge_x, y: badge_y }));
                        let _ = unlock_win.show();
                        let _ = unlock_win.set_always_on_top(true);
                    }
                }
            }

            // Listen to track changed event to automatically resolve lyrics
            let auto_app = handle.clone();
            app.listen("aimp://track-changed", move |event| {
                if let Ok(track) = serde_json::from_str::<AimpTrackInfo>(event.payload()) {
                    let resolver_app = auto_app.clone();
                    tauri::async_runtime::spawn(async move {
                        // Small debounce to avoid rapid song switching burst
                        tokio::time::sleep(Duration::from_millis(150)).await;

                        let state = resolver_app.state::<AppState>();
                        let res = state
                            .lrc_manager
                            .resolve_lyrics(
                                &track.file_name,
                                &track.artist,
                                &track.title,
                                &track.album,
                                track.duration_ms,
                            )
                            .await;

                        match res {
                            Ok(parsed) => {
                                let _ = resolver_app.emit("lyrics://loaded", &parsed);
                            }
                            Err(err) => {
                                let _ = resolver_app.emit(
                                    "lyrics://error",
                                    serde_json::json!({
                                        "error": err,
                                        "artist": track.artist,
                                        "title": track.title
                                    }),
                                );
                            }
                        }
                    });
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    let label = window.label();
                    if label == "settings" || label == "search" || label == "main" || label == "unlock" || label == "editor" {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
                tauri::WindowEvent::Moved(pos) => {
                    if window.label() == "main" {
                        if pos.x > -10000 && pos.y > -10000 {
                            let mut cfg = load_config();
                            cfg.window_x = Some(pos.x);
                            cfg.window_y = Some(pos.y);
                            let _ = save_config(&cfg);

                            if cfg.click_through {
                                if let Some(unlock_win) = window.app_handle().get_webview_window("unlock") {
                                    if let Ok(size) = window.outer_size() {
                                        let badge_x = pos.x + size.width as i32 - 44;
                                        let badge_y = pos.y + 8;
                                        let _ = unlock_win.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: badge_x, y: badge_y }));
                                    }
                                }
                            }
                        }
                    }
                }
                tauri::WindowEvent::Resized(size) => {
                    if window.label() == "main" {
                        if size.width > 200 && size.height > 50 {
                            let mut cfg = load_config();
                            cfg.window_width = Some(size.width);
                            cfg.window_height = Some(size.height);
                            let _ = save_config(&cfg);

                            if cfg.click_through {
                                if let Some(unlock_win) = window.app_handle().get_webview_window("unlock") {
                                    if let Ok(pos) = window.outer_position() {
                                        let badge_x = pos.x + size.width as i32 - 44;
                                        let badge_y = pos.y + 8;
                                        let _ = unlock_win.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: badge_x, y: badge_y }));
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_current_track,
            get_app_config,
            save_app_config,
            set_click_through,
            resolve_lyrics,
            search_lyrics_online,
            apply_lyrics,
            adjust_offset,
            open_settings_window,
            open_search_window,
            open_editor_window,
            clear_cache,
            hide_window,
            minimize_window,
            show_main_window,
            toggle_main_window,
            seek_track,
            dock_window,
            get_album_art,
            control_playback,
            set_aimp_volume,
            save_edited_lyrics,
            get_available_monitors,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
