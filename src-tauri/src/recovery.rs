//! Güncelleme güvenliği: önceki sürümün saklanması, sürüm geçmişi, açılış sağlığı ve geri dönüş.
//!
//! - Güncelleme kurulmadan önce çalışan program `previous/<sürüm>/wakai-desktop.exe` olarak saklanır
//!   (en fazla 2 önceki sürüm).
//! - Her açılışta sürüm `app_versions` tablosuna kaydedilir (göç 006).
//! - Açılış sağlığı (sürüm başına): bir sürüm henüz hiç sağlıklı açılmadıysa her açılışta sayaç
//!   artar; uygulama 15 sn sorunsuz çalışınca o sürüm "sağlıklı" işaretlenir ve bir daha sayılmaz.
//!   Hiç sağlıklı açılmamış bir sürüm art arda 3 kez başarısız olursa saklanan önceki sürüme
//!   otomatik dönülür. Sağlıklı açılmış bir sürüm asla otomatik geri alınmaz.
//! - Geri dönüş çalışan exe'nin üzerine yazmaz: Windows çalışan bir exe'nin yeniden adlandırılmasına
//!   izin verir; mevcut exe `.old` yapılır, önceki sürüm yerine kopyalanır ve uygulama yeniden başlar.
//!
//! Veritabanı geri yüklenmez: göçler yalnızca ekleme yapar (tablo/sütun), bu yüzden eski bir
//! sürüm yeni şemayla sorunsuz açılır (bkz. tests/integration.rs `newer_schema_is_left_untouched`).

use crate::error::{AppError, AppResult};
use serde::Serialize;
use std::path::{Path, PathBuf};

const KEEP_PREVIOUS: usize = 2;
const HEALTH_FILE: &str = "launch-health";
pub const FAILED_LAUNCHES_FOR_ROLLBACK: u32 = 3;

pub fn previous_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("previous")
}

