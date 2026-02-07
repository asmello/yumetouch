use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationMode {
    Notification,
    Dialog,
    Both,
}

impl Default for NotificationMode {
    fn default() -> Self {
        Self::Notification
    }
}

impl std::str::FromStr for NotificationMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "notification" => Ok(Self::Notification),
            "dialog" => Ok(Self::Dialog),
            "both" => Ok(Self::Both),
            _ => Err(format!("invalid notification mode: {s}")),
        }
    }
}

impl std::fmt::Display for NotificationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Notification => write!(f, "notification"),
            Self::Dialog => write!(f, "dialog"),
            Self::Both => write!(f, "both"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationConfig {
    #[serde(default)]
    pub mode: NotificationMode,
    #[serde(default = "default_sound")]
    pub sound: String,
}

fn default_sound() -> String {
    "Funk".to_string()
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            mode: NotificationMode::default(),
            sound: default_sound(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub notification: NotificationConfig,
}

impl Config {
    pub fn load(path: Option<&PathBuf>) -> Self {
        let config_path = path.cloned().unwrap_or_else(default_config_path);

        match std::fs::read_to_string(&config_path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => {
                    log::info!("loaded config from {}", config_path.display());
                    config
                }
                Err(e) => {
                    log::warn!(
                        "failed to parse config at {}: {e}, using defaults",
                        config_path.display()
                    );
                    Self::default()
                }
            },
            Err(_) => {
                log::debug!("no config file at {}, using defaults", config_path.display());
                Self::default()
            }
        }
    }
}

fn default_config_path() -> PathBuf {
    dirs_or_home()
        .join(".config")
        .join("yumetouch")
        .join("config.toml")
}

fn dirs_or_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}
