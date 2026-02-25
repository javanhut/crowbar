use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
    Unknown,
}

#[derive(Debug)]
pub struct MediaInfo {
    pub available: bool,
    pub status: PlaybackStatus,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub player: String,
    pub position: i64,  // microseconds
    pub length: i64,    // microseconds
}

pub fn get_media_info() -> MediaInfo {
    let mut info = MediaInfo {
        available: false,
        status: PlaybackStatus::Unknown,
        title: String::new(),
        artist: String::new(),
        album: String::new(),
        player: String::new(),
        position: 0,
        length: 0,
    };

    // Get player list
    let Ok(output) = Command::new("playerctl").arg("-l").output() else {
        return info;
    };
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return info;
    }

    if let Some(player) = text.lines().next() {
        info.player = player.to_string();
        info.available = true;
    } else {
        return info;
    }

    // Get playback status
    if let Ok(output) = Command::new("playerctl").arg("status").output() {
        let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info.status = match status.as_str() {
            "Playing" => PlaybackStatus::Playing,
            "Paused" => PlaybackStatus::Paused,
            "Stopped" => PlaybackStatus::Stopped,
            _ => PlaybackStatus::Unknown,
        };
    }

    // Get metadata
    if let Ok(output) = Command::new("playerctl")
        .args(["metadata", "--format", "{{title}}|||{{artist}}|||{{album}}"])
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let parts: Vec<&str> = text.split("|||").collect();
        if !parts.is_empty() {
            info.title = parts[0].to_string();
        }
        if parts.len() >= 2 {
            info.artist = parts[1].to_string();
        }
        if parts.len() >= 3 {
            info.album = parts[2].to_string();
        }
    }

    // Get position
    if let Ok(output) = Command::new("playerctl").arg("position").output() {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if let Ok(pos) = text.parse::<f64>() {
            info.position = (pos * 1_000_000.0) as i64;
        }
    }

    // Get length
    if let Ok(output) = Command::new("playerctl")
        .args(["metadata", "mpris:length"])
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if let Ok(length) = text.parse::<i64>() {
            info.length = length;
        }
    }

    info
}

pub fn play_pause() {
    let _ = Command::new("playerctl").arg("play-pause").status();
}

pub fn next() {
    let _ = Command::new("playerctl").arg("next").status();
}

pub fn previous() {
    let _ = Command::new("playerctl").arg("previous").status();
}

pub fn format_duration(microseconds: i64) -> String {
    let seconds = microseconds / 1_000_000;
    let minutes = seconds / 60;
    let secs = seconds % 60;
    format!("{minutes}:{secs:02}")
}

pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s[..max_len].to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
