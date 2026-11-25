//! İmzalı otomatik güncellemeler (resmi Tauri Updater eklentisi + GitHub Releases).
//!
//! - Açılıştan ~20 sn sonra, en fazla 20 saatte bir, eşzamansız denetim (açılışı ve widget'ları
//!   geciktirmez; internet yoksa sessizce vazgeçer). Ayarlardan kapatılabilir; elle denetlenebilir.
//! - Güncelleme asla kullanıcı onayı olmadan kurulmaz.
//! - İmza, uygulamaya gömülü genel anahtarla doğrulanır; imzasız ya da geçersiz yapıtlar reddedilir.
//! - Kurulumdan önce: veritabanı yedeği + çalışan programın kopyası (geri dönüş için).

use crate::error::{AppError, AppResult};
use crate::events;
use crate::state::AppState;
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

const STARTUP_DELAY: Duration = Duration::from_secs(20);
const MIN_INTERVAL_MS: i64 = 20 * 3600 * 1000;
const LAST_CHECK_KEY: &str = "update_last_check";

#[derive(Default)]
pub struct UpdateState {
    pending: Mutex<Option<Update>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub current: String,
    pub version: String,
    pub notes: String,
    pub date: Option<String>,
}

fn info(u: &Update) -> UpdateInfo {
    UpdateInfo {
        current: u.current_version.clone(),
        version: u.version.clone(),
        notes: u.body.clone().unwrap_or_default(),
        date: u.date.map(|d| d.date().to_string()),
    }
}

fn updater(app: &AppHandle) -> AppResult<tauri_plugin_updater::Updater> {
    let mut b = app.updater_builder().timeout(Duration::from_secs(30));
    // Test için uç nokta geçersiz kılma. Güvenlidir: imza yine gömülü anahtarla doğrulanır ve
    // sürüm derlemelerinde HTTPS dışı adresler eklenti tarafından reddedilir.
    // Yalnızca hata ayıklama derlemelerinde geçerlidir.
    if cfg!(debug_assertions)
        && let Ok(url) = std::env::var("WAKAI_UPDATE_ENDPOINT")
        && let Ok(u) = url.parse()
    {
        b = b
            .endpoints(vec![u])
            .map_err(|e| AppError::Other(format!("Güncelleme adresi geçersiz: {e}")))?;
    }
    b.build()
        .map_err(|e| AppError::Other(format!("Güncelleyici hazırlanamadı: {e}")))
}

fn now_ms() -> i64 {
    chrono::Local::now().timestamp_millis()
}

/// Güncelleme var mı? Varsa bekleyen güncelleme olarak saklanır.
pub async fn check(app: &AppHandle) -> AppResult<Option<UpdateInfo>> {
    let result = updater(app)?.check().await.map_err(|e| {
        AppError::Other(format!(
            "Güncelleme denetlenemedi. İnternet bağlantınızı kontrol edin. ({e})"
        ))
    })?;
    let state = app.state::<AppState>();
    let _ = state
        .db
        .with(|c| crate::db::settings::set(c, LAST_CHECK_KEY, &serde_json::json!(now_ms())));
    let out = result.as_ref().map(info);
    *app.state::<UpdateState>()
        .pending
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = result;
    Ok(out)
}

pub fn pending(app: &AppHandle) -> Option<UpdateInfo> {
    app.state::<UpdateState>()
        .pending
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .map(info)
}

pub fn last_check(app: &AppHandle) -> Option<i64> {
    app.state::<AppState>()
        .db
        .with(|c| crate::db::settings::get(c, LAST_CHECK_KEY))
        .ok()
        .flatten()
        .and_then(|v| v.as_i64())
}

/// Açılıştan sonra arka planda bir kez denetler (ayar açıksa ve son denetim eskiyse).
/// Kendi kısa ömürlü iş parçacığında bekler; açılışı geciktirmez.
pub fn spawn_startup_check(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(STARTUP_DELAY);
        let enabled = crate::settings::load(&app.state::<AppState>().db)
            .map(|s| s.auto_update_check)
            .unwrap_or(true);
        let fresh = last_check(&app).is_some_and(|t| now_ms() - t < MIN_INTERVAL_MS);
        if !enabled || fresh {
            return;
        }
        match tauri::async_runtime::block_on(check(&app)) {
            Ok(Some(info)) => {
                events::broadcast(&app, events::UPDATE_AVAILABLE, info.clone());
                notify(&app, &info);
            }
            Ok(None) => {}
            Err(e) => log::warn!("açılış güncelleme denetimi başarısız: {e}"),
        }
    });
}

