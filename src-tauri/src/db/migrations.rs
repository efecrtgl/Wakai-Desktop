//! Sıralı SQL göçleri. `PRAGMA user_version` veritabanının şema sürümünü tutar.
//! Yeni bir göç eklemek için `MIGRATIONS` dizisinin SONUNA yeni bir dosya ekleyin;
//! mevcut göçler asla değiştirilmez.

use crate::error::AppResult;
use rusqlite::Connection;

pub const MIGRATIONS: &[&str] = &[
    include_str!("migrations/001_initial.sql"),
    include_str!("migrations/002_content.sql"),
    include_str!("migrations/003_shopping.sql"),
    include_str!("migrations/004_pets.sql"),
    include_str!("migrations/005_profiles.sql"),
    include_str!("migrations/006_app_versions.sql"),
    include_str!("migrations/007_themes.sql"),
];

pub fn current_version(conn: &Connection) -> AppResult<usize> {
    let v: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    Ok(v.max(0) as usize)
}

pub fn pending(conn: &Connection) -> AppResult<usize> {
    Ok(MIGRATIONS.len().saturating_sub(current_version(conn)?))
}

/// Bekleyen göçleri her biri ayrı bir işlem (transaction) içinde uygular.
/// Bir göç başarısız olursa o göç geri alınır ve hata döndürülür.
pub fn run(conn: &mut Connection) -> AppResult<()> {
    let start = current_version(conn)?;
    for (idx, sql) in MIGRATIONS.iter().enumerate().skip(start) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", (idx + 1) as i64)?;
        tx.commit()?;
        log::info!("veritabanı göçü uygulandı: {}", idx + 1);
    }
    Ok(())
}
