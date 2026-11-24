//! Monitör ve çalışma alanı yardımcıları (fiziksel piksel).

use super::Rect;
use serde::Serialize;
use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITOR_DEFAULTTONEAREST, MONITORINFO,
    MonitorFromRect,
};
use windows::Win32::UI::WindowsAndMessaging::MONITORINFOF_PRIMARY;
use windows::core::BOOL;

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct MonitorInfo {
    pub bounds: Rect,
    pub work_area: Rect,
    pub primary: bool,
}

fn info(mon: HMONITOR) -> Option<MonitorInfo> {
    let mut mi = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    unsafe { GetMonitorInfoW(mon, &mut mi).ok().ok()? };
    let r = |r: RECT| Rect::from_ltrb(r.left, r.top, r.right, r.bottom);
    Some(MonitorInfo {
        bounds: r(mi.rcMonitor),
        work_area: r(mi.rcWork),
        primary: mi.dwFlags & MONITORINFOF_PRIMARY != 0,
    })
}

pub fn all() -> Vec<MonitorInfo> {
    let mut list: Vec<MonitorInfo> = Vec::new();
    unsafe extern "system" fn cb(mon: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
        // SAFETY: data, `all` içindeki yerel vektörü gösterir; EnumDisplayMonitors dönene kadar geçerlidir.
        let list = unsafe { &mut *(data.0 as *mut Vec<MonitorInfo>) };
        if let Some(i) = info(mon) {
            list.push(i);
        }
        BOOL(1)
    }
    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(cb), LPARAM(&mut list as *mut _ as isize));
    }
    list
}

/// Dikdörtgene en yakın monitörün çalışma alanı.
pub fn nearest_work_area(rect: Rect) -> Option<Rect> {
    let r = RECT {
        left: rect.x,
        top: rect.y,
        right: rect.x + rect.width,
        bottom: rect.y + rect.height,
    };
    let mon = unsafe { MonitorFromRect(&r, MONITOR_DEFAULTTONEAREST) };
    info(mon).map(|i| i.work_area)
}

/// Widget'ın görünür kalmasını sağlar: en yakın monitörün çalışma alanına sığdırır.
/// Bağlantısı kesilen bir monitördeki widget böylece görünür bir monitöre taşınır.
pub fn clamp_visible(rect: Rect) -> Rect {
    match nearest_work_area(rect) {
        Some(area) => rect.clamped_into(area),
        None => rect,
    }
}

/// Dikdörtgenin bulunduğu monitörün ölçek katsayısı (ör. %125 → 1.25).
pub fn scale_for(rect: Rect) -> f64 {
    use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
    let r = RECT {
        left: rect.x,
        top: rect.y,
        right: rect.x + rect.width,
        bottom: rect.y + rect.height,
    };
    let mon = unsafe { MonitorFromRect(&r, MONITOR_DEFAULTTONEAREST) };
    let (mut dx, mut dy) = (96u32, 96u32);
    if unsafe { GetDpiForMonitor(mon, MDT_EFFECTIVE_DPI, &mut dx, &mut dy) }.is_err() {
        return 1.0;
    }
    dx as f64 / 96.0
}
