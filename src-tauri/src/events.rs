//! Rust'tan pencerelere yayınlanan olayların adları.
//! Pencereler bu olayları dinleyerek yalnızca etkilendikleri veriyi yeniden ister.

use tauri::{AppHandle, Emitter};

pub const SETTINGS_CHANGED: &str = "settings:changed";

/// Olayı tüm pencerelere yayınlar. Yayın hatası kritik değildir; günlüğe yazılır.
pub fn broadcast<S: serde::Serialize + Clone>(app: &AppHandle, event: &str, payload: S) {
    if let Err(e) = app.emit(event, payload) {
        log::warn!("olay yayınlanamadı ({event}): {e}");
    }
}
