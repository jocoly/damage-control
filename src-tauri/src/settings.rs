use serde::{Deserialize, Serialize};
use std::{fs, path::Path, process::Command, sync::Mutex};
use tauri::Manager;

pub const SETTINGS_FILE_NAME: &str = "settings.json";

const STARTUP_REGISTRY_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const STARTUP_REGISTRY_VALUE_NAME: &str = "Knight Shift";

#[derive(Default)]
pub struct SettingsState {
    settings: Mutex<AppSettings>,
}

impl SettingsState {
    pub fn load(&self, settings: AppSettings) {
        *self.settings.lock().expect("settings lock poisoned") = settings;
    }

    fn current(&self) -> AppSettings {
        self.settings
            .lock()
            .expect("settings lock poisoned")
            .clone()
    }

    fn replace(&self, settings: AppSettings) {
        *self.settings.lock().expect("settings lock poisoned") = settings;
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AppSettings {
    #[serde(default)]
    pub run_on_startup: bool,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default = "default_show_taskbar_icon")]
    pub show_taskbar_icon: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            run_on_startup: false,
            always_on_top: false,
            show_taskbar_icon: true,
        }
    }
}

fn default_show_taskbar_icon() -> bool {
    true
}

#[tauri::command]
pub fn get_app_settings(settings: tauri::State<'_, std::sync::Arc<SettingsState>>) -> AppSettings {
    settings.current()
}

#[tauri::command]
pub fn update_app_settings(
    app: tauri::AppHandle,
    settings_state: tauri::State<'_, std::sync::Arc<SettingsState>>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let previous_settings = settings_state.current();

    apply_window_settings(&app, &settings)?;

    if previous_settings.run_on_startup != settings.run_on_startup {
        set_run_on_startup(settings.run_on_startup).map_err(|error| error.to_string())?;
    }

    let settings_path = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join(SETTINGS_FILE_NAME);

    save_settings_to_path(&settings, &settings_path).map_err(|error| error.to_string())?;
    settings_state.replace(settings.clone());

    Ok(settings)
}

pub fn load_settings_file(path: &Path) -> AppSettings {
    fs::read_to_string(path)
        .ok()
        .and_then(|settings_json| serde_json::from_str::<AppSettings>(&settings_json).ok())
        .unwrap_or_default()
}

fn save_settings_to_path(
    settings: &AppSettings,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let settings_json = serde_json::to_string_pretty(settings)?;
    let temp_path = path.with_extension("json.tmp");

    fs::write(&temp_path, settings_json)?;

    if path.exists() {
        fs::remove_file(path)?;
    }

    fs::rename(temp_path, path)?;

    Ok(())
}

pub fn apply_app_settings(app: &tauri::AppHandle, settings: &AppSettings) -> Result<(), String> {
    apply_window_settings(app, settings)?;
    set_run_on_startup(settings.run_on_startup).map_err(|error| error.to_string())
}

fn apply_window_settings(app: &tauri::AppHandle, settings: &AppSettings) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_always_on_top(settings.always_on_top)
            .map_err(|error| error.to_string())?;
        window
            .set_skip_taskbar(!settings.show_taskbar_icon)
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn set_run_on_startup(enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    let startup_registry_path = format!("HKCU\\{STARTUP_REGISTRY_KEY}");
    let status = if enabled {
        let executable_path = std::env::current_exe()?;
        let executable_value = format!("\"{}\"", executable_path.display());

        Command::new("reg")
            .args([
                "add",
                &startup_registry_path,
                "/v",
                STARTUP_REGISTRY_VALUE_NAME,
                "/t",
                "REG_SZ",
                "/d",
                &executable_value,
                "/f",
            ])
            .status()?
    } else {
        Command::new("reg")
            .args([
                "delete",
                &startup_registry_path,
                "/v",
                STARTUP_REGISTRY_VALUE_NAME,
                "/f",
            ])
            .status()?
    };

    if status.success() || !enabled {
        Ok(())
    } else {
        Err(format!("registry command failed with status {status}").into())
    }
}
