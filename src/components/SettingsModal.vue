<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Settings, X, RotateCcw, Palette, Type, LayoutTemplate, Sparkles, Eye, Check, Languages, Sliders, Anchor } from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { AppConfig } from '../types';

const props = withDefaults(
  defineProps<{
    config?: AppConfig;
    standalone?: boolean;
  }>(),
  {
    standalone: false,
  }
);

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'update-config', newConfig: AppConfig): void;
}>();

const isSaved = ref(false);

const localConfig = ref<AppConfig>({
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
  closeWithAimp: false,
  lrclibAppId: '',
  ...(props.config || {}),
});

const fontFamilies = [
  { name: 'Inter (Modern Clean)', value: 'Inter, system-ui, sans-serif' },
  { name: 'Segoe UI (Windows Default)', value: "'Segoe UI', system-ui, sans-serif" },
  { name: 'Roboto (Material)', value: 'Roboto, sans-serif' },
  { name: 'Poppins (Rounded Modern)', value: 'Poppins, sans-serif' },
  { name: 'Montserrat (Bold Clean)', value: 'Montserrat, sans-serif' },
  { name: 'Arial (Classic Sans)', value: 'Arial, sans-serif' },
  { name: 'Trebuchet MS (Humanist)', value: "'Trebuchet MS', sans-serif" },
];

const colorPresets = [
  { name: 'Sky Blue', hex: '#38BDF8' },
  { name: 'Sunset Gold', hex: '#FBBF24' },
  { name: 'Pure White', hex: '#FFFFFF' },
  { name: 'Neon Green', hex: '#4ADE80' },
  { name: 'Hot Pink', hex: '#F472B6' },
  { name: 'Electric Violet', hex: '#A78BFA' },
];

const strokeColorPresets = [
  { name: 'Pitch Black', hex: '#000000' },
  { name: 'Slate Deep', hex: '#0f172a' },
  { name: 'Midnight Navy', hex: '#1e1b4b' },
  { name: 'Dark Gray', hex: '#374151' },
  { name: 'White Glow', hex: '#FFFFFF' },
];

const availableMonitors = ref<any[]>([]);

const loadFreshConfig = async () => {
  try {
    const cfg = await invoke<AppConfig>('get_app_config');
    if (cfg) {
      localConfig.value = { ...cfg };
    }
  } catch (err) {
    if (props.config) {
      localConfig.value = { ...props.config };
    }
  }
};

onMounted(async () => {
  await loadFreshConfig();
  try {
    const monitors = await invoke<any[]>('get_available_monitors');
    if (monitors && monitors.length > 0) {
      availableMonitors.value = monitors;
    }
  } catch (_) {}

  try {
    await listen<AppConfig>('config://updated', (event) => {
      localConfig.value = { ...event.payload };
    });
  } catch (_) {}
});

const commitConfig = async () => {
  localConfig.value.fontSize = Number(localConfig.value.fontSize) || 28;
  localConfig.value.strokeWidth = Number(localConfig.value.strokeWidth) || 0;

  emit('update-config', { ...localConfig.value });
  try {
    await invoke('save_app_config', { config: localConfig.value });
    isSaved.value = true;
    setTimeout(() => {
      isSaved.value = false;
    }, 1500);
  } catch (err) {
    console.error('Failed to save config:', err);
  }
};

const updateMode = (mode: 'two-line' | 'single-line' | 'karaoke') => {
  localConfig.value.displayMode = mode;
  commitConfig();
};

const updateAlign = (align: 'center' | 'left' | 'right') => {
  localConfig.value.textAlign = align;
  commitConfig();
};

const updateActiveColor = (hex: string) => {
  localConfig.value.activeColor = hex;
  commitConfig();
};

const updateStrokeColor = (hex: string) => {
  localConfig.value.strokeColor = hex;
  commitConfig();
};

const updateDock = async (pos: string) => {
  localConfig.value.dockPosition = pos as any;
  commitConfig();
  try {
    await invoke('dock_window', { position: pos });
  } catch (err) {
    console.error('Failed to dock window:', err);
  }
};

const handleSliderChange = () => {
  commitConfig();
};

const closeSettings = async () => {
  await commitConfig();
  try {
    await invoke('hide_window', { label: 'settings' });
  } catch (_) {
    emit('close');
  }
};

const resetDefaults = () => {
  localConfig.value = {
    ...localConfig.value,
    displayMode: 'two-line',
    fontSize: 28,
    fontFamily: 'Inter, system-ui, sans-serif',
    activeColor: '#38BDF8',
    secondaryColor: 'rgba(255, 255, 255, 0.65)',
    strokeColor: '#000000',
    strokeWidth: 2,
    textAlign: 'center',
    saveToAudioFolder: true,
    closeWithAimp: false,
    enabledSources: ['local', 'lrclib', 'netease', 'lrcred', 'cache'],
    sourcePriority: ['local', 'lrclib', 'netease', 'lrcred', 'cache'],
  };
  commitConfig();
};

const allProviders = [
  {
    id: 'local',
    name: 'Folder Audio Lokal (.lrc)',
    badge: 'Local',
    desc: 'Lirik dari file .lrc di folder lagu yang sama',
    iconColor: 'text-amber-400 border-amber-500/40 bg-amber-500/10',
  },
  {
    id: 'lrclib',
    name: 'LRCLIB Online Database',
    badge: 'Global',
    desc: 'Database lirik sinkron terbuka resmi (lrclib.net)',
    iconColor: 'text-sky-400 border-sky-500/40 bg-sky-500/10',
  },
  {
    id: 'netease',
    name: 'NetEase Cloud Music',
    badge: 'Bilingual & Romaji',
    desc: 'Database musik Asia, Romaji, Pinyin, & Terjemahan',
    iconColor: 'text-rose-400 border-rose-500/40 bg-rose-500/10',
  },
  {
    id: 'lrcred',
    name: 'LRC.RED Provider',
    badge: 'Word Sync',
    desc: 'Penyedia lirik sinkronisasi presisi kata (lrc.red)',
    iconColor: 'text-purple-400 border-purple-500/40 bg-purple-500/10',
  },
  {
    id: 'cache',
    name: 'Offline AppData Cache',
    badge: 'Offline',
    desc: 'Lirik yang pernah diunduh sebelumnya di cache aplikasi',
    iconColor: 'text-emerald-400 border-emerald-500/40 bg-emerald-500/10',
  },
];

const toggleSource = (sourceId: string) => {
  if (!localConfig.value.enabledSources) {
    localConfig.value.enabledSources = ['local', 'lrclib', 'netease', 'lrcred', 'cache'];
  }
  const idx = localConfig.value.enabledSources.indexOf(sourceId);
  if (idx >= 0) {
    // Keep at least one source enabled
    if (localConfig.value.enabledSources.length > 1) {
      localConfig.value.enabledSources.splice(idx, 1);
    }
  } else {
    localConfig.value.enabledSources.push(sourceId);
  }
  handleSliderChange();
};