fn notify(app: &AppHandle, info: &UpdateInfo) {
    let enabled = crate::settings::load(&app.state::<AppState>().db)
        .map(|s| s.notifications_enabled)
        .unwrap_or(true);
    if !enabled {
        return;
    }
    use tauri_plugin_notification::NotificationExt;
    let _ = app
        .notification()
        .builder()
        .title("Yeni bir sürüm hazır!")
        .body(format!(
            "Ivily Desktop {} yüklenebilir. Ayrıntılar için Kontrol Merkezi'ni açın.",
            info.version
        ))
        .show();
}

#[derive(Clone, Serialize)]
struct Progress {
    downloaded: u64,
    total: Option<u64>,
}

/// Kurulum öncesi hazırlık: veritabanı yedeği ve çalışan programın kopyası.
fn prepare(app: &AppHandle, from_version: &str) -> AppResult<()> {
    let state = app.state::<AppState>();
    crate::widgets::manager::flush_rects(app);
    if let Some(store) = app.try_state::<crate::pets::store::PetStore>() {
        store.flush(&state.db)?;
    }
    crate::backup::create(&state.db, &state.data_dir, "-guncelleme-oncesi")?;
    let exe = std::env::current_exe()?;
    crate::recovery::preserve(&state.data_dir, &exe, from_version)?;
    Ok(())
}

/// Bekleyen güncellemeyi indirir (imza doğrulanır) ve kurar. Kurulum başlayınca uygulama kapanır;
/// yükleyici bittiğinde yeniden açılır.
pub async fn download_and_install(app: &AppHandle) -> AppResult<()> {
    let update = app
        .state::<UpdateState>()
        .pending
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .ok_or_else(|| {
            AppError::invalid("Bekleyen bir güncelleme yok. Önce güncellemeleri denetleyin.")
        })?;
    let mut downloaded: u64 = 0;
    let mut last_emit: u64 = 0;
    let a = app.clone();
    let bytes = update
        .download(
            move |chunk, total| {
                downloaded += chunk as u64;
                // İlerlemeyi seyrek gönder (her ~256 KB'de bir).
                if downloaded - last_emit >= 256 * 1024 || total.is_some_and(|t| downloaded >= t) {
                    last_emit = downloaded;
                    events::broadcast(&a, events::UPDATE_PROGRESS, Progress { downloaded, total });
                }
            },
            || {},
        )
        .await
        .map_err(|e| AppError::Other(format!("Güncelleme indirilemedi ya da imzası doğrulanamadı; hiçbir şey değiştirilmedi. ({e})")))?;

    // İmza doğrulandı; ancak şimdi yedek alınır ve çalışan sürüm saklanır (başarısız indirme iz bırakmaz).
    prepare(app, &update.current_version)?;

    // Kuruluma geçmeden önce bekleyen kurtarma bilgisini yaz (yeni sürüm ilk açılışta doğrular).
    let state = app.state::<AppState>();
    let _ = state.db.with(|c| {
        crate::db::settings::set(
            c,
            "update_in_progress",
            &serde_json::json!({ "from": update.current_version, "to": update.version }),
        )
    });
    events::broadcast(
        app,
        events::UPDATE_PROGRESS,
        serde_json::json!({ "installing": true }),
    );
    update.install(bytes).map_err(|e| {
        AppError::Other(format!(
            "Güncelleme kurulamadı; mevcut sürüm çalışmaya devam ediyor. ({e})"
        ))
    })
}

/// Yalnızca indirip imzayı doğrular (kurmaz). Test ve tanı amaçlıdır.
pub async fn verify_only(app: &AppHandle) -> AppResult<u64> {
    let update = app
        .state::<UpdateState>()
        .pending
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .ok_or_else(|| AppError::invalid("Bekleyen bir güncelleme yok."))?;
    let bytes = update
        .download(|_, _| {}, || {})
        .await
        .map_err(|e| AppError::Other(format!("İmza doğrulanamadı: {e}")))?;
    Ok(bytes.len() as u64)
}
