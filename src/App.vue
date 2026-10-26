<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import HoverToolbar from './components/HoverToolbar.vue';
import LyricsDisplay from './components/LyricsDisplay.vue';
import ManualSearchModal from './components/ManualSearchModal.vue';
import SettingsModal from './components/SettingsModal.vue';
import type {
  AimpTrackInfo,
  AimpPositionUpdate,
  ParsedLyrics,
  AppConfig
} from './types';

const currentWindow = getCurrentWebviewWindow();
const windowLabel = currentWindow.label; // 'main' | 'settings' | 'search'

// App state
const track = ref<AimpTrackInfo | null>(null);
const lyrics = ref<ParsedLyrics | null>(null);
const isLoading = ref(false);
const isHovered = ref(false);
const isLocked = ref(false);

const config = ref<AppConfig>({
  clickThrough: false,
  displayMode: 'two-line',
  fontSize: 28,
  fontFamily: 'Inter, system-ui, sans-serif',
  activeColor: '#38BDF8',
  secondaryColor: 'rgba(255, 255, 255, 0.65)',
  strokeColor: '#000000',
  strokeWidth: 2,
  textAlign: 'center',
  saveToAudioFolder: true,
});

// Real-time smooth playback position interpolation
const positionMs = ref(0);
let lastAimpPositionMs = 0;
let lastUpdateTimestamp = 0;
let playbackState = 0;
let animFrameId: number | null = null;
const unlistenFns: UnlistenFn[] = [];

const updateInterpolatedClock = () => {
  if (playbackState === 2) {
    // Playing: smoothly interpolate clock
    const delta = performance.now() - lastUpdateTimestamp;
    positionMs.value = Math.max(0, Math.floor(lastAimpPositionMs + delta));
    animFrameId = requestAnimationFrame(updateInterpolatedClock);
  } else {
    positionMs.value = lastAimpPositionMs;
    animFrameId = null;
  }
};

const ensureClockRunning = () => {
  if (windowLabel === 'main' && playbackState === 2 && animFrameId === null) {
    animFrameId = requestAnimationFrame(updateInterpolatedClock);
  }
};

const openSettingsWindow = async () => {
  try {
    await invoke('open_settings_window');
  } catch (err) {
    console.error('Failed to open settings window:', err);
  }
};

const openSearchWindow = async () => {
  try {
    await invoke('open_search_window');
  } catch (err) {
    console.error('Failed to open search window:', err);
  }
};

onMounted(async () => {
  // 1. Load user configuration
  try {
    const loadedCfg = await invoke<AppConfig>('get_app_config');
    if (loadedCfg) {
      config.value = loadedCfg;
      isLocked.value = loadedCfg.clickThrough;
    }
  } catch (err) {
    console.error('Failed to load config:', err);
  }

  // 2. Start smooth interpolation loop (for main window)
  if (windowLabel === 'main') {
    animFrameId = requestAnimationFrame(updateInterpolatedClock);
  }

  // 3. Register Tauri Event Listeners
  unlistenFns.push(
    await listen<AimpTrackInfo>('aimp://track-changed', async (event) => {
      track.value = event.payload;
      playbackState = event.payload.state;
      ensureClockRunning();

      if (windowLabel === 'main') {
        isLoading.value = true;
        lyrics.value = null;

        // Automatically resolve lyrics
        try {
          const result = await invoke<ParsedLyrics>('resolve_lyrics', {
            audioPath: event.payload.fileName,
            artist: event.payload.artist,
            title: event.payload.title,
            album: event.payload.album,
            durationMs: event.payload.durationMs,
          });
          lyrics.value = result;
        } catch (err) {
          console.warn('Lyrics resolution:', err);
        } finally {
          isLoading.value = false;
        }
      }
    })
  );

  unlistenFns.push(
    await listen<AimpPositionUpdate>('aimp://position-update', (event) => {
      lastAimpPositionMs = event.payload.positionMs;
      playbackState = event.payload.state;
      lastUpdateTimestamp = performance.now();
      if (track.value) {
        track.value.state = event.payload.state;
      }
      ensureClockRunning();
    })
  );

  unlistenFns.push(
    await listen<ParsedLyrics>('lyrics://loaded', (event) => {
      lyrics.value = event.payload;
      isLoading.value = false;
    })
  );

  unlistenFns.push(
    await listen<{ error: string }>('lyrics://error', () => {
      isLoading.value = false;
    })
  );

  unlistenFns.push(
    await listen<boolean>('lyrics://lock-changed', (event) => {
      isLocked.value = event.payload;
      config.value.clickThrough = event.payload;
    })
  );

  unlistenFns.push(
    await listen<AppConfig>('config://updated', (event) => {
      config.value = event.payload;
      isLocked.value = event.payload.clickThrough;
    })
  );

  unlistenFns.push(
    await listen('ui://open-manual-search', () => {
      openSearchWindow();
    })
  );

  unlistenFns.push(
    await listen('ui://open-settings', () => {
      openSettingsWindow();
    })
  );

  // 4. Immediately fetch currently playing track if AIMP is already running
  try {
    const cur = await invoke<AimpTrackInfo | null>('get_current_track');
    if (cur && cur.title) {
      track.value = cur;
      playbackState = cur.state;
      if (windowLabel === 'main') {
        isLoading.value = true;
        try {
          const result = await invoke<ParsedLyrics>('resolve_lyrics', {
            audioPath: cur.fileName,
            artist: cur.artist,
            title: cur.title,
            album: cur.album,
            durationMs: cur.durationMs,
          });
          lyrics.value = result;
        } catch (err) {
          console.warn('Initial lyrics resolution:', err);
        } finally {
          isLoading.value = false;
        }
      }
    }
  } catch (err) {
    console.error('Failed to get current track:', err);
  }

  // Global hotkey: Ctrl + Alt + L to toggle lock
  if (windowLabel === 'main') {
    window.addEventListener('keydown', handleKeydown);
  }
});

