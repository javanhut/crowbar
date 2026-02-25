use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClockConfig {
    pub use_12h: bool,
    pub show_seconds: bool,
    pub show_ntp_status: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub bar: BarConfig,
    pub modules: ModulesConfig,
    pub intervals: IntervalsConfig,
    pub theme: ThemeConfig,
    pub clock: ClockConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BarConfig {
    pub height: i32,
    pub position: String,
    pub monitor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModulesConfig {
    pub left: Vec<String>,
    pub right: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IntervalsConfig {
    pub clock: u32,
    pub battery: u32,
    pub power: u32,
    pub brightness: u32,
    pub connectivity: u32,
    pub media: u32,
    pub app_tracker: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub preset: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeColors {
    pub void_deep: String,
    pub void_dark: String,
    pub void_mid: String,
    pub void_light: String,
    pub frost_dark: String,
    pub frost_mid: String,
    pub frost_light: String,
    pub bifrost_blue: String,
    pub bifrost_cyan: String,
    pub bifrost_teal: String,
    pub bifrost_purple: String,
    pub fire_orange: String,
    pub fire_red: String,
    pub fire_ember: String,
    pub leaf_green: String,
    pub mead_gold: String,
    pub bark_brown: String,
    pub starlight: String,
    pub moonlight: String,
    pub sunlight: String,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            use_12h: false,
            show_seconds: false,
            show_ntp_status: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bar: BarConfig::default(),
            modules: ModulesConfig::default(),
            intervals: IntervalsConfig::default(),
            theme: ThemeConfig::default(),
            clock: ClockConfig::default(),
        }
    }
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            height: 32,
            position: "top".to_string(),
            monitor: String::new(),
        }
    }
}

impl Default for ModulesConfig {
    fn default() -> Self {
        Self {
            left: vec![
                "workspaces".into(),
                "separator".into(),
                "app_tracker".into(),
                "media".into(),
                "app_finder".into(),
            ],
            right: vec![
                "systray".into(),
                "connectivity".into(),
                "audio".into(),
                "brightness".into(),
                "power".into(),
                "battery".into(),
                "separator".into(),
                "clock".into(),
                "settings".into(),
                "power_menu".into(),
            ],
        }
    }
}

impl Default for IntervalsConfig {
    fn default() -> Self {
        Self {
            clock: 1,
            battery: 30,
            power: 10,
            brightness: 5,
            connectivity: 5,
            media: 2,
            app_tracker: 2,
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            preset: "nordic".to_string(),
            colors: ThemeColors::default(),
        }
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::nordic()
    }
}

impl ThemeColors {
    pub fn nordic() -> Self {
        Self {
            void_deep: "#0d0e14".into(),
            void_dark: "#13141c".into(),
            void_mid: "#1a1b26".into(),
            void_light: "#1e2030".into(),
            frost_dark: "#24283b".into(),
            frost_mid: "#2f3549".into(),
            frost_light: "#3b4261".into(),
            bifrost_blue: "#7aa2f7".into(),
            bifrost_cyan: "#7dcfff".into(),
            bifrost_teal: "#73daca".into(),
            bifrost_purple: "#bb9af7".into(),
            fire_orange: "#ff9e64".into(),
            fire_red: "#f7768e".into(),
            fire_ember: "#db4b4b".into(),
            leaf_green: "#9ece6a".into(),
            mead_gold: "#e0af68".into(),
            bark_brown: "#8a6642".into(),
            starlight: "#c0caf5".into(),
            moonlight: "#a9b1d6".into(),
            sunlight: "#ffc777".into(),
        }
    }

