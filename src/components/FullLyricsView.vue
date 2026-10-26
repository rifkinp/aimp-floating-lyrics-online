<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue';
import { Play } from 'lucide-vue-next';
import type { ParsedLyrics, AppConfig } from '../types';

const props = defineProps<{
  lyrics: ParsedLyrics;
  currentPositionMs: number;
  config: AppConfig;
}>();

const emit = defineEmits<{
  (e: 'seek', positionMs: number): void;
}>();

const scrollContainer = ref<HTMLElement | null>(null);

// Find active line index
const activeIndex = computed(() => {
  if (!props.lyrics || !props.lyrics.lines || props.lyrics.lines.length === 0) {
    return -1;
  }
  const lines = props.lyrics.lines;
  const pos = props.currentPositionMs + (props.lyrics.offsetMs || 0);

  let idx = -1;
  for (let i = 0; i < lines.length; i++) {
    if (lines[i].timeMs <= pos) {
      idx = i;
    } else {
      break;
    }
  }
  return idx;
});

// Auto-scroll active lyric into center view
watch(activeIndex, async (newIdx) => {
  if (newIdx < 0 || !scrollContainer.value) return;
  await nextTick();
  const el = scrollContainer.value.querySelector<HTMLElement>(`[data-line-idx="${newIdx}"]`);
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }
});

onMounted(async () => {
  await nextTick();
  if (activeIndex.value >= 0 && scrollContainer.value) {
    const el = scrollContainer.value.querySelector<HTMLElement>(`[data-line-idx="${activeIndex.value}"]`);
    if (el) {
      el.scrollIntoView({ behavior: 'auto', block: 'center' });
    }
  }
});

const formatTime = (ms: number) => {
  const totalSecs = Math.floor(ms / 1000);
  const m = Math.floor(totalSecs / 60);
  const s = totalSecs % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
};

const getSecondaryText = (line: any) => {
  if (!props.config.showDualLyrics) return null;
  if (props.config.dualLyricsType === 'romaji') {
    return line.romaji || line.translation;
  }
  return line.translation || line.romaji;
};
</script>

<template>
  <div
    ref="scrollContainer"
    class="w-full h-full overflow-y-auto px-4 py-2 space-y-1.5 select-none custom-scroll pointer-events-auto"
    :style="{
      fontFamily: config.fontFamily || 'sans-serif',
      textAlign: config.textAlign || 'center',
    }"
  >
    <div
      v-for="(line, idx) in lyrics.lines"
      :key="idx"
      :data-line-idx="idx"
      @click="emit('seek', line.timeMs)"
      class="group py-1.5 px-3 rounded-xl transition-all duration-200 cursor-pointer flex items-center gap-3"
      :class="[
        config.textAlign === 'center' ? 'justify-center' : (config.textAlign === 'right' ? 'justify-end' : 'justify-start'),
        idx === activeIndex
          ? 'bg-sky-500/20 border-l-4 border-sky-400 font-bold scale-[1.01]'
          : 'hover:bg-white/10 opacity-70 hover:opacity-100'
      ]"
    >
      <!-- Clickable Time Badge -->
      <span
        class="text-[10px] font-mono px-1.5 py-0.5 rounded opacity-50 group-hover:opacity-100 transition flex items-center space-x-1"
        :class="idx === activeIndex ? 'bg-sky-400/30 text-sky-200' : 'bg-slate-800 text-slate-300'"
      >
        <Play v-if="idx === activeIndex" class="w-2.5 h-2.5 fill-current" />
        <span>{{ formatTime(line.timeMs) }}</span>
      </span>

      <!-- Lyric Text Content -->
      <div class="flex flex-col">
        <span
          class="tracking-wide lyric-stroke transition-colors text-sm"
          :style="{
            color: idx === activeIndex ? config.activeColor : config.secondaryColor,
            fontSize: idx === activeIndex ? `${Math.min(config.fontSize * 0.75, 20)}px` : `${Math.min(config.fontSize * 0.6, 16)}px`,
            '--stroke-color': config.strokeColor,
            '--stroke-width': `${config.strokeWidth * 0.6}px`,
          }"
        >
          {{ line.text }}
        </span>

        <!-- Optional Dual / Translated / Romaji Sub-line -->
        <span
          v-if="getSecondaryText(line)"
          class="text-xs text-sky-200/80 italic tracking-normal mt-0.5"
        >
          {{ getSecondaryText(line) }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scroll::-webkit-scrollbar {
  width: 4px;
}
.custom-scroll::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scroll::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 4px;
}
.custom-scroll::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.4);
}
</style>
