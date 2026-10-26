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
        }
    }
}

pub fn parse_lrc(content: &str) -> ParsedLyrics {
    let mut title = None;
    let mut artist = None;
    let mut album = None;
    let mut offset_ms: i64 = 0;
    let mut is_instrumental = false;
    let mut lines = Vec::new();

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
            let (clean_text, words) = parse_enhanced_words(remaining_text);
            for ts in timestamps {
                lines.push(LyricLine {
                    time_ms: ts,
                    text: clean_text.clone(),
                    words: words.clone(),
                });
            }
        }
    }

    let is_synced = !lines.is_empty();

    // Sort lines by timestamp
    lines.sort_by_key(|l| l.time_ms);

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
    }
}

fn parse_timestamp(tag: &str) -> Option<u32> {
    // Format: mm:ss or mm:ss.xx or mm:ss.xxx
    let parts: Vec<&str> = tag.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let mins: u32 = parts[0].trim().parse().ok()?;
    let sec_parts: Vec<&str> = parts[1].split('.').collect();

    let secs: u32 = sec_parts[0].trim().parse().ok()?;
    let ms: u32 = if sec_parts.len() > 1 {
        let frac_str = sec_parts[1].trim();
        if frac_str.len() == 1 {
            frac_str.parse::<u32>().ok()? * 100
        } else if frac_str.len() == 2 {
            frac_str.parse::<u32>().ok()? * 10
        } else if frac_str.len() >= 3 {
            frac_str[..3].parse::<u32>().ok()?
        } else {
            0
        }
    } else {
        0
    };

    Some(mins * 60_000 + secs * 1_000 + ms)
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
