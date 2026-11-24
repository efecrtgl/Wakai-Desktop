//! Kedi etkileşimleri ve ilerleme. Hem masaüstü motoru hem Kontrol Merkezi komutları
//! bu fonksiyonları kullanır; kurallar tek yerdedir.

use super::model::{self, FOODS, NeedsDelta, Personality, TOYS};
use super::store::{PetStore, now_ms};
use crate::db::{self, Db, pets::PetRecord};
use crate::error::{AppError, AppResult};
use crate::validate;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct AchievementDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    /// Kazanılınca açılan ödülün açıklaması.
    pub reward: &'static str,
}

pub const ACHIEVEMENTS: &[AchievementDef] = &[
    AchievementDef {
        id: "first_friend",
        name: "İlk dost",
        description: "İlk kedini sahiplen.",
        reward: "Tasma",
    },
    AchievementDef {
        id: "feed_10",
        name: "Mama ustası",
        description: "Kedilerini 10 kez besle.",
        reward: "Fiyonk",
    },
    AchievementDef {
        id: "pet_30",
        name: "Sevgi dolu",
        description: "Kedilerini 30 kez sev.",
        reward: "Çiçek",
    },
    AchievementDef {
        id: "play_10",
        name: "Oyun arkadaşı",
        description: "Kedilerinle 10 kez oyna.",
        reward: "Parti şapkası ve oyuncak fare",
    },
    AchievementDef {
        id: "streak_3",
        name: "Sadık dost",
        description: "Üç gün üst üste kedini besle.",
        reward: "Atkı",
    },
    AchievementDef {
        id: "happy_cat",
        name: "Mutlu kedi",
        description: "Bir kedinin mutluluğunu %95'e çıkar.",
        reward: "Tüy çubuğu",
    },
    AchievementDef {
        id: "collector",
        name: "Koleksiyoncu",
        description: "Dört farklı cins kedin olsun.",
        reward: "Taç",
    },
];

pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn day_number(date: &str) -> i64 {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| d.num_days_from_ce() as i64)
        .unwrap_or(0)
}

use chrono::Datelike;

/// Koşulları sağlanan başarımları açar; yeni açılanların kimliklerini döndürür.
pub fn check_achievements(db: &Db, store: &PetStore) -> AppResult<Vec<&'static str>> {
    let pets = store.snapshot();
    let mut breeds: Vec<&str> = pets.iter().map(|p| p.breed.as_str()).collect();
    breeds.sort();
    breeds.dedup();
    db.with(|c| {
        let conds = [
            ("first_friend", !pets.is_empty()),
            ("feed_10", db::pets::stat(c, "feeds")? >= 10),
            ("pet_30", db::pets::stat(c, "strokes")? >= 30),
            ("play_10", db::pets::stat(c, "plays")? >= 10),
            ("streak_3", db::pets::stat(c, "feed_streak")? >= 3),
            ("happy_cat", pets.iter().any(|p| p.needs.happiness >= 95.0)),
            ("collector", breeds.len() >= 4),
        ];
        let mut new = Vec::new();
        for (id, ok) in conds {
            if ok && db::pets::unlock(c, id)? {
                new.push(id);
            }
        }
        Ok(new)
    })
}

pub fn is_unlocked(db: &Db, achievement: Option<&str>) -> bool {
    match achievement {
        None => true,
        Some(id) => db
            .with(db::pets::achievements)
            .map(|a| a.contains_key(id))
            .unwrap_or(false),
    }
}

pub fn adopt(
    db: &Db,
    store: &PetStore,
    name: &str,
    breed: &str,
    personality: &str,
) -> AppResult<PetRecord> {
    let name = validate::text("Kedi adı", name, 24, true)?;
    if !model::BREEDS.contains(&breed) {
        return Err(AppError::invalid("Bilinmeyen kedi cinsi."));
    }
    let p =
        Personality::parse(personality).ok_or_else(|| AppError::invalid("Bilinmeyen kişilik."))?;
    let rec = db.with(|c| db::pets::insert(c, &name, breed, p, now_ms()))?;
    store.insert(rec.clone());
    check_achievements(db, store)?;
    Ok(rec)
}

pub fn feed(db: &Db, store: &PetStore, id: i64, food: &str) -> AppResult<PetRecord> {
    let f = FOODS
        .iter()
        .find(|f| f.id == food)
        .ok_or_else(|| AppError::invalid("Bilinmeyen yiyecek."))?;
    store.get(id)?;
    let day = today();
    db.with(|c| {
        db::pets::consume(c, food, &day)?;
        db::pets::inc_stat(c, "feeds")?;
        // Art arda besleme günleri
        let today_n = day_number(&day);
        let last = db::pets::stat(c, "feed_day")?;
        if last != today_n {
            let streak = if last == today_n - 1 {
                db::pets::stat(c, "feed_streak")? + 1
            } else {
                1
            };
            db::pets::set_stat(c, "feed_streak", streak)?;
            db::pets::set_stat(c, "feed_day", today_n)?;
        }
        Ok(())
    })?;
    store.apply(
        id,
        NeedsDelta {
            satiety: f.satiety,
            happiness: f.happiness,
            ..Default::default()
        },
    )?;
    let rec = store.persist(db, id)?;
    check_achievements(db, store)?;
    Ok(rec)
}

