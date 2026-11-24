use crate::error::AppResult;
use crate::events;
use crate::focus::{self, FocusState, FocusTimer};
use crate::scheduler;
use crate::state::AppState;
use tauri::{AppHandle, State};

fn publish(app: &AppHandle, s: FocusState) -> FocusState {
    events::broadcast(app, events::FOCUS_CHANGED, s.clone());
    scheduler::poke(app);
    s
}

#[tauri::command]
pub fn focus_state(timer: State<'_, FocusTimer>) -> FocusState {
    timer.state(focus::now_ms())
}

#[tauri::command]
pub fn focus_start(
    app: AppHandle,
    state: State<'_, AppState>,
    timer: State<'_, FocusTimer>,
) -> AppResult<FocusState> {
    let cfg = scheduler::focus_config(&state.db);
    let s = timer.start(&state.db, &cfg, focus::now_ms())?;
    Ok(publish(&app, s))
}

#[tauri::command]
pub fn focus_pause(app: AppHandle, timer: State<'_, FocusTimer>) -> FocusState {
    publish(&app, timer.pause(focus::now_ms()))
}

#[tauri::command]
pub fn focus_reset(
    app: AppHandle,
    state: State<'_, AppState>,
    timer: State<'_, FocusTimer>,
) -> AppResult<FocusState> {
    let cfg = scheduler::focus_config(&state.db);
    let s = timer.reset(&state.db, &cfg, focus::now_ms())?;
    Ok(publish(&app, s))
}

#[tauri::command]
pub fn focus_skip(
    app: AppHandle,
    state: State<'_, AppState>,
    timer: State<'_, FocusTimer>,
) -> AppResult<FocusState> {
    let cfg = scheduler::focus_config(&state.db);
    let s = timer.skip(&state.db, &cfg, focus::now_ms())?;
    Ok(publish(&app, s))
}
