<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
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

// App state
const track = ref<AimpTrackInfo | null>(null);
const lyrics = ref<ParsedLyrics | null>(null);
const isLoading = ref(false);
const isHovered = ref(false);
const isLocked = ref(false);
const showSearchModal = ref(false);
const showSettingsModal = ref(false);

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
    // Playing
    const delta = performance.now() - lastUpdateTimestamp;
    positionMs.value = Math.max(0, Math.floor(lastAimpPositionMs + delta));
  } else {
    positionMs.value = lastAimpPositionMs;
  }
  animFrameId = requestAnimationFrame(updateInterpolatedClock);
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

  // 2. Start smooth interpolation loop
  animFrameId = requestAnimationFrame(updateInterpolatedClock);

  // 3. Register Tauri Event Listeners
  unlistenFns.push(
    await listen<AimpTrackInfo>('aimp://track-changed', async (event) => {
      track.value = event.payload;
      playbackState = event.payload.state;
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
    await listen('ui://open-manual-search', () => {
      showSearchModal.value = true;
    })
  );

  unlistenFns.push(
    await listen('ui://open-settings', () => {
      showSettingsModal.value = true;
    })
  );

  // 4. Immediately fetch currently playing track if AIMP is already running
  try {
    const cur = await invoke<AimpTrackInfo | null>('get_current_track');
    if (cur && cur.title) {
      track.value = cur;
      playbackState = cur.state;
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
  } catch (err) {
    console.error('Failed to get current track:', err);
  }

  // Global hotkey: Ctrl + Alt + L to toggle lock
  window.addEventListener('keydown', handleKeydown);
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
</script>

<template>
  <main
    @mouseenter="isHovered = true"
    @mouseleave="isHovered = false"
    class="relative w-screen h-screen flex flex-col justify-between overflow-hidden transition-colors duration-200"
    :class="[
      !isLocked && (isHovered || showSearchModal || showSettingsModal)
        ? 'bg-slate-950/75 backdrop-blur-md rounded-2xl border border-white/10'
        : 'bg-transparent'
    ]"
  >
    <!-- Hover Toolbar (Unlocked mode only) -->
    <div
      v-if="!isLocked"
      class="w-full pt-2 px-3 transition-opacity duration-200 z-30"
      :class="isHovered || showSearchModal || showSettingsModal ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'"
    >
      <HoverToolbar
        :track="track"
        :offset-ms="lyrics?.offsetMs || 0"
        :is-locked="isLocked"
        @lock="toggleLock"
        @open-search="showSearchModal = true"
        @open-settings="showSettingsModal = true"
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

    <!-- Modals -->
    <ManualSearchModal
      v-if="showSearchModal"
      :track="track"
      @close="showSearchModal = false"
      @lyrics-applied="onLyricsApplied"
    />

    <SettingsModal
      v-if="showSettingsModal"
      :config="config"
      @close="showSettingsModal = false"
      @update-config="handleUpdateConfig"
    />
  </main>
</template>
