//! Tema kitaplığı komutları: içe aktarma (doğrulamalı), silme, dışa aktarma.

use crate::error::{AppError, AppResult};
use crate::state::{AppState, on_main};
use crate::themes::{self, CustomTheme, ThemeSummary};
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn list_custom_themes(state: State<'_, AppState>) -> AppResult<Vec<CustomTheme>> {
    state.db.with(themes::list)
}

/// İçe aktarılan tema dosyasının metni ön yüzde (dosya seçici ile) okunur ve buraya gönderilir;
/// komut hiçbir dosya yolunu kabul etmez.
#[tauri::command]
pub fn import_theme(
    app: AppHandle,
    window: tauri::WebviewWindow,
    text: String,
    replace: bool,
) -> AppResult<ThemeSummary> {
    super::require_main(&window)?;
    let state = app.state::<AppState>();
    let version = app.package_info().version.to_string();
    let summary = state
        .db
        .with(|c| themes::import(c, &text, &version, replace))?;
    log::info!("tema içe aktarıldı: {} {}", summary.id, summary.version);
    crate::events::broadcast(&app, crate::events::THEMES_CHANGED, &summary.id);
    Ok(summary)
}

/// Özel temayı siler. Etkin temaysa varsayılan temaya geçilir; kullanıcı verilerine dokunulmaz.
#[tauri::command]
pub async fn delete_custom_theme(
    app: AppHandle,
    window: tauri::WebviewWindow,
    id: String,
) -> AppResult<()> {
    super::require_main(&window)?;
    on_main(&app, move |app| {
        let state = app.state::<AppState>();
        state.db.with(|c| themes::delete(c, &id))?;
        crate::events::broadcast(app, crate::events::THEMES_CHANGED, &id);
        let s = crate::settings::load(&state.db)?;
        let mut patch = serde_json::Map::new();
        if s.theme == id {
            patch.insert("theme".into(), themes::DEFAULT_THEME.into());
            patch.insert("theme_variant".into(), "".into());
        }
        if s.theme_favorites.contains(&id) {
            let favs: Vec<String> = s.theme_favorites.into_iter().filter(|f| *f != id).collect();
            patch.insert("theme_favorites".into(), serde_json::json!(favs));
        }
        if !patch.is_empty() {
            super::settings::apply_settings_patch(app, serde_json::Value::Object(patch))?;
        }
        Ok(())
    })
}

/// Temayı İndirilenler klasörüne `.ivilytheme` dosyası olarak yazar ve Gezgin'de gösterir.
/// Var olan bir dosyanın üzerine yazılmaz (ad sonuna sayı eklenir).
#[tauri::command]
pub fn export_theme(state: State<'_, AppState>, id: String) -> AppResult<String> {
    if !themes::valid_id(&id) {
        return Err(AppError::invalid("Tema kimliği geçersiz."));
    }
    let manifest = state
        .db
        .with(|c| Ok(themes::manifest(c, &id)))?
        .ok_or(AppError::NotFound)?;
    let text = serde_json::to_string_pretty(&manifest)?;
    let dir = downloads_dir()
        .ok_or_else(|| AppError::Other("İndirilenler klasörü bulunamadı.".into()))?;
    let path = unique_path(&dir, &id);
    std::fs::write(&path, text)?;
    reveal(&path);
    Ok(path.to_string_lossy().to_string())
}

fn unique_path(dir: &std::path::Path, id: &str) -> std::path::PathBuf {
    // id yalnızca [a-z0-9-] içerir (valid_id), bu yüzden dosya adı güvenlidir.
    let mut p = dir.join(format!("{id}.ivilytheme"));
    let mut n = 2;
    while p.exists() {
        p = dir.join(format!("{id}-{n}.ivilytheme"));
        n += 1;
    }
    p
}

#[cfg(windows)]
fn downloads_dir() -> Option<std::path::PathBuf> {
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{FOLDERID_Downloads, KF_FLAG_DEFAULT, SHGetKnownFolderPath};
    // SAFETY: dönen PWSTR CoTaskMemFree ile serbest bırakılır.
    unsafe {
        let p = SHGetKnownFolderPath(&FOLDERID_Downloads, KF_FLAG_DEFAULT, None).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const _));
        s.map(std::path::PathBuf::from).filter(|d| d.is_dir())
    }
}

#[cfg(not(windows))]
fn downloads_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join("Downloads"))
}

fn reveal(path: &std::path::Path) {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("explorer.exe")
            .arg(format!("/select,{}", path.display()))
            .spawn();
    }
    #[cfg(not(windows))]
    let _ = path;
}

#[cfg(test)]
mod tests {
    #[test]
    fn unique_export_names() {
        let d = std::env::temp_dir().join(format!("ivily-export-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let a = super::unique_path(&d, "pixel");
        std::fs::write(&a, "x").unwrap();
        let b = super::unique_path(&d, "pixel");
        assert_ne!(a, b);
        assert!(b.ends_with("pixel-2.ivilytheme"));
    }
}
