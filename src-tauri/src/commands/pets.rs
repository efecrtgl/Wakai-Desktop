//! Kedi komutları. Etkileşimler `pets::service` üzerinden uygulanır; masaüstündeki kedinin
//! animasyonu motora iletilir. Her değişiklik `pets:changed` olayını yayınlar.

use crate::db::pets::PetRecord;
use crate::error::{AppError, AppResult};
use crate::events;
use crate::pets::art;
use crate::pets::engine::{Anim, Cmd, PetEngine};
use crate::pets::model::{self, ACCESSORIES, FOODS, Personality, TOYS};
use crate::pets::service::{self, ACHIEVEMENTS};
use crate::pets::store::PetStore;
use crate::state::AppState;
use serde::Serialize;
use tauri::{AppHandle, State};

fn changed(app: &AppHandle, engine: &PetEngine, reload: bool) {
    if reload {
        engine.send(Cmd::Reload);
    }
    events::broadcast(app, events::PETS_CHANGED, ());
}

#[tauri::command]
pub fn list_pets(store: State<'_, PetStore>) -> Vec<PetRecord> {
    store.snapshot()
}

#[derive(Serialize)]
pub struct CatalogItem {
    id: &'static str,
    name: String,
    unlocked: bool,
    /// Yiyecekler için bugünkü miktar
    count: Option<i64>,
    slot: Option<model::Slot>,
    unlocked_by: Option<&'static str>,
}

#[derive(Serialize)]
pub struct Catalog {
    breeds: Vec<(String, String)>,
    personalities: Vec<&'static str>,
    foods: Vec<CatalogItem>,
    toys: Vec<CatalogItem>,
    accessories: Vec<CatalogItem>,
}

#[tauri::command]
pub fn pet_catalog(state: State<'_, AppState>) -> AppResult<Catalog> {
    let inv = state
        .db
        .with(|c| crate::db::pets::inventory(c, &service::today()))?;
    let unlocked = state.db.with(crate::db::pets::achievements)?;
    let open = |a: Option<&str>| a.is_none_or(|id| unlocked.contains_key(id));
    let sprites = art::sprites();
    Ok(Catalog {
        breeds: model::BREEDS
            .iter()
            .map(|b| {
                (
                    (*b).to_string(),
                    sprites
                        .breeds
                        .get(*b)
                        .map(|x| x.name.clone())
                        .unwrap_or_default(),
                )
            })
            .collect(),
        personalities: Personality::ALL.iter().map(|p| p.as_str()).collect(),
        foods: FOODS
            .iter()
            .map(|f| CatalogItem {
                id: f.id,
                name: f.name.into(),
                unlocked: true,
                count: Some(inv.get(f.id).copied().unwrap_or(0)),
                slot: None,
                unlocked_by: None,
            })
            .collect(),
        toys: TOYS
            .iter()
            .map(|t| CatalogItem {
                id: t.id,
                name: t.name.into(),
                unlocked: open(t.unlocked_by),
                count: None,
                slot: None,
                unlocked_by: t.unlocked_by,
            })
            .collect(),
        accessories: ACCESSORIES
            .iter()
            .map(|a| CatalogItem {
                id: a.id,
                name: sprites
                    .accessories
                    .get(a.id)
                    .map(|x| x.name.clone())
                    .unwrap_or_default(),
                unlocked: open(a.unlocked_by),
                count: None,
                slot: Some(a.slot),
                unlocked_by: a.unlocked_by,
            })
            .collect(),
    })
}

#[derive(Serialize)]
pub struct AchievementView {
    #[serde(flatten)]
    def: service::AchievementDef,
    unlocked_at: Option<String>,
}

#[derive(Serialize)]
pub struct Progress {
    achievements: Vec<AchievementView>,
    feeds: i64,
    strokes: i64,
    plays: i64,
    feed_streak: i64,
}

#[tauri::command]
pub fn pet_progress(state: State<'_, AppState>) -> AppResult<Progress> {
    state.db.with(|c| {
        let unlocked = crate::db::pets::achievements(c)?;
        Ok(Progress {
            achievements: ACHIEVEMENTS
                .iter()
                .map(|d| AchievementView {
                    def: *d,
                    unlocked_at: unlocked.get(d.id).cloned(),
                })
                .collect(),
            feeds: crate::db::pets::stat(c, "feeds")?,
            strokes: crate::db::pets::stat(c, "strokes")?,
            plays: crate::db::pets::stat(c, "plays")?,
            feed_streak: crate::db::pets::stat(c, "feed_streak")?,
        })
    })
}

