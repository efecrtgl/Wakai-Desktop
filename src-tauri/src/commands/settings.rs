use crate::error::{AppError, AppResult};
use crate::events;
use crate::settings::{self, AppSettings};
use crate::state::{AppState, on_main};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppResult<AppSettings> {
    settings::load(&state.db)
}

#[tauri::command]
pub async fn update_settings(app: AppHandle, patch: serde_json::Value) -> AppResult<AppSettings> {
    on_main(&app, move |app| apply_settings_patch(app, patch))
}

fn apply_settings_patch(app: &AppHandle, patch: serde_json::Value) -> AppResult<AppSettings> {
    let state = app.state::<AppState>();
    let before = settings::load(&state.db)?;
    let next = settings::update(&state.db, patch)?;
    if before.launch_on_startup != next.launch_on_startup {
        apply_autostart(app, next.launch_on_startup)?;
    }
    if before.widgets_visible != next.widgets_visible {
        crate::widgets::manager::apply_global_visibility(app, next.widgets_visible)?;
    }
    events::broadcast(app, events::SETTINGS_CHANGED, next.clone());
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
