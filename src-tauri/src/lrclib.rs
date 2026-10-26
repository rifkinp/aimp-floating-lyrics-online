use crate::config::{get_cache_dir, load_config};
use crate::parser::{parse_lrc, ParsedLyrics};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const USER_AGENT: &str = "AIMP-Floating-Lyrics-Online v0.1.0 (https://github.com/local/aimp-floating-lyrics)";
const LRCLIB_CLIENT: &str = "AIMP-Floating-Lyrics-Online v0.1.0";
const API_BASE: &str = "https://lrclib.net/api";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LrclibRecord {
    pub id: i64,
    pub name: Option<String>,
    pub track_name: Option<String>,
    pub artist_name: Option<String>,
    pub album_name: Option<String>,
    pub duration: Option<f64>,
    pub instrumental: Option<bool>,
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
}

pub struct LrcManager {
    client: reqwest::Client,
}

pub fn clean_title_and_artist(artist: &str, title: &str, file_path: &str) -> (String, String) {
    let mut clean_t = title.to_string();

    // 1. Remove URLs
    if let Some(pos) = clean_t.find("http://") {
        clean_t = clean_t[..pos].to_string();
    }
    if let Some(pos) = clean_t.find("https://") {
        clean_t = clean_t[..pos].to_string();
    }
    if let Some(pos) = clean_t.find("www.") {
        clean_t = clean_t[..pos].to_string();
    }

    // 2. Remove pipes (e.g. "Title | Website")
    if let Some(pos) = clean_t.find('|') {
        clean_t = clean_t[..pos].to_string();
    }

    // 3. Remove common bracket tags like [stafaband], [official video], [320kbps]
    while let Some(start) = clean_t.find('[') {
        if let Some(end) = clean_t[start..].find(']') {
            clean_t.replace_range(start..=start + end, "");
        } else {
            break;
        }
    }

    // 4. Remove parenthesized tags if they contain video/audio/remaster
    clean_t = clean_parenthesized_junk(&clean_t);

    let mut clean_t = clean_t.trim().to_string();
    let mut clean_a = artist.trim().to_string();

    // 5. If title has "Artist - Title" format (e.g. "Drive - Akulah Dia")
    if clean_t.contains(" - ") {
        let parts: Vec<&str> = clean_t.splitn(2, " - ").collect();
        if clean_a.is_empty() || clean_a.eq_ignore_ascii_case(parts[0].trim()) {
            clean_a = parts[0].trim().to_string();
            clean_t = parts[1].trim().to_string();
        }
    }

    // 6. Filename fallback if title is empty or if artist is empty
    if clean_t.is_empty() || clean_a.is_empty() {
        if let Some(stem) = Path::new(file_path).file_stem().and_then(|s| s.to_str()) {
            let normalized = stem.replace("_-_", " - ").replace('_', " ");
            if let Some((f_art, f_tit)) = normalized.split_once(" - ") {
                if clean_a.is_empty() {
                    clean_a = f_art.trim().to_string();
                }
                if clean_t.is_empty() {
                    clean_t = f_tit.trim().to_string();
                }
            } else if clean_t.is_empty() {
                clean_t = normalized.trim().to_string();
            }
        }
    }

    (clean_a, clean_t)
}

fn clean_parenthesized_junk(s: &str) -> String {
    let mut res = s.to_string();
    let keywords = [
        "official", "video", "audio", "lyric", "remaster", "bonus", "live", "320kbps",
        "planetlagu", "stafaband", "lagu123", "gudanglagu", "metrolagu", "uyeshare",
    ];
    while let Some(start) = res.find('(') {
        if let Some(end) = res[start..].find(')') {
            let content = &res[start + 1..start + end].to_lowercase();
            if keywords.iter().any(|k| content.contains(k)) {
                res.replace_range(start..=start + end, "");
                continue;
            }
        }
        break;
    }
    res
}

