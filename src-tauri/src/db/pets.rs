//! Kedi kalıcılığı: kediler, envanter, istatistikler, başarımlar ve Kedi Evi eşyaları.

use crate::error::{AppError, AppResult};
use crate::pets::model::{FOODS, Needs, Personality};
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PetRecord {
    pub id: i64,
    pub name: String,
    pub breed: String,
    pub personality: Personality,
    /// "desktop" | "home"
    pub location: String,
    pub paused: bool,
    pub acc_head: Option<String>,
    pub acc_neck: Option<String>,
    pub needs: Needs,
    /// İhtiyaçların en son hesaplandığı an (Unix ms)
    pub needs_at: i64,
}

const COLUMNS: &str = "id, name, breed, personality, location, paused, acc_head, acc_neck, satiety, energy, happiness, rest, needs_at";

fn from_row(r: &Row) -> rusqlite::Result<PetRecord> {
    let personality: String = r.get(3)?;
    Ok(PetRecord {
        id: r.get(0)?,
        name: r.get(1)?,
        breed: r.get(2)?,
        personality: Personality::parse(&personality).unwrap_or(Personality::Calm),
        location: r.get(4)?,
        paused: r.get(5)?,
        acc_head: r.get(6)?,
        acc_neck: r.get(7)?,
        needs: Needs {
            satiety: r.get(8)?,
            energy: r.get(9)?,
            happiness: r.get(10)?,
            rest: r.get(11)?,
        },
        needs_at: r.get(12)?,
    })
}

pub fn list(conn: &Connection) -> AppResult<Vec<PetRecord>> {
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM pets ORDER BY id"))?;
    let rows = stmt
        .query_map([], from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: i64) -> AppResult<PetRecord> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM pets WHERE id = ?1"),
        [id],
        from_row,
    )
    .optional()?
    .ok_or(AppError::NotFound)
}

pub fn insert(
    conn: &Connection,
    name: &str,
    breed: &str,
    p: Personality,
    now_ms: i64,
) -> AppResult<PetRecord> {
    conn.execute(
        "INSERT INTO pets (name, breed, personality, needs_at) VALUES (?1, ?2, ?3, ?4)",
        params![name, breed, p.as_str(), now_ms],
    )?;
    get(conn, conn.last_insert_rowid())
}

