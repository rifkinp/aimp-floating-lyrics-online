<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { Search, Music, Check, X, Clock, Disc, Loader2 } from 'lucide-vue-next';
import type { LrclibRecord, ParsedLyrics, AimpTrackInfo } from '../types';

const props = withDefaults(
  defineProps<{
    track?: AimpTrackInfo | null;
    standalone?: boolean;
  }>(),
  {
    standalone: false,
  }
);

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'lyrics-applied', lyrics: ParsedLyrics): void;
}>();

const currentTrack = ref<AimpTrackInfo | null>(props.track || null);
const query = ref('');
const results = ref<LrclibRecord[]>([]);
const isSearching = ref(false);
const applyingId = ref<number | null>(null);
const errorMessage = ref('');
let unlistenTrackChange: UnlistenFn | null = null;

onMounted(async () => {
  if (!currentTrack.value) {
    try {
      const cur = await invoke<AimpTrackInfo | null>('get_current_track');
      if (cur) currentTrack.value = cur;
    } catch (_) {}
  }

  if (currentTrack.value?.title) {
    query.value = `${currentTrack.value.artist} ${currentTrack.value.title}`.trim();
    if (query.value.length > 0) {
      handleSearch();
    }
  }

  // Listen to track changes if in standalone window
  if (props.standalone) {
    unlistenTrackChange = await listen<AimpTrackInfo>('aimp://track-changed', (event) => {
      currentTrack.value = event.payload;
      if (!query.value || query.value === `${currentTrack.value?.artist} ${currentTrack.value?.title}`.trim()) {
        query.value = `${event.payload.artist} ${event.payload.title}`.trim();
        handleSearch();
      }
    });
  }
});

onUnmounted(() => {
  if (unlistenTrackChange) {
    unlistenTrackChange();
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
      errorMessage.value = 'Tidak ada lirik yang ditemukan di LRCLIB untuk kata kunci ini.';
    }
  } catch (err: any) {
    errorMessage.value = String(err || 'Gagal mencari lirik online.');
  } finally {
    isSearching.value = false;
  }
};

const closeSearch = async () => {
  try {
    await invoke('hide_window', { label: 'search' });
  } catch (_) {
    emit('close');
  }
};

