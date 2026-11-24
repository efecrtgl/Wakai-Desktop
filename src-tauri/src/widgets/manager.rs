//! Widget yaşam döngüsü: oluştur → başlat → göster/gizle → yapılandır → yok et,
//! ve kayıtlı yapılandırmadan geri yükleme.
//!
//! Gizlenen widget'ın penceresi KAPATILIR (yalnızca görünmez yapılmaz); böylece
//! WebView süreci ve belleği serbest kalır. Gösterildiğinde yeniden oluşturulur.

use super::registry::{self, WidgetDefinition};
use super::window as widget_window;
use crate::db::widgets::{self as repo, WidgetInstance};
use crate::desktop::{Rect, monitors};
use crate::error::AppResult;
use crate::events;
use crate::layer;
use crate::state::AppState;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;
use tauri::{AppHandle, Manager};

/// Sürükleme sırasında her harekette veritabanına yazmamak için bekleme süresi.
const RECT_FLUSH_DELAY: Duration = Duration::from_millis(600);
/// Beklenmedik şekilde yok edilen pencerelerin (ör. Explorer çöktü) yeniden deneme gecikmesi.
const RESPAWN_DELAY: Duration = Duration::from_secs(2);

#[derive(Default)]
pub struct WidgetRuntime {
    /// Açık pencereler: örnek kimliği → pencere tanıtıcısı.
    windows: Mutex<HashMap<String, isize>>,
    /// Bilerek kapatılan pencereler (yok edilince yeniden oluşturulmaz).
    closing: Mutex<HashSet<String>>,
    pending_rects: Mutex<HashMap<String, Rect>>,
    flush_scheduled: AtomicBool,
    respawn_scheduled: AtomicBool,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn new_id(kind: &str) -> String {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let millis = chrono::Local::now().timestamp_millis();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{kind}-{millis:x}{n:x}")
}

fn runtime(app: &AppHandle) -> tauri::State<'_, WidgetRuntime> {
    app.state::<WidgetRuntime>()
}

fn db(app: &AppHandle) -> &crate::db::Db {
    &app.state::<AppState>().inner().db
}

fn notify_changed(app: &AppHandle, instance: Option<&WidgetInstance>) {
    events::broadcast(app, events::WIDGETS_CHANGED, ());
    if let Some(inst) = instance {
        events::broadcast(app, events::WIDGET_UPDATED, inst.clone());
    }
}

pub fn id_for_hwnd(app: &AppHandle, hwnd: isize) -> Option<String> {
    lock(&runtime(app).windows)
        .iter()
        .find(|(_, h)| **h == hwnd)
        .map(|(id, _)| id.clone())
}

fn hwnd_for_id(app: &AppHandle, id: &str) -> Option<isize> {
    lock(&runtime(app).windows).get(id).copied()
}

/// Yeni bir widget için varsayılan konum: birincil monitörün sağ üstünden kademeli.
fn default_rect(app: &AppHandle, def: &WidgetDefinition, index: usize) -> Rect {
    let mons = monitors::all();
    let area = mons
        .iter()
        .find(|m| m.primary)
        .or(mons.first())
        .map(|m| m.work_area)
        .unwrap_or(Rect::new(0, 0, 1920, 1040));
    let scale = monitors::scale_for(area) * app.state::<layer::LayerState>().widget_scale();
    let w = (def.default_size.width as f64 * scale).round() as i32;
    let h = (def.default_size.height as f64 * scale).round() as i32;
    let step = (28.0 * scale) as i32 * (index % 8) as i32;
    let margin = (40.0 * scale) as i32;
    Rect::new(
        area.x + area.width - w - margin - step,
        area.y + margin + step,
        w,
        h,
    )
    .clamped_into(area)
}

/// Pencereyi oluşturur ve masaüstü katmanına bağlar.
fn spawn(app: &AppHandle, inst: &WidgetInstance) -> AppResult<()> {
    if hwnd_for_id(app, &inst.id).is_some() {
        return Ok(());
    }
    let window = widget_window::create(app, &inst.id, inst.rect)?;
    let Some(hwnd) = widget_window::hwnd_of(&window) else {
        let _ = window.destroy();
        return Err(crate::error::AppError::Other(
            "Widget penceresi oluşturulamadı.".into(),
        ));
    };
    lock(&runtime(app).windows).insert(inst.id.clone(), hwnd);
    lock(&runtime(app).closing).remove(&inst.id);
    layer::attach_window(app, &window, inst.rect, !inst.click_through);
    Ok(())
}

fn close_window(app: &AppHandle, id: &str) {
    let rt = runtime(app);
    if lock(&rt.windows).remove(id).is_none() {
        return;
    }
    lock(&rt.closing).insert(id.to_string());
    if let Some(w) = app.get_webview_window(&widget_window::label_for(id)) {
        let _ = w.destroy();
    }
}

