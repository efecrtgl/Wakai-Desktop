//! Masaüstü profilleri: ayarların üstünde bir katman. Bir profil uygulanınca yalnızca
//! profilin yönettiği ayarlar ve widget görünürlüğü değişir; widget konumları, kedi durumları
//! ve diğer ayarlar korunur.

use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::pets::behavior::ActivityLevel;
use crate::settings::AnimationQuality;
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

/// Profil uygulanınca widget'ların görünürlüğü nasıl ayarlanır?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetMode {
    /// Widget görünürlüğüne dokunma
    Keep,
    /// Tüm widget'ları göster
    All,
    /// Yalnızca listedeki türleri göster (ör. takvim, görevler)
    Kinds,
    /// Yalnızca seçili widget örneklerini göster
    Selected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProfileConfig {
    /// Masaüstü widget'ları genel olarak görünsün mü?
    pub widgets_visible: bool,
    pub widget_mode: WidgetMode,
    pub widget_kinds: Vec<String>,
    pub widget_ids: Vec<String>,
    pub pets_enabled: bool,
    pub pet_activity: ActivityLevel,
    pub pets_hide_fullscreen: bool,
    pub animation_quality: AnimationQuality,
    pub notifications_enabled: bool,
    /// Profil uygulanınca geçilecek tema (yoksa tema değişmez). 1.1.5
    pub theme: Option<String>,
    pub theme_variant: Option<String>,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            widgets_visible: true,
            widget_mode: WidgetMode::Keep,
            widget_kinds: Vec::new(),
            widget_ids: Vec::new(),
            pets_enabled: true,
            pet_activity: ActivityLevel::Normal,
            pets_hide_fullscreen: true,
            animation_quality: AnimationQuality::Full,
            notifications_enabled: true,
            theme: None,
            theme_variant: None,
        }
    }
}

pub const BUILTINS: &[&str] = &["working", "gaming", "relaxing", "focus"];

pub fn builtin_default(key: &str) -> Option<(&'static str, ProfileConfig)> {
    let d = ProfileConfig::default();
    Some(match key {
        // Çalışma: üretkenlik widget'ları açık, kediler sakin, animasyonlar azaltılmış.
        "working" => (
            "Çalışma",
            ProfileConfig {
                pet_activity: ActivityLevel::Low,
                animation_quality: AnimationQuality::Reduced,
                ..d
            },
        ),
        // Oyun: widget'lar ve kediler gizli, animasyon ve bildirim yok (arka plan işi en aza iner).
        "gaming" => (
            "Oyun",
            ProfileConfig {
                widgets_visible: false,
                pets_enabled: false,
                pet_activity: ActivityLevel::Paused,
                animation_quality: AnimationQuality::Off,
                notifications_enabled: false,
                ..d
            },
        ),
        // Dinlenme: her şey açık, kediler daha hareketli.
        "relaxing" => (
            "Dinlenme",
            ProfileConfig {
                widget_mode: WidgetMode::All,
                pet_activity: ActivityLevel::High,
                ..d
            },
        ),
        // Odaklanma: yalnızca üretkenlik widget'ları, kediler yerinde, animasyon yok.
        "focus" => (
            "Odaklanma",
            ProfileConfig {
                widget_mode: WidgetMode::Kinds,
                widget_kinds: ["calendar", "tasks", "focus", "clock", "note"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                pet_activity: ActivityLevel::Paused,
                animation_quality: AnimationQuality::Off,
                ..d
            },
        ),
        _ => return None,
    })
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Profile {
    pub id: i64,
    pub builtin: Option<String>,
    pub name: String,
    pub config: ProfileConfig,
}

/// Yerleşik profilleri (yoksa) oluşturur.
pub fn seed(db: &Db) -> AppResult<()> {
    db.with_tx(|tx| {
        for key in BUILTINS {
            let (name, cfg) = builtin_default(key).expect("yerleşik profil");
            tx.execute(
                "INSERT OR IGNORE INTO desktop_profiles (builtin, name, config) VALUES (?1, ?2, ?3)",
                params![key, name, serde_json::to_string(&cfg)?],
            )?;
        }
        Ok(())
    })
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<Profile> {
    let cfg: String = r.get(3)?;
    Ok(Profile {
        id: r.get(0)?,
        builtin: r.get(1)?,
        name: r.get(2)?,
        config: serde_json::from_str(&cfg).unwrap_or_default(),
    })
}

pub fn list(db: &Db) -> AppResult<Vec<Profile>> {
    seed(db)?;
    db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT id, builtin, name, config FROM desktop_profiles
             ORDER BY CASE builtin WHEN 'working' THEN 0 WHEN 'gaming' THEN 1 WHEN 'relaxing' THEN 2 WHEN 'focus' THEN 3 ELSE 4 END, id",
        )?;
        Ok(stmt.query_map([], row)?.collect::<Result<Vec<_>, _>>()?)
    })
}

