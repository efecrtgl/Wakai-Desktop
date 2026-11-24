//! Windows masaüstü pencere hiyerarşisinin tespiti.
//!
//! Windows 11 24H2 ve sonrası:
//!   Progman ─┬─ SHELLDLL_DefView (simgeler, üstte)
//!            └─ WorkerW          (duvar kağıdı, altta)
//! Eski düzen:
//!   WorkerW (üst düzey) ── SHELLDLL_DefView
//!   WorkerW (üst düzey, hemen arkasında) ── duvar kağıdı
//!
//! `0x052C` mesajı belgelenmemiştir ama yaygın olarak kullanılır: Explorer'dan duvar
//! kağıdını ayrı bir WorkerW'ye çizmesini ister. Kullanımı bu dosyayla sınırlıdır.

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, FindWindowW, IsWindow, SMTO_NORMAL, SendMessageTimeoutW,
};
use windows::core::{BOOL, PCWSTR, w};

const SPAWN_WORKERW: u32 = 0x052C;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// Windows 11 24H2+: widget'lar Progman altındaki duvar kağıdı WorkerW'sinin alt penceresi olur.
    Modern,
    /// Eski düzen: widget'lar duvar kağıdı WorkerW'sinin alt penceresi olur.
    Legacy,
}

/// Widget'ların bağlanacağı masaüstü konumu.
#[derive(Debug, Clone, Copy)]
pub struct DesktopHost {
    pub layout: Layout,
    /// Widget pencerelerinin ebeveyni.
    pub parent: HWND,
    /// Widget'lar z-sırasında bu pencerenin hemen ALTINA yerleştirilir (yoksa en üste).
    pub insert_below: Option<HWND>,
    /// Simgeleri barındıran kabuk görünümü.
    pub defview: HWND,
    /// Simge listesi (SysListView32).
    pub icons: HWND,
    /// Masaüstünün üst düzey kök penceresi (Progman ya da simgeleri taşıyan WorkerW).
    pub root: HWND,
}

impl DesktopHost {
    pub fn is_alive(&self) -> bool {
        unsafe { IsWindow(Some(self.parent)).as_bool() && IsWindow(Some(self.defview)).as_bool() }
    }
}

fn find_child(parent: Option<HWND>, after: Option<HWND>, class: PCWSTR) -> Option<HWND> {
    unsafe { FindWindowExW(parent, after, class, PCWSTR::null()).ok() }.filter(|h| !h.is_invalid())
}

/// Masaüstü hiyerarşisini bulur. Explorer çalışmıyorsa `None` döner.
pub fn locate() -> Option<DesktopHost> {
    let progman =
        unsafe { FindWindowW(w!("Progman"), PCWSTR::null()).ok() }.filter(|h| !h.is_invalid())?;

    unsafe {
        let mut result = 0usize;
        // Yanıt vermeyen bir Explorer'da takılmamak için zaman aşımı kullanılır.
        let _ = SendMessageTimeoutW(
            progman,
            SPAWN_WORKERW,
            WPARAM(0),
            LPARAM(0),
            SMTO_NORMAL,
            1000,
            Some(&mut result),
        );
    }

    if let Some(defview) = find_child(Some(progman), None, w!("SHELLDLL_DefView")) {
        let icons = find_child(Some(defview), None, w!("SysListView32")).unwrap_or(defview);
        // Doğrudan Progman'a bağlanan bir WebView2 penceresi, z-sırasında DefView'in altında
        // olsa bile simgelerin ÜSTÜNDE birleştirilir (doğrulandı). Duvar kağıdı WorkerW'sinin
        // alt penceresi olunca simgeler doğru şekilde üstte kalır.
        let wallpaper = find_child(Some(progman), None, w!("WorkerW"));
        return Some(DesktopHost {
            layout: Layout::Modern,
            parent: wallpaper.unwrap_or(progman),
            insert_below: if wallpaper.is_some() {
                None
            } else {
                Some(defview)
            },
            defview,
            icons,
            root: progman,
        });
    }

    // Eski düzen: SHELLDLL_DefView'i barındıran üst düzey WorkerW'yi bul.
    let mut icon_host = HWND::default();
    unsafe {
        let _ = EnumWindows(
            Some(find_icon_host),
            LPARAM(&mut icon_host as *mut HWND as isize),
        );
    }
    if icon_host.is_invalid() {
        return None;
    }
    let defview = find_child(Some(icon_host), None, w!("SHELLDLL_DefView"))?;
    let icons = find_child(Some(defview), None, w!("SysListView32")).unwrap_or(defview);
    let wallpaper = find_child(None, Some(icon_host), w!("WorkerW"))?;
    Some(DesktopHost {
        layout: Layout::Legacy,
        parent: wallpaper,
        insert_below: None,
        defview,
        icons,
        root: icon_host,
    })
}

unsafe extern "system" fn find_icon_host(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if find_child(Some(hwnd), None, w!("SHELLDLL_DefView")).is_some() {
        // SAFETY: lparam, `locate` içindeki yerel bir HWND değişkenini gösterir ve
        // EnumWindows dönene kadar geçerlidir.
        unsafe { *(lparam.0 as *mut HWND) = hwnd };
        return BOOL(0);
    }
    BOOL(1)
}
