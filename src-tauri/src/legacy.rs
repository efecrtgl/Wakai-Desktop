//! Wakai Desktop (≤ 1.1.0) kurulumundan Ivily Desktop'a geçişin uygulama tarafı.
//!
//! Asıl taşıma NSIS kancasında yapılır (`windows/installer-hooks.nsh`). Burada yalnızca
//! kancanın yapamadıkları tamamlanır:
//! - Güncelleme sırasında kilitli kalan eski `wakai-desktop.exe` dosyasının silinmesi
//!   (yalnızca kanca eski kaldırma kaydını sildiyse, yani taşıma gerçekten yapıldıysa).
//! - Eski "Wakai Desktop" otomatik başlatma değerinin kaldırılması.
//!
//! Kullanıcı verilerine (`%APPDATA%\com.wakai.desktop`) hiçbir şekilde dokunulmaz.

use std::path::{Path, PathBuf};

pub const LEGACY_PRODUCT: &str = "Wakai Desktop";
const UNINSTALL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Wakai Desktop";
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// Wakai yükleyicisinin eski klasöre koyduğu dosyalar; başka hiçbir şey silinmez.
const LEGACY_FILES: &[&str] = &[
    "wakai-desktop.exe",
    "wakai-desktop.exe.old",
    "uninstall.exe",
];

/// Eski kurulum klasörü: `%LOCALAPPDATA%\Wakai Desktop`.
pub fn legacy_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|d| PathBuf::from(d).join(LEGACY_PRODUCT))
}

fn same_dir(a: &Path, b: &Path) -> bool {
    a.to_string_lossy()
        .trim_end_matches('\\')
        .eq_ignore_ascii_case(b.to_string_lossy().trim_end_matches('\\'))
}

/// Kalan eski program dosyaları silinebilir mi? Yalnızca: uygulama eski klasörden çalışmıyorsa
/// ve eski kurulumun kaldırma kaydı artık yoksa (kanca taşımayı tamamladıysa).
pub fn should_remove_files(legacy: &Path, exe_dir: &Path, uninstall_entry_exists: bool) -> bool {
    !same_dir(legacy, exe_dir)
        && !uninstall_entry_exists
        && legacy.join("wakai-desktop.exe").exists()
}

/// Yalnızca bilinen dosyaları siler, klasör boş kaldıysa klasörü de kaldırır.
pub fn remove_legacy_files(legacy: &Path) -> usize {
    let mut removed = 0;
    for name in LEGACY_FILES {
        let p = legacy.join(name);
        if p.exists() && std::fs::remove_file(&p).is_ok() {
            removed += 1;
        }
    }
    // remove_dir yalnızca boş klasörü siler.
    let _ = std::fs::remove_dir(legacy);
    removed
}

/// Açılışta arka planda çağrılır. Hatalar yalnızca günlüğe yazılır.
pub fn cleanup() {
    if cfg!(debug_assertions) {
        return; // Geliştirme derlemesi kurulu uygulamaya dokunmaz.
    }
    if reg::delete_value(RUN_KEY, LEGACY_PRODUCT) {
        log::info!("eski Wakai Desktop otomatik başlatma kaydı kaldırıldı");
    }
    let (Some(legacy), Ok(exe)) = (legacy_dir(), std::env::current_exe()) else {
        return;
    };
    let Some(exe_dir) = exe.parent() else { return };
    if should_remove_files(&legacy, exe_dir, reg::key_exists(UNINSTALL_KEY)) {
        let n = remove_legacy_files(&legacy);
        log::info!("eski Wakai Desktop klasöründen {n} dosya kaldırıldı");
    }
}

