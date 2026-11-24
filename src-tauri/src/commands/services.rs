//! Sistem izleme, medya ve hava durumu komutları.

use crate::error::{AppError, AppResult};
use crate::media::{Action, MediaService, MediaState};
use crate::monitor::{SystemMonitor, SystemStats};
use crate::state::AppState;
use crate::subscriptions::{self, Subscriptions, Topic};
use crate::weather::{self, Place, WeatherProvider, WeatherResult};
use tauri::{AppHandle, Manager, State, Window};

pub struct WeatherState(pub Box<dyn WeatherProvider>);

fn topic(name: &str) -> AppResult<Topic> {
    Topic::parse(name).ok_or_else(|| AppError::invalid("Bilinmeyen abonelik konusu."))
}

/// Pencere bir arka plan hizmetine abone olur (pencere kapanınca abonelik otomatik silinir).
#[tauri::command]
pub fn subscribe(
    app: AppHandle,
    window: Window,
    subs: State<'_, Subscriptions>,
    name: String,
) -> AppResult<()> {
    let t = topic(&name)?;
    if subs.add(t, window.label()) {
        subscriptions::set_active(&app, t, true);
    }
    Ok(())
}

#[tauri::command]
pub fn unsubscribe(
    app: AppHandle,
    window: Window,
    subs: State<'_, Subscriptions>,
    name: String,
) -> AppResult<()> {
    let t = topic(&name)?;
    if subs.remove(t, window.label()) {
        subscriptions::set_active(&app, t, false);
    }
    Ok(())
}

#[tauri::command]
pub fn system_stats(monitor: State<'_, SystemMonitor>) -> Option<SystemStats> {
    monitor.last()
}

#[tauri::command]
pub fn media_state(media: State<'_, MediaService>) -> MediaState {
    media.last()
}

/// Oynatma komutu. Yanıt birkaç saniye sürebileceği için ana iş parçacığında çalışmaz.
#[tauri::command]
pub async fn media_control(app: AppHandle, action: String) -> AppResult<bool> {
    let action = match action.as_str() {
        "play_pause" => Action::PlayPause,
        "next" => Action::Next,
        "previous" => Action::Previous,
        _ => return Err(AppError::invalid("Bilinmeyen medya komutu.")),
    };
    let media = app.state::<MediaService>().inner().clone();
    tauri::async_runtime::spawn_blocking(move || media.act(action))
        .await
        .map_err(|_| AppError::Other("Medya komutu çalıştırılamadı.".into()))
}

#[tauri::command]
pub async fn weather_get(
    app: AppHandle,
    latitude: f64,
    longitude: f64,
    force: Option<bool>,
) -> AppResult<WeatherResult> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let provider = app.state::<WeatherState>();
        let max_age = crate::settings::load(&state.db)
            .map(|s| s.weather_refresh_minutes)
            .unwrap_or(30);
        weather::get(
            &state.db,
            provider.0.as_ref(),
            latitude,
            longitude,
            max_age,
            force.unwrap_or(false),
        )
    })
    .await
    .map_err(|_| AppError::Other("Hava durumu alınamadı.".into()))?
}

#[tauri::command]
pub async fn weather_search(app: AppHandle, query: String) -> AppResult<Vec<Place>> {
    let query = crate::validate::text("Konum", &query, 80, true)?;
    tauri::async_runtime::spawn_blocking(move || app.state::<WeatherState>().0.search(&query))
        .await
        .map_err(|_| AppError::Other("Konum aranamadı.".into()))?
}
