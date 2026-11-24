//! Alışveriş Kedisi komutları. Her değişiklik `shopping:changed` olayını liste kimliğiyle yayınlar.

use crate::db::shopping::{self, ShoppingItem, ShoppingList};
use crate::error::AppResult;
use crate::events;
use crate::state::AppState;
use tauri::{AppHandle, State};

fn changed(app: &AppHandle, list_id: i64) {
    events::broadcast(app, events::SHOPPING_CHANGED, list_id);
}

#[tauri::command]
pub fn list_shopping_lists(state: State<'_, AppState>) -> AppResult<Vec<ShoppingList>> {
    state.db.with(shopping::lists)
}

#[tauri::command]
pub fn create_shopping_list(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> AppResult<ShoppingList> {
    let l = state.db.with(|c| shopping::create_list(c, &name))?;
    changed(&app, l.id);
    Ok(l)
}

#[tauri::command]
pub fn rename_shopping_list(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
    name: String,
) -> AppResult<ShoppingList> {
    let l = state.db.with(|c| shopping::rename_list(c, id, &name))?;
    changed(&app, id);
    Ok(l)
}

#[tauri::command]
pub fn delete_shopping_list(app: AppHandle, state: State<'_, AppState>, id: i64) -> AppResult<()> {
    state.db.with(|c| shopping::delete_list(c, id))?;
    changed(&app, id);
    Ok(())
}

#[tauri::command]
pub fn list_shopping_items(
    state: State<'_, AppState>,
    list_id: i64,
) -> AppResult<Vec<ShoppingItem>> {
    state.db.with(|c| shopping::items(c, list_id))
}

#[tauri::command]
pub fn add_shopping_item(
    app: AppHandle,
    state: State<'_, AppState>,
    list_id: i64,
    name: String,
    quantity: Option<String>,
) -> AppResult<ShoppingItem> {
    let i = state
        .db
        .with(|c| shopping::add_item(c, list_id, &name, quantity.as_deref().unwrap_or("")))?;
    changed(&app, list_id);
    Ok(i)
}

#[tauri::command]
pub fn update_shopping_item(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
    name: String,
    quantity: String,
) -> AppResult<ShoppingItem> {
    let i = state
        .db
        .with(|c| shopping::update_item(c, id, &name, &quantity))?;
    changed(&app, i.list_id);
    Ok(i)
}

#[tauri::command]
pub fn set_shopping_item_checked(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
    checked: bool,
) -> AppResult<ShoppingItem> {
    let i = state.db.with(|c| shopping::set_checked(c, id, checked))?;
    changed(&app, i.list_id);
    Ok(i)
}

#[tauri::command]
pub fn delete_shopping_item(app: AppHandle, state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let list_id = state.db.with(|c| shopping::delete_item(c, id))?;
    changed(&app, list_id);
    Ok(())
}

#[tauri::command]
pub fn clear_checked_shopping_items(
    app: AppHandle,
    state: State<'_, AppState>,
    list_id: i64,
) -> AppResult<usize> {
    let n = state.db.with(|c| shopping::clear_checked(c, list_id))?;
    changed(&app, list_id);
    Ok(n)
}
