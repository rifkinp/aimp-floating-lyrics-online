<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Search, Music, Check, X, Clock, Disc } from 'lucide-vue-next';
import type { LrclibRecord, ParsedLyrics, AimpTrackInfo } from '../types';

const props = defineProps<{
  track: AimpTrackInfo | null;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'lyrics-applied', lyrics: ParsedLyrics): void;
}>();

const query = ref('');
const results = ref<LrclibRecord[]>([]);
const isSearching = ref(false);
const applyingId = ref<number | null>(null);
const errorMessage = ref('');

onMounted(() => {
  if (props.track) {
    query.value = `${props.track.artist} ${props.track.title}`.trim();
    if (query.value.length > 0) {
      handleSearch();
    }
  }
});

const handleSearch = async () => {
  if (!query.value.trim()) return;
  isSearching.value = true;
  errorMessage.value = '';
  results.value = [];

  try {
    const data = await invoke<LrclibRecord[]>('search_lyrics_online', {
      query: query.value.trim(),
    });
    results.value = data;
    if (data.length === 0) {
      errorMessage.value = 'No lyrics found on LRCLIB for this search.';
    }
  } catch (err: any) {
    errorMessage.value = String(err || 'Failed to search lyrics.');
  } finally {
    isSearching.value = false;
  }
};

const applyRecord = async (record: LrclibRecord) => {
  const rawLrc = record.syncedLyrics || record.plainLyrics;
  if (!rawLrc) {
    errorMessage.value = 'This record contains no lyrics text.';
    return;
  }

  applyingId.value = record.id;
  try {
    const parsed = await invoke<ParsedLyrics>('apply_lyrics', {
      audioPath: props.track?.fileName || '',
      artist: record.artistName || props.track?.artist || '',
      title: record.trackName || props.track?.title || '',
      rawLrc,
    });
    emit('lyrics-applied', parsed);
    emit('close');
  } catch (err: any) {
    errorMessage.value = String(err || 'Failed to apply lyrics.');
  } finally {
    applyingId.value = null;
  }
};

const formatDuration = (secs?: number) => {
  if (!secs) return '0:00';
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}:${s.toString().padStart(2, '0')}`;
};
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-md"
    @click.self="emit('close')"
  >
    <div
      class="bg-slate-900 border border-slate-700/80 rounded-2xl shadow-2xl w-full max-w-lg overflow-hidden flex flex-col max-h-[85vh] text-slate-100 animate-in fade-in zoom-in-95 duration-150"
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-slate-800">
        <div class="flex items-center space-x-2">
          <Music class="w-5 h-5 text-sky-400" />
          <h2 class="font-bold text-sm tracking-wide text-white">Manual Lyrics Search (LRCLIB)</h2>
        </div>
        <button
          @click="emit('close')"
          class="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Search Bar -->
      <form @submit.prevent="handleSearch" class="p-4 border-b border-slate-800 flex gap-2">
        <div class="relative flex-1">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
          <input
            v-model="query"
            type="text"
            placeholder="Search artist, song title..."
            class="w-full bg-slate-800/90 border border-slate-700 rounded-xl pl-9 pr-3 py-2 text-xs text-white placeholder-slate-400 focus:outline-none focus:border-sky-500 focus:ring-1 focus:ring-sky-500 transition"
          />
        </div>
        <button
          type="submit"
          :disabled="isSearching"
          class="px-4 py-2 bg-sky-500 hover:bg-sky-400 disabled:opacity-50 text-white rounded-xl text-xs font-semibold flex items-center space-x-1.5 transition active:scale-95"
        >
          <span v-if="isSearching" class="w-3.5 h-3.5 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
          <span>{{ isSearching ? 'Searching...' : 'Search' }}</span>
        </button>
      </form>

      <!-- Status Message -->
      <div v-if="errorMessage" class="px-4 py-2 bg-red-950/40 border-b border-red-900/30 text-red-300 text-xs">
        {{ errorMessage }}
      </div>

      <!-- Results List -->
      <div class="flex-1 overflow-y-auto p-4 space-y-2.5">
        <div
          v-if="results.length === 0 && !isSearching && !errorMessage"
          class="text-center text-slate-500 text-xs py-8"
        >
          Type a song query above to search LRCLIB database.
        </div>

        <div
          v-for="item in results"
          :key="item.id"
          class="flex items-center justify-between p-3 rounded-xl bg-slate-800/50 hover:bg-slate-800 border border-slate-700/60 hover:border-slate-600 transition"
        >
          <div class="flex-1 min-w-0 pr-3">
            <div class="flex items-center space-x-2">
              <span class="font-semibold text-xs text-white truncate">{{ item.trackName || item.name }}</span>
              <span
                v-if="item.syncedLyrics"
                class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-emerald-500/20 text-emerald-300 border border-emerald-500/30"
              >
                Synced
              </span>
              <span
                v-else-if="item.plainLyrics"
                class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-amber-500/20 text-amber-300 border border-amber-500/30"
              >
                Plain
              </span>
            </div>

            <div class="flex items-center space-x-3 text-[11px] text-slate-400 mt-1">
              <span class="truncate">{{ item.artistName }}</span>
              <span v-if="item.albumName" class="flex items-center space-x-1 truncate max-w-[150px]">
                <Disc class="w-3 h-3 text-slate-500 flex-shrink-0" />
                <span class="truncate">{{ item.albumName }}</span>
              </span>
              <span class="flex items-center space-x-1 flex-shrink-0">
                <Clock class="w-3 h-3 text-slate-500" />
                <span>{{ formatDuration(item.duration) }}</span>
              </span>
            </div>
          </div>

          <button
            @click="applyRecord(item)"
            :disabled="applyingId === item.id || (!item.syncedLyrics && !item.plainLyrics)"
            class="px-3 py-1.5 rounded-lg text-xs font-semibold bg-emerald-600/30 hover:bg-emerald-600/50 text-emerald-200 border border-emerald-500/40 disabled:opacity-40 transition active:scale-95 flex items-center space-x-1"
          >
            <Check class="w-3.5 h-3.5" />
            <span>{{ applyingId === item.id ? 'Saving...' : 'Apply' }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