#[tauri::command]
pub fn adopt_pet(
    app: AppHandle,
    state: State<'_, AppState>,
    store: State<'_, PetStore>,
    engine: State<'_, PetEngine>,
    name: String,
    breed: String,
    personality: String,
) -> AppResult<PetRecord> {
    let p = service::adopt(&state.db, &store, &name, &breed, &personality)?;
    changed(&app, &engine, true);
    Ok(p)
}

#[tauri::command]
pub fn rename_pet(
    app: AppHandle,
    state: State<'_, AppState>,
    store: State<'_, PetStore>,
    engine: State<'_, PetEngine>,
    id: i64,
    name: String,
) -> AppResult<PetRecord> {
    let name = crate::validate::text("Kedi adı", &name, 24, true)?;
    store.update(id, |r| r.name = name)?;
    let p = store.persist(&state.db, id)?;
    changed(&app, &engine, false);
    Ok(p)
}

/// Kediyi koleksiyondan çıkarır (geri alınamaz; arayüz onay ister).
#[tauri::command]
pub fn release_pet(
    app: AppHandle,
    state: State<'_, AppState>,
    store: State<'_, PetStore>,
    engine: State<'_, PetEngine>,
    id: i64,
) -> AppResult<()> {
    state.db.with(|c| crate::db::pets::delete(c, id))?;
    store.remove(id);
    changed(&app, &engine, true);
    Ok(())
}

#[tauri::command]
pub fn set_pet_location(
    app: AppHandle,
    state: State<'_, AppState>,
    store: State<'_, PetStore>,
    engine: State<'_, PetEngine>,
    id: i64,
    location: String,
) -> AppResult<PetRecord> {
    if location != "desktop" && location != "home" {
        return Err(AppError::invalid("Geçersiz konum."));
    }
    store.update(id, |r| r.location = location)?;
    let p = store.persist(&state.db, id)?;
    changed(&app, &engine, true);
    Ok(p)
}

#[tauri::command]
pub fn set_pet_paused(
    app: AppHandle,
    state: State<'_, AppState>,
    store: State<'_, PetStore>,
    engine: State<'_, PetEngine>,
    id: i64,
    paused: bool,
) -> AppResult<PetRecord> {
    store.update(id, |r| r.paused = paused)?;
    let p = store.persist(&state.db, id)?;
    changed(&app, &engine, true);
    Ok(p)
}

#[tauri::command]
pub fn set_pet_accessory(
    app: AppHandle,
    state: State<'_, AppState>,
    store: State<'_, PetStore>,
    engine: State<'_, PetEngine>,
    id: i64,
    slot: String,
    accessory: Option<String>,
) -> AppResult<PetRecord> {
    if slot != "head" && slot != "neck" {
        return Err(AppError::invalid("Geçersiz aksesuar yuvası."));
    }
    let p = service::set_accessory(&state.db, &store, id, &slot, accessory.as_deref())?;
    changed(&app, &engine, true);
    Ok(p)
}

#[tauri::command]
pub fn feed_pet(
    app: AppHandle,
    state: State<'_, AppState>,
    store: State<'_, PetStore>,
    engine: State<'_, PetEngine>,
    id: i64,
    food: String,
) -> AppResult<PetRecord> {
    let p = service::feed(&state.db, &store, id, &food)?;
    engine.send(Cmd::Animate(id, Anim::Eat));
    changed(&app, &engine, false);
    Ok(p)
}

#[tauri::command]
pub fn stroke_pet(
    app: AppHandle,
    state: State<'_, AppState>,
    store: State<'_, PetStore>,
    engine: State<'_, PetEngine>,
    id: i64,
) -> AppResult<PetRecord> {
    let p = service::stroke(&state.db, &store, id)?;
    engine.send(Cmd::Animate(id, Anim::Petted));
    changed(&app, &engine, false);
    Ok(p)
}

#[tauri::command]
pub fn play_with_pet(
    app: AppHandle,
    state: State<'_, AppState>,
    store: State<'_, PetStore>,
    engine: State<'_, PetEngine>,
    id: i64,
    toy: String,
) -> AppResult<PetRecord> {
    let p = service::play(&state.db, &store, id, &toy)?;
    engine.send(Cmd::Animate(id, Anim::Play(toy)));
    changed(&app, &engine, false);
    Ok(p)
}

/// Masaüstündeki kediyi imlecin yanına çağırır (kaybolduysa bulmak için).
#[tauri::command]
pub fn summon_pet(engine: State<'_, PetEngine>, id: i64) {
    engine.send(Cmd::Animate(id, Anim::Summon));
}
