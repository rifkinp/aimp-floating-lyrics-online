<script setup lang="ts">
import {
  GripVertical,
  Lock,
  Search,
  Settings,
  Minus,
  X,
  ListMusic,
  Anchor
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
  }>(),
  {
    showFullLyrics: false,
    dockPosition: 'none',
  }
);

const emit = defineEmits<{
  (e: 'lock'): void;
  (e: 'open-search'): void;
  (e: 'open-settings'): void;
  (e: 'adjust-offset', deltaMs: number): void;
  (e: 'toggle-full-lyrics'): void;
  (e: 'dock', position: string): void;
}>();

import { invoke } from '@tauri-apps/api/core';

const appWindow = getCurrentWebviewWindow();

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
  <div
    @mousedown.self="startDrag"
    class="flex items-center justify-between w-full px-3 py-1.5 bg-slate-900/85 backdrop-blur-md border border-white/10 rounded-xl shadow-2xl transition-all duration-200 select-none text-xs text-slate-200 font-medium"
  >
    <!-- Left: Drag Region & Song Info -->
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

      <div class="truncate flex items-center space-x-1.5 max-w-[260px] pointer-events-none">
        <span class="inline-block w-2 h-2 rounded-full" :class="track?.state === 2 ? 'bg-emerald-400 animate-pulse' : 'bg-amber-400'"></span>
        <span class="text-white font-semibold truncate">{{ track?.title || 'Waiting for AIMP...' }}</span>
        <span v-if="track?.artist" class="text-slate-400 truncate text-[11px]">- {{ track.artist }}</span>
      </div>
    </div>

    <!-- Center: Offset Adjustments -->
    <div class="flex items-center space-x-1 px-2 border-x border-white/10">
      <span class="text-[10px] uppercase tracking-wider text-slate-400 mr-1">Sync:</span>
      <button
        @click="emit('adjust-offset', -500)"
        class="px-1.5 py-0.5 rounded hover:bg-white/15 bg-white/5 active:scale-95 transition text-[11px]"
        title="Delay -0.5s"
      >
        -0.5s
      </button>
      <button
        @click="emit('adjust-offset', -100)"
        class="px-1.5 py-0.5 rounded hover:bg-white/15 bg-white/5 active:scale-95 transition text-[11px]"
        title="Delay -0.1s"
      >
        -0.1s
      </button>

      <button
        type="button"
        @click="emit('adjust-offset', -offsetMs)"
        class="px-1.5 py-0.5 rounded font-mono text-[11px] font-bold hover:bg-white/15 active:scale-95 transition cursor-pointer"
        :class="offsetMs === 0 ? 'text-slate-300' : offsetMs > 0 ? 'text-emerald-400' : 'text-amber-400'"
        title="Klik untuk langsung reset offset ke 0ms"
      >
        {{ offsetMs > 0 ? `+${offsetMs}ms` : `${offsetMs}ms` }}
      </button>

      <button
        @click="emit('adjust-offset', 100)"
        class="px-1.5 py-0.5 rounded hover:bg-white/15 bg-white/5 active:scale-95 transition text-[11px]"
        title="Speed up +0.1s"
      >
        +0.1s
      </button>
      <button
        @click="emit('adjust-offset', 500)"
        class="px-1.5 py-0.5 rounded hover:bg-white/15 bg-white/5 active:scale-95 transition text-[11px]"
        title="Speed up +0.5s"
      >
        +0.5s
      </button>
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
</template>