/// Sevmek: mutluluk artar. Uyuyan kedi uyanmaz ama yine de mutlu olur.
pub fn stroke(db: &Db, store: &PetStore, id: i64) -> AppResult<PetRecord> {
    let p = store.get(id)?;
    let gain = match p.personality {
        Personality::Affectionate => 9.0,
        Personality::Independent => 3.5,
        _ => 6.0,
    };
    store.apply(
        id,
        NeedsDelta {
            happiness: gain,
            ..Default::default()
        },
    )?;
    db.with(|c| db::pets::inc_stat(c, "strokes"))?;
    let rec = store.persist(db, id)?;
    check_achievements(db, store)?;
    Ok(rec)
}

pub fn play(db: &Db, store: &PetStore, id: i64, toy: &str) -> AppResult<PetRecord> {
    let t = TOYS
        .iter()
        .find(|t| t.id == toy)
        .ok_or_else(|| AppError::invalid("Bilinmeyen oyuncak."))?;
    if !is_unlocked(db, t.unlocked_by) {
        return Err(AppError::invalid("Bu oyuncağın kilidi henüz açılmadı."));
    }
    let p = store.get(id)?;
    if p.needs.energy < 15.0 {
        return Err(AppError::invalid(format!(
            "{} çok yorgun; önce biraz dinlenmeli.",
            p.name
        )));
    }
    let fun = if p.personality == Personality::Playful {
        t.fun * 1.4
    } else {
        t.fun
    };
    store.apply(
        id,
        NeedsDelta {
            happiness: fun,
            energy: -t.energy_cost,
            ..Default::default()
        },
    )?;
    db.with(|c| db::pets::inc_stat(c, "plays"))?;
    let rec = store.persist(db, id)?;
    check_achievements(db, store)?;
    Ok(rec)
}

pub fn set_accessory(
    db: &Db,
    store: &PetStore,
    id: i64,
    slot: &str,
    accessory: Option<&str>,
) -> AppResult<PetRecord> {
    if let Some(a) = accessory {
        let def = model::accessory(a).ok_or_else(|| AppError::invalid("Bilinmeyen aksesuar."))?;
        let slot_ok = matches!(
            (slot, def.slot),
            ("head", model::Slot::Head) | ("neck", model::Slot::Neck)
        );
        if !slot_ok {
            return Err(AppError::invalid("Bu aksesuar bu yuvaya takılamaz."));
        }
        if !is_unlocked(db, def.unlocked_by) {
            return Err(AppError::invalid("Bu aksesuarın kilidi henüz açılmadı."));
        }
    }
    let value = accessory.map(str::to_string);
    store.update(id, |r| match slot {
        "head" => r.acc_head = value,
        _ => r.acc_neck = value,
    })?;
    store.persist(db, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Db, PetStore, i64) {
        let db = Db::open_in_memory().unwrap();
        let store = PetStore::default();
        let p = adopt(&db, &store, "Tarçın", "orange", "playful").unwrap();
        (db, store, p.id)
    }

    #[test]
    fn adopt_validates_and_unlocks_first_friend() {
        let (db, store, _) = setup();
        assert!(
            db.with(db::pets::achievements)
                .unwrap()
                .contains_key("first_friend")
        );
        assert!(adopt(&db, &store, "", "orange", "playful").is_err());
        assert!(adopt(&db, &store, "X", "dragon", "playful").is_err());
        assert!(adopt(&db, &store, "X", "orange", "grumpy").is_err());
    }

    #[test]
    fn feeding_raises_satiety_consumes_food_and_tracks_streak() {
        let (db, store, id) = setup();
        store
            .apply(
                id,
                NeedsDelta {
                    satiety: -50.0,
                    ..Default::default()
                },
            )
            .unwrap();
        let before = store.get(id).unwrap().needs.satiety;
        let after = feed(&db, &store, id, "fish").unwrap().needs.satiety;
        assert!(after > before + 30.0);
        assert_eq!(db.with(|c| db::pets::stat(c, "feed_streak")).unwrap(), 1);
        feed(&db, &store, id, "fish").unwrap();
        feed(&db, &store, id, "fish").unwrap();
        assert!(
            feed(&db, &store, id, "fish").is_err(),
            "günlük balık bitmeli"
        );
    }

    #[test]
    fn tired_cat_refuses_to_play_and_locked_toys_are_rejected() {
        let (db, store, id) = setup();
        assert!(play(&db, &store, id, "feather").is_err(), "kilitli oyuncak");
        store
            .apply(
                id,
                NeedsDelta {
                    energy: -100.0,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(play(&db, &store, id, "yarn").is_err());
        store
            .apply(
                id,
                NeedsDelta {
                    energy: 100.0,
                    happiness: -60.0,
                    ..Default::default()
                },
            )
            .unwrap();
        let h = store.get(id).unwrap().needs.happiness;
        assert!(play(&db, &store, id, "yarn").unwrap().needs.happiness > h);
    }

    #[test]
    fn achievements_unlock_rewards() {
        let (db, store, id) = setup();
        assert!(
            set_accessory(&db, &store, id, "head", Some("bow")).is_err(),
            "fiyonk kilitli"
        );
        for _ in 0..30 {
            stroke(&db, &store, id).unwrap();
        }
        let ach = db.with(db::pets::achievements).unwrap();
        assert!(ach.contains_key("pet_30") && ach.contains_key("happy_cat"));
        assert_eq!(
            set_accessory(&db, &store, id, "head", Some("flower"))
                .unwrap()
                .acc_head
                .as_deref(),
            Some("flower")
        );
        assert!(
            set_accessory(&db, &store, id, "neck", Some("flower")).is_err(),
            "yanlış yuva"
        );
        assert!(set_accessory(&db, &store, id, "neck", Some("collar")).is_ok());
        assert!(
            set_accessory(&db, &store, id, "head", None)
                .unwrap()
                .acc_head
                .is_none()
        );
    }
}
