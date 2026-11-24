//! Widget örneklerinin kalıcılığı. Yapılandırma (konum, boyut, ayarlar) burada;
//! widget içerikleri (etkinlik, görev, not) ayrı tablolardadır.

use crate::desktop::Rect;
use crate::error::{AppError, AppResult};
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WidgetInstance {
    pub id: String,
    pub kind: String,
    pub rect: Rect,
    pub visible: bool,
    pub click_through: bool,
    pub config: serde_json::Value,
}

const COLUMNS: &str = "id, kind, x, y, width, height, visible, click_through, config";

fn from_row(r: &Row) -> rusqlite::Result<WidgetInstance> {
    let config: String = r.get(8)?;
    Ok(WidgetInstance {
        id: r.get(0)?,
        kind: r.get(1)?,
        rect: Rect::new(r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?),
        visible: r.get(6)?,
        click_through: r.get(7)?,
        // Bozuk yapılandırma uygulamayı düşürmez; boş nesneye düşer.
        config: serde_json::from_str(&config).unwrap_or_else(|_| serde_json::json!({})),
    })
}

pub fn list(conn: &Connection) -> AppResult<Vec<WidgetInstance>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM widget_instances ORDER BY created_at, id"
    ))?;
    let rows = stmt
        .query_map([], from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<WidgetInstance> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM widget_instances WHERE id = ?1"),
        [id],
        from_row,
    )
    .optional()?
    .ok_or(AppError::NotFound)
}

pub fn insert(conn: &Connection, w: &WidgetInstance) -> AppResult<()> {
    conn.execute(
        "INSERT INTO widget_instances (id, kind, x, y, width, height, visible, click_through, config)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            w.id,
            w.kind,
            w.rect.x,
            w.rect.y,
            w.rect.width,
            w.rect.height,
            w.visible,
            w.click_through,
            w.config.to_string()
        ],
    )?;
    Ok(())
}

fn touch(conn: &Connection, sql: &str, p: impl rusqlite::Params) -> AppResult<()> {
    let n = conn.execute(sql, p)?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn update_rect(conn: &Connection, id: &str, r: Rect) -> AppResult<()> {
    touch(
        conn,
        "UPDATE widget_instances SET x = ?2, y = ?3, width = ?4, height = ?5, updated_at = ?6 WHERE id = ?1",
        params![id, r.x, r.y, r.width, r.height, super::now_local()],
    )
}

pub fn update_visible(conn: &Connection, id: &str, visible: bool) -> AppResult<()> {
    touch(
        conn,
        "UPDATE widget_instances SET visible = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, visible, super::now_local()],
    )
}

pub fn update_click_through(conn: &Connection, id: &str, value: bool) -> AppResult<()> {
    touch(
        conn,
        "UPDATE widget_instances SET click_through = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, value, super::now_local()],
    )
}

pub fn update_config(conn: &Connection, id: &str, config: &serde_json::Value) -> AppResult<()> {
    touch(
        conn,
        "UPDATE widget_instances SET config = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, config.to_string(), super::now_local()],
    )
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    touch(conn, "DELETE FROM widget_instances WHERE id = ?1", [id])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    fn sample(id: &str) -> WidgetInstance {
        WidgetInstance {
            id: id.into(),
            kind: "clock".into(),
            rect: Rect::new(-1500, 20, 300, 160),
            visible: true,
            click_through: false,
            config: serde_json::json!({ "seconds": true }),
        }
    }

    #[test]
    fn crud_roundtrip() {
        let db = Db::open_in_memory().unwrap();
        db.with(|c| insert(c, &sample("a"))).unwrap();
        assert_eq!(db.with(|c| get(c, "a")).unwrap(), sample("a"));

        db.with(|c| update_rect(c, "a", Rect::new(1, 2, 3, 4)))
            .unwrap();
        db.with(|c| update_visible(c, "a", false)).unwrap();
        db.with(|c| update_config(c, "a", &serde_json::json!({"x": 1})))
            .unwrap();
        let w = db.with(|c| get(c, "a")).unwrap();
        assert_eq!(w.rect, Rect::new(1, 2, 3, 4));
        assert!(!w.visible);
        assert_eq!(w.config, serde_json::json!({"x": 1}));

        db.with(|c| delete(c, "a")).unwrap();
        assert!(matches!(db.with(|c| get(c, "a")), Err(AppError::NotFound)));
    }

    #[test]
    fn updating_missing_instance_reports_not_found() {
        let db = Db::open_in_memory().unwrap();
        assert!(matches!(
            db.with(|c| update_visible(c, "yok", true)),
            Err(AppError::NotFound)
        ));
    }

    #[test]
    fn corrupt_config_falls_back_to_empty_object() {
        let db = Db::open_in_memory().unwrap();
        db.with(|c| insert(c, &sample("a"))).unwrap();
        db.with(|c| {
            Ok(c.execute(
                "UPDATE widget_instances SET config = '{bozuk' WHERE id = 'a'",
                [],
            )?)
        })
        .unwrap();
        assert_eq!(
            db.with(|c| get(c, "a")).unwrap().config,
            serde_json::json!({})
        );
    }

    #[test]
    fn invalid_size_is_rejected_by_schema() {
        let db = Db::open_in_memory().unwrap();
        let mut w = sample("a");
        w.rect.width = 0;
        assert!(db.with(|c| insert(c, &w)).is_err());
    }
}
