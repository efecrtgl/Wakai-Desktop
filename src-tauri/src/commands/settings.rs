use crate::error::{AppError, AppResult};
use crate::events;
use crate::settings::{self, AppSettings};
use crate::state::AppState;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppResult<AppSettings> {
    settings::load(&state.db)
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    patch: serde_json::Value,
) -> AppResult<AppSettings> {
    let before = settings::load(&state.db)?;
    let next = settings::update(&state.db, patch)?;
    if before.launch_on_startup != next.launch_on_startup {
        apply_autostart(&app, next.launch_on_startup)?;
    }
    events::broadcast(&app, events::SETTINGS_CHANGED, next.clone());
    Ok(next)
}

/// Windows başlangıç kaydını (HKCU\...\Run) ayarla eşitler.
pub fn apply_autostart(app: &AppHandle, enabled: bool) -> AppResult<()> {
    let launcher = app.autolaunch();
    let result = if enabled {
        launcher.enable()
    } else {
        launcher.disable()
    };
    result.map_err(|e| AppError::Other(format!("Başlangıç ayarı uygulanamadı: {e}")))
}
