export interface LyricWord {
  text: string;
  startMs: number;
  endMs: number;
}

export interface LyricLine {
  timeMs: number;
  text: string;
  translation?: string;
  romaji?: string;
  words?: LyricWord[];
}

export interface ParsedLyrics {
  title?: string;
  artist?: string;
  album?: string;
  offsetMs: number;
  isSynced: boolean;
  isInstrumental: boolean;
  plainText: string;
  rawLrc: string;
  lines: LyricLine[];
  hasTranslation?: boolean;
  hasRomaji?: boolean;
}

export interface AimpTrackInfo {
  artist: string;
  title: string;
  album: string;
  fileName: string;
  durationMs: number;
  state: number; // 0 = stopped, 1 = paused, 2 = playing
}

export interface AimpPositionUpdate {
  positionMs: number;
  durationMs: number;
  state: number;
}

export interface AppConfig {
  clickThrough: boolean;
  displayMode: 'two-line' | 'single-line' | 'karaoke';
  fontSize: number;
  fontFamily: string;
  activeColor: string;
  secondaryColor: string;
  strokeColor: string;
  strokeWidth: number;
  textAlign: 'center' | 'left' | 'right';
  saveToAudioFolder: boolean;
  closeWithAimp?: boolean;
  lrclibAppId?: string;
  windowX?: number;
  windowY?: number;
  windowWidth?: number;
  windowHeight?: number;
  showDualLyrics?: boolean;
  dualLyricsType?: 'translation' | 'romaji';
  backgroundOpacity?: number;
  enableGlassmorphism?: boolean;
  dockPosition?: 'none' | 'bottom-taskbar' | 'top-screen';
  targetMonitor?: number | null;
  enabledSources?: string[];
  sourcePriority?: string[];
}

export interface LrclibRecord {
  id: number;
  name?: string;
  trackName?: string;
  artistName?: string;
  albumName?: string;
  duration?: number;
  instrumental?: boolean;
  plainLyrics?: string;
  syncedLyrics?: string;
  provider?: string;
}
