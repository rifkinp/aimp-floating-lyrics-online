<script setup lang="ts">
import { computed } from 'vue';
import type { AimpTrackInfo, AppConfig, LyricLine, ParsedLyrics } from '../types';

const props = defineProps<{
  track?: AimpTrackInfo | null;
  lyrics: ParsedLyrics | null;
  currentPositionMs: number;
  config: AppConfig;
  isLoading: boolean;
}>();

// Compute active line index
const activeIndex = computed(() => {
  if (!props.lyrics || !props.lyrics.lines || props.lyrics.lines.length === 0) {
    return -1;
  }
  const lines = props.lyrics.lines;
  const pos = props.currentPositionMs + (props.lyrics.offsetMs || 0);

  // Binary or linear search for active line
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

const currentLine = computed<LyricLine | null>(() => {
  if (activeIndex.value >= 0 && props.lyrics?.lines) {
    return props.lyrics.lines[activeIndex.value] || null;
  }
  return null;
});

const nextLine = computed<LyricLine | null>(() => {
  if (activeIndex.value >= 0 && props.lyrics?.lines) {
    return props.lyrics.lines[activeIndex.value + 1] || null;
  }
  if (activeIndex.value === -1 && props.lyrics?.lines && props.lyrics.lines.length > 0) {
    return props.lyrics.lines[0];
  }
  return null;
});

// Calculate karaoke line progress percentage (0 - 100%)
const lineProgress = computed(() => {
  if (!currentLine.value || !props.lyrics?.lines) return 0;
  const start = currentLine.value.timeMs;
  const next = nextLine.value?.timeMs ?? (start + 5000);
  const dur = Math.max(next - start, 500);
  const pos = props.currentPositionMs + (props.lyrics.offsetMs || 0);

  const pct = ((pos - start) / dur) * 100;
  return Math.min(Math.max(pct, 0), 100);
});

const styleVariables = computed(() => {
  return {
    '--font-size': `${props.config.fontSize}px`,
    '--secondary-font-size': `${Math.max(props.config.fontSize * 0.72, 14)}px`,
    '--font-family': props.config.fontFamily || "'Segoe UI', system-ui, -apple-system, sans-serif",
    '--active-color': props.config.activeColor,
    '--secondary-color': props.config.secondaryColor,
    '--stroke-color': props.config.strokeColor,
    '--stroke-width': `${props.config.strokeWidth ?? 2}px`,
    '--progress': `${lineProgress.value.toFixed(1)}%`,
    '--text-align': props.config.textAlign,
  };
});
</script>

<template>
  <div
    class="w-full h-full flex flex-col justify-center px-6 transition-all duration-300 pointer-events-none"
    :style="[styleVariables, { fontFamily: 'var(--font-family)' }]"
    :class="{
      'text-center items-center': config.textAlign === 'center',
      'text-left items-start': config.textAlign === 'left',
      'text-right items-end': config.textAlign === 'right',
    }"
  >
    <!-- Loading State -->
    <div v-if="isLoading" class="flex items-center space-x-2 text-white/70 italic text-sm py-2">
      <span class="inline-block w-2.5 h-2.5 rounded-full border-2 border-sky-400 border-t-transparent animate-spin"></span>
      <span class="lyric-stroke">Searching synchronized lyrics...</span>
    </div>

    <!-- Instrumental Track -->
    <div
      v-else-if="lyrics?.isInstrumental"
      class="text-amber-300 font-medium tracking-wide lyric-stroke py-2"
      :style="{ fontSize: 'var(--font-size)' }"
    >
      🎵 [Instrumental] 🎵
    </div>

    <!-- No Lyrics Loaded Yet -->
    <div
      v-else-if="!lyrics || lyrics.lines.length === 0"
      class="text-white/80 tracking-wide font-medium lyric-stroke py-2 text-center"
      :style="{ fontSize: 'var(--font-size)' }"
    >
      <span v-if="lyrics?.plainText">{{ lyrics.plainText.slice(0, 80) }}...</span>
      <span v-else-if="track && track.title" class="text-white/70">
        🎵 {{ track.artist ? track.artist + ' - ' : '' }}{{ track.title }}
        <span class="text-xs block text-white/50 mt-1 font-normal">(Lirik tidak ditemukan / gunakan Pencarian Manual)</span>
      </span>
      <span v-else>AIMP Floating Lyrics Online</span>
    </div>

    <!-- Synced Lyrics Display -->
    <div v-else class="w-full flex flex-col justify-center overflow-hidden">
      <!-- 1. Two-Line Mode -->
      <div v-if="config.displayMode === 'two-line'" class="flex flex-col space-y-1 w-full">
        <!-- Active Line -->
        <div
          class="font-bold tracking-tight text-white lyric-stroke-thick transition-all duration-200 truncate w-full"
          :style="{
            fontSize: 'var(--font-size)',
            color: 'var(--active-color)'
          }"
        >
          {{ currentLine?.text || '...' }}
        </div>

        <!-- Next Line -->
        <div
          class="font-medium tracking-normal text-white/65 lyric-stroke transition-all duration-300 truncate w-full"
          :style="{
            fontSize: 'var(--secondary-font-size)',
            color: 'var(--secondary-color)'
          }"
        >
          {{ nextLine?.text || ' ' }}
        </div>
      </div>

      <!-- 2. Single-Line Fade Mode -->
      <div v-else-if="config.displayMode === 'single-line'" class="w-full">
        <Transition name="fade-slide" mode="out-in">
          <div
            :key="currentLine?.timeMs || 0"
            class="font-bold tracking-tight text-white lyric-stroke-thick truncate w-full"
            :style="{
              fontSize: 'var(--font-size)',
              color: 'var(--active-color)'
            }"
          >
            {{ currentLine?.text || '...' }}
          </div>
        </Transition>
      </div>

      <!-- 3. Karaoke Word-Highlight / Sweep Mode -->
      <div v-else-if="config.displayMode === 'karaoke'" class="w-full flex flex-col space-y-1">
        <!-- Word-Level Sync if available -->
        <div
          v-if="currentLine?.words && currentLine.words.length > 0"
          class="flex flex-wrap items-center gap-x-2 font-bold tracking-tight lyric-stroke-thick"
          :style="{ fontSize: 'var(--font-size)' }"
        >
          <span
            v-for="(word, wIdx) in currentLine.words"
            :key="wIdx"
            class="transition-colors duration-150"
            :style="{
              color: (currentPositionMs + (lyrics.offsetMs || 0)) >= word.startMs
                ? 'var(--active-color)'
                : 'var(--secondary-color)'
            }"
          >
            {{ word.text }}
          </span>
        </div>

        <!-- Horizontal Gradient Fill Sweep -->
        <div
          v-else
          class="font-bold tracking-tight lyric-stroke-thick karaoke-fill truncate w-full"
          :style="{ fontSize: 'var(--font-size)' }"
        >
          {{ currentLine?.text || '...' }}
        </div>

        <!-- Next Line Preview -->
        <div
          class="font-medium text-white/60 lyric-stroke truncate w-full"
          :style="{
            fontSize: 'var(--secondary-font-size)',
            color: 'var(--secondary-color)'
          }"
        >
          {{ nextLine?.text || ' ' }}
        </div>
      </div>
    </div>
  </div>
</template>
