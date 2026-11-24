//! Sistem tepsisi simgesi ve menüsü (profil hızlı geçişi dahil).

use crate::events;
use crate::state::AppState;
use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

const TRAY_ID: &str = "wakai";
const ID_OPEN: &str = "open";
const ID_WIDGETS: &str = "widgets";
const ID_PETS: &str = "pets";
const ID_UNLOCK: &str = "unlock";
const ID_QUIT: &str = "quit";
const PROFILE_PREFIX: &str = "profile:";

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let state = app.try_state::<AppState>();
    let s = state
        .as_ref()
        .and_then(|st| crate::settings::load(&st.db).ok())
        .unwrap_or_default();
    let profiles = state
        .as_ref()
        .and_then(|st| crate::profiles::list(&st.db).ok())
        .unwrap_or_default();

    let profile_items: Vec<CheckMenuItem<Wry>> = profiles
        .iter()
        .map(|p| {
            CheckMenuItem::with_id(
                app,
                format!("{PROFILE_PREFIX}{}", p.id),
                &p.name,
                true,
                s.active_profile == Some(p.id),
                None::<&str>,
            )
        })
        .collect::<tauri::Result<_>>()?;
    let refs: Vec<&dyn IsMenuItem<Wry>> = profile_items
        .iter()
        .map(|i| i as &dyn IsMenuItem<Wry>)
        .collect();
    let profile_menu = Submenu::with_items(app, "Profil", true, &refs)?;

    Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, ID_OPEN, "Kontrol Merkezi'ni aç", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &profile_menu,
            &CheckMenuItem::with_id(
                app,
                ID_WIDGETS,
                "Widget'ları göster",
                true,
                s.widgets_visible,
                None::<&str>,
            )?,
            &CheckMenuItem::with_id(
                app,
                ID_PETS,
                "Masaüstü kedileri",
                true,
                s.pets_enabled,
                None::<&str>,
            )?,
            &MenuItem::with_id(
                app,
                ID_UNLOCK,
                "Tüm widget'ları etkileşimli yap",
                true,
                None::<&str>,
            )?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, ID_QUIT, "Çıkış", true, None::<&str>)?,
        ],
    )
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Wakai Desktop")
        .menu(&build_menu(app)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            match id {
                ID_OPEN => crate::main_window::show(app),
                ID_WIDGETS => toggle(app, "widgets_visible"),
                ID_PETS => toggle(app, "pets_enabled"),
                ID_UNLOCK => unlock_all(app),
                ID_QUIT => app.exit(0),
                _ => {
                    if let Some(pid) = id
                        .strip_prefix(PROFILE_PREFIX)
                        .and_then(|v| v.parse::<i64>().ok())
                        && let Err(e) = crate::profiles::apply(app, pid)
                    {
                        log::error!("profil uygulanamadı: {e}");
                    }
                }
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::main_window::show(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

/// Ayarlar ya da profiller değişince menüyü (onay işaretleri dahil) yeniden kurar.
pub fn rebuild(app: &AppHandle) {
    if let (Some(tray), Ok(menu)) = (app.tray_by_id(TRAY_ID), build_menu(app)) {
        let _ = tray.set_menu(Some(menu));
    }
}

/// Ayar değişiminde menüyü eşitler.
pub fn sync(app: &AppHandle, _widgets_visible: bool) {
    rebuild(app);
}

fn toggle(app: &AppHandle, key: &str) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let s = serde_json::to_value(crate::settings::load(&state.db).unwrap_or_default())
        .unwrap_or_default();
    let current = s.get(key).and_then(|v| v.as_bool()).unwrap_or(true);
    if let Err(e) =
        crate::commands::settings::apply_settings_patch(app, serde_json::json!({ key: !current }))
    {
        log::error!("ayar değiştirilemedi: {e}");
    }
    rebuild(app);
}

/// Güvenlik çıkışı: tıklama geçirgen yapılmış tüm widget'ları yeniden etkileşimli yapar.
fn unlock_all(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let Ok(list) = state.db.with(crate::db::widgets::list) else {
        return;
    };
    for w in list.into_iter().filter(|w| w.click_through) {
        if let Err(e) = crate::widgets::manager::set_click_through(app, &w.id, false) {
            log::error!("widget kilidi açılamadı: {e}");
        }
    }
    events::broadcast(app, events::WIDGETS_CHANGED, ());
}
