#![allow(dead_code)] // Açılış sağlığı ve geri dönüş 10. aşamada bağlanır.
//! Güncelleme güvenliği: önceki sürümün saklanması, sürüm geçmişi, açılış sağlığı ve geri dönüş.
//!
//! - Güncelleme kurulmadan önce çalışan program `previous/<sürüm>/wakai-desktop.exe` olarak saklanır
//!   (en fazla 2 önceki sürüm).
//! - Her açılışta sürüm `app_versions` tablosuna kaydedilir (göç 006).
//! - Açılış sağlığı: açılışta bir sayaç artırılır, uygulama 15 sn sağlıklı çalışınca sıfırlanır.
//!   Art arda 3 başarısız açılışta saklanan önceki sürüme otomatik dönülür.
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

/// Açılış başında çağrılır; art arda başarısız açılış sayısını döndürür (bu açılış dahil).
pub fn begin_launch(data_dir: &Path) -> u32 {
    let path = data_dir.join(HEALTH_FILE);
    let n = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0)
        + 1;
    let _ = std::fs::write(&path, n.to_string());
    n
}

/// Uygulama sağlıklı çalışınca çağrılır (sayaç sıfırlanır).
pub fn launch_healthy(data_dir: &Path) {
    let _ = std::fs::remove_file(data_dir.join(HEALTH_FILE));
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
    fn launch_health_counter() {
        let data = tmp("health");
        assert_eq!(begin_launch(&data), 1);
        assert_eq!(begin_launch(&data), 2);
        launch_healthy(&data);
        assert_eq!(begin_launch(&data), 1);
    }
}
