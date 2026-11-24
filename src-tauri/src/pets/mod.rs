//! Masaüstü kedi sistemi. Widget motorundan tamamen bağımsızdır:
//! - `model`, `behavior`: saf alan mantığı (kişilik, ihtiyaçlar, davranış seçimi)
//! - `art`: sprite yükleme ve piksel çizimi
//! - `store`: bellekteki tek doğruluk kaynağı (+ seyrek kalıcılık)
//! - `service`: etkileşimler ve ilerleme (motor ve komutlar ortak kullanır)
//! - `native`: Win32 katmanlı pencereler (tüm `unsafe` kod)
//! - `engine`: kedi pencerelerinin sahibi olan iş parçacığı

pub mod art;
pub mod behavior;
pub mod engine;
pub mod model;
pub mod native;
pub mod service;
pub mod store;

use crate::settings::AppSettings;

pub fn engine_settings(s: &AppSettings) -> engine::EngineSettings {
    engine::EngineSettings {
        enabled: s.pets_enabled,
        level: s.pet_activity,
        size: s.pet_size,
        max_active: s.pets_max_active as usize,
        hide_fullscreen: s.pets_hide_fullscreen,
    }
}
