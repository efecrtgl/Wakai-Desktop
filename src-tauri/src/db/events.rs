//! Takvim etkinlikleri ve kategorileri.

use crate::error::{AppError, AppResult};
use crate::validate;
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Event {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub start_at: String,
    pub end_at: Option<String>,
    pub all_day: bool,
    pub category_id: Option<i64>,
    pub color: String,
    pub reminder_minutes: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventInput {
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub start_at: String,
    #[serde(default)]
    pub end_at: Option<String>,
    #[serde(default)]
    pub all_day: bool,
    #[serde(default)]
    pub category_id: Option<i64>,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default)]
    pub reminder_minutes: Option<i64>,
}

fn default_color() -> String {
    "pink".into()
}

/// Doğrulanmış etkinlik girdisi.
struct Valid {
    title: String,
    description: String,
    start_at: String,
    end_at: Option<String>,
    all_day: bool,
    category_id: Option<i64>,
    color: String,
    reminder_minutes: Option<i64>,
}

fn validate(input: &EventInput) -> AppResult<Valid> {
    let title = validate::text("Etkinlik başlığı", &input.title, 200, true)?;
    let description = validate::text("Açıklama", &input.description, 5000, false)?;
    let start = validate::datetime("Başlangıç", &input.start_at)?;
    let start_at = if input.all_day {
        start.date().and_hms_opt(0, 0, 0).expect("geçerli saat")
    } else {
        start
    };
    let end_at = match validate::opt_datetime("Bitiş", input.end_at.as_deref())? {
        Some(e) if !input.all_day => {
            let end = validate::datetime("Bitiş", &e)?;
            if end < start_at {
                return Err(AppError::invalid("Bitiş zamanı başlangıçtan önce olamaz."));
            }
            Some(e)
        }
        _ => None,
    };
    if let Some(m) = input.reminder_minutes
        && !(0..=40_320).contains(&m)
    {
        return Err(AppError::invalid(
            "Hatırlatıcı süresi 0 ile 4 hafta arasında olmalıdır.",
        ));
    }
    Ok(Valid {
        title,
        description,
        start_at: start_at.format(validate::DATETIME_FORMAT).to_string(),
        end_at,
        all_day: input.all_day,
        category_id: input.category_id,
        color: validate::color(&input.color)?,
        reminder_minutes: input.reminder_minutes,
    })
}

const COLUMNS: &str =
    "id, title, description, start_at, end_at, all_day, category_id, color, reminder_minutes";

fn from_row(r: &Row) -> rusqlite::Result<Event> {
    Ok(Event {
        id: r.get(0)?,
        title: r.get(1)?,
        description: r.get(2)?,
        start_at: r.get(3)?,
        end_at: r.get(4)?,
        all_day: r.get(5)?,
        category_id: r.get(6)?,
        color: r.get(7)?,
        reminder_minutes: r.get(8)?,
    })
}

pub fn categories(conn: &Connection) -> AppResult<Vec<Category>> {
    let mut stmt = conn.prepare("SELECT id, name, color FROM event_categories ORDER BY id")?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Category {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn check_category(conn: &Connection, id: Option<i64>) -> AppResult<()> {
    if let Some(id) = id {
        let exists: Option<i64> = conn
            .query_row("SELECT id FROM event_categories WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .optional()?;
        if exists.is_none() {
            return Err(AppError::invalid("Seçilen kategori bulunamadı."));
        }
    }
    Ok(())
}

/// Başlangıcı [from, to) aralığında olan etkinlikler (tarih: YYYY-MM-DD).
pub fn list_range(conn: &Connection, from: &str, to: &str) -> AppResult<Vec<Event>> {
    validate::date("Başlangıç tarihi", from)?;
    validate::date("Bitiş tarihi", to)?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM events WHERE start_at >= ?1 AND start_at < ?2 ORDER BY start_at, id"
    ))?;
    let rows = stmt
        .query_map(params![from, to], from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Şu andan itibaren yaklaşan etkinlikler (bugün başlamış olanlar dahil).
pub fn upcoming(conn: &Connection, now: &str, limit: u32) -> AppResult<Vec<Event>> {
    let today = &now[..10];
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM events
         WHERE start_at >= ?1 OR (all_day = 1 AND substr(start_at, 1, 10) = ?2)
         ORDER BY start_at, id LIMIT ?3"
    ))?;
    let rows = stmt
        .query_map(params![now, today, limit.min(100)], from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: i64) -> AppResult<Event> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM events WHERE id = ?1"),
        [id],
        from_row,
    )
    .optional()?
    .ok_or(AppError::NotFound)
}

pub fn create(conn: &Connection, input: &EventInput) -> AppResult<Event> {
    let v = validate(input)?;
    check_category(conn, v.category_id)?;
    conn.execute(
        "INSERT INTO events (title, description, start_at, end_at, all_day, category_id, color, reminder_minutes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![v.title, v.description, v.start_at, v.end_at, v.all_day, v.category_id, v.color, v.reminder_minutes],
    )?;
    get(conn, conn.last_insert_rowid())
}