/// Uygulama açılışında görünür widget'ları geri yükler.
pub fn restore_all(app: &AppHandle) -> AppResult<()> {
    let settings = crate::settings::load(db(app))?;
    if !settings.widgets_visible {
        return Ok(());
    }
    let list = db(app).with(repo::list)?;
    for inst in list.iter().filter(|i| i.visible) {
        if registry::find(&inst.kind).is_none() {
            log::warn!("bilinmeyen widget türü atlandı: {}", inst.kind);
            continue;
        }
        if let Err(e) = spawn(app, inst) {
            log::error!("widget geri yüklenemedi ({}): {e}", inst.id);
        }
    }
    Ok(())
}

/// "Masaüstü widget'larını göster" ayarı değiştiğinde çağrılır.
pub fn apply_global_visibility(app: &AppHandle, visible: bool) -> AppResult<()> {
    if visible {
        restore_all(app)
    } else {
        let ids: Vec<String> = lock(&runtime(app).windows).keys().cloned().collect();
        for id in ids {
            close_window(app, &id);
        }
        Ok(())
    }
}

pub fn add(app: &AppHandle, kind: &str) -> AppResult<WidgetInstance> {
    let def = registry::require(kind)?;
    let existing = db(app).with(repo::list)?;
    if !def.multi_instance && existing.iter().any(|w| w.kind == kind) {
        return Err(crate::error::AppError::invalid(format!(
            "{} zaten ekli.",
            def.name
        )));
    }
    let inst = WidgetInstance {
        id: new_id(kind),
        kind: kind.to_string(),
        rect: default_rect(app, def, existing.len()),
        visible: true,
        click_through: false,
        config: def.default_config(),
    };
    db(app).with(|c| repo::insert(c, &inst))?;
    if crate::settings::load(db(app))?.widgets_visible {
        spawn(app, &inst)?;
    }
    notify_changed(app, Some(&inst));
    Ok(inst)
}

pub fn remove(app: &AppHandle, id: &str) -> AppResult<()> {
    close_window(app, id);
    db(app).with(|c| repo::delete(c, id))?;
    lock(&runtime(app).pending_rects).remove(id);
    notify_changed(app, None);
    Ok(())
}

pub fn set_visible(app: &AppHandle, id: &str, visible: bool) -> AppResult<WidgetInstance> {
    db(app).with(|c| repo::update_visible(c, id, visible))?;
    let inst = db(app).with(|c| repo::get(c, id))?;
    if visible && crate::settings::load(db(app))?.widgets_visible {
        spawn(app, &inst)?;
    } else if !visible {
        close_window(app, id);
    }
    notify_changed(app, Some(&inst));
    Ok(inst)
}

pub fn set_click_through(app: &AppHandle, id: &str, value: bool) -> AppResult<WidgetInstance> {
    db(app).with(|c| repo::update_click_through(c, id, value))?;
    if let Some(hwnd) = hwnd_for_id(app, id) {
        let state = app.state::<layer::LayerState>();
        let mut l = state.lock();
        l.set_interactive(hwnd, !value);
        if value {
            l.sink(hwnd);
        }
    }
    let inst = db(app).with(|c| repo::get(c, id))?;
    notify_changed(app, Some(&inst));
    Ok(inst)
}

pub fn update_config(
    app: &AppHandle,
    id: &str,
    patch: &serde_json::Value,
) -> AppResult<WidgetInstance> {
    let inst = db(app).with_tx(|tx| {
        let inst = repo::get(tx, id)?;
        let def = registry::require(&inst.kind)?;
        let next = def.merge_config(&inst.config, patch)?;
        repo::update_config(tx, id, &next)?;
        repo::get(tx, id)
    })?;
    notify_changed(app, Some(&inst));
    Ok(inst)
}

/// Konum/boyutu Kontrol Merkezi'nden değiştirir (en küçük boyut ve görünür alan korunur).
pub fn set_rect(app: &AppHandle, id: &str, rect: Rect) -> AppResult<WidgetInstance> {
    let inst = db(app).with(|c| repo::get(c, id))?;
    let def = registry::require(&inst.kind)?;
    let scale = monitors::scale_for(rect) * app.state::<layer::LayerState>().widget_scale();
    let min_w = (def.min_size.width as f64 * scale).round() as i32;
    let min_h = (def.min_size.height as f64 * scale).round() as i32;
    let rect = monitors::clamp_visible(Rect::new(
        rect.x,
        rect.y,
        rect.width.max(min_w),
        rect.height.max(min_h),
    ));
    db(app).with(|c| repo::update_rect(c, id, rect))?;
    if let Some(hwnd) = hwnd_for_id(app, id) {
        app.state::<layer::LayerState>().lock().set_rect(hwnd, rect);
    }
    let inst = db(app).with(|c| repo::get(c, id))?;
    notify_changed(app, Some(&inst));
    Ok(inst)
}

