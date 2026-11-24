//! Olay güdümlü zamanlayıcı: etkinlik/görev hatırlatıcıları ve odak aşaması bitişleri.
//!
//! Tek bir iş parçacığı bir sonraki tetiklenme anına kadar `Condvar` üzerinde uyur.
//! Veri değiştiğinde `poke()` ile uyandırılır ve sonraki anı yeniden hesaplar.
//! Saniyelik yoklama YAPILMAZ. Uyku/saat değişikliklerine karşı en uzun bekleme 5 dakikadır.

use crate::db::{self, Db};
use crate::events;
use crate::focus::{self, FocusConfig, FocusTimer, Phase};
use crate::state::AppState;
use chrono::{Duration as ChronoDuration, Local, NaiveDateTime};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

const MAX_WAIT: Duration = Duration::from_secs(300);
/// Başlangıcından bu kadar dakika fazla geçmiş etkinlik için geç bildirim gösterilmez.
const STALE_MINUTES: i64 = 60;

#[derive(Clone, Default)]
pub struct Scheduler {
    signal: Arc<(Mutex<bool>, Condvar)>,
}

impl Scheduler {
    /// Zamanlayıcıyı uyandırır (veri değişti).
    pub fn poke(&self) {
        let (lock, cv) = &*self.signal;
        *lock.lock().unwrap_or_else(|e| e.into_inner()) = true;
        cv.notify_one();
    }

    pub fn start(&self, app: AppHandle) {
        let signal = self.signal.clone();
        std::thread::Builder::new()
            .name("wakai-scheduler".into())
            .spawn(move || run(app, signal))
            .expect("zamanlayıcı iş parçacığı oluşturulamadı");
    }
}

pub fn poke(app: &AppHandle) {
    if let Some(s) = app.try_state::<Scheduler>() {
        s.poke();
    }
}

/// Odak Kedisi widget'ının yapılandırması (yoksa varsayılanlar).
pub fn focus_config(db: &Db) -> FocusConfig {
    db.with(db::widgets::list)
        .ok()
        .and_then(|list| list.into_iter().find(|w| w.kind == "focus"))
        .and_then(|w| serde_json::from_value::<FocusConfig>(w.config).ok())
        .unwrap_or_default()
        .sanitized()
}

fn parse(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, crate::validate::DATETIME_FORMAT).ok()
}

fn run(app: AppHandle, signal: Arc<(Mutex<bool>, Condvar)>) {
    loop {
        let wait = match tick(&app) {
            Some(d) => d.min(MAX_WAIT),
            None => MAX_WAIT,
        };
        let (lock, cv) = &*signal;
        let mut poked = lock.lock().unwrap_or_else(|e| e.into_inner());
        if !*poked {
            poked = cv
                .wait_timeout(poked, wait)
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
        *poked = false;
    }
}

/// Vadesi gelen işleri yapar; bir sonraki işe kalan süreyi döndürür.
fn tick(app: &AppHandle) -> Option<Duration> {
    let state = app.try_state::<AppState>()?;
    let db = &state.db;
    let notify = crate::settings::load(db)
        .map(|s| s.notifications_enabled)
        .unwrap_or(true);
    let now = Local::now().naive_local();
    let mut next: Option<NaiveDateTime> = None;
    let mut consider = |t: NaiveDateTime| {
        if next.is_none_or(|n| t < n) {
            next = Some(t);
        }
    };

    // Etkinlik hatırlatıcıları
    if let Ok(list) = db.with(db::events::pending_reminders) {
        for (id, title, start, minutes) in list {
            let Some(start_at) = parse(&start) else {
                continue;
            };
            let due = start_at - ChronoDuration::minutes(minutes);
            if due > now {
                consider(due);
                continue;
            }
            if start_at > now - ChronoDuration::minutes(STALE_MINUTES) && notify {
                let body = if start_at <= now {
                    "Şimdi başlıyor".to_string()
                } else {
                    format!("{} saatinde başlıyor", start_at.format("%H:%M"))
                };
                show(app, &format!("📅 {title}"), &body);
            }
            let _ = db.with(|c| db::events::mark_reminded(c, id));
        }
    }

    // Görev hatırlatıcıları
    if let Ok(list) = db.with(db::tasks::pending_reminders) {
        for (id, title, at) in list {
            let Some(due) = parse(&at) else { continue };
            if due > now {
                consider(due);
                continue;
            }
            if notify {
                show(app, "✅ Görev hatırlatıcısı", &title);
            }
            let _ = db.with(|c| db::tasks::mark_reminded(c, id));
        }
    }
    let mut wait = next.map(|t| (t - now).to_std().unwrap_or_default());

    // Odak aşaması
    if let Some(timer) = app.try_state::<FocusTimer>() {
        let cfg = focus_config(db);
        let now_ms = focus::now_ms();
        match timer.complete_if_due(db, &cfg, now_ms) {
            Ok(Some(done)) => {
                events::broadcast(app, events::FOCUS_CHANGED, timer.state(now_ms));
                if notify && cfg.notify {
                    let (title, body) = match done.finished {
                        Phase::Work => (
                            "🐾 Odak süresi bitti!",
                            if done.next == Phase::LongBreak {
                                "Uzun bir mola zamanı."
                            } else {
                                "Kısa bir mola ver."
                            },
                        ),
                        _ => ("☕ Mola bitti", "Hadi yeniden odaklanalım!"),
                    };
                    show(app, title, body);
                }
            }
            Ok(None) => {}
            Err(e) => log::error!("odak aşaması tamamlanamadı: {e}"),
        }
        if let Some(end) = timer.deadline() {
            let ms = (end - focus::now_ms()).max(0) as u64;
            let d = Duration::from_millis(ms + 20);
            wait = Some(wait.map_or(d, |w| w.min(d)));
        }
    }
    wait
}

fn show(app: &AppHandle, title: &str, body: &str) {
    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        log::warn!("bildirim gösterilemedi: {e}");
    }
}
