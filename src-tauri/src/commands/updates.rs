//! Güncelleme komutları. Kurulum yalnızca kullanıcı "Güncellemeyi İndir" dediğinde yapılır.

use crate::error::AppResult;
use crate::events;
use crate::updater::{self, UpdateInfo};
use serde::Serialize;
use tauri::AppHandle;

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
