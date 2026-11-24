//! Veritabanı yedekleme ve geri yükleme.
//!
//! - Yedek: `VACUUM INTO` ile tutarlı bir kopya `backups/` klasörüne yazılır.
//! - Geri yükleme çalışan bir veritabanının üzerine YAZILMAZ: bir işaret dosyası bırakılır,
//!   uygulama yeniden başlatılır ve veritabanı açılmadan ÖNCE yedek yerine konur.
//!   Geri yüklemeden önce mevcut veritabanı da ayrıca yedeklenir.

use crate::db::Db;
use crate::error::{AppError, AppResult};
use serde::Serialize;
use std::path::{Path, PathBuf};

const RESTORE_MARKER: &str = "restore.pending";
const MAX_BACKUPS: usize = 20;

#[derive(Debug, Clone, Serialize)]
pub struct BackupInfo {
    pub name: String,
    pub size: u64,
    /// Dosya adından okunan zaman (YYYY-MM-DD HH:MM)
    pub created: String,
}

pub fn dir(data_dir: &Path) -> PathBuf {
    data_dir.join("backups")
}

fn valid_name(name: &str) -> bool {
    name.starts_with("wakai-")
        && name.ends_with(".db")
        && name.len() < 64
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
}

fn created_label(name: &str) -> String {
    // wakai-20261124-2215[...].db
    let digits: String = name.chars().filter(char::is_ascii_digit).collect();
    if digits.len() >= 12 {
        format!(
            "{}-{}-{} {}:{}",
            &digits[0..4],
            &digits[4..6],
            &digits[6..8],
            &digits[8..10],
            &digits[10..12]
        )
    } else {
        name.to_string()
    }
}

pub fn create(db: &Db, data_dir: &Path, suffix: &str) -> AppResult<BackupInfo> {
    let d = dir(data_dir);
    std::fs::create_dir_all(&d)?;
    let name = format!(
        "wakai-{}{suffix}.db",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );
    let path = d.join(&name);
    db.backup_to(&path)?;
    prune(&d)?;
    let size = std::fs::metadata(&path)?.len();
    Ok(BackupInfo {
        created: created_label(&name),
        name,
        size,
    })
}

pub fn list(data_dir: &Path) -> AppResult<Vec<BackupInfo>> {
    let d = dir(data_dir);
    if !d.exists() {
        return Ok(Vec::new());
    }
    let mut out: Vec<BackupInfo> = std::fs::read_dir(&d)?
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            valid_name(&name).then(|| BackupInfo {
                created: created_label(&name),
                size: e.metadata().map(|m| m.len()).unwrap_or(0),
                name,
            })
        })
        .collect();
    out.sort_by(|a, b| b.name.cmp(&a.name));
    Ok(out)
}

/// En eski yedekleri siler; en fazla `MAX_BACKUPS` tutulur.
fn prune(d: &Path) -> AppResult<()> {
    let mut names: Vec<String> = std::fs::read_dir(d)?
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| valid_name(n))
        .collect();
    names.sort();
    while names.len() > MAX_BACKUPS {
        let oldest = names.remove(0);
        let _ = std::fs::remove_file(d.join(oldest));
    }
    Ok(())
}

pub fn delete(data_dir: &Path, name: &str) -> AppResult<()> {
    if !valid_name(name) {
        return Err(AppError::invalid("Geçersiz yedek adı."));
    }
    std::fs::remove_file(dir(data_dir).join(name))?;
    Ok(())
}

/// Bir sonraki açılışta geri yüklenecek yedeği işaretler.
pub fn schedule_restore(data_dir: &Path, name: &str) -> AppResult<()> {
    if !valid_name(name) || !dir(data_dir).join(name).exists() {
        return Err(AppError::invalid("Yedek bulunamadı."));
    }
    // Yedeğin gerçekten bir Wakai veritabanı olduğunu doğrula.
    let conn = rusqlite::Connection::open_with_flags(
        dir(data_dir).join(name),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?;
    conn.query_row("SELECT COUNT(*) FROM widget_instances", [], |r| {
        r.get::<_, i64>(0)
    })
    .map_err(|_| AppError::invalid("Bu dosya geçerli bir Wakai Desktop yedeği değil."))?;
    std::fs::write(data_dir.join(RESTORE_MARKER), name)?;
    Ok(())
}

/// Açılışta, veritabanı açılmadan önce çağrılır. Bekleyen geri yükleme varsa uygular.
pub fn apply_pending_restore(data_dir: &Path, db_path: &Path) -> AppResult<bool> {
    let marker = data_dir.join(RESTORE_MARKER);
    let Ok(name) = std::fs::read_to_string(&marker) else {
        return Ok(false);
    };
    let _ = std::fs::remove_file(&marker);
    let name = name.trim();
    let source = dir(data_dir).join(name);
    if !valid_name(name) || !source.exists() {
        log::warn!("geri yüklenecek yedek bulunamadı: {name}");
        return Ok(false);
    }
    if db_path.exists() {
        // Geri dönüş yolu: mevcut veritabanının yedeği.
        let current = Db::open(db_path)?;
        create(&current, data_dir, "-geri-yukleme-oncesi")?;
        drop(current);
    }
    for ext in ["db-wal", "db-shm"] {
        let _ = std::fs::remove_file(db_path.with_extension(ext));
    }
    std::fs::copy(&source, db_path)?;
    log::info!("yedek geri yüklendi: {name}");
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("wakai-backup-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn backup_and_restore_roundtrip() {
        let data = temp_dir("roundtrip");
        let db_path = data.join("wakai.db");
        let backup_name;
        {
            let db = Db::open(&db_path).unwrap();
            db.with(|c| crate::db::settings::set(c, "k", &serde_json::json!("eski")))
                .unwrap();
            backup_name = create(&db, &data, "").unwrap().name;
            db.with(|c| crate::db::settings::set(c, "k", &serde_json::json!("yeni")))
                .unwrap();
        }
        assert_eq!(list(&data).unwrap().len(), 1);
        schedule_restore(&data, &backup_name).unwrap();
        assert!(apply_pending_restore(&data, &db_path).unwrap());
        let db = Db::open(&db_path).unwrap();
        assert_eq!(
            db.with(|c| crate::db::settings::get(c, "k")).unwrap(),
            Some(serde_json::json!("eski"))
        );
        // Geri yükleme öncesi durum da yedeklendi.
        assert!(
            list(&data)
                .unwrap()
                .iter()
                .any(|b| b.name.contains("geri-yukleme-oncesi"))
        );
        // İşaret tüketildi.
        assert!(!apply_pending_restore(&data, &db_path).unwrap());
    }

    #[test]
    fn rejects_path_traversal_and_foreign_files() {
        let data = temp_dir("names");
        assert!(delete(&data, "../wakai.db").is_err());
        assert!(schedule_restore(&data, "wakai-x.db").is_err());
        std::fs::create_dir_all(dir(&data)).unwrap();
        std::fs::write(dir(&data).join("wakai-20260101-000000.db"), b"not sqlite").unwrap();
        assert!(schedule_restore(&data, "wakai-20260101-000000.db").is_err());
    }

    #[test]
    fn created_label_parses_name() {
        assert_eq!(
            created_label("wakai-20261124-221530.db"),
            "2026-11-24 22:15"
        );
    }
}
