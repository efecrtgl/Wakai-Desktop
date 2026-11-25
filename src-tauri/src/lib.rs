//! Ivily Desktop (eski adıyla Wakai Desktop) — Rust çekirdeği.

pub mod backup;
mod commands;
pub mod db;
pub mod desktop;
pub mod error;
mod events;
pub mod focus;
mod http;
mod layer;
mod legacy;
mod logging;
mod main_window;
mod media;
mod monitor;
pub mod pets;
mod procstats;
mod profiles;
mod recovery;
mod scheduler;
pub mod settings;
mod state;
mod subscriptions;
pub mod themes;
mod tray;
mod updater;
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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // WAKAI_DATA_DIR: yalnızca yalıtılmış test ortamı içindir (birincil verilere dokunmadan).
            let dir = match std::env::var_os("WAKAI_DATA_DIR") {
                Some(d) => std::path::PathBuf::from(d),
                None => app.path().app_data_dir()?,
            };
            std::fs::create_dir_all(&dir)?;
            logging::init(&dir);
            install_panic_hook();
            let version = app.package_info().version.to_string();
            launch_safety(app.handle(), &dir, &version);
            let db_path = dir.join("wakai.db");
            if let Err(e) = backup::apply_pending_restore(&dir, &db_path) {
                log::error!("yedek geri yüklenemedi: {e}");
            }
            let (db, quarantined) = db::Db::open_or_recover(&db_path)?;
            match backup::apply_pending_selective(&db, &dir) {
                Ok(c) if !c.is_empty() => log::info!("seçmeli geri yükleme: {c:?}"),
                Ok(_) => {}
                Err(e) => log::error!("seçmeli geri yükleme başarısız: {e}"),
            }
            let initial = settings::load(&db).unwrap_or_default();
            record_version(&db, &version);
            app.manage(focus::FocusTimer::new(&db));
            let pet_store = pets::store::PetStore::default();
            if let Err(e) = pet_store.load(&db) {
                log::error!("kediler yüklenemedi: {e}");
            }
            app.manage(pet_store);
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

            // Kedi motoru widget motorundan bağımsız kendi iş parçacığında çalışır.
            let engine = pets::engine::PetEngine::default();
            engine.start(app.handle().clone(), pets::engine_settings(&initial));
            app.manage(engine);
            app.manage(updater::UpdateState::default());
            updater::spawn_startup_check(app.handle());
            app.manage(profiles::AutoGaming::default());
            profiles::sync_auto_gaming(app.handle(), initial.auto_gaming_profile);
            if let Some(q) = quarantined {
                notify_recovered(app.handle(), &q);
            }
            if let Some(from) = recovery::take_rollback_notice(&dir) {
                notify_rolled_back(app.handle(), &from, &version);
            }
            // 15 sn sorunsuz çalışan sürüm "sağlıklı" sayılır; bir daha otomatik geri alınmaz.
            {
                let dir = dir.clone();
                let version = version.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(15));
                    recovery::launch_healthy(&dir, &version);
                });
            }

            // Wakai → Ivily geçişinden kalanlar (eski otomatik başlatma değeri, kilitli kalmış eski
            // program dosyası). Otomatik başlatma açıksa yeni adla yeniden kaydedilir.
            {
                let handle = app.handle().clone();
                let autostart = initial.launch_on_startup;
                std::thread::spawn(move || {
                    legacy::cleanup();
                    if autostart
                        && !cfg!(debug_assertions)
                        && let Err(e) = commands::settings::apply_autostart(&handle, true)
                    {
                        log::warn!("otomatik başlatma yenilenemedi: {e}");
                    }
                });
            }

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
            commands::app::inspect_backup,
            commands::app::restore_backup_selective,
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
            commands::shopping::list_shopping_lists,
            commands::shopping::create_shopping_list,
            commands::shopping::rename_shopping_list,
            commands::shopping::delete_shopping_list,
            commands::shopping::list_shopping_items,
            commands::shopping::add_shopping_item,
            commands::shopping::update_shopping_item,
            commands::shopping::set_shopping_item_checked,
            commands::shopping::delete_shopping_item,
            commands::shopping::clear_checked_shopping_items,
            commands::focus::focus_state,
            commands::focus::focus_start,
            commands::focus::focus_pause,
            commands::focus::focus_reset,
            commands::focus::focus_skip,
            commands::pets::list_pets,
            commands::pets::pet_catalog,
            commands::pets::pet_progress,
            commands::pets::adopt_pet,
            commands::pets::rename_pet,
            commands::pets::release_pet,
            commands::pets::set_pet_location,
            commands::pets::set_pet_paused,
            commands::pets::set_pet_accessory,
            commands::pets::feed_pet,
            commands::pets::stroke_pet,
            commands::pets::play_with_pet,
            commands::pets::summon_pet,
            commands::profiles::list_profiles,
            commands::profiles::apply_profile,
            commands::profiles::create_profile,
            commands::profiles::update_profile,
            commands::profiles::delete_profile,
            commands::profiles::reset_profile,
            commands::pets::list_home_items,
            commands::pets::add_home_item,
            commands::pets::move_home_item,
            commands::pets::delete_home_item,
            commands::pets::get_home_style,
            commands::pets::set_home_style,
            commands::updates::check_for_updates,
            commands::updates::update_status,
            commands::updates::install_update,
            commands::updates::verify_update,
            commands::updates::version_history,
            commands::updates::rollback_to_version,
            commands::updates::brand_notice,
            commands::themes::list_custom_themes,
            commands::themes::import_theme,
            commands::themes::delete_custom_theme,
            commands::themes::export_theme,
            commands::updates::dismiss_brand_notice,
            commands::services::subscribe,
            commands::services::unsubscribe,
            commands::services::system_stats,
            commands::services::media_state,
            commands::services::media_control,
            commands::services::weather_get,
            commands::services::weather_search,
        ])
        .build(tauri::generate_context!())
        .expect("Ivily Desktop başlatılamadı")
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
    if let Some(engine) = app.try_state::<pets::engine::PetEngine>() {
        engine.send(pets::engine::Cmd::Shutdown);
    }
    if let (Some(store), Some(state)) = (
        app.try_state::<pets::store::PetStore>(),
        app.try_state::<AppState>(),
    ) {
        let _ = store.flush(&state.db);
    }
    if let Some(state) = app.try_state::<AppState>() {
        let _ = state
            .db
            .with(|c| Ok(c.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?));
    }
    log::info!("Ivily Desktop kapandı");
}