fn valid_version(v: &str) -> bool {
    !v.is_empty()
        && v.len() < 32
        && v.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

/// Çalışan programı önceki sürüm olarak saklar; en eski kopyaları temizler.
pub fn preserve(data_dir: &Path, exe: &Path, version: &str) -> AppResult<PathBuf> {
    if !valid_version(version) {
        return Err(AppError::invalid("Geçersiz sürüm."));
    }
    let dir = previous_dir(data_dir).join(version);
    std::fs::create_dir_all(&dir)?;
    let target = dir.join("wakai-desktop.exe");
    std::fs::copy(exe, &target)?;
    prune(data_dir, version)?;
    Ok(target)
}

fn prune(data_dir: &Path, keep: &str) -> AppResult<()> {
    let mut versions = list_previous(data_dir)?;
    versions.retain(|p| p.version != keep);
    // Yeniden eskiye sırala; fazlasını sil.
    versions.sort_by(|a, b| compare(&b.version, &a.version));
    for old in versions.into_iter().skip(KEEP_PREVIOUS - 1) {
        let _ = std::fs::remove_dir_all(previous_dir(data_dir).join(&old.version));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PreviousVersion {
    pub version: String,
    pub size: u64,
}

pub fn list_previous(data_dir: &Path) -> AppResult<Vec<PreviousVersion>> {
    let dir = previous_dir(data_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out: Vec<PreviousVersion> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter_map(|e| {
            let version = e.file_name().to_string_lossy().to_string();
            let exe = e.path().join("wakai-desktop.exe");
            (valid_version(&version) && exe.exists()).then(|| PreviousVersion {
                size: std::fs::metadata(&exe).map(|m| m.len()).unwrap_or(0),
                version,
            })
        })
        .collect();
    out.sort_by(|a, b| compare(&b.version, &a.version));
    Ok(out)
}

/// Anlamsal sürüm karşılaştırması ("1.10.0" > "1.9.2").
pub fn compare(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |v: &str| -> Vec<u64> {
        v.split(['.', '-'])
            .map(|p| p.parse().unwrap_or(0))
            .collect()
    };
    parse(a).cmp(&parse(b))
}

/// Önceki sürüme döner: çalışan exe `.old` olur, saklanan sürüm yerine kopyalanır.
/// Çağıran, ardından uygulamayı yeniden başlatmalıdır.
pub fn rollback(data_dir: &Path, exe: &Path, version: &str) -> AppResult<()> {
    if !valid_version(version) {
        return Err(AppError::invalid("Geçersiz sürüm."));
    }
    let source = previous_dir(data_dir)
        .join(version)
        .join("wakai-desktop.exe");
    if !source.exists() {
        return Err(AppError::invalid("Bu sürümün yedeği bulunamadı."));
    }
    let old = exe.with_extension("exe.old");
    let _ = std::fs::remove_file(&old);
    std::fs::rename(exe, &old)?;
    if let Err(e) = std::fs::copy(&source, exe) {
        // Kopyalama başarısızsa çalışan sürümü geri koy: uygulama bozuk bırakılmaz.
        let _ = std::fs::rename(&old, exe);
        return Err(e.into());
    }
    Ok(())
}

/// Önceki açılışlardan kalan `.old` dosyasını temizler.
pub fn cleanup_old(exe: &Path) {
    let _ = std::fs::remove_file(exe.with_extension("exe.old"));
}

fn read_health(data_dir: &Path) -> Option<(String, String)> {
    let text = std::fs::read_to_string(data_dir.join(HEALTH_FILE)).ok()?;
    let mut it = text.split_whitespace();
    Some((it.next()?.to_string(), it.next()?.to_string()))
}

fn write_health(data_dir: &Path, version: &str, value: &str) {
    let _ = std::fs::write(data_dir.join(HEALTH_FILE), format!("{version} {value}"));
}

/// Açılış başında çağrılır. Bu sürüm daha önce sağlıklı açıldıysa 0, değilse art arda
/// başarısız açılış sayısını (bu açılış dahil) döndürür.
pub fn begin_launch(data_dir: &Path, version: &str) -> u32 {
    let n = match read_health(data_dir) {
        Some((v, state)) if v == version => match state.as_str() {
            "ok" => return 0,
            n => n.parse::<u32>().unwrap_or(0) + 1,
        },
        _ => 1,
    };
    write_health(data_dir, version, &n.to_string());
    n
}

/// Uygulama sağlıklı çalışınca çağrılır: bu sürüm bir daha sayılmaz.
pub fn launch_healthy(data_dir: &Path, version: &str) {
    write_health(data_dir, version, "ok");
}

/// Hiç sağlıklı açılmamış sürüm art arda başarısız olduysa dönülecek önceki sürüm.
pub fn auto_rollback_target(data_dir: &Path, version: &str, failed: u32) -> Option<String> {
    if failed < FAILED_LAUNCHES_FOR_ROLLBACK {
        return None;
    }
    list_previous(data_dir)
        .ok()?
        .into_iter()
        .map(|p| p.version)
        .find(|v| v != version)
}

/// Otomatik geri dönüş yapıldı: sağlık kaydı, geri alınan sürüm için döngüye girmesin diye işaretlenir.
pub fn mark_rolled_back(data_dir: &Path, from_version: &str) {
    write_health(data_dir, from_version, "rolled-back");
    let _ = std::fs::write(data_dir.join("rollback-notice"), from_version);
}

/// Bir önceki açılışta otomatik geri dönüş yapıldıysa geri alınan sürümü döndürür (bir kez).
pub fn take_rollback_notice(data_dir: &Path) -> Option<String> {
    let path = data_dir.join("rollback-notice");
    let v = std::fs::read_to_string(&path).ok()?;
    let _ = std::fs::remove_file(path);
    Some(v.trim().to_string())
}

// --- Sürüm geçmişi (app_versions) ---

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct VersionRecord {
    pub version: String,
    pub first_seen: i64,
    pub last_seen: i64,
    pub launches: i64,
}

/// Bu açılışı kaydeder; bir önceki açılıştaki sürüm farklıysa onu döndürür (yükseltme/geri dönüş).
pub fn record_launch(
    conn: &rusqlite::Connection,
    version: &str,
    now: i64,
) -> AppResult<Option<String>> {
    let previous: Option<String> = conn
        .query_row(
            "SELECT version FROM app_versions ORDER BY last_seen DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .ok();
    conn.execute(
        "INSERT INTO app_versions (version, first_seen, last_seen, launches) VALUES (?1, ?2, ?2, 1)
         ON CONFLICT(version) DO UPDATE SET last_seen = ?2, launches = launches + 1",
        rusqlite::params![version, now],
    )?;
    Ok(previous.filter(|p| p != version))
}

pub fn history(conn: &rusqlite::Connection) -> AppResult<Vec<VersionRecord>> {
    let mut stmt = conn.prepare("SELECT version, first_seen, last_seen, launches FROM app_versions ORDER BY first_seen DESC")?;
    let rows = stmt.query_map([], |r| {
        Ok(VersionRecord {
            version: r.get(0)?,
            first_seen: r.get(1)?,
            last_seen: r.get(2)?,
            launches: r.get(3)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("wakai-recovery-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn version_comparison() {
        use std::cmp::Ordering::*;
        assert_eq!(compare("1.10.0", "1.9.2"), Greater);
        assert_eq!(compare("1.1.0", "1.1.0"), Equal);
        assert_eq!(compare("1.0.0", "1.1.0"), Less);
        assert_eq!(compare("2.0", "1.99.99"), Greater);
    }

    #[test]
    fn preserve_keeps_two_previous_versions() {
        let data = tmp("preserve");
        let exe = data.join("app.exe");
        for v in ["1.0.0", "1.1.0", "1.2.0"] {
            std::fs::write(&exe, v).unwrap();
            preserve(&data, &exe, v).unwrap();
        }
        let list: Vec<_> = list_previous(&data)
            .unwrap()
            .into_iter()
            .map(|p| p.version)
            .collect();
        assert_eq!(list, ["1.2.0", "1.1.0"]);
        assert!(preserve(&data, &exe, "../x").is_err());
    }

    #[test]
    fn rollback_swaps_executable_and_keeps_old_copy() {
        let data = tmp("rollback");
        let exe = data.join("wakai-desktop.exe");
        std::fs::write(&exe, "eski").unwrap();
        preserve(&data, &exe, "1.0.0").unwrap();
        std::fs::write(&exe, "yeni-bozuk").unwrap();
        rollback(&data, &exe, "1.0.0").unwrap();
        assert_eq!(std::fs::read_to_string(&exe).unwrap(), "eski");
        assert_eq!(
            std::fs::read_to_string(exe.with_extension("exe.old")).unwrap(),
            "yeni-bozuk"
        );
        cleanup_old(&exe);
        assert!(!exe.with_extension("exe.old").exists());
        assert!(rollback(&data, &exe, "9.9.9").is_err());
        assert_eq!(
            std::fs::read_to_string(&exe).unwrap(),
            "eski",
            "başarısız geri dönüş exe'yi bozmamalı"
        );
    }

    #[test]
    fn launch_health_is_per_version() {
        let data = tmp("health");
        assert_eq!(begin_launch(&data, "1.1.0"), 1);
        assert_eq!(begin_launch(&data, "1.1.0"), 2);
        launch_healthy(&data, "1.1.0");
        assert_eq!(
            begin_launch(&data, "1.1.0"),
            0,
            "sağlıklı sürüm bir daha sayılmaz"
        );
        assert_eq!(begin_launch(&data, "1.2.0"), 1, "yeni sürüm baştan sayılır");
    }

    #[test]
    fn auto_rollback_only_after_repeated_failures_with_backup() {
        let data = tmp("auto");
        let exe = data.join("wakai-desktop.exe");
        assert_eq!(
            auto_rollback_target(&data, "1.2.0", 5),
            None,
            "yedek yokken geri dönülmez"
        );
        std::fs::write(&exe, "x").unwrap();
        preserve(&data, &exe, "1.1.0").unwrap();
        assert_eq!(auto_rollback_target(&data, "1.2.0", 2), None);
        assert_eq!(
            auto_rollback_target(&data, "1.2.0", 3).as_deref(),
            Some("1.1.0")
        );
        assert_eq!(
            auto_rollback_target(&data, "1.1.0", 3),
            None,
            "aynı sürüme dönülmez"
        );
        mark_rolled_back(&data, "1.2.0");
        assert_eq!(
            begin_launch(&data, "1.1.0"),
            1,
            "geri dönülen sürüm sayaçla döngüye girmez"
        );
        assert_eq!(take_rollback_notice(&data).as_deref(), Some("1.2.0"));
        assert_eq!(take_rollback_notice(&data), None);
    }

    #[test]
    fn version_history_records_launches_and_changes() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("db/migrations/006_app_versions.sql"))
            .unwrap();
        assert_eq!(record_launch(&conn, "1.0.0", 1).unwrap(), None);
        assert_eq!(record_launch(&conn, "1.0.0", 2).unwrap(), None);
        assert_eq!(
            record_launch(&conn, "1.1.0", 3).unwrap().as_deref(),
            Some("1.0.0")
        );
        let h = history(&conn).unwrap();
        assert_eq!(h.len(), 2);
        assert_eq!(h[0].version, "1.1.0");
        assert_eq!(h[1].launches, 2);
    }
}
