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

            app.manage(widgets::manager::WidgetRuntime::default());
            if let Err(e) = widgets::manager::restore_all(app.handle()) {
                log::error!("widget'lar geri yüklenemedi: {e}");
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
            commands::widgets::list_widget_definitions,
            commands::widgets::list_widgets,
            commands::widgets::get_widget,
            commands::widgets::add_widget,
            commands::widgets::remove_widget,
            commands::widgets::set_widget_visible,
            commands::widgets::set_widget_click_through,
            commands::widgets::update_widget_config,
            commands::widgets::set_widget_rect,
            commands::widgets::reset_widget_position,
            commands::widgets::reset_widget,
            commands::widgets::open_control_center,
        ])
        .build(tauri::generate_context!())
        .expect("Wakai Desktop başlatılamadı")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                widgets::manager::flush_rects(app);
            }
        });
}

pub(crate) fn show_main_window(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}
