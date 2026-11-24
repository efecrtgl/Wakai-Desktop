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

pub const EVENTS_CHANGED: &str = "events:changed";
pub const TASKS_CHANGED: &str = "tasks:changed";
/// Yük: değişen notun kimliği.
pub const NOTES_CHANGED: &str = "notes:changed";
/// Yük: `FocusState`.
pub const FOCUS_CHANGED: &str = "focus:changed";
/// Yük: `SystemStats`.
pub const SYSTEM_STATS: &str = "system:stats";
/// Yük: `MediaState`.
pub const MEDIA_CHANGED: &str = "media:changed";
/// Yük: değişen alışveriş listesinin kimliği.
pub const SHOPPING_CHANGED: &str = "shopping:changed";
/// Kediler ya da ihtiyaçları değişti (etkileşimlerde; yüksek frekanslı değildir).
pub const PETS_CHANGED: &str = "pets:changed";
/// Profiller ya da etkin profil değişti.
pub const PROFILES_CHANGED: &str = "profiles:changed";
/// Yeni sürüm bulundu. Yük: `UpdateInfo`.
pub const UPDATE_AVAILABLE: &str = "update:available";
/// Güncelleme indirme ilerlemesi. Yük: `{ downloaded, total }` ya da `{ installing: true }`.
pub const UPDATE_PROGRESS: &str = "update:progress";
