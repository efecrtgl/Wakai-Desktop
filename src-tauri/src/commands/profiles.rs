use crate::error::AppResult;
use crate::profiles::{self, Profile, ProfileConfig};
use crate::state::{AppState, on_main};
use tauri::{AppHandle, State};

fn changed(app: &AppHandle) {
    crate::events::broadcast(app, crate::events::PROFILES_CHANGED, ());
    crate::tray::rebuild(app);
}

#[tauri::command]
pub fn list_profiles(state: State<'_, AppState>) -> AppResult<Vec<Profile>> {
    profiles::list(&state.db)
}

/// Profili uygular (widget pencerelerini açıp kapatabileceği için ana iş parçacığında).
#[tauri::command]
pub async fn apply_profile(app: AppHandle, id: i64) -> AppResult<Profile> {
    on_main(&app, move |app| profiles::apply(app, id))
}

#[tauri::command]
pub fn create_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    config: ProfileConfig,
) -> AppResult<Profile> {
    let p = profiles::create(&state.db, &name, &config)?;
    changed(&app);
    Ok(p)
}

#[tauri::command]
pub fn update_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
    name: Option<String>,
    config: Option<ProfileConfig>,
) -> AppResult<Profile> {
    let p = profiles::update(&state.db, id, name.as_deref(), config.as_ref())?;
    changed(&app);
    Ok(p)
}

#[tauri::command]
pub fn delete_profile(app: AppHandle, state: State<'_, AppState>, id: i64) -> AppResult<()> {
    profiles::delete(&state.db, id)?;
    // Silinen profil etkinse "etkin profil yok" olur.
    if crate::settings::load(&state.db)?.active_profile == Some(id) {
        crate::settings::update(&state.db, serde_json::json!({ "active_profile": null }))?;
    }
    changed(&app);
    Ok(())
}

#[tauri::command]
pub fn reset_profile(app: AppHandle, state: State<'_, AppState>, id: i64) -> AppResult<Profile> {
    let p = profiles::reset(&state.db, id)?;
    changed(&app);
    Ok(p)
}
