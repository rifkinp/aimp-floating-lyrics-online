// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE};

fn main() {
    unsafe {
        let _mutex = CreateMutexW(None, true, w!("Global\\AIMP_Floating_Lyrics_SingleInstance_Mutex"));
        if GetLastError() == ERROR_ALREADY_EXISTS {
            // Another instance is already running! Bring existing window to front and exit gracefully
            if let Ok(hwnd) = FindWindowW(PCWSTR::null(), w!("AIMP Floating Lyrics")) {
                if !hwnd.0.is_null() {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                    let _ = SetForegroundWindow(hwnd);
                }
            }
            return;
        }
    }

    std::panic::set_hook(Box::new(|panic_info| {
        let msg = format!("PANIC: {:?}\nLocation: {:?}", panic_info.payload().downcast_ref::<&str>(), panic_info.location());
        let _ = std::fs::write("panic.log", format!("{}\nFull: {}\n", msg, panic_info));
        eprintln!("{}", msg);
    }));

    aimp_floating_lyrics_lib::run();
}
