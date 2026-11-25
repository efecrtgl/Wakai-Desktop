//! Veritabanı yedekleme ve geri yükleme.
//!
//! - Yedek: `VACUUM INTO` ile tutarlı bir kopya `backups/` klasörüne yazılır.
//! - Geri yükleme çalışan bir veritabanının üzerine YAZILMAZ: bir işaret dosyası bırakılır,
//!   uygulama yeniden başlatılır ve veritabanı açılmadan ÖNCE yedek yerine konur.
//!   Geri yüklemeden önce mevcut veritabanı da ayrıca yedeklenir.
//! - Atomik: yedek önce `.tmp` dosyasına yazılır, bütünlüğü denetlenir, sonra yeniden adlandırılır.
//!   Geri yükleme de geçici dosyaya kopyalanıp doğrulandıktan sonra tek adımda yerine konur.
//! - Doğrulama: `PRAGMA quick_check`, Wakai tablosu varlığı ve şema sürümü (uyumluluk) denetlenir.
//! - Seçmeli geri yükleme: yalnızca seçilen kategoriler (ör. görevler, kediler) bir işlem içinde
//!   yedekteki satırlarla değiştirilir; diğer veriler olduğu gibi kalır. Açılışta, veriler
//!   kullanılmadan önce uygulanır.

use crate::db::Db;
use crate::error::{AppError, AppResult};
use serde::Serialize;
use std::path::{Path, PathBuf};

const RESTORE_MARKER: &str = "restore.pending";
const SELECTIVE_MARKER: &str = "restore-selective.pending";

/// Seçmeli geri yükleme kategorileri: kimlik → tablolar (üst tablo önce).
pub const CATEGORIES: &[(&str, &[&str])] = &[
    ("settings", &["settings"]),
    ("widgets", &["widget_instances"]),
    ("calendar", &["event_categories", "events"]),
    ("tasks", &["tasks"]),
    ("notes", &["notes"]),
    ("shopping", &["shopping_lists", "shopping_items"]),
    ("focus", &["focus_sessions"]),
    (
        "pets",
        &[
            "pets",
            "pet_inventory",
            "pet_stats",
            "pet_achievements",
            "pet_home_items",
        ],
    ),
    ("profiles", &["desktop_profiles"]),
];

fn category_tables(id: &str) -> Option<&'static [&'static str]> {
    CATEGORIES.iter().find(|(c, _)| *c == id).map(|(_, t)| *t)
}
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
    let tmp = d.join(format!("{name}.tmp"));
    let _ = std::fs::remove_file(&tmp);
    db.backup_to(&tmp)?;
    if let Err(e) = verify(&tmp) {
        let _ = std::fs::remove_file(&tmp);
        return Err(AppError::Other(format!("Yedek doğrulanamadı: {e}")));
    }
    std::fs::rename(&tmp, &path)?;
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
    // Yedeğin bozulmamış bir Wakai veritabanı olduğunu doğrula.
    verify(&dir(data_dir).join(name))?;
    let _ = std::fs::remove_file(data_dir.join(SELECTIVE_MARKER));
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
    if let Err(e) = verify(&source) {
        log::error!("geri yüklenecek yedek bozuk, atlandı: {e}");
        return Ok(false);
    }
    // Önce geçici dosyaya kopyala ve doğrula; ancak sonra tek adımda yerine koy.
    let tmp = db_path.with_extension("db.restore-tmp");
    std::fs::copy(&source, &tmp)?;
    if let Err(e) = verify(&tmp) {
        let _ = std::fs::remove_file(&tmp);
        log::error!("geri yükleme kopyası doğrulanamadı: {e}");
        return Ok(false);
    }
    for ext in ["db-wal", "db-shm"] {
        let _ = std::fs::remove_file(db_path.with_extension(ext));
    }
    std::fs::rename(&tmp, db_path)?;
    log::info!("yedek geri yüklendi: {name}");
    Ok(true)
}

