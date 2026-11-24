//! Kedilerin bellekteki tek doğruluk kaynağı. Motor (masaüstü) ve komutlar (Kontrol Merkezi)
//! aynı deposu kullanır. İhtiyaçlar geçen süreden hesaplanır; veritabanına yalnızca
//! etkileşimlerde, düzenli aralıklarla (motor) ve kapanışta yazılır.

use super::model::{Activity, Needs, NeedsDelta};
use crate::db::{self, Db, pets::PetRecord};
use crate::error::{AppError, AppResult};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

pub fn now_ms() -> i64 {
    chrono::Local::now().timestamp_millis()
}

struct Entry {
    rec: PetRecord,
    /// `needs_at` anından bu yana kedinin ne yaptığı.
    activity: Activity,
}

#[derive(Default)]
pub struct PetStore {
    pets: Mutex<HashMap<i64, Entry>>,
}

impl PetStore {
    fn lock(&self) -> MutexGuard<'_, HashMap<i64, Entry>> {
        self.pets.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Veritabanından yükler; uygulama kapalıyken geçen süre nazikçe uygulanır.
    pub fn load(&self, db: &Db) -> AppResult<()> {
        let now = now_ms();
        let mut map = HashMap::new();
        for mut rec in db.with(db::pets::list)? {
            let secs = (now - rec.needs_at).max(0) as f64 / 1000.0;
            rec.needs.advance(secs, Activity::Offline, rec.personality);
            rec.needs_at = now;
            map.insert(
                rec.id,
                Entry {
                    rec,
                    activity: Activity::Awake { intensity: 0.2 },
                },
            );
        }
        *self.lock() = map;
        Ok(())
    }

    fn advance(e: &mut Entry, now: i64) {
        let secs = (now - e.rec.needs_at).max(0) as f64 / 1000.0;
        e.rec.needs.advance(secs, e.activity, e.rec.personality);
        e.rec.needs_at = now;
    }

    /// Tüm kedilerin şu anki hâli.
    pub fn snapshot(&self) -> Vec<PetRecord> {
        let now = now_ms();
        let mut map = self.lock();
        let mut v: Vec<PetRecord> = map
            .values_mut()
            .map(|e| {
                Self::advance(e, now);
                e.rec.clone()
            })
            .collect();
        v.sort_by_key(|p| p.id);
        v
    }

    pub fn get(&self, id: i64) -> AppResult<PetRecord> {
        let now = now_ms();
        let mut map = self.lock();
        let e = map.get_mut(&id).ok_or(AppError::NotFound)?;
        Self::advance(e, now);
        Ok(e.rec.clone())
    }

    pub fn needs(&self, id: i64) -> Option<Needs> {
        self.get(id).ok().map(|p| p.needs)
    }

    /// Kedinin şu anki etkinliğini değiştirir (önce geçen süre eski etkinlikle hesaplanır).
    pub fn set_activity(&self, id: i64, activity: Activity) {
        let now = now_ms();
        if let Some(e) = self.lock().get_mut(&id) {
            Self::advance(e, now);
            e.activity = activity;
        }
    }

    pub fn apply(&self, id: i64, delta: NeedsDelta) -> AppResult<PetRecord> {
        self.update(id, |r| r.needs.apply(delta))
    }

    /// Kaydı değiştirir ve döndürür (veritabanına yazmaz; bkz. `persist`).
    pub fn update(&self, id: i64, f: impl FnOnce(&mut PetRecord)) -> AppResult<PetRecord> {
        let now = now_ms();
        let mut map = self.lock();
        let e = map.get_mut(&id).ok_or(AppError::NotFound)?;
        Self::advance(e, now);
        f(&mut e.rec);
        Ok(e.rec.clone())
    }

    pub fn insert(&self, rec: PetRecord) {
        self.lock().insert(
            rec.id,
            Entry {
                rec,
                activity: Activity::Awake { intensity: 0.2 },
            },
        );
    }

    pub fn remove(&self, id: i64) {
        self.lock().remove(&id);
    }

    /// Bir kediyi veritabanına yazar.
    pub fn persist(&self, db: &Db, id: i64) -> AppResult<PetRecord> {
        let rec = self.get(id)?;
        db.with(|c| db::pets::save(c, &rec))?;
        Ok(rec)
    }

    /// Tüm kedileri tek işlemde yazar.
    pub fn flush(&self, db: &Db) -> AppResult<()> {
        let all = self.snapshot();
        db.with_tx(|tx| {
            for p in &all {
                db::pets::save(tx, p)?;
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pets::model::Personality;

    #[test]
    fn load_applies_offline_time_and_flush_persists() {
        let db = Db::open_in_memory().unwrap();
        let hour_ago = now_ms() - 3_600_000;
        let p = db
            .with(|c| db::pets::insert(c, "Duman", "gray", Personality::Sleepy, hour_ago))
            .unwrap();
        let store = PetStore::default();
        store.load(&db).unwrap();
        let loaded = store.get(p.id).unwrap();
        assert!(loaded.needs.rest > 80.0, "kapalıyken dinlenmiş olmalı");
        store
            .apply(
                p.id,
                NeedsDelta {
                    happiness: -30.0,
                    ..Default::default()
                },
            )
            .unwrap();
        store.flush(&db).unwrap();
        let saved = db.with(|c| db::pets::get(c, p.id)).unwrap();
        assert!(saved.needs.happiness < 60.0);
        assert!(saved.needs_at >= hour_ago + 3_000_000);
    }

    #[test]
    fn unknown_pet_is_not_found() {
        let store = PetStore::default();
        assert!(matches!(store.get(1), Err(AppError::NotFound)));
        assert!(store.apply(1, NeedsDelta::default()).is_err());
    }
}
