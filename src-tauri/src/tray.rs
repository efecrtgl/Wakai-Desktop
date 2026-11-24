//! Sistem tepsisi simgesi ve menüsü.

use crate::events;
use crate::state::AppState;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

const ID_OPEN: &str = "open";
const ID_WIDGETS: &str = "widgets";
const ID_UNLOCK: &str = "unlock";
const ID_QUIT: &str = "quit";

/// Menüdeki "Widget'ları göster" onay kutusu; ayar başka yerden değişince güncellenir.
pub struct TrayState(pub CheckMenuItem<Wry>);

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let visible = app
        .try_state::<AppState>()
        .and_then(|s| crate::settings::load(&s.db).ok())
        .map(|s| s.widgets_visible)
        .unwrap_or(true);

    let open = MenuItem::with_id(app, ID_OPEN, "Kontrol Merkezi'ni aç", true, None::<&str>)?;
    let widgets = CheckMenuItem::with_id(
        app,
        ID_WIDGETS,
        "Widget'ları göster",
        true,
        visible,
        None::<&str>,
    )?;
    let unlock = MenuItem::with_id(
        app,
        ID_UNLOCK,
        "Tüm widget'ları etkileşimli yap",
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, ID_QUIT, "Çıkış", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &open,
            &PredefinedMenuItem::separator(app)?,
            &widgets,
            &unlock,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id("wakai")
        .tooltip("Wakai Desktop")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            ID_OPEN => crate::main_window::show(app),
            ID_WIDGETS => toggle_widgets(app),
            ID_UNLOCK => unlock_all(app),
            ID_QUIT => app.exit(0),
            _ => {}
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
    app.manage(TrayState(widgets));
    Ok(())
}

/// Ayar değiştiğinde menüdeki onay işaretini eşitler.
pub fn sync(app: &AppHandle, widgets_visible: bool) {
    if let Some(t) = app.try_state::<TrayState>() {
        let _ = t.0.set_checked(widgets_visible);
    }
}

fn toggle_widgets(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let visible = crate::settings::load(&state.db)
        .map(|s| s.widgets_visible)
        .unwrap_or(true);
    match crate::commands::settings::apply_settings_patch(
        app,
        serde_json::json!({ "widgets_visible": !visible }),
    ) {
        Ok(s) => sync(app, s.widgets_visible),
        Err(e) => log::error!("widget görünürlüğü değiştirilemedi: {e}"),
    }
}

/// Güvenlik çıkışı: tıklama geçirgen yapılmış tüm widget'ları yeniden etkileşimli yapar.
/// Böylece hiçbir widget kalıcı olarak erişilemez hale gelmez.
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
