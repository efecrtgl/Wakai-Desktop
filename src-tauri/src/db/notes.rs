//! Notlar. Not Kedisi widget'ları yapılandırmalarında `note_id` tutar; widget silinse
//! de not kalır.

use crate::error::{AppError, AppResult};
use crate::validate;
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};

pub const MAX_BODY: usize = 100_000;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub color: String,
    pub pinned: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NoteInput {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default = "butter")]
    pub color: String,
    #[serde(default)]
    pub pinned: bool,
}

fn butter() -> String {
    "butter".into()
}

const COLUMNS: &str = "id, title, body, color, pinned, updated_at";

fn from_row(r: &Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: r.get(0)?,
        title: r.get(1)?,
        body: r.get(2)?,
        color: r.get(3)?,
        pinned: r.get(4)?,
        updated_at: r.get(5)?,
    })
}

pub fn list(conn: &Connection) -> AppResult<Vec<Note>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM notes ORDER BY pinned DESC, updated_at DESC, id DESC"
    ))?;
    let rows = stmt
        .query_map([], from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: i64) -> AppResult<Note> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM notes WHERE id = ?1"),
        [id],
        from_row,
    )
    .optional()?
    .ok_or(AppError::NotFound)
}

pub fn create(conn: &Connection, input: &NoteInput) -> AppResult<Note> {
    let title = validate::text("Not başlığı", &input.title, 120, false)?;
    let body = validate::body("Not", &input.body, MAX_BODY)?;
    let color = validate::color(&input.color)?;
    conn.execute(
        "INSERT INTO notes (title, body, color, pinned) VALUES (?1, ?2, ?3, ?4)",
        params![title, body, color, input.pinned],
    )?;
    get(conn, conn.last_insert_rowid())
}

/// Notu günceller. İçerik değişmediyse yazma yapılmaz (otomatik kayıt gereksiz disk yazmasın).
pub fn update(conn: &Connection, id: i64, input: &NoteInput) -> AppResult<Note> {
    let title = validate::text("Not başlığı", &input.title, 120, false)?;
    let body = validate::body("Not", &input.body, MAX_BODY)?;
    let color = validate::color(&input.color)?;
    let current = get(conn, id)?;
    if current.title == title
        && current.body == body
        && current.color == color
        && current.pinned == input.pinned
    {
        return Ok(current);
    }
    conn.execute(
        "UPDATE notes SET title = ?2, body = ?3, color = ?4, pinned = ?5, updated_at = ?6 WHERE id = ?1",
        params![id, title, body, color, input.pinned, super::now_local()],
    )?;
    get(conn, id)
}

pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    if conn.execute("DELETE FROM notes WHERE id = ?1", [id])? == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    fn input(title: &str, body: &str) -> NoteInput {
        NoteInput {
            title: title.into(),
            body: body.into(),
            color: "butter".into(),
            pinned: false,
        }
    }

    #[test]
    fn crud_and_pinned_order() {
        let db = Db::open_in_memory().unwrap();
        let a = db.with(|c| create(c, &input("A", "ilk"))).unwrap();
        let b = db.with(|c| create(c, &input("B", "ikinci"))).unwrap();
        let mut pin = input("A", "ilk");
        pin.pinned = true;
        db.with(|c| update(c, a.id, &pin)).unwrap();
        let ids: Vec<_> = db.with(list).unwrap().into_iter().map(|n| n.id).collect();
        assert_eq!(ids, [a.id, b.id]);
        db.with(|c| delete(c, b.id)).unwrap();
        assert_eq!(db.with(list).unwrap().len(), 1);
    }

    #[test]
    fn unchanged_update_does_not_touch_timestamp() {
        let db = Db::open_in_memory().unwrap();
        let a = db.with(|c| create(c, &input("A", "metin"))).unwrap();
        db.with(|c| Ok(c.execute("UPDATE notes SET updated_at = '2000-01-01T00:00:00'", [])?))
            .unwrap();
        let same = db.with(|c| update(c, a.id, &input("A", "metin"))).unwrap();
        assert_eq!(same.updated_at, "2000-01-01T00:00:00");
    }

    #[test]
    fn body_keeps_whitespace_and_is_limited() {
        let db = Db::open_in_memory().unwrap();
        let n = db
            .with(|c| create(c, &input("", "  girintili\n- madde  ")))
            .unwrap();
        assert_eq!(n.body, "  girintili\n- madde  ");
        let big = "a".repeat(MAX_BODY + 1);
        assert!(db.with(|c| create(c, &input("", &big))).is_err());
    }
}
