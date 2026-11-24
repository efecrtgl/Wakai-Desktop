//! Rust'tan pencerelere yayınlanan olayların adları.
//! Pencereler bu olayları dinleyerek yalnızca etkilendikleri veriyi yeniden ister.

use tauri::{AppHandle, Emitter};

pub const SETTINGS_CHANGED: &str = "settings:changed";
/// Widget listesi değişti (eklendi, silindi, taşındı...). Yük yok; dinleyen listeyi yeniden ister.
pub const WIDGETS_CHANGED: &str = "widgets:changed";
/// Tek bir widget örneği güncellendi. Yük: `WidgetInstance`.
pub const WIDGET_UPDATED: &str = "widget:updated";
/// Kontrol Merkezi bir bölüme yönlendirilsin. Yük: `{ section, widget_id }`.
pub const NAVIGATE: &str = "navigate";

/// Olayı tüm pencerelere yayınlar. Yayın hatası kritik değildir; günlüğe yazılır.
pub fn broadcast<S: serde::Serialize + Clone>(app: &AppHandle, event: &str, payload: S) {
    if let Err(e) = app.emit(event, payload) {
        log::warn!("olay yayınlanamadı ({event}): {e}");
    }
}