impl LrcManager {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            HeaderValue::from_static(USER_AGENT),
        );
        headers.insert("Lrclib-Client", HeaderValue::from_static(LRCLIB_CLIENT));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self { client }
    }

    pub fn sanitize_filename(name: &str) -> String {
        name.chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                _ => c,
            })
            .collect()
    }

    pub fn get_local_lrc_path(audio_path: &str) -> Option<PathBuf> {
        if audio_path.is_empty() {
            return None;
        }
        let p = Path::new(audio_path);
        if p.exists() {
            Some(p.with_extension("lrc"))
        } else {
            None
        }
    }

    pub fn get_cache_lrc_path(artist: &str, title: &str) -> PathBuf {
        let sanitized = Self::sanitize_filename(&format!("{} - {}", artist.trim(), title.trim()));
        get_cache_dir().join(format!("{}.lrc", sanitized))
    }

    /// Primary resolver following Priority Order (Local -> Cache -> LRCLIB get -> LRCLIB search)
    pub async fn resolve_lyrics(
        &self,
        audio_path: &str,
        artist: &str,
        title: &str,
        album: &str,
        duration_ms: u32,
    ) -> Result<ParsedLyrics, String> {
        // Step 1: Check Local File
        if let Some(local_path) = Self::get_local_lrc_path(audio_path) {
            if local_path.exists() {
                if let Ok(content) = fs::read_to_string(&local_path) {
                    let parsed = parse_lrc(&content);
                    if parsed.is_synced || !parsed.plain_text.is_empty() {
                        return Ok(parsed);
                    }
                }
            }
        }

        // Clean query terms (strips URLs, pipes, [stafaband], blogspot, etc.)
        let (clean_artist, clean_title) = clean_title_and_artist(artist, title, audio_path);

        // Step 2: Check App Cache Directory
        let cache_path = Self::get_cache_lrc_path(&clean_artist, &clean_title);
        if cache_path.exists() {
            if let Ok(content) = fs::read_to_string(&cache_path) {
                let parsed = parse_lrc(&content);
                if parsed.is_synced || !parsed.plain_text.is_empty() {
                    return Ok(parsed);
                }
            }
        }

        if clean_title.is_empty() {
            return Err("Empty track title".to_string());
        }

        let duration_secs = (duration_ms / 1000).max(1);

        // Step 3: LRCLIB Exact Match (/api/get)
        let exact_result = self
            .fetch_exact(&clean_artist, &clean_title, album.trim(), duration_secs)
            .await;

        if let Ok(record) = exact_result {
            if let Some(parsed) = self.process_and_save_record(&record, audio_path, &clean_artist, &clean_title) {
                return Ok(parsed);
            }
        }

        // Step 4: LRCLIB Search Fallback (/api/search)
        let mut search_results = self.search_tracks(&clean_artist, &clean_title).await.unwrap_or_default();

        // If no results and artist was given, search by title alone as fallback
        if search_results.is_empty() && !clean_artist.is_empty() {
            if let Ok(by_title) = self.search_tracks("", &clean_title).await {
                search_results = by_title;
            }
        }

        // Pick best matching record
        let target_dur = duration_secs as f64;
        let mut best_record: Option<&LrclibRecord> = None;
        let mut min_diff = f64::MAX;

        // First look for records with syncedLyrics
        for rec in &search_results {
            if rec.synced_lyrics.is_some() {
                let dur = rec.duration.unwrap_or(0.0);
                let diff = (dur - target_dur).abs();
                if diff < min_diff {
                    min_diff = diff;
                    best_record = Some(rec);
                }
            }
        }

        // If no synced lyrics, fallback to plain lyrics
        if best_record.is_none() {
            for rec in &search_results {
                if rec.plain_lyrics.is_some() {
                    let dur = rec.duration.unwrap_or(0.0);
                    let diff = (dur - target_dur).abs();
                    if diff < min_diff {
                        min_diff = diff;
                        best_record = Some(rec);
                    }
                }
            }
        }

        if let Some(record) = best_record {
            if let Some(parsed) = self.process_and_save_record(record, audio_path, &clean_artist, &clean_title) {
                return Ok(parsed);
            }
        }

        Err("No lyrics found online".to_string())
    }

    async fn execute_with_retry<T, F>(&self, make_request: F) -> Result<T, String>
    where
        F: Fn() -> reqwest::RequestBuilder,
        T: serde::de::DeserializeOwned,
    {
        let mut retries = 0;
        loop {
            let req = make_request();
            match req.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS
                        || status == reqwest::StatusCode::SERVICE_UNAVAILABLE
                    {
                        if retries < 3 {
                            retries += 1;
                            let retry_after_secs = resp
                                .headers()
                                .get("Retry-After")
                                .and_then(|v| v.to_str().ok())
                                .and_then(|s| s.parse::<u64>().ok())
                                .unwrap_or(1);
                            tokio::time::sleep(Duration::from_millis(retry_after_secs * 1000 + (retries * 500))).await;
                            continue;
                        }
                    }
                    if status.is_success() {
                        let data = resp.json::<T>().await.map_err(|e| e.to_string())?;
                        return Ok(data);
                    } else {
                        return Err(format!("LRCLIB API status: {}", status));
                    }
                }
                Err(e) => {
                    if retries < 2 {
                        retries += 1;
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        continue;
                    }
                    return Err(e.to_string());
                }
            }
        }
    }

    async fn fetch_exact(
        &self,
        artist: &str,
        title: &str,
        album: &str,
        duration_secs: u32,
    ) -> Result<LrclibRecord, String> {
        let mut query = vec![("track_name", title), ("artist_name", artist)];
        let dur_str = duration_secs.to_string();
        if duration_secs > 0 {
            query.push(("duration", &dur_str));
        }
        if !album.is_empty() {
            query.push(("album_name", album));
        }

        self.execute_with_retry(|| {
            self.client
                .get(format!("{}/get", API_BASE))
                .query(&query)
        })
        .await
    }

    pub async fn search_tracks(
        &self,
        artist: &str,
        title: &str,
    ) -> Result<Vec<LrclibRecord>, String> {
        let mut query = vec![("track_name", title)];
        if !artist.is_empty() {
            query.push(("artist_name", artist));
        }

        let res: Result<Vec<LrclibRecord>, String> = self
            .execute_with_retry(|| {
                self.client
                    .get(format!("{}/search", API_BASE))
                    .query(&query)
            })
            .await;

        match res {
            Ok(list) if !list.is_empty() => Ok(list),
            _ => {
                // Fallback to query (?q=...)
                let q_str = format!("{} {}", artist, title).trim().to_string();
                self.manual_search_query(&q_str).await
            }
        }
    }

    pub async fn manual_search_query(&self, query_str: &str) -> Result<Vec<LrclibRecord>, String> {
        self.execute_with_retry(|| {
            self.client
                .get(format!("{}/search", API_BASE))
                .query(&[("q", query_str)])
        })
        .await
    }

    fn process_and_save_record(
        &self,
        record: &LrclibRecord,
        audio_path: &str,
        artist: &str,
        title: &str,
    ) -> Option<ParsedLyrics> {
        if record.instrumental == Some(true) {
            return Some(ParsedLyrics {
                is_instrumental: true,
                plain_text: "[Instrumental]".to_string(),
                ..Default::default()
            });
        }

        let raw_lrc = if let Some(synced) = &record.synced_lyrics {
            synced.clone()
        } else if let Some(plain) = &record.plain_lyrics {
            plain.clone()
        } else {
            return None;
        };

        let parsed = parse_lrc(&raw_lrc);

        // Auto-save to disk
        let cfg = load_config();
        let mut saved = false;

        if cfg.save_to_audio_folder {
            if let Some(local_path) = Self::get_local_lrc_path(audio_path) {
                if let Ok(_) = fs::write(&local_path, &raw_lrc) {
                    saved = true;
                }
            }
        }

        // Fallback to cache directory
        if !saved {
            let cache_path = Self::get_cache_lrc_path(artist, title);
            let _ = fs::write(&cache_path, &raw_lrc);
        }

        Some(parsed)
    }

    pub fn save_custom_lrc(
        audio_path: &str,
        artist: &str,
        title: &str,
        lrc_text: &str,
    ) -> Result<String, String> {
        let cfg = load_config();
        let mut saved_path = String::new();

        if cfg.save_to_audio_folder {
            if let Some(local_path) = Self::get_local_lrc_path(audio_path) {
                if fs::write(&local_path, lrc_text).is_ok() {
                    saved_path = local_path.to_string_lossy().to_string();
                }
            }
        }

        if saved_path.is_empty() {
            let cache_path = Self::get_cache_lrc_path(artist, title);
            fs::write(&cache_path, lrc_text).map_err(|e| e.to_string())?;
            saved_path = cache_path.to_string_lossy().to_string();
        }

        Ok(saved_path)
    }
}