const applyRecord = async (record: LrclibRecord) => {
  const rawLrc = record.syncedLyrics || record.plainLyrics;
  if (!rawLrc) {
    errorMessage.value = 'Lagu ini tidak memiliki teks lirik.';
    return;
  }

  applyingId.value = record.id;
  try {
    const parsed = await invoke<ParsedLyrics>('apply_lyrics', {
      audioPath: currentTrack.value?.fileName || '',
      artist: record.artistName || currentTrack.value?.artist || '',
      title: record.trackName || currentTrack.value?.title || '',
      rawLrc,
    });
    emit('lyrics-applied', parsed);
    await closeSearch();
  } catch (err: any) {
    errorMessage.value = String(err || 'Gagal menerapkan lirik.');
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
  <!-- Standalone Window View -->
  <div
    v-if="standalone"
    class="w-full h-screen bg-slate-900 flex flex-col text-slate-100 select-none overflow-hidden"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-5 py-3.5 bg-slate-900/90 border-b border-slate-800">
      <div class="flex items-center space-x-2.5">
        <Music class="w-5 h-5 text-sky-400" />
        <h2 class="font-bold text-sm tracking-wide text-white">Manual Lyrics Search (LRCLIB)</h2>
      </div>
      <button
        @click="closeSearch"
        class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition"
        title="Close"
      >
        <X class="w-4 h-4" />
      </button>
    </div>

    <!-- Search Form -->
    <div class="p-5 border-b border-slate-800/80 bg-slate-900/50">
      <form @submit.prevent="handleSearch" class="flex items-center space-x-2.5">
        <div class="relative flex-1">
          <input
            v-model="query"
            type="text"
            placeholder="Cari artis atau judul lagu..."
            class="w-full pl-9 pr-4 py-2.5 bg-slate-800 border border-slate-700/80 rounded-xl text-xs text-white placeholder-slate-400 focus:outline-none focus:border-sky-400 focus:ring-1 focus:ring-sky-400 transition"
          />
          <Search class="w-4 h-4 text-slate-400 absolute left-3 top-3 pointer-events-none" />
        </div>
        <button
          type="submit"
          :disabled="isSearching"
          class="px-5 py-2.5 bg-sky-500 hover:bg-sky-400 disabled:opacity-50 text-white font-semibold text-xs rounded-xl transition flex items-center space-x-1.5 shadow-md shadow-sky-500/20"
        >
          <Loader2 v-if="isSearching" class="w-4 h-4 animate-spin" />
          <span>{{ isSearching ? 'Mencari...' : 'Cari' }}</span>
        </button>
      </form>
    </div>

    <!-- Results List -->
    <div class="flex-1 overflow-y-auto p-5 space-y-2.5 text-xs">
      <div v-if="errorMessage" class="p-4 bg-rose-500/10 border border-rose-500/30 rounded-xl text-rose-300 text-center">
        {{ errorMessage }}
      </div>

      <div
        v-for="record in results"
        :key="record.id"
        class="p-3.5 bg-slate-800/50 hover:bg-slate-800 border border-slate-700/50 hover:border-slate-600 rounded-xl transition flex items-center justify-between group"
      >
        <div class="flex-1 min-w-0 pr-4">
          <div class="flex items-center space-x-2">
            <span class="font-bold text-slate-100 text-sm truncate">{{ record.trackName }}</span>
            <span
              v-if="record.syncedLyrics"
              class="px-2 py-0.5 text-[10px] font-semibold bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 rounded-md"
            >
              Synced LRC
            </span>
            <span
              v-else-if="record.plainLyrics"
              class="px-2 py-0.5 text-[10px] font-semibold bg-amber-500/20 text-amber-300 border border-amber-500/30 rounded-md"
            >
              Plain Text
            </span>
          </div>
          <div class="flex items-center space-x-3 text-slate-400 mt-1 text-[11px]">
            <span class="truncate text-slate-300">{{ record.artistName }}</span>
            <span v-if="record.albumName" class="flex items-center space-x-1 truncate max-w-[200px]">
              <Disc class="w-3 h-3 flex-shrink-0" />
              <span class="truncate">{{ record.albumName }}</span>
            </span>
            <span v-if="record.duration" class="flex items-center space-x-1 flex-shrink-0">
              <Clock class="w-3 h-3" />
              <span>{{ formatDuration(record.duration) }}</span>
            </span>
          </div>
        </div>

        <button
          @click="applyRecord(record)"
          :disabled="applyingId === record.id"
          class="px-4 py-2 bg-sky-500/20 group-hover:bg-sky-500 hover:bg-sky-400 text-sky-300 group-hover:text-white font-semibold rounded-xl border border-sky-400/30 transition flex items-center space-x-1.5 flex-shrink-0"
        >
          <Check v-if="applyingId !== record.id" class="w-3.5 h-3.5" />
          <Loader2 v-else class="w-3.5 h-3.5 animate-spin" />
          <span>{{ applyingId === record.id ? 'Memasang...' : 'Gunakan' }}</span>
        </button>
      </div>

      <div
        v-if="!isSearching && results.length === 0 && !errorMessage"
        class="h-full flex flex-col items-center justify-center text-slate-500 text-center py-12"
      >
        <Music class="w-12 h-12 stroke-1 mb-2 text-slate-600" />
        <p class="text-sm">Ketik nama artis atau judul lagu di atas untuk mencari lirik.</p>
      </div>
    </div>
  </div>

  <!-- Legacy In-Window Modal Fallback -->
  <div
    v-else
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-md"
    @click.self="emit('close')"
  >
    <div
      class="bg-slate-900 border border-slate-700/80 rounded-2xl shadow-2xl w-full max-w-lg overflow-hidden flex flex-col max-h-[85vh] text-slate-100 animate-in fade-in zoom-in-95 duration-150"
    >
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

      <div class="p-3 border-b border-slate-800 bg-slate-800/40">
        <form @submit.prevent="handleSearch" class="flex items-center space-x-2">
          <div class="relative flex-1">
            <input
              v-model="query"
              type="text"
              placeholder="Search by artist, title, or query..."
              class="w-full pl-8 pr-3 py-1.5 bg-slate-800 border border-slate-700 rounded-lg text-xs text-white placeholder-slate-500 focus:outline-none focus:border-sky-400 transition"
            />
            <Search class="w-3.5 h-3.5 text-slate-400 absolute left-2.5 top-2 pointer-events-none" />
          </div>
          <button
            type="submit"
            :disabled="isSearching"
            class="px-3 py-1.5 bg-sky-500 hover:bg-sky-400 disabled:opacity-50 text-white font-semibold text-xs rounded-lg transition flex items-center space-x-1"
          >
            <span>{{ isSearching ? 'Searching...' : 'Search' }}</span>
          </button>
        </form>
      </div>

      <div class="flex-1 overflow-y-auto p-3 space-y-2 text-xs">
        <div v-if="errorMessage" class="p-3 bg-rose-500/10 border border-rose-500/20 rounded-xl text-rose-300 text-center">
          {{ errorMessage }}
        </div>

        <div
          v-for="record in results"
          :key="record.id"
          class="p-2.5 bg-slate-800/40 hover:bg-slate-800/90 border border-slate-700/60 rounded-xl transition flex items-center justify-between"
        >
          <div class="flex-1 min-w-0 pr-3">
            <div class="flex items-center space-x-2">
              <span class="font-semibold text-slate-100 truncate">{{ record.trackName }}</span>
              <span
                v-if="record.syncedLyrics"
                class="px-1.5 py-0.5 text-[9px] font-bold bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 rounded"
              >
                SYNCED
              </span>
              <span
                v-else-if="record.plainLyrics"
                class="px-1.5 py-0.5 text-[9px] font-bold bg-amber-500/20 text-amber-300 border border-amber-500/30 rounded"
              >
                PLAIN
              </span>
            </div>
            <div class="flex items-center space-x-3 text-slate-400 mt-0.5 text-[11px]">
              <span class="truncate">{{ record.artistName }}</span>
              <span v-if="record.albumName" class="flex items-center space-x-1 truncate">
                <Disc class="w-3 h-3" />
                <span class="truncate">{{ record.albumName }}</span>
              </span>
              <span v-if="record.duration" class="flex items-center space-x-1">
                <Clock class="w-3 h-3" />
                <span>{{ formatDuration(record.duration) }}</span>
              </span>
            </div>
          </div>

          <button
            @click="applyRecord(record)"
            :disabled="applyingId === record.id"
            class="px-3 py-1.5 bg-sky-500 hover:bg-sky-400 text-white font-semibold rounded-lg transition flex items-center space-x-1 flex-shrink-0"
          >
            <Check v-if="applyingId !== record.id" class="w-3.5 h-3.5" />
            <span>{{ applyingId === record.id ? 'Applying...' : 'Apply' }}</span>
          </button>
        </div>

        <div
          v-if="!isSearching && results.length === 0 && !errorMessage"
          class="h-32 flex flex-col items-center justify-center text-slate-500 text-center"
        >
          <Music class="w-8 h-8 stroke-1 mb-1 text-slate-600" />
          <p>Search for a song title or artist above.</p>
        </div>
      </div>
    </div>
  </div>
</template>
