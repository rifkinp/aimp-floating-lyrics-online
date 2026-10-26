<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Settings, X, RotateCcw, Palette, Type, LayoutTemplate } from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';
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

onMounted(async () => {
  if (props.config) {
    localConfig.value = { ...props.config };
  } else {
    try {
      const cfg = await invoke<AppConfig>('get_app_config');
      if (cfg) {
        localConfig.value = cfg;
      }
    } catch (e) {
      console.warn('Could not load config in Settings:', e);
    }
  }
});

const colorPresets = [
  { name: 'Sky Blue', hex: '#38BDF8' },
  { name: 'Sunset Gold', hex: '#FBBF24' },
  { name: 'Pure White', hex: '#FFFFFF' },
  { name: 'Neon Green', hex: '#4ADE80' },
  { name: 'Hot Pink', hex: '#F472B6' },
  { name: 'Electric Violet', hex: '#A78BFA' },
];

const commitConfig = async () => {
  emit('update-config', { ...localConfig.value });
  try {
    await invoke('save_app_config', { config: localConfig.value });
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
  };
  commitConfig();
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
        class="px-5 py-2 rounded-xl bg-sky-500 hover:bg-sky-400 text-white font-semibold text-xs transition active:scale-95 shadow-md shadow-sky-500/20"
      >
        Done & Save
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
