<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Edit3, X, Save, Clock, Check } from 'lucide-vue-next';
import type { AimpTrackInfo, ParsedLyrics } from '../types';

const props = withDefaults(
  defineProps<{
    track?: AimpTrackInfo | null;
    lyrics?: ParsedLyrics | null;
    currentPositionMs?: number;
    standalone?: boolean;
  }>(),
  {
    track: null,
    lyrics: null,
    currentPositionMs: 0,
    standalone: false,
  }
);

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'saved', lyrics: ParsedLyrics): void;
}>();

const editorContent = ref('');
const isSaving = ref(false);
const saveSuccess = ref(false);
const textareaRef = ref<HTMLTextAreaElement | null>(null);

const populateContent = () => {
  if (props.lyrics?.rawLrc) {
    editorContent.value = props.lyrics.rawLrc;
  } else if (props.lyrics?.plainText) {
    editorContent.value = props.lyrics.plainText;
  }
};

onMounted(() => {
  populateContent();
});

watch(
  () => props.track?.fileName,
  () => {
    populateContent();
  }
);

watch(
  () => props.lyrics?.rawLrc,
  (newRaw) => {
    if (newRaw && (!editorContent.value || editorContent.value.trim().length === 0)) {
      editorContent.value = newRaw;
    }
  }
);

const formatTimestamp = (ms: number) => {
  const totalSecs = ms / 1000;
  const m = Math.floor(totalSecs / 60);
  const s = (totalSecs % 60).toFixed(2);
  return `[${m.toString().padStart(2, '0')}:${s.padStart(5, '0')}]`;
};

const insertCurrentTimestamp = () => {
  const ts = formatTimestamp(props.currentPositionMs || 0);
  const textarea = textareaRef.value;
  if (!textarea) {
    editorContent.value += `${ts} `;
    return;
  }

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const text = editorContent.value;

  editorContent.value = text.substring(0, start) + ts + ' ' + text.substring(end);
  setTimeout(() => {
    textarea.focus();
    textarea.setSelectionRange(start + ts.length + 1, start + ts.length + 1);
  }, 10);
};