/// Kimlik dışındaki tüm alanları yazar.
pub fn save(conn: &Connection, p: &PetRecord) -> AppResult<()> {
    conn.execute(
        "UPDATE pets SET name = ?2, breed = ?3, personality = ?4, location = ?5, paused = ?6, acc_head = ?7, acc_neck = ?8,
             satiety = ?9, energy = ?10, happiness = ?11, rest = ?12, needs_at = ?13 WHERE id = ?1",
        params![
            p.id,
            p.name,
            p.breed,
            p.personality.as_str(),
            p.location,
            p.paused,
            p.acc_head,
            p.acc_neck,
            p.needs.satiety,
            p.needs.energy,
            p.needs.happiness,
            p.needs.rest,
            p.needs_at
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    if conn.execute("DELETE FROM pets WHERE id = ?1", [id])? == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

// ── Envanter (yiyecekler her gün ücretsiz yenilenir) ───────────────────────

/// Bugünün envanteri; gün değiştiyse her yiyecek en az günlük miktarına tamamlanır.
pub fn inventory(conn: &Connection, today: &str) -> AppResult<BTreeMap<String, i64>> {
    for f in FOODS {
        conn.execute(
            "INSERT INTO pet_inventory (item, count, refilled_on) VALUES (?1, ?2, ?3)
             ON CONFLICT(item) DO UPDATE SET
                count = CASE WHEN refilled_on <> excluded.refilled_on THEN max(count, excluded.count) ELSE count END,
                refilled_on = excluded.refilled_on",
            params![f.id, f.daily, today],
        )?;
    }
    let mut stmt = conn.prepare("SELECT item, count FROM pet_inventory")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn consume(conn: &Connection, item: &str, today: &str) -> AppResult<i64> {
    let inv = inventory(conn, today)?;
    let n = inv
        .get(item)
        .copied()
        .ok_or_else(|| AppError::invalid("Bilinmeyen yiyecek."))?;
    if n <= 0 {
        return Err(AppError::invalid(
            "Bu yiyecek bugünlük bitti; yarın yenilenecek.",
        ));
    }
    conn.execute(
        "UPDATE pet_inventory SET count = count - 1 WHERE item = ?1",
        [item],
    )?;
    Ok(n - 1)
}

// ── İstatistikler ve başarımlar ──────────────────────────────────────────

pub fn stat(conn: &Connection, key: &str) -> AppResult<i64> {
    Ok(conn
        .query_row("SELECT value FROM pet_stats WHERE key = ?1", [key], |r| {
            r.get(0)
        })
        .optional()?
        .unwrap_or(0))
}

pub fn set_stat(conn: &Connection, key: &str, value: i64) -> AppResult<()> {
    conn.execute(
        "INSERT INTO pet_stats (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn inc_stat(conn: &Connection, key: &str) -> AppResult<i64> {
    let v = stat(conn, key)? + 1;
    set_stat(conn, key, v)?;
    Ok(v)
}

pub fn achievements(conn: &Connection) -> AppResult<BTreeMap<String, String>> {
    let mut stmt = conn.prepare("SELECT id, unlocked_at FROM pet_achievements")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Başarımı açar; yeni açıldıysa `true`.
pub fn unlock(conn: &Connection, id: &str) -> AppResult<bool> {
    Ok(conn.execute(
        "INSERT OR IGNORE INTO pet_achievements (id, unlocked_at) VALUES (?1, ?2)",
        params![id, super::now_local()],
    )? > 0)
}

// ── Kedi Evi eşyaları ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HomeItem {
    pub id: i64,
    pub kind: String,
    pub x: i64,
    pub y: i64,
    pub flipped: bool,
}

pub fn home_items(conn: &Connection) -> AppResult<Vec<HomeItem>> {
    let mut stmt =
        conn.prepare("SELECT id, kind, x, y, flipped FROM pet_home_items ORDER BY y, id")?;
    let rows = stmt
        .query_map([], |r| {
            Ok(HomeItem {
                id: r.get(0)?,
                kind: r.get(1)?,
                x: r.get(2)?,
                y: r.get(3)?,
                flipped: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn add_home_item(conn: &Connection, kind: &str, x: i64, y: i64) -> AppResult<HomeItem> {
    conn.execute(
        "INSERT INTO pet_home_items (kind, x, y) VALUES (?1, ?2, ?3)",
        params![kind, x, y],
    )?;
    let id = conn.last_insert_rowid();
    Ok(HomeItem {
        id,
        kind: kind.into(),
        x,
        y,
        flipped: false,
    })
}

pub fn move_home_item(conn: &Connection, id: i64, x: i64, y: i64, flipped: bool) -> AppResult<()> {
    if conn.execute(
        "UPDATE pet_home_items SET x = ?2, y = ?3, flipped = ?4 WHERE id = ?1",
        params![id, x, y, flipped],
    )? == 0
    {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn delete_home_item(conn: &Connection, id: i64) -> AppResult<()> {
    if conn.execute("DELETE FROM pet_home_items WHERE id = ?1", [id])? == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    #[test]
    fn pet_roundtrip_and_delete() {
        let db = Db::open_in_memory().unwrap();
        let mut p = db
            .with(|c| insert(c, "Pamuk", "white", Personality::Curious, 1000))
            .unwrap();
        assert_eq!(p.location, "desktop");
        p.needs.satiety = 42.0;
        p.acc_head = Some("bow".into());
        p.location = "home".into();
        db.with(|c| save(c, &p)).unwrap();
        assert_eq!(db.with(|c| get(c, p.id)).unwrap(), p);
        db.with(|c| delete(c, p.id)).unwrap();
        assert!(db.with(list).unwrap().is_empty());
    }

    #[test]
    fn inventory_refills_daily_and_consumes() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(db.with(|c| inventory(c, "2026-11-24")).unwrap()["fish"], 3);
        for _ in 0..3 {
            db.with(|c| consume(c, "fish", "2026-11-24")).unwrap();
        }
        assert!(db.with(|c| consume(c, "fish", "2026-11-24")).is_err());
        // Ertesi gün yenilenir.
        assert_eq!(db.with(|c| inventory(c, "2026-11-25")).unwrap()["fish"], 3);
        assert!(db.with(|c| consume(c, "pizza", "2026-11-25")).is_err());
    }

    #[test]
    fn stats_achievements_and_home_items() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(db.with(|c| inc_stat(c, "feeds")).unwrap(), 1);
        assert_eq!(db.with(|c| inc_stat(c, "feeds")).unwrap(), 2);
        assert!(db.with(|c| unlock(c, "feed_10")).unwrap());
        assert!(!db.with(|c| unlock(c, "feed_10")).unwrap());
        let it = db.with(|c| add_home_item(c, "bed", 10, 20)).unwrap();
        db.with(|c| move_home_item(c, it.id, 30, 40, true)).unwrap();
        let items = db.with(home_items).unwrap();
        assert_eq!((items[0].x, items[0].y, items[0].flipped), (30, 40, true));
        db.with(|c| delete_home_item(c, it.id)).unwrap();
        assert!(db.with(|c| delete_home_item(c, it.id)).is_err());
    }
}