/// Günceller. Başlangıç ya da hatırlatıcı değişirse hatırlatıcı yeniden kurulur.
pub fn update(conn: &Connection, id: i64, input: &EventInput) -> AppResult<Event> {
    let v = validate(input)?;
    check_category(conn, v.category_id)?;
    let n = conn.execute(
        "UPDATE events SET title = ?2, description = ?3, start_at = ?4, end_at = ?5, all_day = ?6,
             category_id = ?7, color = ?8, reminder_minutes = ?9,
             reminded_at = CASE WHEN start_at = ?4 AND reminder_minutes IS ?9 THEN reminded_at ELSE NULL END,
             updated_at = ?10
         WHERE id = ?1",
        params![
            id,
            v.title,
            v.description,
            v.start_at,
            v.end_at,
            v.all_day,
            v.category_id,
            v.color,
            v.reminder_minutes,
            super::now_local()
        ],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    if conn.execute("DELETE FROM events WHERE id = ?1", [id])? == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// Hatırlatıcısı bekleyen etkinlikler: (id, başlık, başlangıç, dakika).
pub fn pending_reminders(conn: &Connection) -> AppResult<Vec<(i64, String, String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, start_at, reminder_minutes FROM events
         WHERE reminder_minutes IS NOT NULL AND reminded_at IS NULL",
    )?;
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn mark_reminded(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute(
        "UPDATE events SET reminded_at = ?2 WHERE id = ?1",
        params![id, super::now_local()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    fn input(title: &str, start: &str) -> EventInput {
        EventInput {
            title: title.into(),
            description: String::new(),
            start_at: start.into(),
            end_at: None,
            all_day: false,
            category_id: Some(1),
            color: "pink".into(),
            reminder_minutes: Some(15),
        }
    }

    #[test]
    fn create_list_update_delete() {
        let db = Db::open_in_memory().unwrap();
        let e = db
            .with(|c| create(c, &input("Veteriner", "2026-11-25T10:00")))
            .unwrap();
        db.with(|c| create(c, &input("Toplantı", "2026-12-01T09:00")))
            .unwrap();
        let nov = db
            .with(|c| list_range(c, "2026-11-01", "2026-12-01"))
            .unwrap();
        assert_eq!(nov.len(), 1);
        assert_eq!(nov[0].title, "Veteriner");

        let mut changed = input("Veteriner (kontrol)", "2026-11-25T11:00");
        changed.color = "mint".into();
        let u = db.with(|c| update(c, e.id, &changed)).unwrap();
        assert_eq!(u.title, "Veteriner (kontrol)");
        assert_eq!(u.color, "mint");

        db.with(|c| delete(c, e.id)).unwrap();
        assert!(matches!(db.with(|c| get(c, e.id)), Err(AppError::NotFound)));
    }

    #[test]
    fn rejects_bad_input() {
        let db = Db::open_in_memory().unwrap();
        assert!(
            db.with(|c| create(c, &input("", "2026-11-25T10:00")))
                .is_err()
        );
        assert!(db.with(|c| create(c, &input("x", "dün"))).is_err());
        let mut bad = input("x", "2026-11-25T10:00");
        bad.end_at = Some("2026-11-25T09:00".into());
        assert!(db.with(|c| create(c, &bad)).is_err());
        let mut bad_cat = input("x", "2026-11-25T10:00");
        bad_cat.category_id = Some(999);
        assert!(db.with(|c| create(c, &bad_cat)).is_err());
    }

    #[test]
    fn all_day_events_start_at_midnight_and_show_in_upcoming() {
        let db = Db::open_in_memory().unwrap();
        let mut i = input("Doğum günü", "2026-11-25T15:45");
        i.all_day = true;
        let e = db.with(|c| create(c, &i)).unwrap();
        assert_eq!(e.start_at, "2026-11-25T00:00");
        let up = db.with(|c| upcoming(c, "2026-11-25T12:00", 5)).unwrap();
        assert_eq!(up.len(), 1);
    }

    #[test]
    fn reminder_resets_when_start_changes() {
        let db = Db::open_in_memory().unwrap();
        let e = db
            .with(|c| create(c, &input("A", "2026-11-25T10:00")))
            .unwrap();
        db.with(|c| mark_reminded(c, e.id)).unwrap();
        assert!(db.with(pending_reminders).unwrap().is_empty());
        // Yalnızca başlık değişirse hatırlatıcı tekrar çalmaz.
        db.with(|c| update(c, e.id, &input("A2", "2026-11-25T10:00")))
            .unwrap();
        assert!(db.with(pending_reminders).unwrap().is_empty());
        // Başlangıç değişirse yeniden kurulur.
        db.with(|c| update(c, e.id, &input("A2", "2026-11-26T10:00")))
            .unwrap();
        assert_eq!(db.with(pending_reminders).unwrap().len(), 1);
    }

    #[test]
    fn deleting_category_keeps_events() {
        let db = Db::open_in_memory().unwrap();
        let e = db
            .with(|c| create(c, &input("A", "2026-11-25T10:00")))
            .unwrap();
        db.with(|c| Ok(c.execute("DELETE FROM event_categories WHERE id = 1", [])?))
            .unwrap();
        assert_eq!(db.with(|c| get(c, e.id)).unwrap().category_id, None);
    }
}
