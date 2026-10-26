pub mod aimp;
pub mod config;
pub mod lrclib;
pub mod parser;

use aimp::{start_aimp_polling_task, AimpReader, AimpTrackInfo};
use config::{load_config, save_config, AppConfig};
use lrclib::{LrcManager, LrclibRecord};
use parser::{parse_lrc, update_lrc_offset, ParsedLyrics};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
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
fn save_app_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    save_config(&config)?;
    let _ = app.emit("config://updated", &config);
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
fn hide_window(app: AppHandle, label: Option<String>) -> Result<(), String> {
    let lbl = label.as_deref().unwrap_or("main");
    if let Some(window) = app.get_webview_window(lbl) {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn minimize_window(app: AppHandle, label: Option<String>) -> Result<(), String> {
    let lbl = label.as_deref().unwrap_or("main");
    if let Some(window) = app.get_webview_window(lbl) {
        window.minimize().map_err(|e| e.to_string())?;
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
    }
    Ok(())
}

#[tauri::command]
fn toggle_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_always_on_top(true);
            let _ = window.set_focus();
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

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
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
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let label = window.label();
                if label == "settings" || label == "search" || label == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
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
            hide_window,
            minimize_window,
            show_main_window,
            toggle_main_window,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
