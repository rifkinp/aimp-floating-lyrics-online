<script setup lang="ts">
import { computed } from 'vue';
import {
  GripVertical,
  Lock,
  Search,
  Settings,
  Minus,
  X,
  ListMusic,
  Anchor,
  Play,
  Pause,
  SkipBack,
  SkipForward,
  Disc,
  Edit3
} from 'lucide-vue-next';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import type { AimpTrackInfo } from '../types';

const props = withDefaults(
  defineProps<{
    track: AimpTrackInfo | null;
    offsetMs: number;
    isLocked: boolean;
    showFullLyrics?: boolean;
    dockPosition?: string;
    albumArt?: string | null;
    currentPositionMs?: number;
    durationMs?: number;
  }>(),
  {
    showFullLyrics: false,
    dockPosition: 'none',
    albumArt: null,
    currentPositionMs: 0,
    durationMs: 0,
  }
);

const emit = defineEmits<{
  (e: 'lock'): void;
  (e: 'open-search'): void;
  (e: 'open-settings'): void;
  (e: 'open-editor'): void;
  (e: 'adjust-offset', deltaMs: number): void;
  (e: 'toggle-full-lyrics'): void;
  (e: 'dock', position: string): void;
  (e: 'control', command: string): void;
  (e: 'seek', positionMs: number): void;
}>();

import { invoke } from '@tauri-apps/api/core';

const appWindow = getCurrentWebviewWindow();

const progressPct = computed(() => {
  if (!props.durationMs || props.durationMs <= 0) return 0;
  const pct = ((props.currentPositionMs || 0) / props.durationMs) * 100;
  return Math.min(Math.max(pct, 0), 100);
});

const formatTime = (ms: number) => {
  const secs = Math.floor(ms / 1000);
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
};

const handleProgressClick = (e: MouseEvent) => {
  if (!props.durationMs || props.durationMs <= 0) return;
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const clickX = e.clientX - rect.left;
  const pct = Math.min(Math.max(clickX / rect.width, 0), 1);
  const targetMs = Math.floor(pct * props.durationMs);
  emit('seek', targetMs);
};

const startDrag = (e: MouseEvent) => {
  if (e.buttons === 1) {
    appWindow.startDragging();
  }
};

const toggleDock = () => {
  const next = props.dockPosition === 'bottom-taskbar' ? 'none' : 'bottom-taskbar';
  emit('dock', next);
};

const minimizeWindow = async () => {
  try {
    await invoke('hide_window', { label: 'main' });
  } catch (err) {
    console.error('Failed to minimize window:', err);
  }
};

const closeApp = async () => {
  try {
    await invoke('hide_window', { label: 'main' });
  } catch (err) {
    console.error('Failed to close/hide window:', err);
  }
};
</script>

