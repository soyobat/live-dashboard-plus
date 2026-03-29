use std::collections::HashMap;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MusicInfo {
    pub app: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub artist: String,
}

fn music_process_map() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("spotify.exe", "Spotify");
    m.insert("qqmusic.exe", "QQ音乐");
    m.insert("cloudmusic.exe", "网易云音乐");
    m.insert("foobar2000.exe", "foobar2000");
    m.insert("itunes.exe", "Apple Music");
    m.insert("applemusic.exe", "Apple Music");
    m.insert("kugou.exe", "酷狗音乐");
    m.insert("kwmusic.exe", "酷我音乐");
    m.insert("aimp.exe", "AIMP");
    m.insert("musicbee.exe", "MusicBee");
    m.insert("vlc.exe", "VLC");
    m.insert("potplayer.exe", "PotPlayer");
    m.insert("potplayer64.exe", "PotPlayer");
    m.insert("potplayermini.exe", "PotPlayer");
    m.insert("potplayermini64.exe", "PotPlayer");
    m.insert("wmplayer.exe", "Windows Media Player");
    m
}

fn parse_spotify(title: &str) -> Option<(String, String)> {
    if matches!(title, "Spotify" | "Spotify Free" | "Spotify Premium") {
        return None;
    }
    if let Some((artist, song)) = title.split_once(" - ") {
        Some((song.trim().to_string(), artist.trim().to_string()))
    } else {
        Some((title.to_string(), String::new()))
    }
}

fn parse_foobar(title: &str) -> Option<(String, String)> {
    // Remove trailing "[foobar2000 ...]"
    let cleaned = {
        let re_start = title.rfind(" [foobar2000");
        if let Some(pos) = re_start {
            title[..pos].trim().to_string()
        } else {
            title.to_string()
        }
    };
    if cleaned.is_empty() || cleaned == title {
        if let Some((a, b)) = title.split_once(" - ") {
            return Some((b.trim().to_string(), a.trim().to_string()));
        }
        return Some((title.to_string(), String::new()));
    }
    if let Some((artist, song)) = cleaned.split_once(" - ") {
        Some((song.trim().to_string(), artist.trim().to_string()))
    } else {
        Some((cleaned, String::new()))
    }
}

fn parse_dash(title: &str) -> Option<(String, String)> {
    if let Some((song, artist)) = title.split_once(" - ") {
        Some((song.trim().to_string(), artist.trim().to_string()))
    } else {
        Some((title.to_string(), String::new()))
    }
}

/// Scan all visible windows to find a known music player and extract now-playing info.
pub fn get_music_info() -> Option<MusicInfo> {
    let map = music_process_map();
    let mut result: Option<MusicInfo> = None;

    crate::win32::enum_visible_windows(|_hwnd, title, pid| {
        if result.is_some() {
            return false; // stop
        }
        let proc_name = match crate::win32::process_name_by_pid(pid) {
            Some(n) => n,
            None => return true,
        };
        let app_display = match map.get(proc_name.as_str()) {
            Some(&name) => name,
            None => return true,
        };

        let parsed = if proc_name == "spotify.exe" {
            parse_spotify(&title)
        } else if proc_name == "foobar2000.exe" {
            parse_foobar(&title)
        } else {
            parse_dash(&title)
        };

        if let Some((song, artist)) = parsed {
            result = Some(MusicInfo {
                app: app_display.to_string(),
                title: song.chars().take(256).collect(),
                artist: artist.chars().take(256).collect(),
            });
        }
        true
    });

    result
}