const isSourceEnabled = (sourceId: string) => {
  if (!localConfig.value.enabledSources) return true;
  return localConfig.value.enabledSources.includes(sourceId);
};

const moveSourcePriority = (sourceId: string, direction: 'up' | 'down') => {
  if (!localConfig.value.sourcePriority) {
    localConfig.value.sourcePriority = ['local', 'lrclib', 'netease', 'lrcred', 'cache'];
  }
  const list = [...localConfig.value.sourcePriority];
  const idx = list.indexOf(sourceId);
  if (idx < 0) return;
  const targetIdx = direction === 'up' ? idx - 1 : idx + 1;
  if (targetIdx < 0 || targetIdx >= list.length) return;

  const [moved] = list.splice(idx, 1);
  list.splice(targetIdx, 0, moved);
  localConfig.value.sourcePriority = list;
  handleSliderChange();
};

const resetSourcePriority = () => {
  localConfig.value.enabledSources = ['local', 'lrclib', 'netease', 'lrcred', 'cache'];
  localConfig.value.sourcePriority = ['local', 'lrclib', 'netease', 'lrcred', 'cache'];
  handleSliderChange();
};

const isClearingCache = ref(false);
const cacheClearedMessage = ref('');

const clearOfflineCache = async () => {
  if (!confirm('Hapus semua file lirik .lrc yang tersimpan di cache offline? Lirik lokal di samping file lagu tidak akan terpengaruh.')) {
    return;
  }
  isClearingCache.value = true;
  try {
    const count = await invoke<number>('clear_cache');
    cacheClearedMessage.value = `Berhasil menghapus ${count} file lirik dari cache!`;
    setTimeout(() => {
      cacheClearedMessage.value = '';
    }, 3500);
  } catch (err) {
    console.error('Failed to clear cache:', err);
  } finally {
    isClearingCache.value = false;
  }
};
</script>