/// Açılış güvenliği: kalan `.old` dosyasını temizler; hiç sağlıklı açılmamış bu sürüm art arda
/// başarısız olduysa saklanan önceki sürüme döner ve yeniden başlar.
fn launch_safety(app: &tauri::AppHandle, dir: &std::path::Path, version: &str) {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    recovery::cleanup_old(&exe);
    let failed = recovery::begin_launch(dir, version);
    let Some(target) = recovery::auto_rollback_target(dir, version, failed) else {
        return;
    };
    log::error!("{version} art arda {failed} kez sağlıklı açılamadı; {target} sürümüne dönülüyor");
    match recovery::rollback(dir, &exe, &target) {
        Ok(()) => {
            recovery::mark_rolled_back(dir, version);
            app.restart();
        }
        Err(e) => log::error!("otomatik geri dönüş başarısız: {e}"),
    }
}

/// Sürüm geçmişine bu açılışı yazar; sürüm değiştiyse bekleyen güncelleme kaydını temizler.
fn record_version(db: &db::Db, version: &str) {
    let now = chrono::Local::now().timestamp_millis();
    match db.with(|c| recovery::record_launch(c, version, now)) {
        Ok(Some(previous)) => {
            log::info!("sürüm değişti: {previous} → {version}");
            let _ = db.with(|c| db::settings::delete(c, "update_in_progress"));
        }
        Ok(None) => {}
        Err(e) => log::warn!("sürüm geçmişi yazılamadı: {e}"),
    }
}

fn notify_rolled_back(app: &tauri::AppHandle, from: &str, to: &str) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app
        .notification()
        .builder()
        .title("Ivily Desktop önceki sürüme döndü")
        .body(format!(
            "{from} sürümü düzgün açılamadığı için {to} sürümüne dönüldü. Verileriniz korunuyor."
        ))
        .show();
}

/// Bozuk veritabanı karantinaya alındıysa kullanıcıyı bilgilendirir.
fn notify_recovered(app: &tauri::AppHandle, quarantine: &std::path::Path) {
    use tauri_plugin_notification::NotificationExt;
    let name = quarantine
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let _ = app
        .notification()
        .builder()
        .title("Ivily Desktop verileri okunamadı")
        .body(format!(
            "Bozuk veritabanı {name} olarak saklandı ve yeni bir başlangıç yapıldı. Ayarlar → Yedekler bölümünden bir yedeği geri yükleyebilirsiniz."
        ))
        .show();
}

/// Panikler günlük dosyasına yazılır; hata ayıklama için iz bırakır.
fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("beklenmeyen hata: {info}");
        default(info);
    }));
}
