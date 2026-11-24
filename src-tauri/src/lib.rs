//! Wakai Desktop — Rust çekirdeği.

mod backup;
mod commands;
pub mod db;
pub mod desktop;
pub mod error;
mod events;
mod focus;
mod http;
mod layer;
mod logging;
mod main_window;
mod media;
mod monitor;
mod procstats;
mod scheduler;
pub mod settings;
mod state;
mod subscriptions;
mod tray;
mod validate;
mod weather;
mod widgets;

use state::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            main_window::show(app);
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
            install_panic_hook();
            let db_path = dir.join("wakai.db");
            if let Err(e) = backup::apply_pending_restore(&dir, &db_path) {
                log::error!("yedek geri yüklenemedi: {e}");
            }
            let db = db::Db::open(&db_path)?;
            let initial = settings::load(&db).unwrap_or_default();
            app.manage(focus::FocusTimer::new(&db));
            app.manage(AppState {
                db,
                data_dir: dir.clone(),
            });
            app.manage(layer::LayerState::new(initial.widget_scale));
            app.manage(subscriptions::Subscriptions::default());
            app.manage(monitor::SystemMonitor::default());
            app.manage(media::MediaService::default());
            app.manage(commands::services::WeatherState(Box::new(
                weather::OpenMeteo,
            )));
            layer::start(app.handle());

            let scheduler = scheduler::Scheduler::default();
            scheduler.start(app.handle().clone());
            app.manage(scheduler);

            app.manage(widgets::manager::WidgetRuntime::default());
            if let Err(e) = widgets::manager::restore_all(app.handle()) {
                log::error!("widget'lar geri yüklenemedi: {e}");
            }

            tray::create(app.handle())?;

            // Windows ile başlatıldıysa yalnızca widget'lar gelir; Kontrol Merkezi açılmaz.
            let autostarted = std::env::args().any(|a| a == "--autostart");
            if !autostarted {
                main_window::show(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == main_window::LABEL
                && let tauri::WindowEvent::CloseRequested { api, .. } = event
            {
                match main_window::on_close_requested(window.app_handle()) {
                    // Pencereyi gizlemek yerine kapatmaya izin ver: WebView belleği serbest kalır.
                    main_window::CloseAction::CloseWindowOnly => {}
                    main_window::CloseAction::ExitApp => {
                        api.prevent_close();
                        window.app_handle().exit(0);
                    }
                }
            }
            if let tauri::WindowEvent::Destroyed = event {
                subscriptions::on_window_destroyed(window.app_handle(), window.label());
            }
            if widgets::window::id_from_label(window.label()).is_some()
                && let Some(w) = window.app_handle().get_webview_window(window.label())
            {
                layer::on_widget_window_event(&w, event);
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::app_info,
            commands::app::create_backup,
            commands::app::list_backups,
            commands::app::delete_backup,
            commands::app::restore_backup,
            commands::app::send_test_notification,
            commands::app::app_resource_usage,
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
            commands::content::list_event_categories,
            commands::content::list_events,
            commands::content::upcoming_events,
            commands::content::create_event,
            commands::content::update_event,
            commands::content::delete_event,
            commands::content::list_tasks,
            commands::content::create_task,
            commands::content::update_task,
            commands::content::set_task_completed,
            commands::content::delete_task,
            commands::content::clear_completed_tasks,
            commands::content::list_notes,
            commands::content::get_note,
            commands::content::create_note,
            commands::content::update_note,
            commands::content::delete_note,
            commands::focus::focus_state,
            commands::focus::focus_start,
            commands::focus::focus_pause,
            commands::focus::focus_reset,
            commands::focus::focus_skip,
            commands::services::subscribe,
            commands::services::unsubscribe,
            commands::services::system_stats,
            commands::services::media_state,
            commands::services::media_control,
            commands::services::weather_get,
            commands::services::weather_search,
        ])
        .build(tauri::generate_context!())
        .expect("Wakai Desktop başlatılamadı")
        .run(|app, event| match event {
            // Son pencere kapandığında (ör. Kontrol Merkezi kapatıldı ve widget'lar gizli)
            // uygulama tepside çalışmaya devam eder. Açık çıkış (`app.exit`) bir kod taşır.
            tauri::RunEvent::ExitRequested {
                code: None, api, ..
            } => api.prevent_exit(),
            tauri::RunEvent::Exit => shutdown(app),
            _ => {}
        });
}

/// Düzenli kapanış: bekleyen konumları yaz, WAL'ı veritabanına aktar.
fn shutdown(app: &tauri::AppHandle) {
    widgets::manager::flush_rects(app);
    if let Some(state) = app.try_state::<AppState>() {
        let _ = state
            .db
            .with(|c| Ok(c.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?));
    }
    log::info!("Wakai Desktop kapandı");
}

/// Panikler günlük dosyasına yazılır; hata ayıklama için iz bırakır.
fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("beklenmeyen hata: {info}");
        default(info);
    }));
}
