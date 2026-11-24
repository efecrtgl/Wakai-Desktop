use crate::error::AppResult;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct AppInfo {
    version: String,
    schema_version: usize,
    database_path: Option<String>,
}

#[tauri::command]
pub fn app_info(app: tauri::AppHandle, state: State<'_, AppState>) -> AppResult<AppInfo> {
    let schema_version = state.db.with(crate::db::migrations::current_version)?;
    Ok(AppInfo {
        version: app.package_info().version.to_string(),
        schema_version,
        database_path: state.db.path().map(|p| p.display().to_string()),
    })
}
