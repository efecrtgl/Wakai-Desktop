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

pub fn apply_settings_patch(app: &AppHandle, patch: serde_json::Value) -> AppResult<AppSettings> {
    let state = app.state::<AppState>();
    let before = settings::load(&state.db)?;
    let next = settings::update(&state.db, patch)?;
    if before.launch_on_startup != next.launch_on_startup {
        apply_autostart(app, next.launch_on_startup)?;
    }
    if (before.widget_scale - next.widget_scale).abs() > f64::EPSILON {
        crate::widgets::manager::apply_widget_scale(app, before.widget_scale, next.widget_scale)?;
    }
    if before.widgets_visible != next.widgets_visible {
        crate::widgets::manager::apply_global_visibility(app, next.widgets_visible)?;
    }
    if before.appearance != next.appearance
        && let Some(w) = app.get_webview_window(crate::main_window::LABEL)
    {
        crate::main_window::apply_theme(app, &w);
    }
    let pets_before = crate::pets::engine_settings(&before);
    let pets_next = crate::pets::engine_settings(&next);
    if pets_before != pets_next
        && let Some(engine) = app.try_state::<crate::pets::engine::PetEngine>()
    {
        engine.send(crate::pets::engine::Cmd::Settings(pets_next));
    }
    if before.widgets_visible != next.widgets_visible
        || before.pets_enabled != next.pets_enabled
        || before.active_profile != next.active_profile
    {
        crate::tray::sync(app, next.widgets_visible);
    }
    if next.auto_gaming_profile && !before.auto_gaming_profile {
        crate::profiles::sync_auto_gaming(app, true);
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
