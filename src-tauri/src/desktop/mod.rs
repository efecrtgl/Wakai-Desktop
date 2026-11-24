//! Windows 11 masaüstü katmanı entegrasyonu.
//!
//! Tüm `unsafe` Win32 kodu bu modülün içindedir. Dışarıya güvenli bir API açılır:
//! [`DesktopLayer`] widget pencerelerini masaüstü katmanına bağlar, kaldırır ve taşır;
//! [`watcher`] sistem olaylarını [`DesktopEvent`] olarak bildirir.
//!
//! Etkileşim modeli ("üzerine gelince kaldır"):
//! - Dinlenme: widget Progman'ın alt penceresidir → simgelerin altında, duvar kağıdının üstünde.
//! - İmleç widget'ın üzerine gelir ve altında gerçekten masaüstü vardır → widget masaüstünün
//!   hemen üstüne, tüm uygulama pencerelerinin altına üst düzey pencere olarak taşınır.
//!   Artık fare ve klavye girdisini doğal yoldan alır.
//! - İmleç ayrılır ve widget odakta değildir → widget katmana geri döner.

mod hierarchy;
pub mod monitors;
mod placement;
mod watcher;

pub use hierarchy::Layout;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetForegroundWindow};

/// Fiziksel piksel cinsinden ekran dikdörtgeni.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
    pub fn from_ltrb(l: i32, t: i32, r: i32, b: i32) -> Self {
        Self {
            x: l,
            y: t,
            width: r - l,
            height: b - t,
        }
    }
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.width && y < self.y + self.height
    }
    /// Dikdörtgeni alanın içine taşır; alandan büyükse boyutunu da küçültür.
    pub fn clamped_into(&self, area: Rect) -> Rect {
        let width = self.width.min(area.width).max(1);
        let height = self.height.min(area.height).max(1);
        let x = self.x.clamp(area.x, area.x + area.width - width);
        let y = self.y.clamp(area.y, area.y + area.height - height);
        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

/// İzleyici iş parçacığından gelen olaylar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopEvent {
    ExplorerRestarted,
    DisplayChanged,
    SessionEnding,
    /// İmleç masaüstündeki bir widget'ın üzerine geldi (değer: pencere tanıtıcısı).
    HoverEnter(isize),
    /// İmleç bir widget'tan ayrıldı.
    HoverLeave(isize),
}

/// İzleyicinin okuduğu, iş parçacıkları arası paylaşılan hedef bilgisi.
#[derive(Debug, Clone)]
struct Target {
    hwnd: isize,
    rect: Rect,
    lifted: bool,
    interactive: bool,
}

/// Masaüstü pencerelerinin tanıtıcıları (Send olabilmesi için tamsayı olarak).
#[derive(Debug, Clone, Copy)]
struct HostSnapshot {
    icons: isize,
    defview: isize,
    root: isize,
    parent: isize,
}

impl HostSnapshot {
    fn contains(&self, h: isize) -> bool {
        h != 0 && (h == self.icons || h == self.defview || h == self.root || h == self.parent)
    }
}

static TARGETS: RwLock<Vec<Target>> = RwLock::new(Vec::new());
static HOST: RwLock<Option<HostSnapshot>> = RwLock::new(None);
static HANDLER: OnceLock<Box<dyn Fn(DesktopEvent) + Send + Sync>> = OnceLock::new();

fn emit(event: DesktopEvent) {
    if let Some(h) = HANDLER.get() {
        h(event);
    }
}

/// İzleyiciyi başlatır. Yalnızca bir kez çağrılmalıdır.
pub fn start_watcher(handler: impl Fn(DesktopEvent) + Send + Sync + 'static) {
    if HANDLER.set(Box::new(handler)).is_ok() {
        watcher::spawn();
    }
}

pub fn cursor_position() -> Option<(i32, i32)> {
    let mut pt = POINT::default();
    unsafe { GetCursorPos(&mut pt).ok()? };
    Some((pt.x, pt.y))
}

pub fn is_foreground(hwnd: isize) -> bool {
    unsafe { GetForegroundWindow().0 as isize == hwnd }
}

pub fn window_rect(hwnd: isize) -> Option<Rect> {
    placement::screen_rect(h(hwnd))
}

