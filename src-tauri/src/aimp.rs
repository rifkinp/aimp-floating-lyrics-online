use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use windows::core::w;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM, WPARAM};
use windows::Win32::System::Memory::{
    MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_READ, MEMORY_MAPPED_VIEW_ADDRESS,
};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, SendMessageW};

const WM_AIMP_PROPERTY: u32 = 0x0477; // WM_USER (0x0400) + 0x77
const AIMP_PROP_PLAYER_STATE: usize = 0x40; // 0 = Stopped, 1 = Paused, 2 = Playing
const AIMP_PROP_TRACK_POSITION: usize = 0x20; // in milliseconds
const AIMP_PROP_TRACK_DURATION: usize = 0x10; // in milliseconds

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AimpRemoteFileInfo {
    pub cb_size: u32,
    pub active: u32,
    pub bit_rate: u32,
    pub channels: u32,
    pub duration: u32,
    pub file_size_low: u32,
    pub file_size_high: u32,
    pub file_mark: u32,
    pub sample_rate: u32,
    pub track_number: u32,
    pub album_length: u32,
    pub artist_length: u32,
    pub date_length: u32,
    pub file_name_length: u32,
    pub genre_length: u32,
    pub title_length: u32,
    pub deprecated2: [u32; 6],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct AimpTrackInfo {
    pub artist: String,
    pub title: String,
    pub album: String,
    pub file_name: String,
    pub duration_ms: u32,
    pub state: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AimpPositionUpdate {
    pub position_ms: u32,
    pub duration_ms: u32,
    pub state: u32,
}

pub struct AimpReader;

impl AimpReader {
    pub fn get_current_track() -> Option<AimpTrackInfo> {
        let hwnd_opt = Self::find_aimp_window();
        if let Some(mut info) = Self::read_shared_memory() {
            if let Some(hwnd) = hwnd_opt {
                let (state, _pos, dur) = Self::get_playback_state(hwnd);
                info.state = state;
                if dur > 0 && info.duration_ms == 0 {
                    info.duration_ms = dur;
                }
            } else if info.state == 0 {
                info.state = 2; // Default to playing when active in memory
            }
            return Some(info);
        }
        None
    }

    pub fn find_aimp_window() -> Option<HWND> {
        unsafe {
            // Try AIMP2_RemoteInfo first
            let hwnd1 = FindWindowW(w!("AIMP2_RemoteInfo"), w!("AIMP2_RemoteInfo"));
            if let Ok(h) = hwnd1 {
                if !h.0.is_null() {
                    return Some(h);
                }
            }

            // Try AIMPRemoteAccessClass
            let hwnd2 = FindWindowW(w!("AIMPRemoteAccessClass"), w!("AIMPRemoteAccessClass"));
            if let Ok(h) = hwnd2 {
                if !h.0.is_null() {
                    return Some(h);
                }
            }

            None
        }
    }

    pub fn get_playback_state(hwnd: HWND) -> (u32, u32, u32) {
        unsafe {
            let state_res = SendMessageW(
                hwnd,
                WM_AIMP_PROPERTY,
                WPARAM(AIMP_PROP_PLAYER_STATE),
                LPARAM(0),
            );
            let pos_res = SendMessageW(
                hwnd,
                WM_AIMP_PROPERTY,
                WPARAM(AIMP_PROP_TRACK_POSITION),
                LPARAM(0),
            );
            let dur_res = SendMessageW(
                hwnd,
                WM_AIMP_PROPERTY,
                WPARAM(AIMP_PROP_TRACK_DURATION),
                LPARAM(0),
            );

            let state = (state_res.0 as u32).min(3);
            let pos = state_res_to_u32(pos_res.0);
            let dur = state_res_to_u32(dur_res.0);

            (state, pos, dur)
        }
    }

    pub fn read_shared_memory() -> Option<AimpTrackInfo> {
        unsafe {
            let mut handle: Option<HANDLE> = None;

            // Try AIMP2_RemoteInfo
            if let Ok(h) = OpenFileMappingW(FILE_MAP_READ.0, false, w!("AIMP2_RemoteInfo")) {
                if !h.0.is_null() {
                    handle = Some(h);
                }
            }

            // Fallback to AIMPRemoteAccessClass
            if handle.is_none() {
                if let Ok(h) = OpenFileMappingW(FILE_MAP_READ.0, false, w!("AIMPRemoteAccessClass"))
                {
                    if !h.0.is_null() {
                        handle = Some(h);
                    }
                }
            }

            let file_map = handle?;

            let view: MEMORY_MAPPED_VIEW_ADDRESS = MapViewOfFile(
                file_map,
                FILE_MAP_READ,
                0,
                0,
                0,
            );

            if view.Value.is_null() {
                let _ = CloseHandle(file_map);
                return None;
            }

            let header_ptr = view.Value as *const AimpRemoteFileInfo;
            let header = *header_ptr;

            let total_chars = (header.album_length
                + header.artist_length
                + header.date_length
                + header.file_name_length
                + header.genre_length
                + header.title_length) as usize;

            // Max safeguard: 32K chars
            if total_chars > 32768 {
                let _ = UnmapViewOfFile(view);
                let _ = CloseHandle(file_map);
                return None;
            }

            let str_start = (view.Value as *const u8)
                .add(std::mem::size_of::<AimpRemoteFileInfo>())
                as *const u16;

            let mut offset = 0;
            let album = read_utf16_str(str_start, offset, header.album_length as usize);
            offset += header.album_length as usize;

            let artist = read_utf16_str(str_start, offset, header.artist_length as usize);
            offset += header.artist_length as usize;

            let _date = read_utf16_str(str_start, offset, header.date_length as usize);
            offset += header.date_length as usize;

            let file_name = read_utf16_str(str_start, offset, header.file_name_length as usize);
            offset += header.file_name_length as usize;

            let _genre = read_utf16_str(str_start, offset, header.genre_length as usize);
            offset += header.genre_length as usize;

            let title = read_utf16_str(str_start, offset, header.title_length as usize);

            let _ = UnmapViewOfFile(view);
            let _ = CloseHandle(file_map);

            let duration_ms = header.duration;

            Some(AimpTrackInfo {
                artist: artist.trim().to_string(),
                title: title.trim().to_string(),
                album: album.trim().to_string(),
                file_name: file_name.trim().to_string(),
                duration_ms,
                state: 0,
            })
        }
    }
}

fn state_res_to_u32(val: isize) -> u32 {
    if val < 0 {
        0
    } else {
        val as u32
    }
}

unsafe fn read_utf16_str(base: *const u16, offset: usize, len: usize) -> String {
    if len == 0 {
        return String::new();
    }
    let slice = std::slice::from_raw_parts(base.add(offset), len);
    String::from_utf16_lossy(slice)
}

pub fn start_aimp_polling_task(
    app: AppHandle,
    running: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("aimp-polling".into())
        .spawn(move || {
            let mut last_track = AimpTrackInfo::default();
            let mut last_state = 0;
            let mut last_pos = 0;
            let mut idle_counter: u32 = 0;

            while running.load(Ordering::Relaxed) {
                let hwnd_opt = AimpReader::find_aimp_window();
                let mem_opt = if hwnd_opt.is_some() {
                    AimpReader::read_shared_memory()
                } else {
                    None
                };

                if let Some(mut info) = mem_opt {
                    idle_counter = 0;
                    let (state, pos, dur) = if let Some(hwnd) = hwnd_opt {
                        AimpReader::get_playback_state(hwnd)
                    } else {
                        let s = if info.state != 0 { info.state } else { 2 };
                        (s, 0, info.duration_ms)
                    };

                    info.state = state;
                    if dur > 0 && info.duration_ms == 0 {
                        info.duration_ms = dur;
                    }

                    // Check if track changed (by file_name or artist+title)
                    let track_changed = info.file_name != last_track.file_name
                        || info.title != last_track.title
                        || info.artist != last_track.artist;

                    if track_changed {
                        last_track = info.clone();
                        let _ = app.emit("aimp://track-changed", &info);
                    }

                    // Emit position update when playing or paused, or when state changed
                    if state != last_state || (state == 2 && pos != last_pos) {
                        last_state = state;
                        last_pos = pos;
                        let _ = app.emit(
                            "aimp://position-update",
                            &AimpPositionUpdate {
                                position_ms: pos,
                                duration_ms: if dur > 0 { dur } else { info.duration_ms },
                                state,
                            },
                        );
                    }
                } else {
                    // Shared memory not found or empty
                    if last_state != 0 {
                        last_state = 0;
                        last_pos = 0;
                        let _ = app.emit(
                            "aimp://position-update",
                            &AimpPositionUpdate {
                                position_ms: 0,
                                duration_ms: 0,
                                state: 0,
                            },
                        );
                    }

                    idle_counter = idle_counter.wrapping_add(1);
                    // Check close_with_aimp every ~2 seconds (40 * 50ms) to avoid continuous disk I/O
                    if idle_counter % 40 == 0 {
                        let cfg = crate::config::load_config();
                        if cfg.close_with_aimp && hwnd_opt.is_none() && last_track.duration_ms > 0 {
                            app.exit(0);
                            break;
                        }
                    }
                }

                std::thread::sleep(Duration::from_millis(50));
            }
        })
        .expect("failed to spawn aimp polling thread")
}

pub fn seek_track(position_ms: u32) -> Result<(), String> {
    unsafe {
        let hwnd = FindWindowW(w!("AIMPRemoteAccessClass"), w!("AIMPRemoteAccessClass"))
            .map_err(|e| e.to_string())?;
        if hwnd.0.is_null() {
            return Err("AIMP window not found".to_string());
        }
        SendMessageW(
            hwnd,
            WM_AIMP_PROPERTY,
            WPARAM(AIMP_PROP_TRACK_POSITION | 1),
            LPARAM(position_ms as isize),
        );
        Ok(())
    }
}
