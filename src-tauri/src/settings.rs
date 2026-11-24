//! Uygulama ayarları: türlü yapı, varsayılanlar ve doğrulama.
//! Ayarlar tek bir JSON nesnesi olarak `settings` tablosunda `app` anahtarıyla saklanır.

use crate::db::{self, Db};
use crate::error::AppResult;
use serde::{Deserialize, Serialize};

const KEY: &str = "app";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationQuality {
    Full,
    Reduced,
    Off,
}

/// Görünüm modu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Appearance {
    /// Özgün pastel Wakai görünümü
    Wakai,
    /// Siyah görünüm
    Dark,
    /// Beyaz görünüm
    Light,
}

/// Yazı tipi seçimi (hepsi Windows 11 ile gelir).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontFamily {
    System,
    Modern,
    Mono,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub appearance: Appearance,
    /// Masaüstü kedileri açık mı?
    pub pets_enabled: bool,
    /// Kedilerin genel etkinlik düzeyi (profiller değiştirir).
    pub pet_activity: crate::pets::behavior::ActivityLevel,
    /// Kedi boyutu: sprite pikseli başına ekran pikseli (2–5).
    pub pet_size: u32,
    /// Masaüstünde aynı anda dolaşabilecek en fazla kedi (1–12).
    pub pets_max_active: u32,
    /// Tam ekran bir uygulama (oyun, video) öndeyken kedileri gizle.
    pub pets_hide_fullscreen: bool,
    /// Etkin masaüstü profili (yoksa `None`).
    pub active_profile: Option<i64>,
    /// Tam ekran uygulama algılanınca "Oyun" profiline otomatik geç.
    pub auto_gaming_profile: bool,
    pub font_family: FontFamily,
    /// Ana pencere kapatıldığında uygulama tepside çalışmaya devam eder.
    pub close_to_tray: bool,
    /// Windows açıldığında Wakai Desktop'u başlat.
    pub launch_on_startup: bool,
    /// Masaüstü widget'larını göster.
    pub widgets_visible: bool,
    /// Widget opaklığı (0.4 – 1.0).
    pub widget_opacity: f64,
    /// Widget ölçeği (0.75 – 1.5).
    pub widget_scale: f64,
    pub animation_quality: AnimationQuality,
    pub reduced_motion: bool,
    pub shadows: bool,
    /// Köşe yuvarlaklığı piksel cinsinden (0 – 28).
    pub corner_radius: u32,
    /// Sistem izleme yenileme aralığı saniye cinsinden (1 – 60).
    pub monitor_interval_secs: u32,
    /// Hava durumu yenileme aralığı dakika cinsinden (15 – 180).
    pub weather_refresh_minutes: u32,
    /// Düşük kaynak modu: animasyonları kapatır, aralıkları uzatır.
    pub reduced_resource_mode: bool,
    pub notifications_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            appearance: Appearance::Wakai,
            pets_enabled: true,
            pet_activity: crate::pets::behavior::ActivityLevel::Normal,
            pet_size: 3,
            pets_max_active: 5,
            pets_hide_fullscreen: true,
            active_profile: None,
            auto_gaming_profile: false,
            font_family: FontFamily::System,
            close_to_tray: true,
            launch_on_startup: false,
            widgets_visible: true,
            widget_opacity: 1.0,
            widget_scale: 1.0,
            animation_quality: AnimationQuality::Full,
            reduced_motion: false,
            shadows: true,
            corner_radius: 18,
            monitor_interval_secs: 3,
            weather_refresh_minutes: 30,
            reduced_resource_mode: false,
            notifications_enabled: true,
        }
    }
}

impl AppSettings {
    /// Değerleri izin verilen aralıklara sıkıştırır.
    pub fn sanitized(mut self) -> Self {
        let clamp_f =
            |v: f64, lo: f64, hi: f64, d: f64| if v.is_finite() { v.clamp(lo, hi) } else { d };
        self.widget_opacity = clamp_f(self.widget_opacity, 0.4, 1.0, 1.0);
        self.widget_scale = clamp_f(self.widget_scale, 0.75, 1.5, 1.0);
        self.corner_radius = self.corner_radius.min(28);
        self.pet_size = self.pet_size.clamp(2, 5);
        self.pets_max_active = self.pets_max_active.clamp(1, 12);
        self.monitor_interval_secs = self.monitor_interval_secs.clamp(1, 60);
        self.weather_refresh_minutes = self.weather_refresh_minutes.clamp(15, 180);
        self
    }
}

pub fn load(db: &Db) -> AppResult<AppSettings> {
    db.with(|c| {
        // Bozuk JSON kaydı yok sayılır (varsayılanlara düşülür); gerçek veritabanı hataları iletilir.
        let value = match db::settings::get(c, KEY) {
            Ok(v) => v,
            Err(crate::error::AppError::Json(_)) => None,
            Err(e) => return Err(e),
        };
        // Bozuk ya da eski biçimli kayıt varsayılanlara düşer; eksik alanlar `serde(default)` ile tamamlanır.
        let parsed = value
            .and_then(|v| serde_json::from_value::<AppSettings>(v).ok())
            .unwrap_or_default();
        Ok(parsed.sanitized())
    })
}

/// Kısmi bir JSON yamasını mevcut ayarlarla birleştirir, doğrular ve kaydeder.
pub fn update(db: &Db, patch: serde_json::Value) -> AppResult<AppSettings> {
    let serde_json::Value::Object(patch) = patch else {
        return Err(crate::error::AppError::invalid(
            "Ayar güncellemesi bir nesne olmalıdır.",
        ));
    };
    let current = load(db)?;
    let mut merged = serde_json::to_value(&current)?;
    if let serde_json::Value::Object(map) = &mut merged {
        for (k, v) in patch {
            if map.contains_key(&k) {
                map.insert(k, v);
            }
        }
    }
    let next: AppSettings = serde_json::from_value(merged)
        .map_err(|e| crate::error::AppError::invalid(format!("Geçersiz ayar değeri: {e}")))?;
    let next = next.sanitized();
    if next != current {
        let value = serde_json::to_value(&next)?;
        db.with(|c| db::settings::set(c, KEY, &value))?;
    }
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_empty() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(load(&db).unwrap(), AppSettings::default());
    }

    #[test]
    fn partial_update_is_merged_and_clamped() {
        let db = Db::open_in_memory().unwrap();
        let s = update(
            &db,
            serde_json::json!({ "widget_opacity": 5.0, "shadows": false, "unknown": 1 }),
        )
        .unwrap();
        assert_eq!(s.widget_opacity, 1.0);
        assert!(!s.shadows);
        assert_eq!(load(&db).unwrap(), s);
    }

    #[test]
    fn appearance_roundtrip_and_validation() {
        let db = Db::open_in_memory().unwrap();
        let s = update(
            &db,
            serde_json::json!({ "appearance": "dark", "font_family": "modern" }),
        )
        .unwrap();
        assert_eq!(s.appearance, Appearance::Dark);
        assert_eq!(load(&db).unwrap().font_family, FontFamily::Modern);
        assert!(update(&db, serde_json::json!({ "appearance": "neon" })).is_err());
    }

    #[test]
    fn invalid_type_is_rejected() {
        let db = Db::open_in_memory().unwrap();
        assert!(update(&db, serde_json::json!({ "shadows": "evet" })).is_err());
        assert!(update(&db, serde_json::json!([1, 2])).is_err());
    }
}