fn h(raw: isize) -> HWND {
    HWND(raw as *mut _)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Placement {
    /// Masaüstü katmanında (simgelerin altında).
    Sunk,
    /// Etkileşim için masaüstünün hemen üstünde.
    Lifted,
    /// Masaüstü bulunamadı; pencere normal üst düzey pencere olarak duruyor.
    Detached,
}

#[derive(Debug, Clone)]
struct Entry {
    rect: Rect,
    placement: Placement,
    interactive: bool,
}

/// Widget pencerelerinin masaüstü katmanındaki durumunu yönetir.
/// Pencere işlemleri ana (UI) iş parçacığından çağrılmalıdır.
#[derive(Default)]
pub struct DesktopLayer {
    host: Option<hierarchy::DesktopHost>,
    windows: HashMap<isize, Entry>,
}

// SAFETY: HWND değerleri yalnızca tanıtıcıdır; tüm pencere işlemleri ana iş parçacığında yapılır.
unsafe impl Send for DesktopLayer {}

impl DesktopLayer {
    pub fn new() -> Self {
        let mut layer = Self::default();
        layer.refresh_host();
        layer
    }

    /// Masaüstü hiyerarşisini yeniden bulur.
    pub fn refresh_host(&mut self) -> bool {
        self.host = hierarchy::locate();
        let snapshot = self.host.map(|h| HostSnapshot {
            icons: h.icons.0 as isize,
            defview: h.defview.0 as isize,
            root: h.root.0 as isize,
            parent: h.parent.0 as isize,
        });
        *HOST.write().unwrap_or_else(|e| e.into_inner()) = snapshot;
        match &self.host {
            Some(h) => log::info!("masaüstü katmanı bulundu: {:?}", h.layout),
            None => log::warn!("masaüstü katmanı bulunamadı (Explorer çalışmıyor olabilir)"),
        }
        self.host.is_some()
    }

    pub fn layout(&self) -> Option<Layout> {
        self.host.map(|h| h.layout)
    }

    fn sync_targets(&self) {
        let list = self
            .windows
            .iter()
            .map(|(hwnd, e)| Target {
                hwnd: *hwnd,
                rect: e.rect,
                lifted: e.placement != Placement::Sunk,
                interactive: e.interactive,
            })
            .collect();
        *TARGETS.write().unwrap_or_else(|e| e.into_inner()) = list;
    }

    fn live_host(&mut self) -> Option<hierarchy::DesktopHost> {
        if !self.host.map(|h| h.is_alive()).unwrap_or(false) {
            self.refresh_host();
        }
        self.host
    }

    /// Pencereyi katmana ekler ve yerleştirir.
    pub fn attach(&mut self, hwnd: isize, rect: Rect, interactive: bool) -> Placement {
        let rect = monitors::clamp_visible(rect);
        if !placement::install_dpi_guard(h(hwnd)) {
            log::warn!("DPI koruması kurulamadı");
        }
        let placement = match self.live_host() {
            Some(host) if placement::sink(h(hwnd), &host, rect) => Placement::Sunk,
            _ => {
                placement::move_top_level(h(hwnd), rect);
                Placement::Detached
            }
        };
        self.windows.insert(
            hwnd,
            Entry {
                rect,
                placement,
                interactive,
            },
        );
        self.sync_targets();
        placement
    }

    pub fn detach(&mut self, hwnd: isize) {
        self.windows.remove(&hwnd);
        self.sync_targets();
    }

    pub fn contains(&self, hwnd: isize) -> bool {
        self.windows.contains_key(&hwnd)
    }

    pub fn placement(&self, hwnd: isize) -> Option<Placement> {
        self.windows.get(&hwnd).map(|e| e.placement)
    }

    pub fn rect(&self, hwnd: isize) -> Option<Rect> {
        self.windows.get(&hwnd).map(|e| e.rect)
    }

    /// Tıklama geçirgenliği: etkileşimsiz widget'lar hiç kaldırılmaz.
    pub fn set_interactive(&mut self, hwnd: isize, interactive: bool) {
        if let Some(e) = self.windows.get_mut(&hwnd) {
            e.interactive = interactive;
        }
        self.sync_targets();
    }

    /// Konum/boyut değişikliğini uygular.
    pub fn set_rect(&mut self, hwnd: isize, rect: Rect) {
        let host = self.live_host();
        let Some(e) = self.windows.get_mut(&hwnd) else {
            return;
        };
        e.rect = rect;
        match (e.placement, host) {
            (Placement::Sunk, Some(host)) => {
                placement::move_sunk(h(hwnd), &host, rect);
            }
            _ => {
                placement::move_top_level(h(hwnd), rect);
            }
        }
        self.sync_targets();
    }

    /// Kullanıcı pencereyi kendisi taşıdığında/boyutlandırdığında (kaldırılmış durumda)
    /// gerçek konumu kaydeder; pencereyi hareket ettirmez.
    pub fn record_actual_rect(&mut self, hwnd: isize) -> Option<Rect> {
        let actual = placement::screen_rect(h(hwnd))?;
        let e = self.windows.get_mut(&hwnd)?;
        if e.placement == Placement::Sunk || e.rect == actual {
            return None;
        }
        e.rect = actual;
        self.sync_targets();
        Some(actual)
    }

    pub fn lift(&mut self, hwnd: isize) -> bool {
        let Some(host) = self.live_host() else {
            return false;
        };
        let Some(e) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        if e.placement != Placement::Sunk || !e.interactive {
            return false;
        }
        if placement::lift(h(hwnd), &host, e.rect) {
            e.placement = Placement::Lifted;
            self.sync_targets();
            true
        } else {
            false
        }
    }

    pub fn sink(&mut self, hwnd: isize) -> bool {
        let Some(host) = self.live_host() else {
            return false;
        };
        let Some(e) = self.windows.get_mut(&hwnd) else {
            return false;
        };
        if e.placement == Placement::Sunk {
            return true;
        }
        if let Some(actual) = placement::screen_rect(h(hwnd)) {
            e.rect = actual;
        }
        if placement::sink(h(hwnd), &host, e.rect) {
            e.placement = Placement::Sunk;
            self.sync_targets();
            true
        } else {
            false
        }
    }

    /// Kaldırılmış bir widget şu an katmana dönebilir mi? (odakta değil ve imleç dışarıda)
    pub fn should_sink(&self, hwnd: isize) -> bool {
        let Some(e) = self.windows.get(&hwnd) else {
            return false;
        };
        if e.placement != Placement::Lifted || is_foreground(hwnd) {
            return false;
        }
        match cursor_position() {
            Some((x, y)) => !e.rect.contains(x, y),
            None => true,
        }
    }

    /// Explorer yeniden başladıktan sonra yaşayan pencereleri yeni masaüstüne bağlar.
    /// Ölen pencerelerin tanıtıcılarını döndürür (yeniden oluşturulmaları gerekir).
    pub fn reattach_all(&mut self) -> Vec<isize> {
        self.refresh_host();
        let mut dead = Vec::new();
        let handles: Vec<isize> = self.windows.keys().copied().collect();
        for hwnd in handles {
            if !placement::is_window(h(hwnd)) {
                dead.push(hwnd);
                self.windows.remove(&hwnd);
                continue;
            }
            let e = self.windows.get_mut(&hwnd).expect("anahtar mevcut");
            e.rect = monitors::clamp_visible(e.rect);
            e.placement = match self.host {
                Some(host) if placement::sink(h(hwnd), &host, e.rect) => Placement::Sunk,
                _ => Placement::Detached,
            };
        }
        self.sync_targets();
        dead
    }

    /// Ekran yapılandırması değişti: her widget'ı görünür bir alana sığdırır.
    /// Konumu değişen pencereleri ve yeni dikdörtgenlerini döndürür.
    pub fn on_display_changed(&mut self) -> Vec<(isize, Rect)> {
        self.live_host();
        let mut changed = Vec::new();
        let handles: Vec<isize> = self.windows.keys().copied().collect();
        for hwnd in handles {
            let rect = self.windows[&hwnd].rect;
            let clamped = monitors::clamp_visible(rect);
            // Progman yeniden boyutlanmış olabilir; konum değişmese bile yeniden uygula.
            self.set_rect(hwnd, clamped);
            if clamped != rect {
                changed.push((hwnd, clamped));
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains_is_half_open() {
        let r = Rect::new(10, 10, 100, 50);
        assert!(r.contains(10, 10));
        assert!(r.contains(109, 59));
        assert!(!r.contains(110, 30));
        assert!(!r.contains(50, 60));
    }

    #[test]
    fn clamp_moves_offscreen_rect_inside() {
        let area = Rect::new(0, 0, 1920, 1040);
        assert_eq!(
            Rect::new(3000, 500, 300, 200).clamped_into(area),
            Rect::new(1620, 500, 300, 200)
        );
        assert_eq!(
            Rect::new(-500, -50, 300, 200).clamped_into(area),
            Rect::new(0, 0, 300, 200)
        );
        // Alandan büyük dikdörtgen küçültülür.
        assert_eq!(Rect::new(0, 0, 4000, 3000).clamped_into(area), area);
    }

    #[test]
    fn negative_origin_monitor_is_supported() {
        let left_monitor = Rect::new(-1920, 0, 1920, 1080);
        assert_eq!(
            Rect::new(-100, 10, 300, 200).clamped_into(left_monitor),
            Rect::new(-300, 10, 300, 200)
        );
    }

    #[test]
    fn host_snapshot_ignores_null_handle() {
        let s = HostSnapshot {
            icons: 1,
            defview: 2,
            root: 3,
            parent: 3,
        };
        assert!(s.contains(1) && s.contains(3));
        assert!(!s.contains(0) && !s.contains(9));
    }
}
