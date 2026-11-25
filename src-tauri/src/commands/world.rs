//! Kedi Evi dünyası komutları. Anlık hareket ön yüzdedir; burada yalnızca anlamlı olaylar,
//! yerleşim ve konumlar doğrulanıp kaydedilir.

use crate::error::{AppError, AppResult};
use crate::events;
use crate::pets::model::Activity;
use crate::pets::store::{PetStore, now_ms};
use crate::state::AppState;
use crate::world::{self, EventInput, EventOutcome, Memory, WorldItem, WorldSnapshot};
use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, State};

/// Olay ödülü bekleme süreleri: (kedi, olay) → son ödül anı (ms).
#[derive(Default)]
pub struct Cooldowns(Mutex<HashMap<(i64, String), i64>>);

#[derive(Serialize)]
pub struct WorldView {
    #[serde(flatten)]
    pub world: WorldSnapshot,
    pub pets: Vec<crate::db::pets::PetRecord>,
}

#[tauri::command]
pub fn get_world(state: State<'_, AppState>, store: State<'_, PetStore>) -> AppResult<WorldView> {
    let pets = store.snapshot();
    let home: Vec<_> = pets
        .iter()
        .filter(|p| p.location == "home")
        .cloned()
        .collect();
    let world = state.db.with_tx(|c| {
        world::init(c)?;
        let away = world::offline_progress(c, now_ms(), &home)?;
        world::snapshot(c, away)
    })?;
    Ok(WorldView { world, pets })
}

#[tauri::command]
pub fn world_event(
    app: AppHandle,
    state: State<'_, AppState>,
    store: State<'_, PetStore>,
    cooldowns: State<'_, Cooldowns>,
    input: EventInput,
) -> AppResult<EventOutcome> {
    let kind = input
        .kind
        .clone()
        .ok_or_else(|| AppError::invalid("Olay türü eksik."))?;
    let mut pet = store.get(input.pet_id)?;
    let now = now_ms();
    let key = (pet.id, format!("{kind:?}"));
    let ready = {
        let mut map = cooldowns.0.lock().unwrap_or_else(|e| e.into_inner());
        let last = map.get(&key).copied().unwrap_or(0);
        let ok = now - last >= kind.cooldown_secs() * 1000;
        if ok {
            map.insert(key, now);
        }
        ok
    };
    if !ready {
        return Ok(EventOutcome {
            level: world::level_for(pet.xp),
            household_level: 0,
            pet,
            rewarded: false,
            unlocked: vec![],
            memory: None,
        });
    }
    let hour = chrono::Local::now().hour();
    let (unlocked, memory, household_level) = state
        .db
        .with_tx(|c| world::apply_event(c, &mut pet, &kind, &input, hour))?;
    let updated = pet.clone();
    store.update(pet.id, |r| {
        r.needs = updated.needs;
        r.xp = updated.xp;
        r.habits = updated.habits.clone();
    })?;
    let pet = store.persist(&state.db, pet.id)?;
    crate::pets::service::check_achievements(&state.db, &store)?;
    if !unlocked.is_empty() || memory.is_some() {
        events::broadcast(&app, events::PETS_CHANGED, ());
    }
    Ok(EventOutcome {
        level: world::level_for(pet.xp),
        pet,
        rewarded: true,
        household_level,
        unlocked,
        memory,
    })
}

/// Dünyadaki kedinin uyuyup uyumadığı (ihtiyaç hesabı buna göre yapılır).
#[tauri::command]
pub fn set_world_activity(store: State<'_, PetStore>, pet_id: i64, sleeping: bool) {
    store.set_activity(
        pet_id,
        if sleeping {
            Activity::Sleeping
        } else {
            Activity::Awake { intensity: 0.35 }
        },
    );
}

#[derive(Deserialize)]
pub struct Position {
    id: i64,
    room: String,
    x: f64,
    y: f64,
}

/// Kedilerin oda ve konumlarını toplu kaydeder (her karede değil; aralıklarla ve kapanışta).
#[tauri::command]
pub fn save_world_positions(
    state: State<'_, AppState>,
    store: State<'_, PetStore>,
    positions: Vec<Position>,
) -> AppResult<()> {
    for p in positions.iter().take(32) {
        if !["living", "bedroom", "playroom"].contains(&p.room.as_str())
            || !p.x.is_finite()
            || !p.y.is_finite()
        {
            continue;
        }
        let _ = store.update(p.id, |r| {
            r.room = p.room.clone();
            r.wx = Some(p.x.clamp(0.0, 320.0));
            r.wy = Some(p.y.clamp(0.0, 192.0));
        });
    }
    store.flush(&state.db)
}

#[tauri::command]
pub fn place_world_item(
    state: State<'_, AppState>,
    kind: String,
    room: String,
    tx: i64,
    ty: i64,
    rot: i64,
) -> AppResult<WorldItem> {
    state
        .db
        .with_tx(|c| world::place(c, &kind, &room, tx, ty, rot))
}

#[tauri::command]
pub fn move_world_item(
    state: State<'_, AppState>,
    id: i64,
    room: String,
    tx: i64,
    ty: i64,
    rot: i64,
) -> AppResult<WorldItem> {
    state
        .db
        .with_tx(|c| world::move_item(c, id, &room, tx, ty, rot))
}

#[tauri::command]
pub fn remove_world_item(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    state.db.with_tx(|c| world::remove(c, id))
}

#[tauri::command]
pub fn fill_world_bowl(
    state: State<'_, AppState>,
    id: i64,
    food: Option<String>,
) -> AppResult<WorldItem> {
    let today = crate::pets::service::today();
    state
        .db
        .with_tx(|c| world::fill(c, id, food.as_deref(), &today))
}

#[tauri::command]
pub fn set_world_room_style(
    state: State<'_, AppState>,
    room: String,
    floor: Option<String>,
    wall: Option<String>,
) -> AppResult<()> {
    state
        .db
        .with(|c| world::set_room_style(c, &room, floor.as_deref(), wall.as_deref()))
}

#[tauri::command]
pub fn pet_memories(state: State<'_, AppState>, id: i64) -> AppResult<Vec<Memory>> {
    state.db.with(|c| world::memories(c, id))
}
