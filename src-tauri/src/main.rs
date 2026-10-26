// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        let msg = format!("PANIC: {:?}\nLocation: {:?}", panic_info.payload().downcast_ref::<&str>(), panic_info.location());
        let _ = std::fs::write("panic.log", format!("{}\nFull: {}\n", msg, panic_info));
        eprintln!("{}", msg);
    }));

    aimp_floating_lyrics_lib::run();
}
