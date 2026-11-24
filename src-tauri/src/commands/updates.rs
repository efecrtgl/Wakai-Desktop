//! Güncelleme komutları. Kurulum yalnızca kullanıcı "Güncellemeyi İndir" dediğinde yapılır.

use crate::error::AppResult;
use crate::events;
use crate::recovery::{self, PreviousVersion, VersionRecord};
use crate::state::AppState;
use crate::updater::{self, UpdateInfo};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

#[derive(Serialize)]
pub struct UpdateStatus {
    pub current: String,
    pub pending: Option<UpdateInfo>,
    pub last_check: Option<i64>,
}

/// Elle denetim. Güncelleme bulunursa Kontrol Merkezi'ndeki pencere açılır.
#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> AppResult<Option<UpdateInfo>> {
    let found = updater::check(&app).await?;
    if let Some(info) = &found {
        let mut payload = serde_json::to_value(info)?;
        payload["manual"] = serde_json::Value::Bool(true);
        events::broadcast(&app, events::UPDATE_AVAILABLE, payload);
    }
    Ok(found)
}

#[tauri::command]
pub fn update_status(app: AppHandle) -> UpdateStatus {
    UpdateStatus {
        current: app.package_info().version.to_string(),
        pending: updater::pending(&app),
        last_check: updater::last_check(&app),
    }
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> AppResult<()> {
    updater::download_and_install(&app).await
}

/// İndirir ve imzayı doğrular, kurmaz (tanı/test).
#[tauri::command]
pub async fn verify_update(app: AppHandle) -> AppResult<u64> {
    updater::verify_only(&app).await
}

#[derive(Serialize)]
pub struct VersionHistory {
    pub current: String,
    pub history: Vec<VersionRecord>,
    pub previous: Vec<PreviousVersion>,
}

/// Bu bilgisayarda çalışmış sürümler ve geri dönülebilecek saklı sürümler.
#[tauri::command]
pub fn version_history(app: AppHandle, state: State<'_, AppState>) -> AppResult<VersionHistory> {
    let current = app.package_info().version.to_string();
    let previous = recovery::list_previous(&state.data_dir)?
        .into_iter()
        .filter(|p| p.version != current)
        .collect();
    Ok(VersionHistory {
        current,
        history: state.db.with(recovery::history)?,
        previous,
    })
}

/// Saklanan önceki sürüme döner ve uygulamayı yeniden başlatır. Önce veritabanı yedeklenir;
/// veriler geri alınmaz (göçler eski sürümlerle uyumludur).
#[tauri::command]
pub fn rollback_to_version(app: AppHandle, version: String) -> AppResult<()> {
    let state = app.state::<AppState>();
    // Önce doğrula: geçersiz istek yedek ya da başka bir iz bırakmasın.
    if !recovery::list_previous(&state.data_dir)?
        .iter()
        .any(|p| p.version == version)
    {
        return Err(crate::error::AppError::invalid(
            "Bu sürümün yedeği bulunamadı.",
        ));
    }
    crate::widgets::manager::flush_rects(&app);
    if let Some(store) = app.try_state::<crate::pets::store::PetStore>() {
        store.flush(&state.db)?;
    }
    crate::backup::create(&state.db, &state.data_dir, "-geri-donus-oncesi")?;
    let exe = std::env::current_exe()?;
    recovery::rollback(&state.data_dir, &exe, &version)?;
    log::info!(
        "elle geri dönüş: {} → {version}",
        app.package_info().version
    );
    app.restart();
}
