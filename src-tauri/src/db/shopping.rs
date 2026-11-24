//! Alışveriş listeleri ve maddeleri.

use crate::error::{AppError, AppResult};
use crate::validate;
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ShoppingList {
    pub id: i64,
    pub name: String,
    pub open_items: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ShoppingItem {
    pub id: i64,
    pub list_id: i64,
    pub name: String,
    pub quantity: String,
    pub checked: bool,
}

fn name(v: &str) -> AppResult<String> {
    validate::text("Madde adı", v, 120, true)
}

fn quantity(v: &str) -> AppResult<String> {
    validate::text("Miktar", v, 30, false)
}

pub fn lists(conn: &Connection) -> AppResult<Vec<ShoppingList>> {
    let mut stmt = conn.prepare(
        "SELECT l.id, l.name, (SELECT COUNT(*) FROM shopping_items i WHERE i.list_id = l.id AND i.checked = 0)
         FROM shopping_lists l ORDER BY l.id",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(ShoppingList {
                id: r.get(0)?,
                name: r.get(1)?,
                open_items: r.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_list(conn: &Connection, id: i64) -> AppResult<ShoppingList> {
    lists(conn)?
        .into_iter()
        .find(|l| l.id == id)
        .ok_or(AppError::NotFound)
}

pub fn create_list(conn: &Connection, list_name: &str) -> AppResult<ShoppingList> {
    let n = validate::text("Liste adı", list_name, 60, true)?;
    conn.execute("INSERT INTO shopping_lists (name) VALUES (?1)", [n])?;
    get_list(conn, conn.last_insert_rowid())
}

pub fn rename_list(conn: &Connection, id: i64, list_name: &str) -> AppResult<ShoppingList> {
    let n = validate::text("Liste adı", list_name, 60, true)?;
    if conn.execute(
        "UPDATE shopping_lists SET name = ?2 WHERE id = ?1",
        params![id, n],
    )? == 0
    {
        return Err(AppError::NotFound);
    }
    get_list(conn, id)
}

pub fn delete_list(conn: &Connection, id: i64) -> AppResult<()> {
    if conn.execute("DELETE FROM shopping_lists WHERE id = ?1", [id])? == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// Alınmamışlar önce (eklenme sırasıyla), sonra sepettekiler (en son alınan önce).
pub fn items(conn: &Connection, list_id: i64) -> AppResult<Vec<ShoppingItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, list_id, name, quantity, checked FROM shopping_items WHERE list_id = ?1
         ORDER BY checked, CASE WHEN checked = 1 THEN checked_at END DESC, id",
    )?;
    let rows = stmt
        .query_map([list_id], |r| {
            Ok(ShoppingItem {
                id: r.get(0)?,
                list_id: r.get(1)?,
                name: r.get(2)?,
                quantity: r.get(3)?,
                checked: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn get_item(conn: &Connection, id: i64) -> AppResult<ShoppingItem> {
    conn.query_row(
        "SELECT id, list_id, name, quantity, checked FROM shopping_items WHERE id = ?1",
        [id],
        |r| {
            Ok(ShoppingItem {
                id: r.get(0)?,
                list_id: r.get(1)?,
                name: r.get(2)?,
                quantity: r.get(3)?,
                checked: r.get(4)?,
            })
        },
    )
    .optional()?
    .ok_or(AppError::NotFound)
}

pub fn add_item(
    conn: &Connection,
    list_id: i64,
    item_name: &str,
    qty: &str,
) -> AppResult<ShoppingItem> {
    get_list(conn, list_id)?;
    conn.execute(
        "INSERT INTO shopping_items (list_id, name, quantity) VALUES (?1, ?2, ?3)",
        params![list_id, name(item_name)?, quantity(qty)?],
    )?;
    get_item(conn, conn.last_insert_rowid())
}

pub fn update_item(
    conn: &Connection,
    id: i64,
    item_name: &str,
    qty: &str,
) -> AppResult<ShoppingItem> {
    if conn.execute(
        "UPDATE shopping_items SET name = ?2, quantity = ?3 WHERE id = ?1",
        params![id, name(item_name)?, quantity(qty)?],
    )? == 0
    {
        return Err(AppError::NotFound);
    }
    get_item(conn, id)
}

pub fn set_checked(conn: &Connection, id: i64, checked: bool) -> AppResult<ShoppingItem> {
    if conn.execute(
        "UPDATE shopping_items SET checked = ?2, checked_at = CASE WHEN ?2 THEN ?3 ELSE NULL END WHERE id = ?1",
        params![id, checked, super::now_local()],
    )? == 0
    {
        return Err(AppError::NotFound);
    }
    get_item(conn, id)
}

pub fn delete_item(conn: &Connection, id: i64) -> AppResult<i64> {
    let item = get_item(conn, id)?;
    conn.execute("DELETE FROM shopping_items WHERE id = ?1", [id])?;
    Ok(item.list_id)
}

pub fn clear_checked(conn: &Connection, list_id: i64) -> AppResult<usize> {
    Ok(conn.execute(
        "DELETE FROM shopping_items WHERE list_id = ?1 AND checked = 1",
        [list_id],
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    #[test]
    fn list_and_item_lifecycle() {
        let db = Db::open_in_memory().unwrap();
        let l = db.with(|c| create_list(c, "Market")).unwrap();
        let milk = db.with(|c| add_item(c, l.id, "Süt", "2 L")).unwrap();
        let bread = db.with(|c| add_item(c, l.id, "Ekmek", "")).unwrap();
        db.with(|c| add_item(c, l.id, "Mama", "1 paket")).unwrap();
        db.with(|c| set_checked(c, milk.id, true)).unwrap();
        let names: Vec<_> = db
            .with(|c| items(c, l.id))
            .unwrap()
            .into_iter()
            .map(|i| i.name)
            .collect();
        assert_eq!(names, ["Ekmek", "Mama", "Süt"]);
        assert_eq!(db.with(|c| get_list(c, l.id)).unwrap().open_items, 2);
        let u = db
            .with(|c| update_item(c, bread.id, "Tam buğday ekmeği", "1"))
            .unwrap();
        assert_eq!(u.quantity, "1");
        assert_eq!(db.with(|c| clear_checked(c, l.id)).unwrap(), 1);
        assert_eq!(db.with(|c| items(c, l.id)).unwrap().len(), 2);
    }

    #[test]
    fn validation_and_cascade() {
        let db = Db::open_in_memory().unwrap();
        let l = db.with(|c| create_list(c, "Ev")).unwrap();
        assert!(db.with(|c| add_item(c, l.id, "  ", "")).is_err());
        assert!(db.with(|c| add_item(c, 999, "Süt", "")).is_err());
        assert!(db.with(|c| create_list(c, "")).is_err());
        db.with(|c| add_item(c, l.id, "Süt", "")).unwrap();
        db.with(|c| delete_list(c, l.id)).unwrap();
        let left: i64 = db
            .with(|c| Ok(c.query_row("SELECT COUNT(*) FROM shopping_items", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(left, 0);
    }
}
