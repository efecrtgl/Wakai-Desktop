//! Takvim, görev ve not komutları. Her değişiklik ilgili olayı yayınlar ve
//! hatırlatıcı zamanlayıcısını uyandırır.

use crate::db::{
    self,
    events::{Category, Event, EventInput},
    notes::{Note, NoteInput},
    tasks::{Task, TaskInput},
};
use crate::error::AppResult;
use crate::events;
use crate::scheduler;
use crate::state::AppState;
use tauri::{AppHandle, State};

fn changed(app: &AppHandle, event: &str) {
    events::broadcast(app, event, ());
    scheduler::poke(app);
}

// ── Takvim ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_event_categories(state: State<'_, AppState>) -> AppResult<Vec<Category>> {
    state.db.with(db::events::categories)
}

#[tauri::command]
pub fn list_events(state: State<'_, AppState>, from: String, to: String) -> AppResult<Vec<Event>> {
    state.db.with(|c| db::events::list_range(c, &from, &to))
}

#[tauri::command]
pub fn upcoming_events(state: State<'_, AppState>, limit: Option<u32>) -> AppResult<Vec<Event>> {
    let now = chrono::Local::now()
        .format(crate::validate::DATETIME_FORMAT)
        .to_string();
    state
        .db
        .with(|c| db::events::upcoming(c, &now, limit.unwrap_or(5)))
}

#[tauri::command]
pub fn create_event(
    app: AppHandle,
    state: State<'_, AppState>,
    input: EventInput,
) -> AppResult<Event> {
    let e = state.db.with(|c| db::events::create(c, &input))?;
    changed(&app, events::EVENTS_CHANGED);
    Ok(e)
}

#[tauri::command]
pub fn update_event(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
    input: EventInput,
) -> AppResult<Event> {
    let e = state.db.with(|c| db::events::update(c, id, &input))?;
    changed(&app, events::EVENTS_CHANGED);
    Ok(e)
}

#[tauri::command]
pub fn delete_event(app: AppHandle, state: State<'_, AppState>, id: i64) -> AppResult<()> {
    state.db.with(|c| db::events::delete(c, id))?;
    changed(&app, events::EVENTS_CHANGED);
    Ok(())
}

// ── Görevler ──────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_tasks(
    state: State<'_, AppState>,
    include_completed: Option<bool>,
) -> AppResult<Vec<Task>> {
    state
        .db
        .with(|c| db::tasks::list(c, include_completed.unwrap_or(true)))
}

#[tauri::command]
pub fn create_task(
    app: AppHandle,
    state: State<'_, AppState>,
    input: TaskInput,
) -> AppResult<Task> {
    let t = state.db.with(|c| db::tasks::create(c, &input))?;
    changed(&app, events::TASKS_CHANGED);
    Ok(t)
}

#[tauri::command]
pub fn update_task(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
    input: TaskInput,
) -> AppResult<Task> {
    let t = state.db.with(|c| db::tasks::update(c, id, &input))?;
    changed(&app, events::TASKS_CHANGED);
    Ok(t)
}

#[tauri::command]
pub fn set_task_completed(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
    completed: bool,
) -> AppResult<Task> {
    let t = state
        .db
        .with(|c| db::tasks::set_completed(c, id, completed))?;
    changed(&app, events::TASKS_CHANGED);
    Ok(t)
}

#[tauri::command]
pub fn delete_task(app: AppHandle, state: State<'_, AppState>, id: i64) -> AppResult<()> {
    state.db.with(|c| db::tasks::delete(c, id))?;
    changed(&app, events::TASKS_CHANGED);
    Ok(())
}

#[tauri::command]
pub fn clear_completed_tasks(app: AppHandle, state: State<'_, AppState>) -> AppResult<usize> {
    let n = state.db.with(db::tasks::clear_completed)?;
    changed(&app, events::TASKS_CHANGED);
    Ok(n)
}

// ── Notlar ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_notes(state: State<'_, AppState>) -> AppResult<Vec<Note>> {
    state.db.with(db::notes::list)
}

#[tauri::command]
pub fn get_note(state: State<'_, AppState>, id: i64) -> AppResult<Note> {
    state.db.with(|c| db::notes::get(c, id))
}

#[tauri::command]
pub fn create_note(
    app: AppHandle,
    state: State<'_, AppState>,
    input: NoteInput,
) -> AppResult<Note> {
    let n = state.db.with(|c| db::notes::create(c, &input))?;
    events::broadcast(&app, events::NOTES_CHANGED, n.id);
    Ok(n)
}

/// Otomatik kayıt bu komutu kullanır; içerik değişmediyse veritabanına yazılmaz.
#[tauri::command]
pub fn update_note(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
    input: NoteInput,
) -> AppResult<Note> {
    let before = state.db.with(|c| db::notes::get(c, id))?;
    let n = state.db.with(|c| db::notes::update(c, id, &input))?;
    if n != before {
        events::broadcast(&app, events::NOTES_CHANGED, n.id);
    }
    Ok(n)
}

#[tauri::command]
pub fn delete_note(app: AppHandle, state: State<'_, AppState>, id: i64) -> AppResult<()> {
    state.db.with(|c| db::notes::delete(c, id))?;
    events::broadcast(&app, events::NOTES_CHANGED, id);
    Ok(())
}