pub fn reset_position(app: &AppHandle, id: &str) -> AppResult<WidgetInstance> {
    let inst = db(app).with(|c| repo::get(c, id))?;
    let def = registry::require(&inst.kind)?;
    let index = db(app)
        .with(repo::list)?
        .iter()
        .position(|w| w.id == id)
        .unwrap_or(0);
    set_rect(app, id, default_rect(app, def, index))
}

/// Widget'ı varsayılan ayarlarına ve konumuna döndürür. İçerik verisine dokunmaz.
pub fn reset(app: &AppHandle, id: &str) -> AppResult<WidgetInstance> {
    let inst = db(app).with(|c| repo::get(c, id))?;
    let def = registry::require(&inst.kind)?;
    db(app).with(|c| repo::update_config(c, id, &def.default_config()))?;
    if inst.click_through {
        set_click_through(app, id, false)?;
    }
    reset_position(app, id)
}

/// Widget ölçeği ayarı değiştiğinde tüm widget'ları orantılı büyütür/küçültür.
pub fn apply_widget_scale(app: &AppHandle, old: f64, new: f64) -> AppResult<()> {
    if old <= 0.0 || (old - new).abs() < f64::EPSILON {
        return Ok(());
    }
    layer::set_widget_scale(app, new);
    let factor = new / old;
    for inst in db(app).with(repo::list)? {
        let r = inst.rect;
        let scaled = Rect::new(
            r.x,
            r.y,
            (r.width as f64 * factor).round() as i32,
            (r.height as f64 * factor).round() as i32,
        );
        set_rect(app, &inst.id, scaled)?;
    }
    Ok(())
}

/// Katman, kullanıcının taşıdığı/boyutlandırdığı pencerelerin yeni konumunu bildirir.
/// Yazma işlemi kısa bir gecikmeyle tek bir işlemde toplanır.
pub fn on_rects_changed(app: &AppHandle, changed: Vec<(isize, Rect)>) {
    if changed.is_empty() {
        return;
    }
    let rt = runtime(app);
    {
        let mut pending = lock(&rt.pending_rects);
        for (hwnd, rect) in changed {
            if let Some(id) = id_for_hwnd(app, hwnd) {
                pending.insert(id, rect);
            }
        }
    }
    if rt.flush_scheduled.swap(true, Ordering::AcqRel) {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(RECT_FLUSH_DELAY);
        flush_rects(&app);
    });
}

/// Bekleyen konum değişikliklerini hemen yazar (kapanışta da çağrılır).
pub fn flush_rects(app: &AppHandle) {
    let rt = runtime(app);
    rt.flush_scheduled.store(false, Ordering::Release);
    let pending: Vec<(String, Rect)> = lock(&rt.pending_rects).drain().collect();
    if pending.is_empty() {
        return;
    }
    let result = db(app).with_tx(|tx| {
        for (id, rect) in &pending {
            match repo::update_rect(tx, id, *rect) {
                Ok(()) | Err(crate::error::AppError::NotFound) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    });
    match result {
        Ok(()) => events::broadcast(app, events::WIDGETS_CHANGED, ()),
        Err(e) => log::error!("widget konumları kaydedilemedi: {e}"),
    }
}

/// Bir widget penceresi yok edildi. Bilerek kapatılmadıysa (ör. Explorer çöktü ve
/// ebeveyniyle birlikte yok edildi) kısa bir gecikmeyle yeniden oluşturulur.
pub fn on_window_destroyed(app: &AppHandle, id: &str) {
    let rt = runtime(app);
    if lock(&rt.closing).remove(id) {
        return;
    }
    if lock(&rt.windows).remove(id).is_some() {
        log::warn!("widget penceresi beklenmedik şekilde kapandı: {id}");
        schedule_respawn(app);
    }
}

pub fn on_windows_lost(app: &AppHandle, dead: Vec<isize>) {
    lock(&runtime(app).windows).retain(|_, h| !dead.contains(h));
    // Explorer yeniden başladı: eksik kalan tüm pencereler yeni masaüstünde oluşturulur.
    respawn_missing(app);
}

fn schedule_respawn(app: &AppHandle) {
    if runtime(app).respawn_scheduled.swap(true, Ordering::AcqRel) {
        return;
    }
    let app2 = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(RESPAWN_DELAY);
        let app3 = app2.clone();
        let _ = app2.run_on_main_thread(move || {
            runtime(&app3)
                .respawn_scheduled
                .store(false, Ordering::Release);
            respawn_missing(&app3);
        });
    });
}

/// Görünür olması gereken ama penceresi olmayan widget'ları yeniden oluşturur.
fn respawn_missing(app: &AppHandle) {
    if let Err(e) = restore_all(app) {
        log::error!("widget'lar yeniden oluşturulamadı: {e}");
    }
}