/// Yedeğin bozulmamış bir Wakai Desktop veritabanı olduğunu doğrular; şema sürümünü döndürür.
pub fn verify(path: &Path) -> AppResult<usize> {
    let invalid =
        || AppError::invalid("Bu dosya geçerli bir Ivily Desktop yedeği değil ya da bozuk.");
    let conn =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|_| invalid())?;
    let check: String = conn
        .query_row("PRAGMA quick_check", [], |r| r.get(0))
        .map_err(|_| invalid())?;
    if check != "ok" {
        return Err(invalid());
    }
    if !table_exists(&conn, "main", "widget_instances")? {
        return Err(invalid());
    }
    crate::db::migrations::current_version(&conn)
}

fn table_exists(conn: &rusqlite::Connection, schema: &str, table: &str) -> AppResult<bool> {
    let n: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM {schema}.sqlite_master WHERE type = 'table' AND name = ?1"),
        [table],
        |r| r.get(0),
    )?;
    Ok(n > 0)
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryInfo {
    pub id: String,
    /// Kategorinin ana tablosundaki kayıt sayısı (tablo yedekte yoksa `None`).
    pub count: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupDetails {
    pub name: String,
    pub created: String,
    pub size: u64,
    pub valid: bool,
    pub error: Option<String>,
    pub schema_version: usize,
    pub supported_version: usize,
    /// Yedek bu sürümden daha yeni bir Wakai Desktop ile alınmış.
    pub newer: bool,
    pub categories: Vec<CategoryInfo>,
}

/// Bir yedeği inceler: bütünlük, şema uyumluluğu ve kategori başına kayıt sayıları.
pub fn inspect(data_dir: &Path, name: &str) -> AppResult<BackupDetails> {
    if !valid_name(name) {
        return Err(AppError::invalid("Geçersiz yedek adı."));
    }
    let path = dir(data_dir).join(name);
    let size = std::fs::metadata(&path)
        .map_err(|_| AppError::invalid("Yedek bulunamadı."))?
        .len();
    let supported = crate::db::migrations::MIGRATIONS.len();
    let mut out = BackupDetails {
        name: name.to_string(),
        created: created_label(name),
        size,
        valid: false,
        error: None,
        schema_version: 0,
        supported_version: supported,
        newer: false,
        categories: Vec::new(),
    };
    match verify(&path) {
        Ok(v) => {
            out.valid = true;
            out.schema_version = v;
            out.newer = v > supported;
        }
        Err(e) => {
            out.error = Some(e.to_string());
            return Ok(out);
        }
    }
    let conn =
        rusqlite::Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    for (id, tables) in CATEGORIES {
        let main = tables[0];
        let count = if table_exists(&conn, "main", main)? {
            Some(
                conn.query_row(&format!("SELECT COUNT(*) FROM \"{main}\""), [], |r| {
                    r.get(0)
                })?,
            )
        } else {
            None
        };
        out.categories.push(CategoryInfo {
            id: id.to_string(),
            count,
        });
    }
    Ok(out)
}

/// Seçmeli geri yüklemeyi bir sonraki açılış için işaretler.
pub fn schedule_selective_restore(
    data_dir: &Path,
    name: &str,
    categories: &[String],
) -> AppResult<()> {
    if categories.is_empty() {
        return Err(AppError::invalid("En az bir kategori seçin."));
    }
    if let Some(bad) = categories.iter().find(|c| category_tables(c).is_none()) {
        return Err(AppError::invalid(format!("Bilinmeyen kategori: {bad}")));
    }
    if !valid_name(name) || !dir(data_dir).join(name).exists() {
        return Err(AppError::invalid("Yedek bulunamadı."));
    }
    verify(&dir(data_dir).join(name))?;
    let _ = std::fs::remove_file(data_dir.join(RESTORE_MARKER));
    let body = serde_json::json!({ "name": name, "categories": categories });
    std::fs::write(data_dir.join(SELECTIVE_MARKER), body.to_string())?;
    Ok(())
}

/// Açılışta, veritabanı açıldıktan sonra ve veriler kullanılmadan önce çağrılır.
/// Bekleyen seçmeli geri yüklemeyi uygular; uygulanan kategorileri döndürür.
pub fn apply_pending_selective(db: &Db, data_dir: &Path) -> AppResult<Vec<String>> {
    let marker = data_dir.join(SELECTIVE_MARKER);
    let Ok(text) = std::fs::read_to_string(&marker) else {
        return Ok(Vec::new());
    };
    let _ = std::fs::remove_file(&marker);
    let v: serde_json::Value = serde_json::from_str(&text)?;
    let name = v["name"].as_str().unwrap_or_default().to_string();
    let categories: Vec<String> = v["categories"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|c| c.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let source = dir(data_dir).join(&name);
    if !valid_name(&name) || !source.exists() || verify(&source).is_err() {
        log::warn!("seçmeli geri yükleme atlandı: yedek yok ya da bozuk ({name})");
        return Ok(Vec::new());
    }
    create(db, data_dir, "-secmeli-oncesi")?;
    db.with(|c| restore_categories(c, &source, &categories))?;
    log::info!("seçmeli geri yükleme uygulandı: {name} {categories:?}");
    Ok(categories)
}

/// Seçilen kategorilerin tablolarını yedekteki satırlarla değiştirir (tek işlem; hata olursa
/// hiçbir şey değişmez). Yalnızca iki tarafta da bulunan sütunlar kopyalanır, böylece daha eski
/// ya da daha yeni şemalı yedekler de kullanılabilir.
pub fn restore_categories(
    conn: &rusqlite::Connection,
    source: &Path,
    categories: &[String],
) -> AppResult<()> {
    let tables: Vec<&str> = categories
        .iter()
        .filter_map(|c| category_tables(c))
        .flatten()
        .copied()
        .collect();
    conn.execute("ATTACH DATABASE ?1 AS bk", [source.to_string_lossy()])?;
    let result = (|| -> AppResult<()> {
        let tx = conn.unchecked_transaction()?;
        let present: Vec<&str> = tables
            .iter()
            .copied()
            .filter(|t| {
                table_exists(&tx, "bk", t).unwrap_or(false)
                    && table_exists(&tx, "main", t).unwrap_or(false)
            })
            .collect();
        for t in present.iter().rev() {
            tx.execute(&format!("DELETE FROM main.\"{t}\""), [])?;
        }
        for t in &present {
            let cols = common_columns(&tx, t)?;
            if cols.is_empty() {
                continue;
            }
            let list = cols
                .iter()
                .map(|c| format!("\"{c}\""))
                .collect::<Vec<_>>()
                .join(", ");
            tx.execute(
                &format!("INSERT INTO main.\"{t}\" ({list}) SELECT {list} FROM bk.\"{t}\""),
                [],
            )?;
        }
        tx.commit()?;
        Ok(())
    })();
    let _ = conn.execute("DETACH DATABASE bk", []);
    result
}

fn columns(conn: &rusqlite::Connection, schema: &str, table: &str) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare(&format!("PRAGMA {schema}.table_info(\"{table}\")"))?;
    let cols = stmt
        .query_map([], |r| r.get::<_, String>(1))?
        .collect::<Result<_, _>>()?;
    Ok(cols)
}

fn common_columns(conn: &rusqlite::Connection, table: &str) -> AppResult<Vec<String>> {
    let backup = columns(conn, "bk", table)?;
    Ok(columns(conn, "main", table)?
        .into_iter()
        .filter(|c| backup.contains(c))
        .collect())
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
        // Geçici dosya kalmadı.
        assert!(!db_path.with_extension("db.restore-tmp").exists());
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

    #[test]
    fn corrupt_and_foreign_files_are_rejected() {
        let data = temp_dir("corrupt");
        let d = dir(&data);
        std::fs::create_dir_all(&d).unwrap();
        // Rastgele baytlar
        std::fs::write(
            d.join("wakai-20260101-000000.db"),
            b"bu bir veritabani degil",
        )
        .unwrap();
        assert!(schedule_restore(&data, "wakai-20260101-000000.db").is_err());
        let info = inspect(&data, "wakai-20260101-000000.db").unwrap();
        assert!(!info.valid && info.error.is_some());
        // Wakai tablosu olmayan geçerli SQLite
        let other = rusqlite::Connection::open(d.join("wakai-20260101-000001.db")).unwrap();
        other.execute_batch("CREATE TABLE x (a);").unwrap();
        drop(other);
        assert!(schedule_restore(&data, "wakai-20260101-000001.db").is_err());
        // Bozuk dosya varken bekleyen geri yükleme mevcut veritabanını bozmaz.
        let db_path = data.join("wakai.db");
        drop(Db::open(&db_path).unwrap());
        std::fs::write(data.join(RESTORE_MARKER), "wakai-20260101-000000.db").unwrap();
        assert!(!apply_pending_restore(&data, &db_path).unwrap());
        assert!(Db::open(&db_path).is_ok());
    }

    #[test]
    fn inspect_reports_counts_and_compatibility() {
        let data = temp_dir("inspect");
        let db = Db::open(&data.join("wakai.db")).unwrap();
        db.with(|c| {
            c.execute("INSERT INTO tasks (title, created_at, updated_at) VALUES ('a', '', ''), ('b', '', '')", [])?;
            Ok(())
        })
        .unwrap();
        let name = create(&db, &data, "").unwrap().name;
        let info = inspect(&data, &name).unwrap();
        assert!(info.valid);
        assert!(!info.newer);
        assert_eq!(info.schema_version, crate::db::migrations::MIGRATIONS.len());
        let tasks = info.categories.iter().find(|c| c.id == "tasks").unwrap();
        assert_eq!(tasks.count, Some(2));
        assert!(
            !dir(&data).read_dir().unwrap().any(|e| e
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp"))
        );
    }

    #[test]
    fn selective_restore_replaces_only_chosen_categories() {
        let data = temp_dir("selective");
        let db = Db::open(&data.join("wakai.db")).unwrap();
        let add_task = |t: &str| {
            db.with(|c| {
                Ok(c.execute(
                    "INSERT INTO tasks (title, created_at, updated_at) VALUES (?1, '', '')",
                    [t],
                )?)
            })
            .unwrap()
        };
        let add_note = |t: &str| {
            db.with(|c| Ok(c.execute("INSERT INTO notes (title, body, created_at, updated_at) VALUES (?1, '', '', '')", [t])?)).unwrap()
        };
        add_task("eski görev");
        add_note("eski not");
        let name = create(&db, &data, "").unwrap().name;
        add_task("yeni görev");
        add_note("yeni not");

        assert!(schedule_selective_restore(&data, &name, &[]).is_err());
        assert!(schedule_selective_restore(&data, &name, &["bilinmeyen".into()]).is_err());
        schedule_selective_restore(&data, &name, &["tasks".into()]).unwrap();
        assert_eq!(apply_pending_selective(&db, &data).unwrap(), ["tasks"]);

        let count = |sql: &str| {
            db.with(|c| Ok(c.query_row(sql, [], |r| r.get::<_, i64>(0))?))
                .unwrap()
        };
        assert_eq!(
            count("SELECT COUNT(*) FROM tasks"),
            1,
            "görevler yedekteki hâline döndü"
        );
        assert_eq!(
            count("SELECT COUNT(*) FROM notes"),
            2,
            "notlara dokunulmadı"
        );
        assert!(
            list(&data)
                .unwrap()
                .iter()
                .any(|b| b.name.contains("secmeli-oncesi"))
        );
        assert!(
            apply_pending_selective(&db, &data).unwrap().is_empty(),
            "işaret tüketildi"
        );
    }
}