<template>
  <div class="flex flex-col w-full bg-slate-900/85 backdrop-blur-md border border-white/10 rounded-xl shadow-2xl transition-all duration-200 select-none text-xs text-slate-200 font-medium overflow-hidden">
    <div
      @mousedown.self="startDrag"
      class="flex items-center justify-between w-full px-3 py-1.5"
    >
      <!-- Left: Drag Region, Album Art & Song Info -->
      <div
        data-tauri-drag-region
        @mousedown="startDrag"
        class="flex items-center space-x-2 flex-1 min-w-0 cursor-grab active:cursor-grabbing"
      >
        <div
          class="flex items-center space-x-1 p-1 hover:bg-white/10 rounded-md transition-colors"
          title="Drag to move overlay"
        >
          <GripVertical class="w-3.5 h-3.5 text-slate-400" />
        </div>

        <!-- Album Art Thumbnail -->
        <div class="relative w-6 h-6 rounded-md overflow-hidden bg-slate-800 flex-shrink-0 flex items-center justify-center border border-white/15 shadow-sm">
          <img v-if="albumArt" :src="albumArt" alt="Album Art" class="w-full h-full object-cover" />
          <Disc v-else class="w-3.5 h-3.5 text-slate-400" />
        </div>

        <div class="truncate flex items-center space-x-1.5 max-w-[210px] pointer-events-none">
          <span class="inline-block w-2 h-2 rounded-full flex-shrink-0" :class="track?.state === 2 ? 'bg-emerald-400 animate-pulse' : 'bg-amber-400'"></span>
          <span class="text-white font-semibold truncate">{{ track?.title || 'Waiting for AIMP...' }}</span>
          <span v-if="track?.artist" class="text-slate-400 truncate text-[11px]">- {{ track.artist }}</span>
        </div>
      </div>

      <!-- Center: Music Controller & Offset -->
      <div class="flex items-center space-x-1.5 px-2 border-x border-white/10">
        <!-- Playback Controller -->
        <div class="flex items-center space-x-0.5 px-1 py-0.5 bg-white/5 rounded-lg border border-white/10">
          <button
            type="button"
            @click="emit('control', 'prev')"
            class="p-1 rounded hover:bg-white/15 text-slate-300 hover:text-white transition active:scale-90"
            title="Previous Track (AIMP)"
          >
            <SkipBack class="w-3 h-3" />
          </button>

          <button
            type="button"
            @click="emit('control', 'toggle')"
            class="p-1 rounded hover:bg-white/15 text-sky-300 hover:text-white transition active:scale-90"
            :title="track?.state === 2 ? 'Pause' : 'Play'"
          >
            <Pause v-if="track?.state === 2" class="w-3.5 h-3.5 fill-current" />
            <Play v-else class="w-3.5 h-3.5 fill-current" />
          </button>

          <button
            type="button"
            @click="emit('control', 'next')"
            class="p-1 rounded hover:bg-white/15 text-slate-300 hover:text-white transition active:scale-90"
            title="Next Track (AIMP)"
          >
            <SkipForward class="w-3 h-3" />
          </button>
        </div>

        <!-- Sync Offset -->
        <div class="flex items-center space-x-0.5">
          <button
            @click="emit('adjust-offset', -500)"
            class="px-1.5 py-0.5 rounded hover:bg-white/15 bg-white/5 active:scale-95 transition text-[10px]"
            title="Delay -0.5s"
          >
            -0.5s
          </button>

          <button
            type="button"
            @click="emit('adjust-offset', -offsetMs)"
            class="px-1.5 py-0.5 rounded font-mono text-[10px] font-bold hover:bg-white/15 active:scale-95 transition cursor-pointer"
            :class="offsetMs === 0 ? 'text-slate-300' : offsetMs > 0 ? 'text-emerald-400' : 'text-amber-400'"
            title="Klik untuk langsung reset offset ke 0ms"
          >
            {{ offsetMs > 0 ? `+${offsetMs}ms` : `${offsetMs}ms` }}
          </button>

          <button
            @click="emit('adjust-offset', 500)"
            class="px-1.5 py-0.5 rounded hover:bg-white/15 bg-white/5 active:scale-95 transition text-[10px]"
            title="Speed up +0.5s"
          >
            +0.5s
          </button>
        </div>
      </div>

      <!-- Right: Action Buttons -->
      <div class="flex items-center space-x-1 ml-2">
        <!-- Full Lyrics on Hover Toggle -->
        <button
          type="button"
          @click="emit('toggle-full-lyrics')"
          class="flex items-center space-x-1 px-2 py-0.5 rounded transition active:scale-95 text-[11px]"
          :class="showFullLyrics ? 'bg-sky-500 text-white font-bold shadow-sm shadow-sky-500/30' : 'bg-slate-800/80 hover:bg-slate-700 text-slate-300 hover:text-white'"
          title="Full Lyrics / Songbook (Auto-returns to synced timestamp when mouse leaves)"
        >
          <ListMusic class="w-3 h-3" />
          <span>{{ showFullLyrics ? 'Synced' : 'Full' }}</span>
        </button>

        <!-- Docking Toggle -->
        <button
          type="button"
          @click="toggleDock"
          class="p-1 rounded hover:bg-white/15 transition active:scale-95"
          :class="dockPosition === 'bottom-taskbar' ? 'text-sky-400 bg-sky-500/20' : 'text-slate-400 hover:text-white'"
          :title="dockPosition === 'bottom-taskbar' ? 'Undock (Free Float)' : 'Dock right above Taskbar'"
        >
          <Anchor class="w-3.5 h-3.5" />
        </button>

        <!-- Edit LRC Lyric Button -->
        <button
          @click="emit('open-editor')"
          class="p-1 rounded hover:bg-white/15 text-slate-300 hover:text-white transition active:scale-95"
          title="Edit LRC Lyrics (Teks & Timestamp)"
        >
          <Edit3 class="w-3.5 h-3.5" />
        </button>

        <button
          @click="emit('open-search')"
          class="p-1 rounded hover:bg-white/15 text-slate-300 hover:text-white transition active:scale-95"
          title="Search Lyrics Online (LRCLIB)"
        >
          <Search class="w-3.5 h-3.5" />
        </button>

        <button
          @click="emit('open-settings')"
          class="p-1 rounded hover:bg-white/15 text-slate-300 hover:text-white transition active:scale-95"
          title="Lyrics Appearance & Settings"
        >
          <Settings class="w-3.5 h-3.5" />
        </button>

        <button
          @click="emit('lock')"
          class="flex items-center space-x-1 px-2 py-0.5 rounded bg-sky-500/20 hover:bg-sky-500/30 text-sky-300 border border-sky-500/30 transition active:scale-95"
          title="Lock Window (Click-Through mode). Unlock anytime via System Tray!"
        >
          <Lock class="w-3 h-3" />
          <span class="text-[11px]">Lock</span>
        </button>

        <div class="h-3 w-px bg-white/10 mx-1"></div>

        <button
          @click="minimizeWindow"
          class="p-1 rounded hover:bg-white/15 text-slate-400 hover:text-white transition"
          title="Minimize to System Tray"
        >
          <Minus class="w-3.5 h-3.5" />
        </button>

        <button
          @click="closeApp"
          class="p-1 rounded hover:bg-red-500/30 text-slate-400 hover:text-red-300 transition"
          title="Hide to System Tray (Restore anytime from Tray icon)"
        >
          <X class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Mini Progress Bar along the bottom of the toolbar -->
    <div
      v-if="durationMs && durationMs > 0"
      @click="handleProgressClick"
      class="w-full h-1 bg-white/10 hover:h-2 transition-all cursor-pointer overflow-hidden relative group"
      :title="`Seek: ${formatTime(currentPositionMs || 0)} / ${formatTime(durationMs)}`"
    >
      <div
        class="h-full bg-gradient-to-r from-sky-500 to-sky-400 transition-all duration-100"
        :style="{ width: `${progressPct}%` }"
      ></div>
    </div>
  </div>
</template>
