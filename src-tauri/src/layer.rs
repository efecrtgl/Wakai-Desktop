//! Masaüstü katmanı olaylarını Tauri pencereleriyle buluşturan yapıştırıcı katman.
//! Tüm pencere işlemleri ana iş parçacığında çalışır.

use crate::desktop::{DesktopEvent, DesktopLayer, Placement, Rect, monitors};
use crate::state::AppState;
use crate::widgets::window as widget_window;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use tauri::{AppHandle, Manager, WebviewWindow, WindowEvent};

pub struct LayerState {
    layer: Mutex<DesktopLayer>,
    /// Her pencereye en son uygulanan WebView ölçeği (gereksiz çağrıları önler).
    scales: Mutex<HashMap<isize, f64>>,
    /// Kullanıcının "widget ölçeği" ayarı; monitör ölçeğiyle çarpılır.
    widget_scale: Mutex<f64>,
}

impl LayerState {
    pub fn new(widget_scale: f64) -> Self {
        Self {
            layer: Mutex::new(DesktopLayer::new()),
            scales: Mutex::default(),
            widget_scale: Mutex::new(widget_scale),
        }
    }

    pub fn widget_scale(&self) -> f64 {
        *self.widget_scale.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn lock(&self) -> MutexGuard<'_, DesktopLayer> {
        self.layer.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// İzleyiciyi başlatır; olaylar ana iş parçacığına aktarılır.
pub fn start(app: &AppHandle) {
    let handle = app.clone();
    crate::desktop::start_watcher(move |event| {
        let app = handle.clone();
        let _ = handle.run_on_main_thread(move || on_desktop_event(&app, event));
    });
}

fn window_by_hwnd(app: &AppHandle, hwnd: isize) -> Option<WebviewWindow> {
    app.webview_windows().into_values().find(|w| {
        widget_window::id_from_label(w.label()).is_some() && widget_window::hwnd_of(w) == Some(hwnd)
    })
}

/// Pencerenin bulunduğu monitöre uygun WebView ölçeğini uygular (değiştiyse).
fn sync_scale(app: &AppHandle, window: &WebviewWindow, hwnd: isize) {
    let state = app.state::<LayerState>();
    let Some(rect) = state.lock().rect(hwnd) else {
        return;
    };
    let scale = monitors::scale_for(rect) * state.widget_scale();
    let mut scales = state.scales.lock().unwrap_or_else(|e| e.into_inner());
    if scales
        .get(&hwnd)
        .is_some_and(|s| (s - scale).abs() < f64::EPSILON)
    {
        return;
    }
    scales.insert(hwnd, scale);
    drop(scales);
    widget_window::set_raster_scale(window, scale);
}

fn sync_scale_by_hwnd(app: &AppHandle, hwnd: isize) {
    if let Some(w) = window_by_hwnd(app, hwnd) {
        sync_scale(app, &w, hwnd);
    }
}

/// Widget ölçeği ayarı değişti: tüm pencerelerin içerik ölçeğini yeniden uygular.
pub fn set_widget_scale(app: &AppHandle, scale: f64) {
    let state = app.state::<LayerState>();
    *state.widget_scale.lock().unwrap_or_else(|e| e.into_inner()) = scale;
    state
        .scales
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
    for w in app.webview_windows().into_values() {
        if let Some(hwnd) = widget_window::hwnd_of(&w).filter(|h| state.lock().contains(*h)) {
            sync_scale(app, &w, hwnd);
        }
    }
}

/// Widget penceresini masaüstü katmanına bağlar ve gösterir.
pub fn attach_window(
    app: &AppHandle,
    window: &WebviewWindow,
    rect: Rect,
    interactive: bool,
) -> Option<Placement> {
    let hwnd = widget_window::hwnd_of(window)?;
    let placement = app
        .state::<LayerState>()
        .lock()
        .attach(hwnd, rect, interactive);
    sync_scale(app, window, hwnd);
    Some(placement)
}

fn on_desktop_event(app: &AppHandle, event: DesktopEvent) {
    let state = app.state::<LayerState>();
    match event {
        DesktopEvent::HoverEnter(hwnd) => {
            let lifted = state.lock().lift(hwnd);
            if lifted {
                sync_scale_by_hwnd(app, hwnd);
            }
        }
        DesktopEvent::HoverLeave(hwnd) => {
            let sunk = {
                let mut l = state.lock();
                l.should_sink(hwnd) && l.sink(hwnd)
            };
            if sunk {
                sync_scale_by_hwnd(app, hwnd);
            }
        }
        DesktopEvent::ExplorerRestarted => {
            let dead = state.lock().reattach_all();
            log::info!("Explorer yeniden başladı; kaybolan pencere: {}", dead.len());
            crate::widgets::manager::on_windows_lost(app, dead);
        }
        DesktopEvent::DisplayChanged => {
            let changed = state.lock().on_display_changed();
            for w in app.webview_windows().into_values() {
                if let Some(hwnd) = widget_window::hwnd_of(&w).filter(|h| state.lock().contains(*h))
                {
                    sync_scale(app, &w, hwnd);
                }
            }
            crate::widgets::manager::on_rects_changed(app, changed);
        }
        DesktopEvent::SessionEnding => {
            log::info!("Windows oturumu kapanıyor");
            if let Some(state) = app.try_state::<AppState>() {
                let _ = state
                    .db
                    .with(|c| Ok(c.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?));
            }
        }
    }
}

/// Widget pencerelerinin olaylarını işler (odak kaybı, taşıma, boyutlandırma).
pub fn on_widget_window_event(window: &WebviewWindow, event: &WindowEvent) {
    let Some(hwnd) = widget_window::hwnd_of(window) else {
        return;
    };
    let app = window.app_handle();
    let state = app.state::<LayerState>();
    match event {
        WindowEvent::Focused(false) => {
            let sunk = {
                let mut l = state.lock();
                l.should_sink(hwnd) && l.sink(hwnd)
            };
            if sunk {
                sync_scale(app, window, hwnd);
            }
        }
        WindowEvent::Moved(_) | WindowEvent::Resized(_) => {
            let rect = state.lock().record_actual_rect(hwnd);
            if let Some(rect) = rect {
                sync_scale(app, window, hwnd);
                crate::widgets::manager::on_rects_changed(app, vec![(hwnd, rect)]);
            }
        }
        WindowEvent::Destroyed => {
            state.lock().detach(hwnd);
            state
                .scales
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&hwnd);
            if let Some(id) = widget_window::id_from_label(window.label()) {
                crate::widgets::manager::on_window_destroyed(app, id);
            }
        }
        _ => {}
    }
}
