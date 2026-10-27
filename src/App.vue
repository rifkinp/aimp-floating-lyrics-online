<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import HoverToolbar from './components/HoverToolbar.vue';
import LyricsDisplay from './components/LyricsDisplay.vue';
import FullLyricsView from './components/FullLyricsView.vue';
import ManualSearchModal from './components/ManualSearchModal.vue';
import SettingsModal from './components/SettingsModal.vue';
import LrcEditorModal from './components/LrcEditorModal.vue';
import { Lock, Unlock } from 'lucide-vue-next';
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
const albumArt = ref<string | null>(null);
const showEditor = ref(false);

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

        try {
          albumArt.value = await invoke<string | null>('get_album_art', {
            filePath: event.payload.fileName,
          });
        } catch (_) {
          albumArt.value = null;
        }

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
          albumArt.value = await invoke<string | null>('get_album_art', {
            filePath: cur.fileName,
          });
        } catch (_) {
          albumArt.value = null;
        }
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

const showFullLyrics = ref(false);

const handleMouseEnter = () => {
  if (!isLocked.value) {
    isHovered.value = true;
  }
};

const handleMouseLeave = () => {
  isHovered.value = false;
  showFullLyrics.value = false; // Always revert to synchronized timestamp view!
};

const handleSeekTrack = async (posMs: number) => {
  try {
    await invoke('seek_track', { positionMs: posMs });
  } catch (err) {
    console.error('Failed to seek track:', err);
  }
};

const handleDock = async (position: string) => {
  try {
    await invoke('dock_window', { position });
  } catch (err) {
    console.error('Failed to dock window:', err);
  }
};

const handleControlPlayback = async (command: string) => {
  try {
    await invoke('control_playback', { command });
  } catch (err) {
    console.error('Failed to control playback in AIMP:', err);
  }
};

const onLyricsSaved = (updated: ParsedLyrics) => {
  lyrics.value = updated;
};

const backgroundStyle = computed(() => {
  let opacity = config.value.backgroundOpacity ?? 0.15;
  if (!isLocked.value && isHovered.value) {
    opacity = Math.max(opacity, 0.88);
  } else if (isLocked.value) {
    opacity = config.value.backgroundOpacity ?? 0.0;
  }

  const isGlass = config.value.enableGlassmorphism ?? true;
  return {
    backgroundColor: opacity > 0.01 ? `rgba(15, 23, 42, ${opacity})` : 'transparent',
    backdropFilter: isGlass && opacity > 0.05 ? 'blur(16px)' : 'none',
    WebkitBackdropFilter: isGlass && opacity > 0.05 ? 'blur(16px)' : 'none',
    border: opacity > 0.05 ? `1px solid rgba(255, 255, 255, ${Math.min(opacity * 0.25, 0.2)})` : 'none',
    boxShadow: opacity > 0.1 ? '0 20px 25px -5px rgba(0, 0, 0, 0.6)' : 'none',
  };
});

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

const unlockFromBadge = async () => {
  try {
    await invoke('set_click_through', { locked: false });
    isLocked.value = false;
  } catch (err) {
    console.error('Failed to unlock from badge:', err);
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

  <!-- Window 4: Dedicated Mini Unlock Button for Click-Through Mode -->
  <div
    v-else-if="windowLabel === 'unlock'"
    @click="unlockFromBadge"
    class="w-full h-full flex items-center justify-center cursor-pointer select-none group bg-transparent p-0.5"
    title="Jendela terkunci (Click-Through). Klik di sini atau tekan Ctrl+Alt+L untuk membuka kunci"
  >
    <div
      class="w-8 h-8 rounded-full bg-slate-900/95 hover:bg-red-500 border border-amber-400/80 hover:border-red-300 text-amber-300 hover:text-white flex items-center justify-center shadow-2xl transition-all duration-200 transform hover:scale-110 active:scale-90 animate-pulse hover:animate-none"
    >
      <Lock class="w-4 h-4 group-hover:hidden" />
      <Unlock class="w-4 h-4 hidden group-hover:block" />
    </div>
  </div>

  <!-- Window 3: Main Floating Lyrics Overlay -->
  <main
    v-else
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
    @mousemove="handleMouseEnter"
    class="relative w-screen h-screen flex flex-col justify-between overflow-hidden transition-all duration-200 rounded-2xl"
    :style="backgroundStyle"
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
        :show-full-lyrics="showFullLyrics"
        :dock-position="config.dockPosition || 'none'"
        :album-art="albumArt"
        :current-position-ms="positionMs"
        :duration-ms="track?.durationMs || 0"
        @lock="toggleLock"
        @open-search="openSearchWindow"
        @open-settings="openSettingsWindow"
        @open-editor="showEditor = true"
        @adjust-offset="handleAdjustOffset"
        @toggle-full-lyrics="showFullLyrics = !showFullLyrics"
        @dock="handleDock"
        @control="handleControlPlayback"
        @seek="handleSeekTrack"
      />
    </div>

    <!-- Lyrics Display Core or Full Lyrics on Hover -->
    <div class="flex-1 w-full flex items-center justify-center min-h-0 overflow-hidden">
      <!-- Full Lyrics on Hover (reverts automatically on mouseleave) -->
      <FullLyricsView
        v-if="showFullLyrics && isHovered && lyrics && lyrics.lines.length > 0"
        :lyrics="lyrics"
        :current-position-ms="positionMs"
        :config="config"
        @seek="handleSeekTrack"
      />

      <!-- Synced Floating Lyrics Overlay -->
      <LyricsDisplay
        v-else
        :track="track"
        :lyrics="lyrics"
        :current-position-ms="positionMs"
        :config="config"
        :is-loading="isLoading"
      />
    </div>
  </main>

  <!-- Quick LRC Editor Modal -->
  <LrcEditorModal
    v-if="showEditor"
    :track="track"
    :lyrics="lyrics"
    :current-position-ms="positionMs"
    @close="showEditor = false"
    @saved="onLyricsSaved"
  />
</template>
