use crate::db::widgets::{self as repo, WidgetInstance};
use crate::desktop::Rect;
use crate::error::{AppError, AppResult};
use crate::state::{AppState, on_main};
use crate::widgets::manager;
use crate::widgets::registry::{self, DEFINITIONS, WidgetDefinition};
use serde::Serialize;
use tauri::{AppHandle, State};

fn check_id(id: &str) -> AppResult<()> {
    let ok = !id.is_empty()
        && id.len() <= 64
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    if ok {
        Ok(())
    } else {
        Err(AppError::invalid("Geçersiz widget kimliği."))
    }
}

#[tauri::command]
pub fn list_widget_definitions() -> Vec<WidgetDefinition> {
    DEFINITIONS.to_vec()
}

#[tauri::command]
pub fn list_widgets(state: State<'_, AppState>) -> AppResult<Vec<WidgetInstance>> {
    state.db.with(repo::list)
}

#[derive(Serialize)]
pub struct WidgetDetail {
    instance: WidgetInstance,
    definition: WidgetDefinition,
}

#[tauri::command]
pub fn get_widget(state: State<'_, AppState>, id: String) -> AppResult<WidgetDetail> {
    check_id(&id)?;
    let instance = state.db.with(|c| repo::get(c, &id))?;
    let definition = registry::require(&instance.kind)?.clone();
    Ok(WidgetDetail {
        instance,
        definition,
    })
}

#[tauri::command]
pub async fn add_widget(app: AppHandle, kind: String) -> AppResult<WidgetInstance> {
    on_main(&app, move |app| manager::add(app, &kind))
}

#[tauri::command]
pub async fn remove_widget(app: AppHandle, id: String) -> AppResult<()> {
    check_id(&id)?;
    on_main(&app, move |app| manager::remove(app, &id))
}

#[tauri::command]
pub async fn set_widget_visible(
    app: AppHandle,
    id: String,
    visible: bool,
) -> AppResult<WidgetInstance> {
    check_id(&id)?;
    on_main(&app, move |app| manager::set_visible(app, &id, visible))
}

#[tauri::command]
pub async fn set_widget_click_through(
    app: AppHandle,
    id: String,
    value: bool,
) -> AppResult<WidgetInstance> {
    check_id(&id)?;
    on_main(&app, move |app| manager::set_click_through(app, &id, value))
}

#[tauri::command]
pub async fn update_widget_config(
    app: AppHandle,
    id: String,
    patch: serde_json::Value,
) -> AppResult<WidgetInstance> {
    check_id(&id)?;
    on_main(&app, move |app| manager::update_config(app, &id, &patch))
}

#[tauri::command]
pub async fn set_widget_rect(app: AppHandle, id: String, rect: Rect) -> AppResult<WidgetInstance> {
    check_id(&id)?;
    if rect.width <= 0 || rect.height <= 0 || rect.width > 10_000 || rect.height > 10_000 {
        return Err(AppError::invalid("Geçersiz widget boyutu."));
    }
    on_main(&app, move |app| manager::set_rect(app, &id, rect))
}

#[tauri::command]
pub async fn reset_widget_position(app: AppHandle, id: String) -> AppResult<WidgetInstance> {
    check_id(&id)?;
    on_main(&app, move |app| manager::reset_position(app, &id))
}

#[tauri::command]
pub async fn reset_widget(app: AppHandle, id: String) -> AppResult<WidgetInstance> {
    check_id(&id)?;
    on_main(&app, move |app| manager::reset(app, &id))
}

/// Kontrol Merkezi'ni açar ve isteğe bağlı olarak bir bölüme yönlendirir.
#[tauri::command]
pub fn open_control_center(
    app: AppHandle,
    section: Option<String>,
    widget_id: Option<String>,
) -> AppResult<()> {
    crate::main_window::show(&app);
    if section.is_some() || widget_id.is_some() {
        crate::events::broadcast(
            &app,
            crate::events::NAVIGATE,
            serde_json::json!({ "section": section, "widget_id": widget_id }),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check_id;

    #[test]
    fn id_validation() {
        assert!(check_id("clock-19a3f0").is_ok());
        assert!(check_id("").is_err());
        assert!(check_id("../etc").is_err());
        assert!(check_id(&"a".repeat(65)).is_err());
    }
}