const handleSave = async () => {
  if (!editorContent.value.trim()) return;
  isSaving.value = true;
  saveSuccess.value = false;

  try {
    const updated = await invoke<ParsedLyrics>('save_edited_lyrics', {
      audioPath: props.track?.fileName || '',
      rawLrc: editorContent.value,
    });
    saveSuccess.value = true;
    emit('saved', updated);
    setTimeout(() => {
      emit('close');
    }, 800);
  } catch (err) {
    console.error('Failed to save edited lyrics:', err);
  } finally {
    isSaving.value = false;
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
        <Edit3 class="w-4 h-4 text-sky-400" />
        <div>
          <h2 class="font-bold text-sm tracking-wide text-white">LRC Lyrics Editor</h2>
          <p class="text-[10px] text-slate-400 truncate max-w-[420px]">
            {{ track?.title || 'Unknown Title' }} <span v-if="track?.artist">- {{ track.artist }}</span>
          </p>
        </div>
      </div>
      <button
        @click="emit('close')"
        class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition"
        title="Tutup Jendela Editor"
      >
        <X class="w-4 h-4" />
      </button>
    </div>

    <!-- Action Toolbar -->
    <div class="px-5 py-2.5 bg-slate-950/60 border-b border-slate-800 flex items-center justify-between">
      <div class="flex items-center space-x-2">
        <button
          type="button"
          @click="insertCurrentTimestamp"
          class="px-2.5 py-1 rounded-lg bg-sky-500/20 hover:bg-sky-500/30 text-sky-300 border border-sky-400/30 transition flex items-center space-x-1.5 active:scale-95"
          title="Sisipkan timestamp pemutaran AIMP saat ini pada posisi kursor"
        >
          <Clock class="w-3.5 h-3.5" />
          <span>Sisipkan Timestamp ({{ formatTimestamp(currentPositionMs || 0) }})</span>
        </button>
      </div>

      <div class="text-[11px] text-slate-400">
        Format: <code class="text-sky-300 font-mono">[mm:ss.xx] Teks lirik</code>
      </div>
    </div>

    <!-- Textarea Editor -->
    <div class="flex-1 p-4 bg-slate-950/40 min-h-0 flex flex-col">
      <textarea
        ref="textareaRef"
        v-model="editorContent"
        placeholder="[00:12.34] Ketik atau tempel lirik berformat LRC di sini..."
        class="w-full flex-1 bg-slate-900/90 border border-slate-700/80 rounded-xl p-3.5 text-xs text-slate-100 font-mono leading-relaxed placeholder-slate-500 focus:outline-none focus:border-sky-400 resize-none selection:bg-sky-500/30"
        spellcheck="false"
      ></textarea>
    </div>

    <!-- Footer -->
    <div class="px-5 py-3 bg-slate-900 border-t border-slate-800 flex items-center justify-between">
      <span v-if="saveSuccess" class="text-emerald-400 font-semibold flex items-center space-x-1 text-xs animate-pulse">
        <Check class="w-4 h-4" />
        <span>Lirik berhasil disimpan dan diperbarui!</span>
      </span>
      <span v-else class="text-slate-400 text-[11px]">
        Lirik yang diedit akan otomatis disimpan ke folder lagu dan cache offline.
      </span>

      <div class="flex items-center space-x-2.5">
        <button
          type="button"
          @click="emit('close')"
          class="px-4 py-1.5 rounded-xl text-slate-400 hover:text-white hover:bg-slate-800 transition"
        >
          Tutup
        </button>

        <button
          type="button"
          @click="handleSave"
          :disabled="isSaving"
          class="px-5 py-2 rounded-xl bg-sky-500 hover:bg-sky-400 disabled:opacity-50 text-white font-semibold transition active:scale-95 flex items-center space-x-1.5 shadow-md shadow-sky-500/20"
        >
          <Save class="w-3.5 h-3.5" />
          <span>{{ isSaving ? 'Menyimpan...' : 'Simpan & Terapkan' }}</span>
        </button>
      </div>
    </div>
  </div>

  <!-- In-Overlay Modal View -->
  <div
    v-else
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md select-none"
    @click.self="emit('close')"
  >
    <div
      class="bg-slate-900 border border-slate-700/80 rounded-2xl shadow-2xl w-full max-w-xl h-[82vh] flex flex-col overflow-hidden text-slate-100 text-xs animate-in fade-in zoom-in-95 duration-150"
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-3.5 bg-slate-900/90 border-b border-slate-800">
        <div class="flex items-center space-x-2.5">
          <Edit3 class="w-4 h-4 text-sky-400" />
          <div>
            <h2 class="font-bold text-sm tracking-wide text-white">LRC Lyrics Editor</h2>
            <p class="text-[10px] text-slate-400 truncate max-w-[340px]">
              {{ track?.title || 'Unknown Title' }} <span v-if="track?.artist">- {{ track.artist }}</span>
            </p>
          </div>
        </div>
        <button
          @click="emit('close')"
          class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Action Toolbar -->
      <div class="px-5 py-2.5 bg-slate-950/60 border-b border-slate-800 flex items-center justify-between">
        <div class="flex items-center space-x-2">
          <button
            type="button"
            @click="insertCurrentTimestamp"
            class="px-2.5 py-1 rounded-lg bg-sky-500/20 hover:bg-sky-500/30 text-sky-300 border border-sky-400/30 transition flex items-center space-x-1.5 active:scale-95"
            title="Sisipkan timestamp pemutaran AIMP saat ini pada posisi kursor"
          >
            <Clock class="w-3.5 h-3.5" />
            <span>Sisipkan Timestamp ({{ formatTimestamp(currentPositionMs || 0) }})</span>
          </button>
        </div>

        <div class="text-[11px] text-slate-400">
          Format: <code class="text-sky-300 font-mono">[mm:ss.xx] Teks lirik</code>
        </div>
      </div>

      <!-- Textarea Editor -->
      <div class="flex-1 p-4 bg-slate-950/40 min-h-0">
        <textarea
          ref="textareaRef"
          v-model="editorContent"
          placeholder="[00:12.34] Ketik atau tempel lirik berformat LRC di sini..."
          class="w-full h-full bg-slate-900/90 border border-slate-700/80 rounded-xl p-3.5 text-xs text-slate-100 font-mono leading-relaxed placeholder-slate-500 focus:outline-none focus:border-sky-400 resize-none selection:bg-sky-500/30"
          spellcheck="false"
        ></textarea>
      </div>

      <!-- Footer -->
      <div class="px-5 py-3 bg-slate-900 border-t border-slate-800 flex items-center justify-between">
        <span v-if="saveSuccess" class="text-emerald-400 font-semibold flex items-center space-x-1 text-xs animate-pulse">
          <Check class="w-4 h-4" />
          <span>Lirik berhasil disimpan dan diperbarui!</span>
        </span>
        <span v-else class="text-slate-400 text-[11px]">
          Lirik yang diedit akan otomatis disimpan ke folder lagu dan cache offline.
        </span>

        <div class="flex items-center space-x-2.5">
          <button
            type="button"
            @click="emit('close')"
            class="px-4 py-1.5 rounded-xl text-slate-400 hover:text-white hover:bg-slate-800 transition"
          >
            Batal
          </button>

          <button
            type="button"
            @click="handleSave"
            :disabled="isSaving"
            class="px-5 py-2 rounded-xl bg-sky-500 hover:bg-sky-400 disabled:opacity-50 text-white font-semibold transition active:scale-95 flex items-center space-x-1.5 shadow-md shadow-sky-500/20"
          >
            <Save class="w-3.5 h-3.5" />
            <span>{{ isSaving ? 'Menyimpan...' : 'Simpan & Terapkan' }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