<template>
  <!-- Standalone Window View -->
  <div
    v-if="standalone"
    class="w-full h-screen bg-slate-900 flex flex-col text-slate-100 select-none overflow-hidden"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-5 py-3.5 bg-slate-900/90 border-b border-slate-800">
      <div class="flex items-center space-x-2.5">
        <Settings class="w-5 h-5 text-sky-400" />
        <h2 class="font-bold text-sm tracking-wide text-white">Appearance & Settings</h2>
      </div>
      <button
        @click="closeSettings"
        class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition"
        title="Close"
      >
        <X class="w-4 h-4" />
      </button>
    </div>

    <!-- Scrollable Content -->
    <div class="flex-1 p-5 space-y-5 overflow-y-auto text-xs">
      <!-- Live Interactive Preview Card -->
      <div class="p-4 bg-slate-950/80 border border-slate-700/80 rounded-2xl shadow-inner relative overflow-hidden">
        <div class="flex items-center justify-between mb-2 pb-2 border-b border-slate-800/80">
          <div class="flex items-center space-x-1.5 text-[11px] font-semibold text-slate-300">
            <Eye class="w-3.5 h-3.5 text-sky-400" />
            <span>Live Preview</span>
          </div>
          <div class="flex items-center space-x-2">
            <span v-if="isSaved" class="text-[10px] text-emerald-400 font-medium flex items-center space-x-1 animate-pulse">
              <Check class="w-3 h-3" />
              <span>Saved</span>
            </span>
            <span class="text-[10px] font-mono text-slate-500 uppercase">{{ localConfig.displayMode }}</span>
          </div>
        </div>

        <div
          class="py-3 px-2 flex flex-col justify-center min-h-[76px] transition-all duration-200"
          :style="{
            fontFamily: localConfig.fontFamily || 'sans-serif',
            textAlign: localConfig.textAlign,
            alignItems: localConfig.textAlign === 'left' ? 'flex-start' : (localConfig.textAlign === 'right' ? 'flex-end' : 'center'),
          }"
        >
          <!-- Two-Line Mode Preview -->
          <div v-if="localConfig.displayMode === 'two-line'" class="space-y-1 w-full">
            <div
              class="font-bold tracking-tight lyric-stroke-thick truncate w-full"
              :style="{
                fontSize: `${Math.min(localConfig.fontSize, 34)}px`,
                color: localConfig.activeColor,
                '--stroke-color': localConfig.strokeColor,
                '--stroke-width': `${localConfig.strokeWidth}px`,
              }"
            >
              🎵 Never gonna give you up
            </div>
            <div
              v-if="localConfig.showDualLyrics"
              class="font-medium italic tracking-wide text-sky-200/90 lyric-stroke truncate w-full"
              :style="{ fontSize: `${Math.max(Math.min(localConfig.fontSize, 34) * 0.52, 11)}px` }"
            >
              {{ localConfig.dualLyricsType === 'romaji' ? '🔤 Kimi wo kesshite akiramenai' : '🌐 Takkan pernah menyerah padamu' }}
            </div>
            <div
              class="font-medium tracking-normal lyric-stroke truncate w-full"
              :style="{
                fontSize: `${Math.max(Math.min(localConfig.fontSize, 34) * 0.72, 12)}px`,
                color: localConfig.secondaryColor,
                '--stroke-color': localConfig.strokeColor,
                '--stroke-width': `${localConfig.strokeWidth}px`,
              }"
            >
              Never gonna let you down
            </div>
          </div>

          <!-- Single-Line Mode Preview -->
          <div v-else-if="localConfig.displayMode === 'single-line'" class="w-full">
            <div
              class="font-bold tracking-tight lyric-stroke-thick truncate w-full"
              :style="{
                fontSize: `${Math.min(localConfig.fontSize, 34)}px`,
                color: localConfig.activeColor,
                '--stroke-color': localConfig.strokeColor,
                '--stroke-width': `${localConfig.strokeWidth}px`,
              }"
            >
              🎵 Never gonna run around and desert you
            </div>
          </div>

          <!-- Karaoke Mode Preview -->
          <div v-else class="space-y-1 w-full">
            <div
              class="font-bold tracking-tight lyric-stroke-thick karaoke-fill truncate w-full"
              :style="{
                fontSize: `${Math.min(localConfig.fontSize, 34)}px`,
                '--active-color': localConfig.activeColor,
                '--secondary-color': localConfig.secondaryColor,
                '--stroke-color': localConfig.strokeColor,
                '--stroke-width': `${localConfig.strokeWidth}px`,
                '--progress': '65%',
              }"
            >
              🎵 Never gonna make you cry
            </div>
            <div
              class="font-medium lyric-stroke truncate w-full"
              :style="{
                fontSize: `${Math.max(Math.min(localConfig.fontSize, 34) * 0.72, 12)}px`,
                color: localConfig.secondaryColor,
                '--stroke-color': localConfig.strokeColor,
                '--stroke-width': `${localConfig.strokeWidth}px`,
              }"
            >
              Never gonna say goodbye
            </div>
          </div>
        </div>
      </div>

      <!-- Display Mode -->
      <div class="space-y-2">
        <label class="flex items-center space-x-1.5 font-semibold text-slate-300">
          <LayoutTemplate class="w-3.5 h-3.5 text-sky-400" />
          <span>Display Mode</span>
        </label>
        <div class="grid grid-cols-3 gap-2.5">
          <button
            v-for="mode in [
              { id: 'two-line', label: 'Two-Line Classic' },
              { id: 'single-line', label: 'Single-Line Fade' },
              { id: 'karaoke', label: 'Karaoke Sweep' }
            ]"
            :key="mode.id"
            type="button"
            @click="updateMode(mode.id as any)"
            class="p-2.5 rounded-xl border text-center transition font-medium"
            :class="localConfig.displayMode === mode.id
              ? 'bg-sky-500/20 border-sky-400 text-sky-200 shadow-sm shadow-sky-500/20'
              : 'bg-slate-800/60 border-slate-700/80 text-slate-300 hover:bg-slate-800'"
          >
            {{ mode.label }}
          </button>
        </div>
      </div>

      <!-- Font Family Dropdown -->
      <div class="space-y-2">
        <label class="flex items-center space-x-1.5 font-semibold text-slate-300">
          <Type class="w-3.5 h-3.5 text-sky-400" />
          <span>Font Family</span>
        </label>
        <select
          v-model="localConfig.fontFamily"
          @change="handleSliderChange"
          class="w-full bg-slate-800 border border-slate-700/80 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-sky-400 cursor-pointer"
        >
          <option v-for="font in fontFamilies" :key="font.value" :value="font.value">
            {{ font.name }}
          </option>
        </select>
      </div>

      <!-- Font Size Slider -->
      <div class="space-y-2">
        <div class="flex items-center justify-between font-semibold text-slate-300">
          <label class="flex items-center space-x-1.5">
            <Type class="w-3.5 h-3.5 text-sky-400" />
            <span>Font Size</span>
          </label>
          <span class="font-mono text-sky-400 text-sm font-bold">{{ localConfig.fontSize }}px</span>
        </div>
        <input
          v-model.number="localConfig.fontSize"
          @input="handleSliderChange"
          type="range"
          min="16"
          max="54"
          step="1"
          class="w-full accent-sky-400 h-2 bg-slate-700 rounded-lg cursor-pointer"
        />
      </div>

      <!-- Text Outline / Stroke -->
      <div class="space-y-2.5 pt-3 border-t border-slate-800">
        <div class="flex items-center justify-between font-semibold text-slate-300">
          <label class="flex items-center space-x-1.5">
            <Sparkles class="w-3.5 h-3.5 text-sky-400" />
            <span>Outline Stroke Width</span>
          </label>
          <span class="font-mono text-sky-400 text-sm font-bold">{{ localConfig.strokeWidth }}px</span>
        </div>
        <input
          v-model.number="localConfig.strokeWidth"
          @input="handleSliderChange"
          type="range"
          min="0"
          max="6"
          step="0.5"
          class="w-full accent-sky-400 h-2 bg-slate-700 rounded-lg cursor-pointer"
        />

        <div class="space-y-1.5">
          <label class="text-[11px] font-semibold text-slate-400">Outline Stroke Color</label>
          <div class="flex items-center space-x-3">
            <button
              v-for="color in strokeColorPresets"
              :key="color.hex"
              type="button"
              @click="updateStrokeColor(color.hex)"
              class="w-6 h-6 rounded-full border border-white/20 transition-transform hover:scale-110 relative"
              :style="{ backgroundColor: color.hex }"
              :title="color.name"
            >
              <span
                v-if="localConfig.strokeColor.toLowerCase() === color.hex.toLowerCase()"
                class="absolute inset-0 flex items-center justify-center font-bold text-[10px]"
                :class="color.hex === '#FFFFFF' ? 'text-black' : 'text-white'"
              >✓</span>
            </button>
            <input
              v-model="localConfig.strokeColor"
              @input="handleSliderChange"
              type="color"
              class="w-6 h-6 rounded cursor-pointer bg-transparent border-0"
              title="Custom Stroke Color"
            />
          </div>
        </div>
      </div>

      <!-- Text Alignment -->
      <div class="space-y-2">
        <label class="font-semibold text-slate-300">Text Alignment</label>
        <div class="grid grid-cols-3 gap-2.5">
          <button
            v-for="align in ['left', 'center', 'right']"
            :key="align"
            type="button"
            @click="updateAlign(align as any)"
            class="py-2 rounded-lg border capitalize text-center transition font-medium"
            :class="localConfig.textAlign === align
              ? 'bg-sky-500/20 border-sky-400 text-sky-200'
              : 'bg-slate-800/60 border-slate-700/80 text-slate-300 hover:bg-slate-800'"
          >
            {{ align }}
          </button>
        </div>
      </div>

      <!-- Active Color Presets -->
      <div class="space-y-2">
        <label class="flex items-center space-x-1.5 font-semibold text-slate-300">
          <Palette class="w-3.5 h-3.5 text-sky-400" />
          <span>Active Lyric Color</span>
        </label>
        <div class="flex items-center space-x-3">
          <button
            v-for="color in colorPresets"
            :key="color.hex"
            type="button"
            @click="updateActiveColor(color.hex)"
            class="w-7 h-7 rounded-full border border-white/20 transition-transform hover:scale-110 relative"
            :style="{ backgroundColor: color.hex }"
            :title="color.name"
          >
            <span
              v-if="localConfig.activeColor.toLowerCase() === color.hex.toLowerCase()"
              class="absolute inset-0 flex items-center justify-center text-black font-bold text-xs"
            >✓</span>
          </button>
          <input
            v-model="localConfig.activeColor"
            @input="handleSliderChange"
            type="color"
            class="w-7 h-7 rounded cursor-pointer bg-transparent border-0"
            title="Custom Hex Color"
          />
        </div>
      </div>

      <!-- Dual Lyrics / Translation & Romaji -->
      <div class="space-y-3 pt-3 border-t border-slate-800">
        <div class="flex items-center justify-between">
          <label class="flex items-center space-x-1.5 font-semibold text-slate-300">
            <Languages class="w-3.5 h-3.5 text-sky-400" />
            <span>Show Dual Lyrics (Translation / Romaji)</span>
          </label>
          <input
            v-model="localConfig.showDualLyrics"
            @change="handleSliderChange"
            type="checkbox"
            class="w-4 h-4 rounded text-sky-500 accent-sky-500 bg-slate-800 border-slate-700 cursor-pointer"
          />
        </div>
        <div v-if="localConfig.showDualLyrics" class="space-y-1.5 pl-5">
          <div class="text-[11px] text-slate-400">Pilihan Baris Sub-Lirik:</div>
          <div class="grid grid-cols-2 gap-2">
            <button
              type="button"
              @click="localConfig.dualLyricsType = 'translation'; commitConfig()"
              class="py-1.5 px-3 rounded-lg border text-center transition font-medium text-xs"
              :class="localConfig.dualLyricsType !== 'romaji'
                ? 'bg-sky-500/20 border-sky-400 text-sky-200'
                : 'bg-slate-800/60 border-slate-700 text-slate-400 hover:bg-slate-800'"
            >
              🌐 Translation (Arti)
            </button>
            <button
              type="button"
              @click="localConfig.dualLyricsType = 'romaji'; commitConfig()"
              class="py-1.5 px-3 rounded-lg border text-center transition font-medium text-xs"
              :class="localConfig.dualLyricsType === 'romaji'
                ? 'bg-sky-500/20 border-sky-400 text-sky-200'
                : 'bg-slate-800/60 border-slate-700 text-slate-400 hover:bg-slate-800'"
            >
              🔤 Romaji / Pinyin
            </button>
          </div>
        </div>
      </div>

      <!-- Background & Glassmorphism -->
      <div class="space-y-2.5 pt-3 border-t border-slate-800">
        <div class="flex items-center justify-between font-semibold text-slate-300">
          <label class="flex items-center space-x-1.5">
            <Sliders class="w-3.5 h-3.5 text-sky-400" />
            <span>Background Opacity</span>
          </label>
          <span class="font-mono text-sky-400 text-sm font-bold">{{ Math.round((localConfig.backgroundOpacity ?? 0.15) * 100) }}%</span>
        </div>
        <input
          v-model.number="localConfig.backgroundOpacity"
          @input="handleSliderChange"
          type="range"
          min="0"
          max="1"
          step="0.05"
          class="w-full accent-sky-400 h-2 bg-slate-700 rounded-lg cursor-pointer"
        />

        <div class="flex items-center justify-between pt-1">
          <div>
            <div class="font-semibold text-slate-200 text-xs">Enable Glassmorphism (Frosted Glass Blur)</div>
            <div class="text-[11px] text-slate-400">Efek blur latar belakang transparan modern Windows 11</div>
          </div>
          <input
            v-model="localConfig.enableGlassmorphism"
            @change="handleSliderChange"
            type="checkbox"
            class="w-4 h-4 rounded text-sky-500 accent-sky-500 bg-slate-800 border-slate-700 cursor-pointer"
          />
        </div>
      </div>

      <!-- Screen Docking Mode -->
      <div class="space-y-2 pt-3 border-t border-slate-800">
        <label class="flex items-center space-x-1.5 font-semibold text-slate-300">
          <Anchor class="w-3.5 h-3.5 text-sky-400" />
          <span>Screen Docking Position</span>
        </label>
        <div class="grid grid-cols-3 gap-2">
          <button
            v-for="dock in [
              { id: 'none', label: '📌 Free Float' },
              { id: 'bottom-taskbar', label: '⬇️ Above Taskbar' },
              { id: 'top-screen', label: '⬆️ Top Screen' }
            ]"
            :key="dock.id"
            type="button"
            @click="updateDock(dock.id)"
            class="py-2 px-1 rounded-xl border text-center transition font-medium text-[11px]"
            :class="(localConfig.dockPosition || 'none') === dock.id
              ? 'bg-sky-500/20 border-sky-400 text-sky-200'
              : 'bg-slate-800/60 border-slate-700/80 text-slate-400 hover:bg-slate-800'"
          >
            {{ dock.label }}
          </button>
        </div>

        <!-- Target Monitor Selector -->
        <div v-if="availableMonitors.length > 1" class="space-y-1 pt-1.5">
          <label class="text-[11px] font-semibold text-slate-400">Target Display Monitor:</label>
          <select
            v-model="localConfig.targetMonitor"
            @change="handleSliderChange"
            class="w-full bg-slate-800 border border-slate-700/80 rounded-xl px-3 py-1.5 text-xs text-white focus:outline-none focus:border-sky-400 cursor-pointer"
          >
            <option :value="null">Default (Auto / Active Window Monitor)</option>
            <option v-for="m in availableMonitors" :key="m.index" :value="m.index">
              {{ m.name }} ({{ m.width }}x{{ m.height }}{{ m.isPrimary ? ' - Primary' : '' }})
            </option>
          </select>
        </div>
      </div>

      <!-- Local File Auto-Save Toggle -->
      <div class="pt-3 border-t border-slate-800 flex items-center justify-between">
        <div>
          <div class="font-semibold text-slate-200 text-xs">Auto-Save .lrc Alongside Audio</div>
          <div class="text-[11px] text-slate-400">Simpan otomatis lirik yang diunduh ke folder lagu</div>
        </div>
        <input
          v-model="localConfig.saveToAudioFolder"
          @change="handleSliderChange"
          type="checkbox"
          class="w-4 h-4 rounded text-sky-500 accent-sky-500 bg-slate-800 border-slate-700 cursor-pointer"
        />
      </div>

      <!-- Auto-Close When AIMP Exits -->
      <div class="pt-3 border-t border-slate-800 flex items-center justify-between">
        <div>
          <div class="font-semibold text-slate-200 text-xs">Auto-Close When AIMP Exits</div>
          <div class="text-[11px] text-slate-400">Tutup otomatis Floating Lyrics saat pemutar musik AIMP di-close</div>
        </div>
        <input
          v-model="localConfig.closeWithAimp"
          @change="handleSliderChange"
          type="checkbox"
          class="w-4 h-4 rounded text-sky-500 accent-sky-500 bg-slate-800 border-slate-700 cursor-pointer"
        />
      </div>

      <!-- Lyrics Sources & Priority Selection -->
      <div class="pt-3 border-t border-slate-800 space-y-2">
        <div class="flex items-center justify-between">
          <div>
            <div class="font-semibold text-slate-200 text-xs">Pilihan Sumber & Prioritas Lirik (Lyrics Sources)</div>
            <div class="text-[11px] text-slate-400">Aktifkan sumber dan atur urutan prioritas pencarian lirik</div>
          </div>
          <button
            type="button"
            @click="resetSourcePriority"
            class="text-[10px] text-sky-400 hover:text-sky-300 underline font-medium transition"
          >
            Reset Prioritas
          </button>
        </div>

        <div class="space-y-1.5 pt-1">
          <div
            v-for="(srcId, pIdx) in (localConfig.sourcePriority || ['local', 'lrclib', 'netease', 'lrcred', 'cache'])"
            :key="srcId"
            class="flex items-center justify-between p-2 rounded-xl border transition-all"
            :class="isSourceEnabled(srcId)
              ? 'bg-slate-800/80 border-slate-700/80 text-white'
              : 'bg-slate-800/30 border-slate-800 text-slate-500 opacity-60'"
          >
            <div class="flex items-center space-x-2.5 min-w-0 flex-1 mr-2">
              <span
                class="w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold flex-shrink-0"
                :class="pIdx === 0
                  ? 'bg-sky-500/20 text-sky-400 border border-sky-500/40'
                  : 'bg-slate-700 text-slate-300'"
              >
                {{ pIdx + 1 }}
              </span>

              <div class="min-w-0 flex-1">
                <div class="flex items-center space-x-1.5">
                  <span class="font-semibold text-xs truncate">
                    {{ allProviders.find(p => p.id === srcId)?.name || srcId }}
                  </span>
                  <span
                    class="text-[9px] px-1.5 py-0.5 rounded border uppercase font-mono font-bold"
                    :class="allProviders.find(p => p.id === srcId)?.iconColor"
                  >
                    {{ allProviders.find(p => p.id === srcId)?.badge }}
                  </span>
                </div>
                <div class="text-[10px] text-slate-400 truncate">
                  {{ allProviders.find(p => p.id === srcId)?.desc }}
                </div>
              </div>
            </div>

            <div class="flex items-center space-x-1 flex-shrink-0">
              <button
                type="button"
                :disabled="pIdx === 0"
                @click="moveSourcePriority(srcId, 'up')"
                class="w-6 h-6 rounded flex items-center justify-center hover:bg-white/10 disabled:opacity-20 disabled:hover:bg-transparent transition text-slate-300 active:scale-90"
                title="Geser Prioritas Naik"
              >
                ▲
              </button>
              <button
                type="button"
                :disabled="pIdx === (localConfig.sourcePriority?.length || 5) - 1"
                @click="moveSourcePriority(srcId, 'down')"
                class="w-6 h-6 rounded flex items-center justify-center hover:bg-white/10 disabled:opacity-20 disabled:hover:bg-transparent transition text-slate-300 active:scale-90"
                title="Geser Prioritas Turun"
              >
                ▼
              </button>
              <input
                type="checkbox"
                :checked="isSourceEnabled(srcId)"
                @change="toggleSource(srcId)"
                class="w-4 h-4 rounded text-sky-500 accent-sky-500 bg-slate-800 border-slate-700 cursor-pointer ml-1"
                title="Aktifkan / Nonaktifkan Sumber"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- LRCLIB Application ID -->
      <div class="pt-3 border-t border-slate-800 space-y-1.5">
        <div class="flex items-center justify-between">
          <span class="font-semibold text-slate-200 text-xs">LRCLIB Application ID</span>
          <span class="text-[10px] text-sky-400 font-mono">Lrclib-App-Id</span>
        </div>
        <input
          v-model="localConfig.lrclibAppId"
          @input="handleSliderChange"
          type="text"
          placeholder="e.g. eJ3m5Pbtu1XIMGEEro1ykAFNPBrwf9tD2Q5f3Wzi42E"
          class="w-full bg-slate-800/80 border border-slate-700/80 rounded-lg px-3 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-sky-400 font-mono transition"
        />
        <div class="text-[11px] text-slate-400 leading-relaxed">
          Application ID dari LRCLIB Developer Portal. Jika ID belum aktif/terverifikasi, sistem otomatis fallback menggunakan client publik agar pencarian lirik tetap berjalan lancar.
        </div>
      </div>

      <!-- Offline Cache Management -->
      <div class="pt-3 border-t border-slate-800 space-y-1.5">
        <div class="flex items-center justify-between">
          <span class="font-semibold text-slate-200 text-xs">Offline Lyrics Cache</span>
          <button
            type="button"
            @click="clearOfflineCache"
            :disabled="isClearingCache"
            class="px-2.5 py-1 rounded-lg bg-rose-500/20 hover:bg-rose-500/30 text-rose-300 border border-rose-500/30 text-[11px] font-medium transition active:scale-95 disabled:opacity-50"
          >
            {{ isClearingCache ? 'Membersihkan...' : 'Bersihkan Cache' }}
          </button>
        </div>
        <div v-if="cacheClearedMessage" class="text-[11px] text-emerald-400 font-semibold animate-pulse">
          {{ cacheClearedMessage }}
        </div>
        <div v-else class="text-[11px] text-slate-400 leading-relaxed">
          Menghapus seluruh file lirik yang tersimpan di cache AppData. Gunakan fitur ini jika ada lirik salah yang terlanjur tersimpan di cache.
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="px-5 py-3.5 bg-slate-900 border-t border-slate-800 flex items-center justify-between">
      <button
        @click="resetDefaults"
        class="flex items-center space-x-1.5 text-xs text-slate-400 hover:text-white transition"
      >
        <RotateCcw class="w-3.5 h-3.5" />
        <span>Reset Defaults</span>
      </button>

      <button
        @click="closeSettings"
        class="px-5 py-2 rounded-xl bg-sky-500 hover:bg-sky-400 text-white font-semibold text-xs transition active:scale-95 shadow-md shadow-sky-500/20 flex items-center space-x-1.5"
      >
        <Check v-if="isSaved" class="w-3.5 h-3.5" />
        <span>{{ isSaved ? 'Saved!' : 'Done & Save' }}</span>
      </button>
    </div>
  </div>

  <!-- Legacy In-Window Modal Fallback -->
  <div
    v-else
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-md"
    @click.self="emit('close')"
  >
    <div
      class="bg-slate-900 border border-slate-700/80 rounded-2xl shadow-2xl w-full max-w-md overflow-hidden flex flex-col text-slate-100 text-xs animate-in fade-in zoom-in-95 duration-150"
    >
      <div class="flex items-center justify-between px-4 py-3 border-b border-slate-800">
        <div class="flex items-center space-x-2">
          <Settings class="w-4 h-4 text-sky-400" />
          <h2 class="font-bold text-sm tracking-wide text-white">Appearance & Settings</h2>
        </div>
        <button
          @click="emit('close')"
          class="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="p-4 space-y-4 max-h-[75vh] overflow-y-auto">
        <!-- Live Interactive Preview Card -->
        <div class="p-3 bg-slate-950/80 border border-slate-700/80 rounded-xl shadow-inner relative overflow-hidden">
          <div class="flex items-center justify-between mb-1.5 pb-1.5 border-b border-slate-800/80">
            <div class="flex items-center space-x-1.5 text-[10px] font-semibold text-slate-300">
              <Eye class="w-3 h-3 text-sky-400" />
              <span>Live Preview</span>
            </div>
            <span v-if="isSaved" class="text-[9px] text-emerald-400 font-medium flex items-center space-x-1 animate-pulse">
              <Check class="w-3 h-3" />
              <span>Saved</span>
            </span>
          </div>

          <div
            class="py-2 px-1 flex flex-col justify-center min-h-[60px] transition-all duration-200"
            :style="{
              fontFamily: localConfig.fontFamily || 'sans-serif',
              textAlign: localConfig.textAlign,
              alignItems: localConfig.textAlign === 'left' ? 'flex-start' : (localConfig.textAlign === 'right' ? 'flex-end' : 'center'),
            }"
          >
            <!-- Two-Line Mode Preview -->
            <div v-if="localConfig.displayMode === 'two-line'" class="space-y-0.5 w-full">
              <div
                class="font-bold tracking-tight lyric-stroke-thick truncate w-full"
                :style="{
                  fontSize: `${Math.min(localConfig.fontSize, 30)}px`,
                  color: localConfig.activeColor,
                  '--stroke-color': localConfig.strokeColor,
                  '--stroke-width': `${localConfig.strokeWidth}px`,
                }"
              >
                🎵 Never gonna give you up
              </div>
              <div
                class="font-medium tracking-normal lyric-stroke truncate w-full"
                :style="{
                  fontSize: `${Math.max(Math.min(localConfig.fontSize, 30) * 0.72, 11)}px`,
                  color: localConfig.secondaryColor,
                  '--stroke-color': localConfig.strokeColor,
                  '--stroke-width': `${localConfig.strokeWidth}px`,
                }"
              >
                Never gonna let you down
              </div>
            </div>

            <!-- Single-Line Mode Preview -->
            <div v-else-if="localConfig.displayMode === 'single-line'" class="w-full">
              <div
                class="font-bold tracking-tight lyric-stroke-thick truncate w-full"
                :style="{
                  fontSize: `${Math.min(localConfig.fontSize, 30)}px`,
                  color: localConfig.activeColor,
                  '--stroke-color': localConfig.strokeColor,
                  '--stroke-width': `${localConfig.strokeWidth}px`,
                }"
              >
                🎵 Never gonna run around and desert you
              </div>
            </div>

            <!-- Karaoke Mode Preview -->
            <div v-else class="space-y-0.5 w-full">
              <div
                class="font-bold tracking-tight lyric-stroke-thick karaoke-fill truncate w-full"
                :style="{
                  fontSize: `${Math.min(localConfig.fontSize, 30)}px`,
                  '--active-color': localConfig.activeColor,
                  '--secondary-color': localConfig.secondaryColor,
                  '--stroke-color': localConfig.strokeColor,
                  '--stroke-width': `${localConfig.strokeWidth}px`,
                  '--progress': '65%',
                }"
              >
                🎵 Never gonna make you cry
              </div>
              <div
                class="font-medium lyric-stroke truncate w-full"
                :style="{
                  fontSize: `${Math.max(Math.min(localConfig.fontSize, 30) * 0.72, 11)}px`,
                  color: localConfig.secondaryColor,
                  '--stroke-color': localConfig.strokeColor,
                  '--stroke-width': `${localConfig.strokeWidth}px`,
                }"
              >
                Never gonna say goodbye
              </div>
            </div>
          </div>
        </div>
        <!-- Display Mode -->
        <div class="space-y-1.5">
          <label class="flex items-center space-x-1.5 font-semibold text-slate-300">
            <LayoutTemplate class="w-3.5 h-3.5 text-sky-400" />
            <span>Display Mode</span>
          </label>
          <div class="grid grid-cols-3 gap-2">
            <button
              v-for="mode in [
                { id: 'two-line', label: 'Two-Line Classic' },
                { id: 'single-line', label: 'Single-Line Fade' },
                { id: 'karaoke', label: 'Karaoke Sweep' }
              ]"
              :key="mode.id"
              type="button"
              @click="updateMode(mode.id as any)"
              class="p-2 rounded-xl border text-center transition font-medium"
              :class="localConfig.displayMode === mode.id
                ? 'bg-sky-500/20 border-sky-400 text-sky-200'
                : 'bg-slate-800/60 border-slate-700/80 text-slate-300 hover:bg-slate-800'"
            >
              {{ mode.label }}
            </button>
          </div>
        </div>

        <!-- Font Family Dropdown -->
        <div class="space-y-1.5">
          <label class="flex items-center space-x-1.5 font-semibold text-slate-300">
            <Type class="w-3.5 h-3.5 text-sky-400" />
            <span>Font Family</span>
          </label>
          <select
            v-model="localConfig.fontFamily"
            @change="handleSliderChange"
            class="w-full bg-slate-800 border border-slate-700/80 rounded-lg px-2.5 py-1.5 text-xs text-white focus:outline-none focus:border-sky-400 cursor-pointer"
          >
            <option v-for="font in fontFamilies" :key="font.value" :value="font.value">
              {{ font.name }}
            </option>
          </select>
        </div>

        <!-- Font Size Slider -->
        <div class="space-y-1.5">
          <div class="flex items-center justify-between font-semibold text-slate-300">
            <label class="flex items-center space-x-1.5">
              <Type class="w-3.5 h-3.5 text-sky-400" />
              <span>Font Size</span>
            </label>
            <span class="font-mono text-sky-400">{{ localConfig.fontSize }}px</span>
          </div>
          <input
            v-model.number="localConfig.fontSize"
            @input="handleSliderChange"
            type="range"
            min="16"
            max="54"
            step="1"
            class="w-full accent-sky-400 h-1.5 bg-slate-700 rounded-lg cursor-pointer"
          />
        </div>

        <!-- Text Outline / Stroke -->
        <div class="space-y-2 pt-2 border-t border-slate-800">
          <div class="flex items-center justify-between font-semibold text-slate-300">
            <label class="flex items-center space-x-1.5">
              <Sparkles class="w-3.5 h-3.5 text-sky-400" />
              <span>Outline Stroke Width</span>
            </label>
            <span class="font-mono text-sky-400 text-xs font-bold">{{ localConfig.strokeWidth }}px</span>
          </div>
          <input
            v-model.number="localConfig.strokeWidth"
            @input="handleSliderChange"
            type="range"
            min="0"
            max="6"
            step="0.5"
            class="w-full accent-sky-400 h-1.5 bg-slate-700 rounded-lg cursor-pointer"
          />

          <div class="space-y-1">
            <label class="text-[10px] font-semibold text-slate-400">Outline Stroke Color</label>
            <div class="flex items-center space-x-2">
              <button
                v-for="color in strokeColorPresets"
                :key="color.hex"
                type="button"
                @click="updateStrokeColor(color.hex)"
                class="w-5 h-5 rounded-full border border-white/20 transition-transform hover:scale-110 relative"
                :style="{ backgroundColor: color.hex }"
                :title="color.name"
              >
                <span
                  v-if="localConfig.strokeColor.toLowerCase() === color.hex.toLowerCase()"
                  class="absolute inset-0 flex items-center justify-center font-bold text-[9px]"
                  :class="color.hex === '#FFFFFF' ? 'text-black' : 'text-white'"
                >✓</span>
              </button>
              <input
                v-model="localConfig.strokeColor"
                @input="handleSliderChange"
                type="color"
                class="w-5 h-5 rounded cursor-pointer bg-transparent border-0"
                title="Custom Stroke Color"
              />
            </div>
          </div>
        </div>

        <!-- Text Alignment -->
        <div class="space-y-1.5">
          <label class="font-semibold text-slate-300">Text Alignment</label>
          <div class="grid grid-cols-3 gap-2">
            <button
              v-for="align in ['left', 'center', 'right']"
              :key="align"
              type="button"
              @click="updateAlign(align as any)"
              class="py-1.5 rounded-lg border capitalize text-center transition font-medium"
              :class="localConfig.textAlign === align
                ? 'bg-sky-500/20 border-sky-400 text-sky-200'
                : 'bg-slate-800/60 border-slate-700/80 text-slate-300 hover:bg-slate-800'"
            >
              {{ align }}
            </button>
          </div>
        </div>

        <!-- Active Color Presets -->
        <div class="space-y-1.5">
          <label class="flex items-center space-x-1.5 font-semibold text-slate-300">
            <Palette class="w-3.5 h-3.5 text-sky-400" />
            <span>Active Lyric Color</span>
          </label>
          <div class="flex items-center space-x-2">
            <button
              v-for="color in colorPresets"
              :key="color.hex"
              type="button"
              @click="updateActiveColor(color.hex)"
              class="w-6 h-6 rounded-full border border-white/20 transition-transform hover:scale-110 relative"
              :style="{ backgroundColor: color.hex }"
              :title="color.name"
            >
              <span
                v-if="localConfig.activeColor.toLowerCase() === color.hex.toLowerCase()"
                class="absolute inset-0 flex items-center justify-center text-black font-bold text-[10px]"
              >✓</span>
            </button>
            <input
              v-model="localConfig.activeColor"
              @input="handleSliderChange"
              type="color"
              class="w-6 h-6 rounded cursor-pointer bg-transparent border-0"
              title="Custom Hex Color"
            />
          </div>
        </div>

        <!-- Dual Lyrics / Translation & Romaji -->
      <div class="space-y-3 pt-3 border-t border-slate-800">
        <div class="flex items-center justify-between">
          <label class="flex items-center space-x-1.5 font-semibold text-slate-300">
            <Languages class="w-3.5 h-3.5 text-sky-400" />
            <span>Show Dual Lyrics (Translation / Romaji)</span>
          </label>
          <input
            v-model="localConfig.showDualLyrics"
            @change="handleSliderChange"
            type="checkbox"
            class="w-4 h-4 rounded text-sky-500 accent-sky-500 bg-slate-800 border-slate-700 cursor-pointer"
          />
        </div>
        <div v-if="localConfig.showDualLyrics" class="space-y-1.5 pl-5">
          <div class="text-[11px] text-slate-400">Pilihan Baris Sub-Lirik:</div>
          <div class="grid grid-cols-2 gap-2">
            <button
              type="button"
              @click="localConfig.dualLyricsType = 'translation'; commitConfig()"
              class="py-1.5 px-3 rounded-lg border text-center transition font-medium text-xs"
              :class="localConfig.dualLyricsType !== 'romaji'
                ? 'bg-sky-500/20 border-sky-400 text-sky-200'
                : 'bg-slate-800/60 border-slate-700 text-slate-400 hover:bg-slate-800'"
            >
              🌐 Translation (Arti)
            </button>
            <button
              type="button"
              @click="localConfig.dualLyricsType = 'romaji'; commitConfig()"
              class="py-1.5 px-3 rounded-lg border text-center transition font-medium text-xs"
              :class="localConfig.dualLyricsType === 'romaji'
                ? 'bg-sky-500/20 border-sky-400 text-sky-200'
                : 'bg-slate-800/60 border-slate-700 text-slate-400 hover:bg-slate-800'"
            >
              🔤 Romaji / Pinyin
            </button>
          </div>
        </div>
      </div>

      <!-- Background & Glassmorphism -->
      <div class="space-y-2.5 pt-3 border-t border-slate-800">
        <div class="flex items-center justify-between font-semibold text-slate-300">
          <label class="flex items-center space-x-1.5">
            <Sliders class="w-3.5 h-3.5 text-sky-400" />
            <span>Background Opacity</span>
          </label>
          <span class="font-mono text-sky-400 text-sm font-bold">{{ Math.round((localConfig.backgroundOpacity ?? 0.15) * 100) }}%</span>
        </div>
        <input
          v-model.number="localConfig.backgroundOpacity"
          @input="handleSliderChange"
          type="range"
          min="0"
          max="1"
          step="0.05"
          class="w-full accent-sky-400 h-2 bg-slate-700 rounded-lg cursor-pointer"
        />

        <div class="flex items-center justify-between pt-1">
          <div>
            <div class="font-semibold text-slate-200 text-xs">Enable Glassmorphism (Frosted Glass Blur)</div>
            <div class="text-[11px] text-slate-400">Efek blur latar belakang transparan modern Windows 11</div>
          </div>
          <input
            v-model="localConfig.enableGlassmorphism"
            @change="handleSliderChange"
            type="checkbox"
            class="w-4 h-4 rounded text-sky-500 accent-sky-500 bg-slate-800 border-slate-700 cursor-pointer"
          />
        </div>
      </div>

      <!-- Screen Docking Mode -->
      <div class="space-y-2 pt-3 border-t border-slate-800">
        <label class="flex items-center space-x-1.5 font-semibold text-slate-300">
          <Anchor class="w-3.5 h-3.5 text-sky-400" />
          <span>Screen Docking Position</span>
        </label>
        <div class="grid grid-cols-3 gap-2">
          <button
            v-for="dock in [
              { id: 'none', label: '📌 Free Float' },
              { id: 'bottom-taskbar', label: '⬇️ Above Taskbar' },
              { id: 'top-screen', label: '⬆️ Top Screen' }
            ]"
            :key="dock.id"
            type="button"
            @click="updateDock(dock.id)"
            class="py-2 px-1 rounded-xl border text-center transition font-medium text-[11px]"
            :class="(localConfig.dockPosition || 'none') === dock.id
              ? 'bg-sky-500/20 border-sky-400 text-sky-200'
              : 'bg-slate-800/60 border-slate-700/80 text-slate-400 hover:bg-slate-800'"
          >
            {{ dock.label }}
          </button>
        </div>

        <!-- Target Monitor Selector -->
        <div v-if="availableMonitors.length > 1" class="space-y-1 pt-1.5">
          <label class="text-[11px] font-semibold text-slate-400">Target Display Monitor:</label>
          <select
            v-model="localConfig.targetMonitor"
            @change="handleSliderChange"
            class="w-full bg-slate-800 border border-slate-700/80 rounded-xl px-3 py-1.5 text-xs text-white focus:outline-none focus:border-sky-400 cursor-pointer"
          >
            <option :value="null">Default (Auto / Active Window Monitor)</option>
            <option v-for="m in availableMonitors" :key="m.index" :value="m.index">
              {{ m.name }} ({{ m.width }}x{{ m.height }}{{ m.isPrimary ? ' - Primary' : '' }})
            </option>
          </select>
        </div>
      </div>

      <!-- Local File Auto-Save Toggle -->
        <div class="pt-2 border-t border-slate-800 flex items-center justify-between">
          <div>
            <div class="font-semibold text-slate-200">Auto-Save .lrc Alongside Audio</div>
            <div class="text-[10px] text-slate-400">Save downloaded lyrics directly next to the audio file</div>
          </div>
          <input
            v-model="localConfig.saveToAudioFolder"
            @change="handleSliderChange"
            type="checkbox"
            class="w-4 h-4 rounded text-sky-500 accent-sky-500 bg-slate-800 border-slate-700 cursor-pointer"
          />
        </div>

        <!-- Lyrics Sources & Priority Selection -->
      <div class="pt-3 border-t border-slate-800 space-y-2">
        <div class="flex items-center justify-between">
          <div>
            <div class="font-semibold text-slate-200 text-xs">Pilihan Sumber & Prioritas Lirik (Lyrics Sources)</div>
            <div class="text-[11px] text-slate-400">Aktifkan sumber dan atur urutan prioritas pencarian lirik</div>
          </div>
          <button
            type="button"
            @click="resetSourcePriority"
            class="text-[10px] text-sky-400 hover:text-sky-300 underline font-medium transition"
          >
            Reset Prioritas
          </button>
        </div>

        <div class="space-y-1.5 pt-1">
          <div
            v-for="(srcId, pIdx) in (localConfig.sourcePriority || ['local', 'lrclib', 'netease', 'lrcred', 'cache'])"
            :key="srcId"
            class="flex items-center justify-between p-2 rounded-xl border transition-all"
            :class="isSourceEnabled(srcId)
              ? 'bg-slate-800/80 border-slate-700/80 text-white'
              : 'bg-slate-800/30 border-slate-800 text-slate-500 opacity-60'"
          >
            <div class="flex items-center space-x-2.5 min-w-0 flex-1 mr-2">
              <span
                class="w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold flex-shrink-0"
                :class="pIdx === 0
                  ? 'bg-sky-500/20 text-sky-400 border border-sky-500/40'
                  : 'bg-slate-700 text-slate-300'"
              >
                {{ pIdx + 1 }}
              </span>

              <div class="min-w-0 flex-1">
                <div class="flex items-center space-x-1.5">
                  <span class="font-semibold text-xs truncate">
                    {{ allProviders.find(p => p.id === srcId)?.name || srcId }}
                  </span>
                  <span
                    class="text-[9px] px-1.5 py-0.5 rounded border uppercase font-mono font-bold"
                    :class="allProviders.find(p => p.id === srcId)?.iconColor"
                  >
                    {{ allProviders.find(p => p.id === srcId)?.badge }}
                  </span>
                </div>
                <div class="text-[10px] text-slate-400 truncate">
                  {{ allProviders.find(p => p.id === srcId)?.desc }}
                </div>
              </div>
            </div>

            <div class="flex items-center space-x-1 flex-shrink-0">
              <button
                type="button"
                :disabled="pIdx === 0"
                @click="moveSourcePriority(srcId, 'up')"
                class="w-6 h-6 rounded flex items-center justify-center hover:bg-white/10 disabled:opacity-20 disabled:hover:bg-transparent transition text-slate-300 active:scale-90"
                title="Geser Prioritas Naik"
              >
                ▲
              </button>
              <button
                type="button"
                :disabled="pIdx === (localConfig.sourcePriority?.length || 5) - 1"
                @click="moveSourcePriority(srcId, 'down')"
                class="w-6 h-6 rounded flex items-center justify-center hover:bg-white/10 disabled:opacity-20 disabled:hover:bg-transparent transition text-slate-300 active:scale-90"
                title="Geser Prioritas Turun"
              >
                ▼
              </button>
              <input
                type="checkbox"
                :checked="isSourceEnabled(srcId)"
                @change="toggleSource(srcId)"
                class="w-4 h-4 rounded text-sky-500 accent-sky-500 bg-slate-800 border-slate-700 cursor-pointer ml-1"
                title="Aktifkan / Nonaktifkan Sumber"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- LRCLIB Application ID -->
        <div class="pt-2 border-t border-slate-800 space-y-1">
          <div class="flex items-center justify-between">
            <span class="font-semibold text-slate-200">LRCLIB Application ID</span>
            <span class="text-[10px] text-sky-400 font-mono">Lrclib-App-Id</span>
          </div>
          <input
            v-model="localConfig.lrclibAppId"
            @input="handleSliderChange"
            type="text"
            placeholder="e.g. eJ3m5Pbtu1XIMGEEro1ykAFNPBrwf9tD2Q5f3Wzi42E"
            class="w-full bg-slate-800/80 border border-slate-700/80 rounded-lg px-2.5 py-1.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-sky-400 font-mono transition"
          />
          <div class="text-[10px] text-slate-400">
            Mengumumkan Application ID ke LRCLIB API untuk verifikasi & bypass rate-limit. Juga dapat diisi via file <code class="text-sky-300">.env</code>.
          </div>
        </div>

        <!-- Offline Cache Management -->
        <div class="pt-2 border-t border-slate-800 space-y-1">
          <div class="flex items-center justify-between">
            <span class="font-semibold text-slate-200">Offline Lyrics Cache</span>
            <button
              type="button"
              @click="clearOfflineCache"
              :disabled="isClearingCache"
              class="px-2 py-0.5 rounded bg-rose-500/20 hover:bg-rose-500/30 text-rose-300 border border-rose-500/30 text-[10px] font-medium transition active:scale-95 disabled:opacity-50"
            >
              {{ isClearingCache ? 'Membersihkan...' : 'Bersihkan Cache' }}
            </button>
          </div>
          <div v-if="cacheClearedMessage" class="text-[10px] text-emerald-400 font-semibold animate-pulse">
            {{ cacheClearedMessage }}
          </div>
          <div v-else class="text-[10px] text-slate-400">
            Menghapus cache lirik offline di folder AppData.
          </div>
        </div>
      </div>

      <div class="px-4 py-3 bg-slate-800/50 border-t border-slate-800 flex items-center justify-between">
        <button
          @click="resetDefaults"
          class="flex items-center space-x-1 text-slate-400 hover:text-white transition"
        >
          <RotateCcw class="w-3 h-3" />
          <span>Reset Defaults</span>
        </button>

        <button
          @click="emit('close')"
          class="px-4 py-1.5 rounded-xl bg-sky-500 hover:bg-sky-400 text-white font-semibold transition active:scale-95"
        >
          Done
        </button>
      </div>
    </div>
  </div>
</template>
