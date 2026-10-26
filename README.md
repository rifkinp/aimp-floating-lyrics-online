# AIMP Floating Lyrics Online

A production-ready, lightweight, transparent, always-on-top, click-through desktop overlay window built using **Tauri v2 (Rust backend)** and **Vue 3 + Tailwind CSS (reactive frontend)** for Windows 11.

It automatically detects currently playing tracks from the **AIMP Music Player** via Win32 Shared Memory (`AIMP2_RemoteInfo` / `AIMPRemoteAccessClass`), checks for local `.lrc` files, and fetches synchronized lyrics on-the-fly from the **LRCLIB API (`https://lrclib.net`)** with auto-caching.

---

## Features

- **Transparent & Frameless Overlay:** Runs as an always-on-top desktop overlay with customizable font sizes, glow effects, text outlines, and colors.
- **Click-Through Mode:** Easily lock/unlock mouse interactions via global hotkey (`Ctrl + Alt + L`), hover toolbar button, or System Tray icon.
- **Native AIMP Integration:** Uses Win32 Shared Memory mapping (88-byte Delphi unpadded header) and `SendMessageW` (`WM_AIMP_PROPERTY` = `0x0477`) at a 50ms polling interval.
- **LRCLIB Client with Rate-Limit & Overload Resilience:** Automatically fetches from `/api/get` and `/api/search` with retry and exponential backoff on `503 ServerOverloaded` and `429 Too Many Requests`.
- **Automatic Offline Caching:** Automatically saves fetched `.lrc` files next to audio files (or falls back to `%APPDATA%/aimp-floating-lyrics/cache/`).
- **Real-Time Smooth Interpolation:** Uses `requestAnimationFrame` interpolation between AIMP's 50ms polling ticks for silky smooth synchronized line and karaoke transitions.
- **3 Display Modes:**
  - *Two-Line Classic:* Active current line + upcoming line preview.
  - *Single-Line Fade:* Clean minimal display with animated fade transitions.
  - *Karaoke Sweep:* Word-level / syllable-level karaoke progress highlight.
- **Interactive Hover Toolbar:**
  - Drag handle to reposition anywhere on screen.
  - Sync Offset adjustment: `-0.5s`, `-0.1s`, `+0.1s`, `+0.5s`.
  - Manual lyrics search modal (for obscure or mis-tagged tracks).
  - Appearance settings modal (colors, font sizes, display modes).
- **System Tray Integration:** Quick context menu for click-through toggle, manual search, settings, and application exit.

---

## Quick Start (Run Standalone)

You can run the pre-built standalone application directly without needing Node.js or Rust:

1. Navigate to:
   ```
   d:\0 Project Portofolio\Lyrics Aimp Generator\release\
   ```
2. Double-click **`AIMP Floating Lyrics.exe`** (ensure `WebView2Loader.dll` remains in the same folder).
3. Play any track in AIMP; lyrics will immediately appear and auto-download!

---

## Keyboard Shortcuts & Controls

| Shortcut / Action | Description |
|-------------------|-------------|
| `Ctrl + Alt + L` | Toggle Click-Through Lock / Unlock |
| Hover on window | Reveals the drag handle and quick sync toolbar |
| Tray Icon Click | Toggle window visibility / Focus overlay |
| Tray Right-Click | Open context menu (Lock, Search, Settings, Quit) |

---

## Development & Building from Source

### Prerequisites
- Node.js 18+ & npm
- Rust 1.81+ (`x86_64-pc-windows-gnu`) & MinGW UCRT

### Build Frontend:
```bash
npm install
npm run build
```

### Build Standalone Release:
```bash
cd src-tauri
cargo build --release
```
The compiled self-contained executable will be generated at `src-tauri/target/release/aimp-floating-lyrics.exe`.
