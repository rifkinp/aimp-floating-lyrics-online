use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricWord {
    pub text: String,
    pub start_ms: u32,
    pub end_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    pub time_ms: u32,
    pub text: String,
    pub translation: Option<String>,
    pub romaji: Option<String>,
    pub words: Option<Vec<LyricWord>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedLyrics {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub offset_ms: i64,
    pub is_synced: bool,
    pub is_instrumental: bool,
    pub plain_text: String,
    pub raw_lrc: String,
    pub lines: Vec<LyricLine>,
    pub has_translation: bool,
    pub has_romaji: bool,
}

impl Default for ParsedLyrics {
    fn default() -> Self {
        Self {
            title: None,
            artist: None,
            album: None,
            offset_ms: 0,
            is_synced: false,
            is_instrumental: false,
            plain_text: String::new(),
            raw_lrc: String::new(),
            lines: Vec::new(),
            has_translation: false,
            has_romaji: false,
        }
    }
}

pub fn parse_lrc(content: &str) -> ParsedLyrics {
    let mut title = None;
    let mut artist = None;
    let mut album = None;
    let mut offset_ms: i64 = 0;
    let mut is_instrumental = false;
    let mut lines: Vec<LyricLine> = Vec::new();

    let trimmed = content.trim();
    if trimmed.is_empty() {
        return ParsedLyrics::default();
    }

    if trimmed.eq_ignore_ascii_case("[instrumental]") {
        return ParsedLyrics {
            is_instrumental: true,
            plain_text: "[Instrumental]".to_string(),
            raw_lrc: content.to_string(),
            ..Default::default()
        };
    }

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        // Check metadata tags
        if line.starts_with("[ti:") && line.ends_with(']') {
            title = Some(line[4..line.len() - 1].trim().to_string());
            continue;
        }
        if line.starts_with("[ar:") && line.ends_with(']') {
            artist = Some(line[4..line.len() - 1].trim().to_string());
            continue;
        }
        if line.starts_with("[al:") && line.ends_with(']') {
            album = Some(line[4..line.len() - 1].trim().to_string());
            continue;
        }
        if line.starts_with("[offset:") && line.ends_with(']') {
            if let Ok(val) = line[8..line.len() - 1].trim().parse::<i64>() {
                offset_ms = val;
            }
            continue;
        }
        if line.eq_ignore_ascii_case("[instrumental]") {
            is_instrumental = true;
            continue;
        }

        // Parse timestamps: e.g. [00:12.34][01:25.50] Lirik
        let mut timestamps = Vec::new();
        let mut idx = 0;
        let bytes = line.as_bytes();

        while idx < bytes.len() && bytes[idx] == b'[' {
            if let Some(close_bracket) = line[idx..].find(']') {
                let tag = &line[idx + 1..idx + close_bracket];
                if let Some(ms) = parse_timestamp(tag) {
                    timestamps.push(ms);
                    idx += close_bracket + 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let remaining_text = line[idx..].trim();

        if !timestamps.is_empty() {
            // Check if this line is an explicit translation line [tr] or [trans]
            if remaining_text.starts_with("[tr]") || remaining_text.starts_with("[trans]") {
                let clean_tr = if remaining_text.starts_with("[tr]") {
                    remaining_text[4..].trim()
                } else {
                    remaining_text[7..].trim()
                };
                for ts in &timestamps {
                    if let Some(existing) = lines.iter_mut().find(|l| (l.time_ms as i64 - *ts as i64).abs() < 500) {
                        existing.translation = Some(clean_tr.to_string());
                    } else {
                        lines.push(LyricLine {
                            time_ms: *ts,
                            text: clean_tr.to_string(),
                            translation: Some(clean_tr.to_string()),
                            romaji: None,
                            words: None,
                        });
                    }
                }
                continue;
            }

            // Check if this line is an explicit romaji line [ro] or [roma]
            if remaining_text.starts_with("[ro]") || remaining_text.starts_with("[roma]") {
                let clean_ro = if remaining_text.starts_with("[ro]") {
                    remaining_text[4..].trim()
                } else {
                    remaining_text[6..].trim()
                };
                for ts in &timestamps {
                    if let Some(existing) = lines.iter_mut().find(|l| (l.time_ms as i64 - *ts as i64).abs() < 500) {
                        existing.romaji = Some(clean_ro.to_string());
                    }
                }
                continue;
            }

            let (clean_text, words) = parse_enhanced_words(remaining_text);
            let (main_text, inline_trans) = if clean_text.contains(" // ") {
                let parts: Vec<&str> = clean_text.splitn(2, " // ").collect();
                (parts[0].trim().to_string(), Some(parts[1].trim().to_string()))
            } else {
                (clean_text, None)
            };

            for ts in timestamps {
                lines.push(LyricLine {
                    time_ms: ts,
                    text: main_text.clone(),
                    translation: inline_trans.clone(),
                    romaji: None,
                    words: words.clone(),
                });
            }
        }
    }

    let is_synced = !lines.is_empty();

    // Sort lines by timestamp
    lines.sort_by_key(|l| l.time_ms);

    let has_translation = lines.iter().any(|l| l.translation.is_some());
    let has_romaji = lines.iter().any(|l| l.romaji.is_some());

    let plain_text = if is_synced {
        lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join("\n")
    } else {
        content.to_string()
    };

    ParsedLyrics {
        title,
        artist,
        album,
        offset_ms,
        is_synced,
        is_instrumental,
        plain_text,
        raw_lrc: content.to_string(),
        lines,
        has_translation,
        has_romaji,
    }
}

pub fn merge_translations(
    parsed: &mut ParsedLyrics,
    tlyric: Option<&str>,
    romalrc: Option<&str>,
) {
    if let Some(t_content) = tlyric {
        let t_parsed = parse_lrc(t_content);
        for t_line in t_parsed.lines {
            if let Some(match_line) = parsed.lines.iter_mut().min_by_key(|l| {
                (l.time_ms as i64 - t_line.time_ms as i64).abs()
            }) {
                if (match_line.time_ms as i64 - t_line.time_ms as i64).abs() < 600 {
                    if !t_line.text.trim().is_empty() {
                        match_line.translation = Some(t_line.text);
                        parsed.has_translation = true;
                    }
                }
            }
        }
    }

    if let Some(r_content) = romalrc {
        let r_parsed = parse_lrc(r_content);
        for r_line in r_parsed.lines {
            if let Some(match_line) = parsed.lines.iter_mut().min_by_key(|l| {
                (l.time_ms as i64 - r_line.time_ms as i64).abs()
            }) {
                if (match_line.time_ms as i64 - r_line.time_ms as i64).abs() < 600 {
                    if !r_line.text.trim().is_empty() {
                        match_line.romaji = Some(r_line.text);
                        parsed.has_romaji = true;
                    }
                }
            }
        }
    }
}

fn parse_timestamp(tag: &str) -> Option<u32> {
    // Normalizes comma to dot: e.g. "01:23,45" -> "01:23.45"
    let normalized = tag.replace(',', ".");
    let parts: Vec<&str> = normalized.split(':').collect();

    // Supports [mm:ss.xx] and [hh:mm:ss.xx]
    if parts.len() == 2 {
        let mins: u32 = parts[0].trim().parse().ok()?;
        let sec_parts: Vec<&str> = parts[1].split('.').collect();
        let secs: u32 = sec_parts[0].trim().parse().ok()?;
        let ms: u32 = parse_fraction(sec_parts.get(1).copied().unwrap_or(""));
        Some(mins * 60_000 + secs * 1_000 + ms)
    } else if parts.len() == 3 {
        let hours: u32 = parts[0].trim().parse().ok()?;
        let mins: u32 = parts[1].trim().parse().ok()?;
        let sec_parts: Vec<&str> = parts[2].split('.').collect();
        let secs: u32 = sec_parts[0].trim().parse().ok()?;
        let ms: u32 = parse_fraction(sec_parts.get(1).copied().unwrap_or(""));
        Some(hours * 3_600_000 + mins * 60_000 + secs * 1_000 + ms)
    } else {
        None
    }
}

fn parse_fraction(frac_str: &str) -> u32 {
    let s = frac_str.trim();
    if s.is_empty() {
        return 0;
    }
    if s.len() == 1 {
        s.parse::<u32>().unwrap_or(0) * 100
    } else if s.len() == 2 {
        s.parse::<u32>().unwrap_or(0) * 10
    } else if s.len() >= 3 {
        s[..3].parse::<u32>().unwrap_or(0)
    } else {
        0
    }
}

fn parse_enhanced_words(raw: &str) -> (String, Option<Vec<LyricWord>>) {
    // Check if word tags exist, e.g. <00:12.34>word or <00:12.34>
    if !raw.contains('<') || !raw.contains('>') {
        return (raw.to_string(), None);
    }

    let mut words = Vec::new();
    let mut clean_text = String::new();
    let mut cur = 0;
    let mut last_ms: Option<u32> = None;
    let mut cur_word_buf = String::new();

    while cur < raw.len() {
        if let Some(open_rel) = raw[cur..].find('<') {
            let open_idx = cur + open_rel;
            if open_idx > cur {
                let piece = &raw[cur..open_idx];
                cur_word_buf.push_str(piece);
                clean_text.push_str(piece);
            }

            if let Some(close_rel) = raw[open_idx..].find('>') {
                let close_idx = open_idx + close_rel;
                let tag = &raw[open_idx + 1..close_idx];
                let ts = parse_timestamp(tag);

                if let Some(new_ms) = ts {
                    if let Some(prev_ms) = last_ms {
                        let trimmed = cur_word_buf.trim();
                        if !trimmed.is_empty() {
                            words.push(LyricWord {
                                text: trimmed.to_string(),
                                start_ms: prev_ms,
                                end_ms: new_ms,
                            });
                        }
                        cur_word_buf.clear();
                    }
                    last_ms = Some(new_ms);
                }
                cur = close_idx + 1;
            } else {
                let rest = &raw[open_idx..];
                cur_word_buf.push_str(rest);
                clean_text.push_str(rest);
                break;
            }
        } else {
            let piece = &raw[cur..];
            cur_word_buf.push_str(piece);
            clean_text.push_str(piece);
            break;
        }
    }

    if let Some(prev_ms) = last_ms {
        let trimmed = cur_word_buf.trim();
        if !trimmed.is_empty() {
            words.push(LyricWord {
                text: trimmed.to_string(),
                start_ms: prev_ms,
                end_ms: prev_ms + 1000,
            });
        }
    }

    let clean = clean_text.trim().to_string();
    let word_list = if words.is_empty() { None } else { Some(words) };
    (clean, word_list)
}

pub fn update_lrc_offset(lrc_content: &str, delta_ms: i64) -> (String, i64) {
    let mut new_lines = Vec::new();
    let mut found_offset = false;
    let mut current_offset: i64 = 0;

    for line in lrc_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[offset:") && trimmed.ends_with(']') {
            if let Ok(val) = trimmed[8..trimmed.len() - 1].trim().parse::<i64>() {
                current_offset = val;
            }
            let new_offset = current_offset + delta_ms;
            new_lines.push(format!("[offset:{}]", new_offset));
            current_offset = new_offset;
            found_offset = true;
        } else {
            new_lines.push(line.to_string());
        }
    }

    if !found_offset {
        current_offset = delta_ms;
        new_lines.insert(0, format!("[offset:{}]", current_offset));
    }

    (new_lines.join("\n"), current_offset)
}
