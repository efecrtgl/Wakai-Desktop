use crate::error::AppResult;
use crate::state::AppState;
use serde::Serialize;
use tauri::{Manager, State};

#[derive(Serialize)]
pub struct AppInfo {
    version: String,
    schema_version: usize,
    database_path: Option<String>,
}

#[tauri::command]
pub fn app_info(app: tauri::AppHandle, state: State<'_, AppState>) -> AppResult<AppInfo> {
    let schema_version = state.db.with(crate::db::migrations::current_version)?;
    Ok(AppInfo {
        version: app.package_info().version.to_string(),
        schema_version,
        database_path: state.db.path().map(|p| p.display().to_string()),
    })
}

#[tauri::command]
pub fn create_backup(state: State<'_, AppState>) -> AppResult<crate::backup::BackupInfo> {
    crate::backup::create(&state.db, &state.data_dir, "")
}

#[tauri::command]
pub fn list_backups(state: State<'_, AppState>) -> AppResult<Vec<crate::backup::BackupInfo>> {
    crate::backup::list(&state.data_dir)
}

#[tauri::command]
pub fn delete_backup(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    name: String,
) -> AppResult<()> {
    super::require_main(&window)?;
    crate::backup::delete(&state.data_dir, &name)
}

/// Yedeği bir sonraki açılış için işaretler ve uygulamayı yeniden başlatır.
#[tauri::command]
pub fn restore_backup(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    name: String,
) -> AppResult<()> {
    super::require_main(&window)?;
    crate::backup::schedule_restore(&state.data_dir, &name)?;
    crate::widgets::manager::flush_rects(&app);
    if let Some(store) = app.try_state::<crate::pets::store::PetStore>() {
        let _ = store.flush(&state.db);
    }
    app.restart();
}

/// Yedeği inceler: bütünlük, şema uyumluluğu, kategori başına kayıt sayısı.
#[tauri::command]
pub fn inspect_backup(
    state: State<'_, AppState>,
    name: String,
) -> AppResult<crate::backup::BackupDetails> {
    crate::backup::inspect(&state.data_dir, &name)
}

/// Yalnızca seçilen kategorileri bir sonraki açılışta geri yükler ve uygulamayı yeniden başlatır.
#[tauri::command]
pub fn restore_backup_selective(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    name: String,
    categories: Vec<String>,
) -> AppResult<()> {
    super::require_main(&window)?;
    crate::backup::schedule_selective_restore(&state.data_dir, &name, &categories)?;
    crate::widgets::manager::flush_rects(&app);
    if let Some(store) = app.try_state::<crate::pets::store::PetStore>() {
        let _ = store.flush(&state.db);
    }
    app.restart();
}

#[tauri::command]
pub fn send_test_notification(app: tauri::AppHandle) -> AppResult<()> {
    use tauri_plugin_notification::NotificationExt;
    app.notification()
        .builder()
        .title("🐾 Ivily Desktop")
        .body("Bildirimler çalışıyor! Kedin seni hatırlatacak.")
        .show()
        .map_err(|e| crate::error::AppError::Other(format!("Bildirim gösterilemedi: {e}")))
}

/// Uygulamanın kendi bellek/CPU kullanımı (ana süreç + WebView2 süreçleri).
#[tauri::command]
pub fn app_resource_usage(app: tauri::AppHandle) -> crate::procstats::ResourceUsage {
    use tauri::Manager;
    let widget_windows = app
        .webview_windows()
        .keys()
        .filter(|l| crate::widgets::window::id_from_label(l).is_some())
        .count();
    crate::procstats::usage(widget_windows)
}