onUnmounted(() => {
  if (animFrameId) cancelAnimationFrame(animFrameId);
  unlistenFns.forEach((fn) => fn());
  window.removeEventListener('keydown', handleKeydown);
});

const handleKeydown = (e: KeyboardEvent) => {
  if (e.ctrlKey && e.altKey && e.key.toLowerCase() === 'l') {
    e.preventDefault();
    toggleLock();
  }
};

const toggleLock = async () => {
  const nextState = !isLocked.value;
  try {
    await invoke('set_click_through', { locked: nextState });
    isLocked.value = nextState;
  } catch (err) {
    console.error('Failed to toggle click-through:', err);
  }
};

const handleAdjustOffset = async (deltaMs: number) => {
  if (!lyrics.value?.rawLrc) return;
  try {
    const updated = await invoke<ParsedLyrics>('adjust_offset', {
      audioPath: track.value?.fileName || '',
      artist: track.value?.artist || '',
      title: track.value?.title || '',
      currentRawLrc: lyrics.value.rawLrc,
      deltaMs,
    });
    lyrics.value = updated;
  } catch (err) {
    console.error('Failed to adjust offset:', err);
  }
};

const handleUpdateConfig = async (newConfig: AppConfig) => {
  config.value = newConfig;
  try {
    await invoke('save_app_config', { config: newConfig });
  } catch (err) {
    console.error('Failed to save config:', err);
  }
};

const onLyricsApplied = (newLyrics: ParsedLyrics) => {
  lyrics.value = newLyrics;
};

const hideCurrentWindow = async () => {
  try {
    await invoke('hide_window', { label: windowLabel });
  } catch (_) {
    currentWindow.hide();
  }
};
</script>

<template>
  <!-- Window 1: Dedicated Settings Popup -->
  <SettingsModal
    v-if="windowLabel === 'settings'"
    :standalone="true"
    :config="config"
    @close="hideCurrentWindow"
    @update-config="handleUpdateConfig"
  />

  <!-- Window 2: Dedicated Manual Search Popup -->
  <ManualSearchModal
    v-else-if="windowLabel === 'search'"
    :standalone="true"
    :track="track"
    @close="hideCurrentWindow"
    @lyrics-applied="onLyricsApplied"
  />

  <!-- Window 3: Main Floating Lyrics Overlay -->
  <main
    v-else
    @mouseenter="isHovered = true"
    @mouseleave="isHovered = false"
    @mousemove="isHovered = true"
    class="relative w-screen h-screen flex flex-col justify-between overflow-hidden transition-all duration-200"
    :class="[
      !isLocked && isHovered
        ? 'bg-slate-950/85 backdrop-blur-md rounded-2xl border border-white/15 shadow-2xl'
        : (!isLocked ? 'bg-slate-950/[0.02]' : 'bg-transparent')
    ]"
  >
    <!-- Hover Toolbar (Unlocked mode only) -->
    <div
      v-if="!isLocked"
      class="w-full pt-2 px-3 transition-opacity duration-200 z-30"
      :class="isHovered ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'"
    >
      <HoverToolbar
        :track="track"
        :offset-ms="lyrics?.offsetMs || 0"
        :is-locked="isLocked"
        @lock="toggleLock"
        @open-search="openSearchWindow"
        @open-settings="openSettingsWindow"
        @adjust-offset="handleAdjustOffset"
      />
    </div>

    <!-- Lyrics Display Core -->
    <div class="flex-1 w-full flex items-center justify-center min-h-0">
      <LyricsDisplay
        :track="track"
        :lyrics="lyrics"
        :current-position-ms="positionMs"
        :config="config"
        :is-loading="isLoading"
      />
    </div>
  </main>
</template>
