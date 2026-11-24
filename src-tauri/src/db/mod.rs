//! SQLite erişim katmanı. Tek bir bağlantı bir `Mutex` ile korunur; sorgular kısa
//! ve parametrelidir, bu nedenle kilit çekişmesi pratikte oluşmaz.

pub mod events;
pub mod migrations;
pub mod notes;
pub mod settings;
pub mod tasks;
pub mod widgets;

use crate::error::AppResult;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

pub struct Db {
    conn: Mutex<Connection>,
    path: Option<PathBuf>,
}

impl Db {
    /// Veritabanını açar, ayarlar ve bekleyen göçleri uygular.
    /// Mevcut bir veritabanında göç bekliyorsa önce yedek alınır.
    pub fn open(path: &Path) -> AppResult<Self> {
        let existed = path.exists();
        let mut conn = Connection::open(path)?;
        configure(&conn)?;
        if existed && migrations::pending(&conn)? > 0 {
            let version = migrations::current_version(&conn)?;
            let backup = path.with_extension(format!("db.bak-v{version}"));
            backup_into(&conn, &backup)?;
        }
        migrations::run(&mut conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path: Some(path.to_path_buf()),
        })
    }

    /// Veritabanını açar; dosya bozuksa (SQLite okuyamıyorsa) onu `*.db.bozuk-<zaman>` adıyla
    /// karantinaya alır ve boş bir veritabanıyla başlar. Karantina dosyasının yolunu döndürür.
    /// Dosya izni gibi G/Ç hatalarında karantina YAPILMAZ (veri kaybı riskini önlemek için).
    pub fn open_or_recover(path: &Path) -> AppResult<(Self, Option<PathBuf>)> {
        match Self::open(path) {
            Ok(db) => Ok((db, None)),
            Err(crate::error::AppError::Db(e)) if path.exists() => {
                log::error!("veritabanı açılamadı, karantinaya alınıyor: {e}");
                let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
                let quarantine = path.with_extension(format!("db.bozuk-{stamp}"));
                std::fs::rename(path, &quarantine)?;
                for ext in ["db-wal", "db-shm"] {
                    let side = path.with_extension(ext);
                    if side.exists() {
                        let _ = std::fs::rename(
                            &side,
                            quarantine.with_extension(format!("{ext}.bozuk-{stamp}")),
                        );
                    }
                }
                Ok((Self::open(path)?, Some(quarantine)))
            }
            Err(e) => Err(e),
        }
    }

    /// Testler için bellek içi veritabanı.
    pub fn open_in_memory() -> AppResult<Self> {
        let mut conn = Connection::open_in_memory()?;
        configure(&conn)?;
        migrations::run(&mut conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path: None,
        })
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    fn lock(&self) -> MutexGuard<'_, Connection> {
        // Bir iş parçacığı panik yapsa bile bağlantı tutarlıdır (işlemler atomiktir).
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn with<T>(&self, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        f(&self.lock())
    }

    /// Kapalı bir işlem içinde çalıştırır; hata olursa tüm değişiklikler geri alınır.
    pub fn with_tx<T>(
        &self,
        f: impl FnOnce(&rusqlite::Transaction) -> AppResult<T>,
    ) -> AppResult<T> {
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        let out = f(&tx)?;
        tx.commit()?;
        Ok(out)
    }

    /// Veritabanının tutarlı bir kopyasını verilen yola yazar.
    pub fn backup_to(&self, target: &Path) -> AppResult<()> {
        backup_into(&self.lock(), target)
    }
}

fn configure(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 3000;",
    )?;
    Ok(())
}

fn backup_into(conn: &Connection, target: &Path) -> AppResult<()> {
    if target.exists() {
        std::fs::remove_file(target)?;
    }
    let target = target.to_string_lossy().to_string();
    conn.execute("VACUUM INTO ?1", [target])?;
    Ok(())
}

pub fn now_local() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("wakai-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("wakai.db")
    }

    #[test]
    fn migrations_bring_schema_to_latest() {
        let db = Db::open_in_memory().unwrap();
        let v = db.with(migrations::current_version).unwrap();
        assert_eq!(v, migrations::MIGRATIONS.len());
        let fk: i64 = db
            .with(|c| Ok(c.query_row("PRAGMA foreign_keys", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(fk, 1);
    }

    #[test]
    fn reopening_is_idempotent_and_keeps_data() {
        let path = temp_db_path("reopen");
        {
            let db = Db::open(&path).unwrap();
            db.with(|c| settings::set(c, "k", &serde_json::json!({"a": 1})))
                .unwrap();
        }
        let db = Db::open(&path).unwrap();
        let v = db.with(|c| settings::get(c, "k")).unwrap();
        assert_eq!(v, Some(serde_json::json!({"a": 1})));
    }

    #[test]
    fn older_database_is_backed_up_before_migrating() {
        let path = temp_db_path("backup");
        {
            let mut conn = Connection::open(&path).unwrap();
            configure(&conn).unwrap();
            // Şema sürümü 0 olan (hiç göç uygulanmamış) bir veritabanı bırak.
            conn.execute_batch("CREATE TABLE legacy (x INTEGER); INSERT INTO legacy VALUES (7);")
                .unwrap();
            let _ = &mut conn;
        }
        let _db = Db::open(&path).unwrap();
        let backup = path.with_extension("db.bak-v0");
        assert!(backup.exists(), "yedek dosyası oluşturulmalı");
        let conn = Connection::open(&backup).unwrap();
        let x: i64 = conn
            .query_row("SELECT x FROM legacy", [], |r| r.get(0))
            .unwrap();
        assert_eq!(x, 7);
    }

    #[test]
    fn corrupt_file_is_quarantined_and_fresh_db_created() {
        let path = temp_db_path("corrupt");
        std::fs::write(
            &path,
            b"bu bir sqlite dosyasi degil, bozuk veri bozuk veri bozuk veri",
        )
        .unwrap();
        let (db, quarantined) = Db::open_or_recover(&path).unwrap();
        let q = quarantined.expect("karantina dosyası");
        assert!(q.exists());
        assert_eq!(std::fs::read(&q).unwrap()[..4], *b"bu b");
        assert_eq!(
            db.with(migrations::current_version).unwrap(),
            migrations::MIGRATIONS.len()
        );
    }

    #[test]
    fn healthy_file_is_not_quarantined() {
        let path = temp_db_path("healthy");
        drop(Db::open(&path).unwrap());
        let (_db, q) = Db::open_or_recover(&path).unwrap();
        assert!(q.is_none());
    }

    #[test]
    fn failed_transaction_rolls_back() {
        let db = Db::open_in_memory().unwrap();
        let res: AppResult<()> = db.with_tx(|tx| {
            settings::set(tx, "k", &serde_json::json!(1))?;
            Err(crate::error::AppError::invalid("iptal"))
        });
        assert!(res.is_err());
        assert_eq!(db.with(|c| settings::get(c, "k")).unwrap(), None);
    }
}
