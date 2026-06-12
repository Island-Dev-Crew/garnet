use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const MODES: &[&str] = &["simple", "power"];
pub const THEMES: &[&str] = &["dark", "light", "system"];

const COMMAND_TIMEOUT_DEFAULT: u64 = 900;
const COMMAND_TIMEOUT_MIN: u64 = 30;
const COMMAND_TIMEOUT_MAX: u64 = 14_400;
const MATRIX_TIMEOUT_DEFAULT: u64 = 5_400;
const MATRIX_TIMEOUT_MIN: u64 = 60;
const MATRIX_TIMEOUT_MAX: u64 = 21_600;

/// Persisted Studio UI/runtime preferences.
///
/// Stored as JSON under the per-user config directory; every field has a safe
/// default so a missing or corrupt file never blocks startup, and every write
/// passes through [`StudioSettings::normalized`] so out-of-range values can
/// never persist.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StudioSettings {
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_command_timeout")]
    pub command_timeout_secs: u64,
    #[serde(default = "default_matrix_timeout")]
    pub matrix_timeout_secs: u64,
}

fn default_mode() -> String {
    "simple".to_string()
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_command_timeout() -> u64 {
    COMMAND_TIMEOUT_DEFAULT
}

fn default_matrix_timeout() -> u64 {
    MATRIX_TIMEOUT_DEFAULT
}

impl Default for StudioSettings {
    fn default() -> Self {
        StudioSettings {
            mode: default_mode(),
            theme: default_theme(),
            command_timeout_secs: default_command_timeout(),
            matrix_timeout_secs: default_matrix_timeout(),
        }
    }
}

impl StudioSettings {
    pub fn normalized(mut self) -> Self {
        let mode = self.mode.trim().to_lowercase();
        self.mode = if MODES.contains(&mode.as_str()) {
            mode
        } else {
            default_mode()
        };
        let theme = self.theme.trim().to_lowercase();
        self.theme = if THEMES.contains(&theme.as_str()) {
            theme
        } else {
            default_theme()
        };
        self.command_timeout_secs = self
            .command_timeout_secs
            .clamp(COMMAND_TIMEOUT_MIN, COMMAND_TIMEOUT_MAX);
        self.matrix_timeout_secs = self
            .matrix_timeout_secs
            .clamp(MATRIX_TIMEOUT_MIN, MATRIX_TIMEOUT_MAX);
        self
    }
}

pub fn settings_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
    });
    base.join("garnet-studio").join("settings.json")
}

/// Load settings, falling back to defaults on a missing or unreadable file.
/// A corrupt file is never fatal: the Studio must always boot.
pub fn load() -> StudioSettings {
    let path = settings_path();
    let Ok(raw) = fs::read_to_string(&path) else {
        return StudioSettings::default();
    };
    serde_json::from_str::<StudioSettings>(&raw)
        .map(StudioSettings::normalized)
        .unwrap_or_default()
}

pub fn save(settings: StudioSettings) -> Result<StudioSettings, String> {
    let normalized = settings.normalized();
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create settings directory: {err}"))?;
    }
    let payload = serde_json::to_string_pretty(&normalized)
        .map_err(|err| format!("failed to serialize settings: {err}"))?
        + "\n";
    fs::write(&path, payload).map_err(|err| format!("failed to write settings: {err}"))?;
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_simple_dark_with_sane_timeouts() {
        let settings = StudioSettings::default();
        assert_eq!(settings.mode, "simple");
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.command_timeout_secs, COMMAND_TIMEOUT_DEFAULT);
        assert_eq!(settings.matrix_timeout_secs, MATRIX_TIMEOUT_DEFAULT);
    }

    #[test]
    fn normalization_clamps_timeouts_and_rejects_unknown_modes() {
        let settings = StudioSettings {
            mode: "ULTRA".to_string(),
            theme: "neon".to_string(),
            command_timeout_secs: 1,
            matrix_timeout_secs: u64::MAX,
        }
        .normalized();
        assert_eq!(settings.mode, "simple");
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.command_timeout_secs, COMMAND_TIMEOUT_MIN);
        assert_eq!(settings.matrix_timeout_secs, MATRIX_TIMEOUT_MAX);
    }

    #[test]
    fn normalization_accepts_known_values_case_insensitively() {
        let settings = StudioSettings {
            mode: " Power ".to_string(),
            theme: "SYSTEM".to_string(),
            command_timeout_secs: 600,
            matrix_timeout_secs: 7200,
        }
        .normalized();
        assert_eq!(settings.mode, "power");
        assert_eq!(settings.theme, "system");
        assert_eq!(settings.command_timeout_secs, 600);
        assert_eq!(settings.matrix_timeout_secs, 7200);
    }

    #[test]
    fn corrupt_settings_json_falls_back_to_defaults() {
        let parsed = serde_json::from_str::<StudioSettings>("{\"mode\": 42}");
        assert!(parsed.is_err());
        // load() maps this error to defaults; assert the contract here too.
        assert_eq!(
            serde_json::from_str::<StudioSettings>("{}")
                .map(StudioSettings::normalized)
                .unwrap_or_default(),
            StudioSettings::default()
        );
    }
}
