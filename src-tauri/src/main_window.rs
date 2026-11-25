//! Kontrol Merkezi penceresi. Bellek tasarrufu için tepsiye küçültülünce pencere GİZLENMEZ,
//! tamamen KAPATILIR (WebView süreci serbest kalır) ve gerektiğinde yeniden oluşturulur.

use crate::error::AppResult;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub const LABEL: &str = "main";

fn build(app: &AppHandle) -> AppResult<WebviewWindow> {
    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
        .title("Ivily Desktop")
        .inner_size(1100.0, 720.0)
        .min_inner_size(860.0, 560.0)
        .center()
        .visible(false)
        .additional_browser_args(crate::widgets::window::BROWSER_ARGS)
        .build()?;
    Ok(window)
}

/// Kontrol Merkezi'ni gösterir; kapalıysa yeniden oluşturur.
pub fn show(app: &AppHandle) {
    let window = match app.get_webview_window(LABEL) {
        Some(w) => w,
        None => match build(app) {
            Ok(w) => w,
            Err(e) => {
                log::error!("Kontrol Merkezi açılamadı: {e}");
                return;
            }
        },
    };
    apply_theme(app, &window);
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

/// Ayara göre Kontrol Merkezi kapatıldığında ne olacağı.
/// `close_to_tray` açıksa yalnızca pencere kapanır, uygulama tepside çalışmaya devam eder.
pub fn on_close_requested(app: &AppHandle) -> CloseAction {
    let to_tray = app
        .try_state::<crate::state::AppState>()
        .and_then(|s| crate::settings::load(&s.db).ok())
        .map(|s| s.close_to_tray)
        .unwrap_or(true);
    if to_tray {
        CloseAction::CloseWindowOnly
    } else {
        CloseAction::ExitApp
    }
}

pub enum CloseAction {
    CloseWindowOnly,
    ExitApp,
}

/// Pencere çerçevesini (başlık çubuğu) seçili görünüme uydurur.
pub fn apply_theme(app: &AppHandle, window: &WebviewWindow) {
    let dark = app
        .try_state::<crate::state::AppState>()
        .is_some_and(|state| {
            let Ok(s) = crate::settings::load(&state.db) else {
                return false;
            };
            state
                .db
                .with(|c| Ok(crate::themes::is_dark(c, &s.theme, &s.theme_variant)))
                .unwrap_or(false)
        });
    let _ = window.set_theme(Some(if dark {
        tauri::Theme::Dark
    } else {
        tauri::Theme::Light
    }));
}
