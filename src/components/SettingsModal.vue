<script setup lang="ts">
import { ref } from 'vue';
import { Settings, X, RotateCcw, Palette, Type, LayoutTemplate } from 'lucide-vue-next';
import type { AppConfig } from '../types';

const props = defineProps<{
  config: AppConfig;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'update-config', newConfig: AppConfig): void;
}>();

const localConfig = ref<AppConfig>({ ...props.config });

const colorPresets = [
  { name: 'Sky Blue', hex: '#38BDF8' },
  { name: 'Sunset Gold', hex: '#FBBF24' },
  { name: 'Pure White', hex: '#FFFFFF' },
  { name: 'Neon Green', hex: '#4ADE80' },
  { name: 'Hot Pink', hex: '#F472B6' },
  { name: 'Electric Violet', hex: '#A78BFA' },
];

const updateMode = (mode: 'two-line' | 'single-line' | 'karaoke') => {
  localConfig.value.displayMode = mode;
  emit('update-config', { ...localConfig.value });
};

const updateAlign = (align: 'center' | 'left' | 'right') => {
  localConfig.value.textAlign = align;
  emit('update-config', { ...localConfig.value });
};

const updateActiveColor = (hex: string) => {
  localConfig.value.activeColor = hex;
  emit('update-config', { ...localConfig.value });
};

const handleSliderChange = () => {
  emit('update-config', { ...localConfig.value });
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
  };
  emit('update-config', { ...localConfig.value });
};
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-md"
    @click.self="emit('close')"
  >
    <div
      class="bg-slate-900 border border-slate-700/80 rounded-2xl shadow-2xl w-full max-w-md overflow-hidden flex flex-col text-slate-100 text-xs animate-in fade-in zoom-in-95 duration-150"
    >
      <!-- Header -->
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
      </div>

      <!-- Footer -->
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
