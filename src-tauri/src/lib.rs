//! Wakai Desktop — Rust çekirdeği.

mod commands;
pub mod db;
pub mod desktop;
pub mod error;
mod events;
mod layer;
mod logging;
pub mod settings;
mod state;
mod widgets;
mod widgets_runtime_hook;

use state::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            logging::init(&dir);
            let db = db::Db::open(&dir.join("wakai.db"))?;
            app.manage(AppState { db });
            app.manage(layer::LayerState::new());
            layer::start(app.handle());

            // Aşama 2 doğrulama penceresi (Aşama 3'te widget yöneticisiyle değişecek).
            if let Ok(spec) = std::env::var("WAKAI_LAYER_PROBE") {
                let n: Vec<i32> = spec
                    .split(',')
                    .filter_map(|v| v.trim().parse().ok())
                    .collect();
                let rect = if n.len() == 4 {
                    desktop::Rect::new(n[0], n[1], n[2], n[3])
                } else {
                    desktop::Rect::new(120, 120, 340, 200)
                };
                let w = widgets::window::create(app.handle(), "probe", rect)?;
                let p = layer::attach_window(app.handle(), &w, rect, true);
                eprintln!("probe placement: {p:?}");
            }

            let autostarted = std::env::args().any(|a| a == "--autostart");
            if !autostarted {
                show_main_window(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if widgets::window::id_from_label(window.label()).is_some()
                && let Some(w) = window.app_handle().get_webview_window(window.label())
            {
                layer::on_widget_window_event(&w, event);
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::app_info,
            commands::settings::get_settings,
            commands::settings::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("Wakai Desktop başlatılamadı");
}

pub(crate) fn show_main_window(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}
