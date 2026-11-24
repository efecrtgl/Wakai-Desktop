//! Görevler.

use crate::error::{AppError, AppResult};
use crate::validate;
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub notes: String,
    /// 0 düşük, 1 normal, 2 yüksek.
    pub priority: i64,
    pub due_at: Option<String>,
    pub category: String,
    pub completed: bool,
    pub completed_at: Option<String>,
    pub reminder_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskInput {
    pub title: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default = "normal")]
    pub priority: i64,
    #[serde(default)]
    pub due_at: Option<String>,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub reminder_at: Option<String>,
}

fn normal() -> i64 {
    1
}

struct Valid {
    title: String,
    notes: String,
    priority: i64,
    due_at: Option<String>,
    category: String,
    reminder_at: Option<String>,
}

fn validate(i: &TaskInput) -> AppResult<Valid> {
    if !(0..=2).contains(&i.priority) {
        return Err(AppError::invalid("Geçersiz öncelik."));
    }
    Ok(Valid {
        title: validate::text("Görev başlığı", &i.title, 200, true)?,
        notes: validate::text("Görev notu", &i.notes, 2000, false)?,
        priority: i.priority,
        due_at: validate::opt_datetime("Son tarih", i.due_at.as_deref())?,
        category: validate::text("Kategori", &i.category, 40, false)?,
        reminder_at: validate::opt_datetime("Hatırlatıcı", i.reminder_at.as_deref())?,
    })
}

const COLUMNS: &str =
    "id, title, notes, priority, due_at, category, completed, completed_at, reminder_at";

fn from_row(r: &Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: r.get(0)?,
        title: r.get(1)?,
        notes: r.get(2)?,
        priority: r.get(3)?,
        due_at: r.get(4)?,
        category: r.get(5)?,
        completed: r.get(6)?,
        completed_at: r.get(7)?,
        reminder_at: r.get(8)?,
    })
}

/// Açık görevler önce (yüksek öncelik, yakın son tarih), sonra tamamlananlar (en yeni önce).
pub fn list(conn: &Connection, include_completed: bool) -> AppResult<Vec<Task>> {
    let filter = if include_completed {
        ""
    } else {
        "WHERE completed = 0"
    };
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM tasks {filter}
         ORDER BY completed,
                  CASE WHEN completed = 0 THEN -priority END,
                  CASE WHEN completed = 0 THEN coalesce(due_at, '9999') END,
                  CASE WHEN completed = 1 THEN completed_at END DESC,
                  id"
    ))?;
    let rows = stmt
        .query_map([], from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: i64) -> AppResult<Task> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM tasks WHERE id = ?1"),
        [id],
        from_row,
    )
    .optional()?
    .ok_or(AppError::NotFound)
}

pub fn create(conn: &Connection, input: &TaskInput) -> AppResult<Task> {
    let v = validate(input)?;
    conn.execute(
        "INSERT INTO tasks (title, notes, priority, due_at, category, reminder_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![v.title, v.notes, v.priority, v.due_at, v.category, v.reminder_at],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, input: &TaskInput) -> AppResult<Task> {
    let v = validate(input)?;
    let n = conn.execute(
        "UPDATE tasks SET title = ?2, notes = ?3, priority = ?4, due_at = ?5, category = ?6, reminder_at = ?7,
             reminded_at = CASE WHEN reminder_at IS ?7 THEN reminded_at ELSE NULL END,
             updated_at = ?8
         WHERE id = ?1",
        params![id, v.title, v.notes, v.priority, v.due_at, v.category, v.reminder_at, super::now_local()],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

pub fn set_completed(conn: &Connection, id: i64, completed: bool) -> AppResult<Task> {
    let now = super::now_local();
    let n = conn.execute(
        "UPDATE tasks SET completed = ?2,
             completed_at = CASE WHEN ?2 THEN ?3 ELSE NULL END,
             updated_at = ?3
         WHERE id = ?1",
        params![id, completed, now],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    if conn.execute("DELETE FROM tasks WHERE id = ?1", [id])? == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn clear_completed(conn: &Connection) -> AppResult<usize> {
    Ok(conn.execute("DELETE FROM tasks WHERE completed = 1", [])?)
}

/// Hatırlatıcısı bekleyen açık görevler: (id, başlık, hatırlatma zamanı).
pub fn pending_reminders(conn: &Connection) -> AppResult<Vec<(i64, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, reminder_at FROM tasks
         WHERE reminder_at IS NOT NULL AND reminded_at IS NULL AND completed = 0",
    )?;
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn mark_reminded(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute(
        "UPDATE tasks SET reminded_at = ?2 WHERE id = ?1",
        params![id, super::now_local()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    fn input(title: &str, priority: i64, due: Option<&str>) -> TaskInput {
        TaskInput {
            title: title.into(),
            notes: String::new(),
            priority,
            due_at: due.map(Into::into),
            category: String::new(),
            reminder_at: None,
        }
    }

    #[test]
    fn ordering_and_completion() {
        let db = Db::open_in_memory().unwrap();
        let low = db.with(|c| create(c, &input("Mama al", 0, None))).unwrap();
        db.with(|c| create(c, &input("Rapor", 2, Some("2026-11-30T17:00"))))
            .unwrap();
        db.with(|c| create(c, &input("Fatura", 2, Some("2026-11-26T12:00"))))
            .unwrap();
        let titles: Vec<_> = db
            .with(|c| list(c, false))
            .unwrap()
            .into_iter()
            .map(|t| t.title)
            .collect();
        assert_eq!(titles, ["Fatura", "Rapor", "Mama al"]);

        let done = db.with(|c| set_completed(c, low.id, true)).unwrap();
        assert!(done.completed && done.completed_at.is_some());
        assert_eq!(db.with(|c| list(c, false)).unwrap().len(), 2);
        assert_eq!(
            db.with(|c| list(c, true)).unwrap().last().unwrap().id,
            low.id
        );

        let undone = db.with(|c| set_completed(c, low.id, false)).unwrap();
        assert!(!undone.completed && undone.completed_at.is_none());
    }

    #[test]
    fn validation() {
        let db = Db::open_in_memory().unwrap();
        assert!(db.with(|c| create(c, &input(" ", 1, None))).is_err());
        assert!(db.with(|c| create(c, &input("x", 5, None))).is_err());
        assert!(
            db.with(|c| create(c, &input("x", 1, Some("yarın"))))
                .is_err()
        );
    }

    #[test]
    fn completed_tasks_do_not_remind() {
        let db = Db::open_in_memory().unwrap();
        let mut i = input("Ara", 1, None);
        i.reminder_at = Some("2026-11-25T09:00".into());
        let t = db.with(|c| create(c, &i)).unwrap();
        assert_eq!(db.with(pending_reminders).unwrap().len(), 1);
        db.with(|c| set_completed(c, t.id, true)).unwrap();
        assert!(db.with(pending_reminders).unwrap().is_empty());
    }
}