pub fn get(db: &Db, id: i64) -> AppResult<Profile> {
    db.with(|c| {
        Ok(c.query_row(
            "SELECT id, builtin, name, config FROM desktop_profiles WHERE id = ?1",
            [id],
            row,
        )
        .optional()?)
    })?
    .ok_or(AppError::NotFound)
}

pub fn create(db: &Db, name: &str, config: &ProfileConfig) -> AppResult<Profile> {
    let name = crate::validate::text("Profil adı", name, 40, true)?;
    let id = db.with(|c| {
        c.execute(
            "INSERT INTO desktop_profiles (name, config) VALUES (?1, ?2)",
            params![name, serde_json::to_string(config)?],
        )?;
        Ok(c.last_insert_rowid())
    })?;
    get(db, id)
}

pub fn update(
    db: &Db,
    id: i64,
    name: Option<&str>,
    config: Option<&ProfileConfig>,
) -> AppResult<Profile> {
    let current = get(db, id)?;
    let name = match name {
        Some(n) => crate::validate::text("Profil adı", n, 40, true)?,
        None => current.name,
    };
    let config = config.cloned().unwrap_or(current.config);
    db.with(|c| {
        Ok(c.execute(
            "UPDATE desktop_profiles SET name = ?2, config = ?3 WHERE id = ?1",
            params![id, name, serde_json::to_string(&config)?],
        )?)
    })?;
    get(db, id)
}

pub fn delete(db: &Db, id: i64) -> AppResult<()> {
    if get(db, id)?.builtin.is_some() {
        return Err(AppError::invalid(
            "Yerleşik profiller silinemez; varsayılana döndürebilirsiniz.",
        ));
    }
    db.with(|c| Ok(c.execute("DELETE FROM desktop_profiles WHERE id = ?1", [id])?))?;
    Ok(())
}

pub fn reset(db: &Db, id: i64) -> AppResult<Profile> {
    let p = get(db, id)?;
    let key = p.builtin.ok_or_else(|| {
        AppError::invalid("Yalnızca yerleşik profiller varsayılana döndürülebilir.")
    })?;
    let (name, cfg) = builtin_default(&key).ok_or(AppError::NotFound)?;
    update(db, id, Some(name), Some(&cfg))
}

/// Profilin ayar yaması (yalnızca profilin yönettiği alanlar).
pub fn settings_patch(cfg: &ProfileConfig, id: i64) -> serde_json::Value {
    let mut patch = serde_json::json!({
        "widgets_visible": cfg.widgets_visible,
        "pets_enabled": cfg.pets_enabled,
        "pet_activity": cfg.pet_activity,
        "pets_hide_fullscreen": cfg.pets_hide_fullscreen,
        "animation_quality": cfg.animation_quality,
        "notifications_enabled": cfg.notifications_enabled,
        "active_profile": id,
    });
    if let Some(theme) = cfg.theme.as_deref().filter(|t| crate::themes::valid_id(t)) {
        patch["theme"] = theme.into();
        patch["theme_variant"] = cfg.theme_variant.clone().unwrap_or_default().into();
    }
    patch
}

