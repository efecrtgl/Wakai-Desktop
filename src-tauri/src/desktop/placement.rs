//! Pencereleri masaüstü katmanına yerleştiren ve oradan çıkaran düşük seviye işlemler.
//! Yalnızca KENDİ pencerelerimizin stilleri ve ebeveyni değiştirilir; Explorer
//! pencerelerine dokunulmaz.

use super::Rect;
use super::hierarchy::DesktopHost;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::UI::WindowsAndMessaging::{
    GW_HWNDPREV, GWL_EXSTYLE, GWL_STYLE, GetWindow, GetWindowLongPtrW, GetWindowRect, HWND_BOTTOM,
    HWND_TOP, IsWindow, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOOWNERZORDER, SWP_SHOWWINDOW,
    SetParent, SetWindowLongPtrW, SetWindowPos, WS_CAPTION, WS_CHILD, WS_CLIPSIBLINGS,
    WS_EX_APPWINDOW, WS_EX_TOOLWINDOW, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU,
    WS_THICKFRAME,
};

pub fn is_window(hwnd: HWND) -> bool {
    unsafe { IsWindow(Some(hwnd)).as_bool() }
}

/// Pencerenin ekran koordinatlarındaki dikdörtgeni.
pub fn screen_rect(hwnd: HWND) -> Option<Rect> {
    let mut r = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut r).ok()? };
    Some(Rect::from_ltrb(r.left, r.top, r.right, r.bottom))
}

fn set_styles(hwnd: HWND, child: bool) {
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        let frame = (WS_CAPTION | WS_THICKFRAME | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX).0;
        let mut next = style & !frame;
        if child {
            next = (next & !WS_POPUP.0) | WS_CHILD.0 | WS_CLIPSIBLINGS.0;
        } else {
            next = (next & !WS_CHILD.0) | WS_POPUP.0;
        }
        SetWindowLongPtrW(hwnd, GWL_STYLE, next as isize);

        // Görev çubuğunda ve Alt+Tab listesinde görünmesin.
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let ex_next = (ex & !WS_EX_APPWINDOW.0) | WS_EX_TOOLWINDOW.0;
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_next as isize);
    }
}

/// Ekran koordinatını ebeveynin istemci koordinatına çevirir.
fn to_parent_client(parent: HWND, rect: Rect) -> POINT {
    let mut pt = POINT {
        x: rect.x,
        y: rect.y,
    };
    unsafe {
        let _ = ScreenToClient(parent, &mut pt);
    }
    pt
}

/// Pencereyi masaüstü katmanına (simgelerin altı, duvar kağıdının üstü) yerleştirir.
pub fn sink(hwnd: HWND, host: &DesktopHost, rect: Rect) -> bool {
    set_styles(hwnd, true);
    unsafe {
        if SetParent(hwnd, Some(host.parent)).is_err() {
            log::warn!("SetParent başarısız oldu");
            return false;
        }
        let pt = to_parent_client(host.parent, rect);
        let after = host.insert_below.unwrap_or(HWND_TOP);

        SetWindowPos(
            hwnd,
            Some(after),
            pt.x,
            pt.y,
            rect.width,
            rect.height,
            SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW | SWP_NOOWNERZORDER,
        )
        .is_ok()
    }
}

/// Katmandaki bir pencereyi yeniden konumlandırır (z-sırasını da düzeltir).
pub fn move_sunk(hwnd: HWND, host: &DesktopHost, rect: Rect) -> bool {
    let pt = to_parent_client(host.parent, rect);
    let after = host.insert_below.unwrap_or(HWND_TOP);
    unsafe {
        SetWindowPos(
            hwnd,
            Some(after),
            pt.x,
            pt.y,
            rect.width,
            rect.height,
            SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        )
        .is_ok()
    }
}

/// Pencereyi katmandan çıkarıp masaüstünün HEMEN ÜSTÜNE, diğer tüm uygulama
/// pencerelerinin altına üst düzey pencere olarak yerleştirir. Etkinleştirmez.
pub fn lift(hwnd: HWND, host: &DesktopHost, rect: Rect) -> bool {
    set_styles(hwnd, false);
    unsafe {
        if SetParent(hwnd, None).is_err() {
            return false;
        }
        // z-sırasında masaüstü kökünün hemen üstündeki pencerenin altına gir.
        let above_root = GetWindow(host.root, GW_HWNDPREV)
            .ok()
            .filter(|h| !h.is_invalid() && *h != hwnd);
        let after = above_root.unwrap_or(HWND_BOTTOM);
        SetWindowPos(
            hwnd,
            Some(after),
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW | SWP_NOOWNERZORDER,
        )
        .is_ok()
    }
}

/// Üst düzey (kaldırılmış) bir pencereyi taşır; z-sırasını değiştirmez.
pub fn move_top_level(hwnd: HWND, rect: Rect) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::SWP_NOZORDER;
    unsafe {
        SetWindowPos(
            hwnd,
            None,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            SWP_NOACTIVATE | SWP_NOZORDER,
        )
        .is_ok()
    }
}

/// Widget penceresinin `WM_DPICHANGED` mesajını yutan alt sınıf.
/// Masaüstü katmanına bağlanan pencere ebeveyninin DPI değerini alır; pencere
/// çerçevesinin (tao) önerilen dikdörtgene göre kendini yeniden boyutlandırması
/// konumu bozar. Geometriyi bu modül yönettiği için mesaj burada sonlandırılır.
/// WebView2 içeriğin ölçeğini monitöre göre kendisi ayarlamaya devam eder.
pub fn install_dpi_guard(hwnd: HWND) -> bool {
    use windows::Win32::UI::Shell::SetWindowSubclass;
    unsafe { SetWindowSubclass(hwnd, Some(dpi_guard_proc), DPI_GUARD_ID, 0).as_bool() }
}

const DPI_GUARD_ID: usize = 0x5741_4B41; // "WAKA"

unsafe extern "system" fn dpi_guard_proc(
    hwnd: HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
    _id: usize,
    _data: usize,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass};
    use windows::Win32::UI::WindowsAndMessaging::{WM_DPICHANGED, WM_NCDESTROY};
    match msg {
        WM_DPICHANGED => windows::Win32::Foundation::LRESULT(0),
        WM_NCDESTROY => unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(dpi_guard_proc), DPI_GUARD_ID);
            DefSubclassProc(hwnd, msg, wparam, lparam)
        },
        _ => unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) },
    }
}
