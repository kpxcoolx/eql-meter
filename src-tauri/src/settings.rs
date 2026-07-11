use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowGeometry {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Default for WindowGeometry {
    fn default() -> Self {
        Self {
            x: 100.0,
            y: 100.0,
            width: 1100.0,
            height: 720.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub last_log_path: Option<String>,
    pub auto_monitor_on_start: bool,
    /// Deprecated: ignored. Multi-mob pulls always track every NPC.
    pub focus_primary: bool,
    /// Drop finished fights below this total damage (0 = keep all).
    pub min_fight_damage: u64,
    /// When true (default), ignore other players' combat lines in your log.
    pub self_only: bool,
    pub main_window: Option<WindowGeometry>,
    pub overlay_window: Option<WindowGeometry>,
    /// Optional path to EverQuest `spells_us.txt` for ability name enrichment.
    pub spells_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            last_log_path: None,
            auto_monitor_on_start: true,
            focus_primary: false,
            min_fight_damage: 0,
            self_only: true,
            main_window: None,
            overlay_window: None,
            spells_path: None,
        }
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("config dir: {e}"))?;
    fs::create_dir_all(&dir).map_err(|e| format!("create config dir: {e}"))?;
    Ok(dir.join("settings.json"))
}

pub fn load_settings(app: &AppHandle) -> AppSettings {
    let Ok(path) = settings_path(app) else {
        return AppSettings::default();
    };
    let Ok(text) = fs::read_to_string(path) else {
        return AppSettings::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let text =
        serde_json::to_string_pretty(settings).map_err(|e| format!("serialize settings: {e}"))?;
    fs::write(path, text).map_err(|e| format!("write settings: {e}"))
}

pub fn remember_log_path(app: &AppHandle, path: &str) -> Result<(), String> {
    let mut settings = load_settings(app);
    settings.last_log_path = Some(path.to_string());
    save_settings(app, &settings)
}

pub fn remember_window(
    app: &AppHandle,
    label: &str,
    geometry: WindowGeometry,
) -> Result<AppSettings, String> {
    let mut settings = load_settings(app);
    match label {
        "main" => settings.main_window = Some(geometry),
        "overlay" => settings.overlay_window = Some(geometry),
        _ => return Err(format!("unknown window label: {label}")),
    }
    save_settings(app, &settings)?;
    Ok(settings)
}

pub fn remember_spells_path(app: &AppHandle, path: &str) -> Result<(), String> {
    let mut settings = load_settings(app);
    settings.spells_path = Some(path.to_string());
    save_settings(app, &settings)
}