    pub fn light() -> Self {
        Self {
            void_deep: "#e8e8ed".into(),
            void_dark: "#f0f0f4".into(),
            void_mid: "#f5f5f8".into(),
            void_light: "#fafafe".into(),
            frost_dark: "#d0d0d8".into(),
            frost_mid: "#c0c0cc".into(),
            frost_light: "#b0b0be".into(),
            bifrost_blue: "#3d6bb5".into(),
            bifrost_cyan: "#2a8fa8".into(),
            bifrost_teal: "#2d9a7e".into(),
            bifrost_purple: "#7c5cbf".into(),
            fire_orange: "#c97030".into(),
            fire_red: "#c4455a".into(),
            fire_ember: "#a83030".into(),
            leaf_green: "#5a8a2a".into(),
            mead_gold: "#b08030".into(),
            bark_brown: "#6a4a2a".into(),
            starlight: "#2a2a3a".into(),
            moonlight: "#3a3a4a".into(),
            sunlight: "#c09030".into(),
        }
    }

    pub fn warm() -> Self {
        Self {
            void_deep: "#1a1410".into(),
            void_dark: "#201a14".into(),
            void_mid: "#2a2018".into(),
            void_light: "#33281e".into(),
            frost_dark: "#3d3028".into(),
            frost_mid: "#4a3a30".into(),
            frost_light: "#5a4a3a".into(),
            bifrost_blue: "#c49a6c".into(),
            bifrost_cyan: "#d4aa7c".into(),
            bifrost_teal: "#8cb070".into(),
            bifrost_purple: "#c08a9a".into(),
            fire_orange: "#ff9e64".into(),
            fire_red: "#e07060".into(),
            fire_ember: "#c05040".into(),
            leaf_green: "#a0b868".into(),
            mead_gold: "#f0c060".into(),
            bark_brown: "#8a6642".into(),
            starlight: "#e0d0c0".into(),
            moonlight: "#c8b8a0".into(),
            sunlight: "#ffd080".into(),
        }
    }

    pub fn frost() -> Self {
        Self {
            void_deep: "#0a1628".into(),
            void_dark: "#0f1d35".into(),
            void_mid: "#142540".into(),
            void_light: "#1a2d4a".into(),
            frost_dark: "#203858".into(),
            frost_mid: "#2a4a6a".into(),
            frost_light: "#3a5a7a".into(),
            bifrost_blue: "#88ccff".into(),
            bifrost_cyan: "#a0e8ff".into(),
            bifrost_teal: "#70e0d0".into(),
            bifrost_purple: "#a0b0ff".into(),
            fire_orange: "#ffb880".into(),
            fire_red: "#ff8898".into(),
            fire_ember: "#e06070".into(),
            leaf_green: "#80d888".into(),
            mead_gold: "#e8c878".into(),
            bark_brown: "#6a8090".into(),
            starlight: "#e0f0ff".into(),
            moonlight: "#c0d8f0".into(),
            sunlight: "#ffe8a0".into(),
        }
    }

    pub fn for_preset(preset: &str) -> Self {
        match preset {
            "light" => Self::light(),
            "warm" => Self::warm(),
            "frost" => Self::frost(),
            _ => Self::nordic(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::find_config();
        match config_path {
            Some(path) => {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        match toml::from_str(&content) {
                            Ok(config) => config,
                            Err(e) => {
                                eprintln!("Warning: Failed to parse config: {e}");
                                Config::default()
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to read config: {e}");
                        Config::default()
                    }
                }
            }
            None => Config::default(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let home = std::env::var("HOME").map_err(|e| format!("Could not get HOME: {e}"))?;
        let config_dir = PathBuf::from(format!("{home}/.config/crowbar"));
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .map_err(|e| format!("Could not create config dir: {e}"))?;
        }
        let config_path = config_dir.join("config.toml");
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Could not serialize config: {e}"))?;
        std::fs::write(&config_path, content)
            .map_err(|e| format!("Could not write config: {e}"))?;
        Ok(())
    }

    fn find_config() -> Option<PathBuf> {
        let home = std::env::var("HOME").unwrap_or_default();
        let locations = [
            PathBuf::from(format!("{home}/.config/crowbar/config.toml")),
        ];

        for loc in &locations {
            if loc.exists() {
                return Some(loc.clone());
            }
        }
        None
    }
}