#[cfg(windows)]
mod reg {
    use windows::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_READ, RegCloseKey, RegDeleteKeyValueW, RegOpenKeyExW,
    };
    use windows::core::HSTRING;

    pub fn key_exists(path: &str) -> bool {
        let mut key = HKEY::default();
        // SAFETY: geçerli HSTRING ve çıkış tutamacı; açılan anahtar hemen kapatılır.
        unsafe {
            let ok = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                &HSTRING::from(path),
                None,
                KEY_READ,
                &mut key,
            )
            .is_ok();
            if ok {
                let _ = RegCloseKey(key);
            }
            ok
        }
    }

    /// Değer vardıysa ve silindiyse `true`.
    pub fn delete_value(path: &str, name: &str) -> bool {
        // SAFETY: yalnızca HKCU altında, adı verilen tek bir değer silinir.
        unsafe {
            RegDeleteKeyValueW(
                HKEY_CURRENT_USER,
                &HSTRING::from(path),
                &HSTRING::from(name),
            )
            .is_ok()
        }
    }
}

#[cfg(not(windows))]
mod reg {
    pub fn key_exists(_: &str) -> bool {
        false
    }
    pub fn delete_value(_: &str, _: &str) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ivily-legacy-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn removes_only_after_installer_migrated() {
        let legacy = tmp("cond");
        let exe_dir = tmp("cond-new");
        std::fs::write(legacy.join("wakai-desktop.exe"), b"x").unwrap();
        // Kaldırma kaydı hâlâ duruyorsa (taşıma yapılmadı) dokunma.
        assert!(!should_remove_files(&legacy, &exe_dir, true));
        // Uygulama eski klasörden çalışıyorsa dokunma (büyük/küçük harf ve sondaki ayraç önemsiz).
        let upper = PathBuf::from(legacy.to_string_lossy().to_uppercase() + "\\");
        assert!(!should_remove_files(&legacy, &upper, false));
        assert!(should_remove_files(&legacy, &exe_dir, false));
    }

    #[test]
    fn keeps_unknown_files_and_folder() {
        let legacy = tmp("files");
        std::fs::write(legacy.join("wakai-desktop.exe"), b"x").unwrap();
        std::fs::write(legacy.join("uninstall.exe"), b"x").unwrap();
        std::fs::write(legacy.join("kullanici-notu.txt"), b"x").unwrap();
        assert_eq!(remove_legacy_files(&legacy), 2);
        assert!(legacy.join("kullanici-notu.txt").exists());
        std::fs::remove_file(legacy.join("kullanici-notu.txt")).unwrap();
        remove_legacy_files(&legacy);
        assert!(!legacy.exists());
    }
}

/// "Wakai Desktop artık Ivily Desktop." bildirimi gösterilmeli mi? Yalnızca bu bilgisayarda daha
/// önce bir Wakai sürümü (< 1.1.5) çalışmışsa ve kullanıcı bildirimi henüz kapatmadıysa.
pub fn brand_notice_pending(conn: &rusqlite::Connection) -> crate::error::AppResult<bool> {
    if crate::db::settings::get(conn, BRAND_NOTICE_KEY)?.is_some() {
        return Ok(false);
    }
    Ok(crate::recovery::history(conn)?
        .iter()
        .any(|v| crate::recovery::compare(&v.version, "1.1.5").is_lt()))
}

pub const BRAND_NOTICE_KEY: &str = "brand_notice_dismissed";

#[cfg(test)]
mod notice_tests {
    use super::*;

    #[test]
    fn notice_only_for_wakai_users_until_dismissed() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        // Yeni kurulum: yalnızca 1.1.5 görülmüş.
        crate::recovery::record_launch(&conn, "1.1.5", 1).unwrap();
        assert!(!brand_notice_pending(&conn).unwrap());
        // Wakai'den gelen kullanıcı.
        crate::recovery::record_launch(&conn, "1.1.0", 0).unwrap();
        assert!(brand_notice_pending(&conn).unwrap());
        crate::db::settings::set(&conn, BRAND_NOTICE_KEY, &serde_json::json!(true)).unwrap();
        assert!(!brand_notice_pending(&conn).unwrap());
    }
}