/// Bir widget bu profilde görünür mü? `None`: profil görünürlüğe dokunmaz.
pub fn widget_visible(cfg: &ProfileConfig, id: &str, kind: &str) -> Option<bool> {
    match cfg.widget_mode {
        WidgetMode::Keep => None,
        WidgetMode::All => Some(true),
        WidgetMode::Kinds => Some(cfg.widget_kinds.iter().any(|k| k == kind)),
        WidgetMode::Selected => Some(cfg.widget_ids.iter().any(|w| w == id)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_theme_is_applied_only_when_set() {
        let mut cfg = ProfileConfig::default();
        assert!(
            settings_patch(&cfg, 1).get("theme").is_none(),
            "tema korunur"
        );
        cfg.theme = Some("cozy".into());
        cfg.theme_variant = Some("evening".into());
        let p = settings_patch(&cfg, 1);
        assert_eq!(
            (p["theme"].as_str(), p["theme_variant"].as_str()),
            (Some("cozy"), Some("evening"))
        );
        cfg.theme = Some("../kotu".into());
        assert!(settings_patch(&cfg, 1).get("theme").is_none());
    }

    #[test]
    fn builtins_are_seeded_once_and_protected() {
        let db = Db::open_in_memory().unwrap();
        let all = list(&db).unwrap();
        assert_eq!(all.len(), 4);
        assert_eq!(list(&db).unwrap().len(), 4);
        assert_eq!(
            all.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
            ["Çalışma", "Oyun", "Dinlenme", "Odaklanma"]
        );
        assert!(delete(&db, all[0].id).is_err());
    }

    #[test]
    fn custom_profiles_crud_and_reset() {
        let db = Db::open_in_memory().unwrap();
        let gaming = list(&db)
            .unwrap()
            .into_iter()
            .find(|p| p.builtin.as_deref() == Some("gaming"))
            .unwrap();
        assert!(!gaming.config.pets_enabled && !gaming.config.widgets_visible);
        let edited = update(
            &db,
            gaming.id,
            Some("Oyun+"),
            Some(&ProfileConfig {
                pets_enabled: true,
                ..gaming.config.clone()
            }),
        )
        .unwrap();
        assert!(edited.config.pets_enabled);
        let restored = reset(&db, gaming.id).unwrap();
        assert_eq!(restored.name, "Oyun");
        assert!(!restored.config.pets_enabled);

        let mine = create(&db, "Gece modu", &ProfileConfig::default()).unwrap();
        assert!(reset(&db, mine.id).is_err());
        assert!(create(&db, " ", &ProfileConfig::default()).is_err());
        delete(&db, mine.id).unwrap();
        assert!(matches!(get(&db, mine.id), Err(AppError::NotFound)));
    }

    #[test]
    fn widget_rules() {
        let (_, focus) = builtin_default("focus").unwrap();
        assert_eq!(widget_visible(&focus, "x", "calendar"), Some(true));
        assert_eq!(widget_visible(&focus, "x", "music"), Some(false));
        let (_, working) = builtin_default("working").unwrap();
        assert_eq!(widget_visible(&working, "x", "music"), None);
        let sel = ProfileConfig {
            widget_mode: WidgetMode::Selected,
            widget_ids: vec!["clock-1".into()],
            ..Default::default()
        };
        assert_eq!(widget_visible(&sel, "clock-1", "clock"), Some(true));
        assert_eq!(widget_visible(&sel, "clock-2", "clock"), Some(false));
    }
}

// ── Uygulama ve otomatik geçiş ────────────────────────────────────────────

use crate::state::AppState;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use tauri::{AppHandle, Manager};

/// Profili uygular. Pencere işlemleri yaptığı için ana iş parçacığında çağrılmalıdır.
pub fn apply(app: &AppHandle, id: i64) -> AppResult<Profile> {
    let state = app.state::<AppState>();
    let p = get(&state.db, id)?;
    crate::commands::settings::apply_settings_patch(app, settings_patch(&p.config, id))?;
    if p.config.widget_mode != WidgetMode::Keep {
        for inst in state.db.with(crate::db::widgets::list)? {
            if let Some(v) = widget_visible(&p.config, &inst.id, &inst.kind)
                && v != inst.visible
            {
                crate::widgets::manager::set_visible(app, &inst.id, v)?;
            }
        }
    }
    crate::events::broadcast(app, crate::events::PROFILES_CHANGED, ());
    crate::tray::rebuild(app);
    Ok(p)
}

/// Tam ekran uygulama algılanınca "Oyun" profiline geçen isteğe bağlı izleyici.
/// Yalnızca ayar açıkken çalışır (3 sn'de bir hafif bir kontrol); kapatılınca iş parçacığı biter.
#[derive(Default)]
pub struct AutoGaming {
    running: AtomicBool,
    /// Otomatik geçişten önceki profil (geri dönmek için); -1: yok
    previous: AtomicI64,
    active: AtomicBool,
}

pub fn sync_auto_gaming(app: &AppHandle, enabled: bool) {
    let auto = app.state::<AutoGaming>();
    if !enabled || auto.running.swap(true, Ordering::AcqRel) {
        return;
    }
    let app = app.clone();
    std::thread::Builder::new()
        .name("wakai-auto-profile".into())
        .spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3));
                let state = app.state::<AppState>();
                let settings = crate::settings::load(&state.db).unwrap_or_default();
                let auto = app.state::<AutoGaming>();
                if !settings.auto_gaming_profile {
                    auto.running.store(false, Ordering::Release);
                    return;
                }
                let fullscreen = crate::pets::native::fullscreen_app_active();
                let active = auto.active.load(Ordering::Acquire);
                let target = if fullscreen && !active {
                    auto.previous
                        .store(settings.active_profile.unwrap_or(-1), Ordering::Release);
                    auto.active.store(true, Ordering::Release);
                    list(&state.db).ok().and_then(|l| {
                        l.into_iter()
                            .find(|p| p.builtin.as_deref() == Some("gaming"))
                            .map(|p| p.id)
                    })
                } else if !fullscreen && active {
                    auto.active.store(false, Ordering::Release);
                    Some(auto.previous.load(Ordering::Acquire)).filter(|id| *id >= 0)
                } else {
                    None
                };
                if let Some(id) = target {
                    let a = app.clone();
                    let _ = app.run_on_main_thread(move || {
                        if let Err(e) = apply(&a, id) {
                            log::warn!("otomatik profil uygulanamadı: {e}");
                        }
                    });
                }
            }
        })
        .expect("otomatik profil iş parçacığı");
}
