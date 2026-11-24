//! Gizli bir üst düzey pencere üzerinden sistem olaylarını dinleyen iş parçacığı.
//!
//! - `TaskbarCreated`: Explorer yeniden başladı → masaüstü hiyerarşisi yeniden kurulmalı.
//! - `WM_DISPLAYCHANGE` / `WM_SETTINGCHANGE(SPI_SETWORKAREA)`: çözünürlük/monitör değişti.
//! - `WM_ENDSESSION`: Windows kapanıyor → bekleyen veriler yazılmalı.
//! - `WM_INPUT` (Raw Input, RIDEV_INPUTSINK): imleç bir widget'ın üzerine geldi mi?
//!
//! Raw Input yalnızca fare HAREKET ETTİĞİNDE mesaj üretir; boşta iken bu iş parçacığı
//! `GetMessageW` içinde uyur ve CPU kullanmaz.

use super::{DesktopEvent, HOST, TARGETS, emit};
use std::cell::Cell;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::{RAWINPUTDEVICE, RIDEV_INPUTSINK, RegisterRawInputDevices};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetCursorPos, GetMessageW, MSG,
    RegisterClassW, RegisterWindowMessageW, SPI_SETWORKAREA, TranslateMessage, WINDOW_EX_STYLE,
    WM_DISPLAYCHANGE, WM_ENDSESSION, WM_INPUT, WM_QUERYENDSESSION, WM_SETTINGCHANGE, WNDCLASSW,
    WS_EX_TOOLWINDOW, WS_POPUP, WindowFromPoint,
};
use windows::core::w;

thread_local! {
    static TASKBAR_CREATED: Cell<u32> = const { Cell::new(0) };
    static LAST_POINT: Cell<(i32, i32)> = const { Cell::new((i32::MIN, i32::MIN)) };
    /// İmlecin şu an üzerinde bulunduğu widget kimliği (TARGETS indeksi değil, hwnd).
    static HOVERED: Cell<isize> = const { Cell::new(0) };
}

pub fn spawn() {
    std::thread::Builder::new()
        .name("wakai-desktop-watcher".into())
        .spawn(|| {
            if let Err(e) = run() {
                log::error!("masaüstü izleyici başlatılamadı: {e}");
            }
        })
        .expect("izleyici iş parçacığı oluşturulamadı");
}

fn run() -> windows::core::Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class = w!("WakaiDesktopWatcher");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: class,
            ..Default::default()
        };
        RegisterClassW(&wc);
        TASKBAR_CREATED.with(|c| c.set(RegisterWindowMessageW(w!("TaskbarCreated"))));

        // Yayın mesajlarını (TaskbarCreated, WM_DISPLAYCHANGE) yalnızca üst düzey pencereler
        // alır; bu yüzden mesaj-yalnız (HWND_MESSAGE) değil, hiç gösterilmeyen bir pencere kullanılır.
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(WS_EX_TOOLWINDOW.0),
            class,
            w!("Wakai Desktop Watcher"),
            WS_POPUP,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        let device = RAWINPUTDEVICE {
            usUsagePage: 0x01, // Generic Desktop
            usUsage: 0x02,     // Mouse
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        };
        RegisterRawInputDevices(&[device], std::mem::size_of::<RAWINPUTDEVICE>() as u32)?;

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_INPUT => on_mouse_input(),
        WM_DISPLAYCHANGE => emit(DesktopEvent::DisplayChanged),
        WM_SETTINGCHANGE if wparam.0 as u32 == SPI_SETWORKAREA.0 => {
            emit(DesktopEvent::DisplayChanged)
        }
        WM_QUERYENDSESSION => return LRESULT(1),
        WM_ENDSESSION if wparam.0 != 0 => emit(DesktopEvent::SessionEnding),
        m if m != 0 && m == TASKBAR_CREATED.with(|c| c.get()) => {
            emit(DesktopEvent::ExplorerRestarted)
        }
        _ => {}
    }
    // WM_INPUT dahil tüm mesajlar varsayılan işleyiciye iletilir (Raw Input belleği temizlenir).
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn on_mouse_input() {
    let mut pt = POINT::default();
    if unsafe { GetCursorPos(&mut pt) }.is_err() {
        return;
    }
    let same = LAST_POINT.with(|c| {
        let prev = c.replace((pt.x, pt.y));
        prev == (pt.x, pt.y)
    });
    if same {
        return; // Yalnızca tıklama/tekerlek; konum değişmedi.
    }

    let targets = TARGETS.read().unwrap_or_else(|e| e.into_inner());
    if targets.is_empty() {
        return;
    }
    let hit = targets
        .iter()
        .find(|t| t.interactive && t.rect.contains(pt.x, pt.y));
    let prev = HOVERED.with(|c| c.get());

    if prev != 0 && hit.map(|t| t.hwnd) != Some(prev) {
        HOVERED.with(|c| c.set(0));
        emit(DesktopEvent::HoverLeave(prev));
    }

    let Some(t) = hit else { return };
    if t.lifted {
        HOVERED.with(|c| c.set(t.hwnd));
        return;
    }
    if prev == t.hwnd {
        return;
    }
    // Widget yalnızca imlecin altında GERÇEKTEN masaüstü varsa kaldırılır;
    // üstünde bir uygulama penceresi varsa hiçbir şey yapılmaz.
    let under = unsafe { WindowFromPoint(pt) };
    let host = HOST.read().unwrap_or_else(|e| e.into_inner());
    let on_desktop = host.map(|h| h.contains(under.0 as isize)).unwrap_or(false);
    if on_desktop {
        HOVERED.with(|c| c.set(t.hwnd));
        emit(DesktopEvent::HoverEnter(t.hwnd));
    }
}
