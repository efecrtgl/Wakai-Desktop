//! Kediler için yerel Win32 katmanı. Kedilerle ilgili TÜM `unsafe` kod buradadır.
//!
//! Her kedi küçük bir katmanlı penceredir (`WS_EX_LAYERED`), içeriği `UpdateLayeredWindow`
//! ile piksel başına saydamlıkla çizilir:
//! - Saydam piksellerden tıklama geçer: kedi yalnızca kendi şeklinin olduğu yerde fareyi yakalar.
//! - `WS_EX_NOACTIVATE` + `WS_EX_TOOLWINDOW`: odak çalmaz, görev çubuğunda/Alt+Tab'da görünmez.
//! - `WS_EX_TOPMOST`: uygulama pencerelerinin üstünde dolaşabilir.
//!
//! WebView kullanılmaz; kedi başına bellek maliyeti birkaç yüz KB'dir.

use std::cell::RefCell;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
use windows::Win32::Graphics::Gdi::{
    AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION,
    CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, HBITMAP,
    HDC, HGDIOBJ, ReleaseDC, SelectObject,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ReleaseCapture, SetCapture, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent,
};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{BOOL, HSTRING, PCWSTR, w};

/// Pencereden motora iletilen fare olayları (ekran koordinatı, fiziksel piksel).
#[derive(Debug, Clone, Copy)]
pub enum Input {
    Down(isize, i32, i32),
    Move(isize, i32, i32, bool),
    Up(isize, i32, i32),
    Leave(isize),
    Menu(isize, i32, i32),
}

/// `WM_MOUSELEAVE` (winuser.h). Tek sabit için büyük `Win32_UI_Controls` özelliği eklenmez.
const WM_MOUSELEAVE: u32 = 0x02A3;

thread_local! {
    static INPUTS: RefCell<Vec<Input>> = const { RefCell::new(Vec::new()) };
    static TRACKING: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
}

pub fn drain_inputs() -> Vec<Input> {
    INPUTS.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

fn cursor() -> (i32, i32) {
    let mut p = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut p);
    }
    (p.x, p.y)
}

pub fn cursor_pos() -> (i32, i32) {
    cursor()
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let h = hwnd.0 as isize;
    let (x, y) = cursor();
    let push = |i: Input| INPUTS.with(|q| q.borrow_mut().push(i));
    match msg {
        WM_LBUTTONDOWN => {
            unsafe { SetCapture(hwnd) };
            push(Input::Down(h, x, y));
            return LRESULT(0);
        }
        WM_LBUTTONUP => {
            let _ = unsafe { ReleaseCapture() };
            push(Input::Up(h, x, y));
            return LRESULT(0);
        }
        WM_MOUSEMOVE => {
            let tracked = TRACKING.with(|t| t.borrow().contains(&h));
            if !tracked {
                let mut tme = TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                let _ = unsafe { TrackMouseEvent(&mut tme) };
                TRACKING.with(|t| t.borrow_mut().push(h));
            }
            push(Input::Move(h, x, y, wp.0 & 0x0001 != 0));
            return LRESULT(0);
        }
        WM_MOUSELEAVE => {
            TRACKING.with(|t| t.borrow_mut().retain(|v| *v != h));
            push(Input::Leave(h));
            return LRESULT(0);
        }
        WM_RBUTTONUP => {
            push(Input::Menu(h, x, y));
            return LRESULT(0);
        }
        // Tıklamada etkinleşme: asla odak alma.
        WM_MOUSEACTIVATE => return LRESULT(MA_NOACTIVATE as isize),
        _ => {}
    }
    unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
}

fn ensure_class() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| unsafe {
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: GetModuleHandleW(None).unwrap_or_default().into(),
            lpszClassName: w!("WakaiPet"),
            hCursor: LoadCursorW(None, IDC_HAND).unwrap_or_default(),
            ..Default::default()
        };
        RegisterClassW(&wc);
    });
}

/// Katmanlı pencere + DIB yüzeyi.
pub struct Sprite {
    pub hwnd: HWND,
    hdc: HDC,
    bitmap: HBITMAP,
    old: HGDIOBJ,
    bits: *mut u32,
    pub w: i32,
    pub h: i32,
}

impl Sprite {
    pub fn new(title: &str) -> Option<Self> {
        ensure_class();
        unsafe {
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST,
                w!("WakaiPet"),
                &HSTRING::from(title),
                WS_POPUP,
                0,
                0,
                1,
                1,
                None,
                None,
                Some(GetModuleHandleW(None).ok()?.into()),
                None,
            )
            .ok()?;
            let screen = GetDC(None);
            let hdc = CreateCompatibleDC(Some(screen));
            ReleaseDC(None, screen);
            Some(Self {
                hwnd,
                hdc,
                bitmap: HBITMAP::default(),
                old: HGDIOBJ::default(),
                bits: std::ptr::null_mut(),
                w: 0,
                h: 0,
            })
        }
    }

    /// Penceresi olmayan, hiçbir şey çizmeyen yer tutucu (motor içi geçici değiş tokuş için).
    pub fn placeholder() -> Self {
        Self {
            hwnd: HWND::default(),
            hdc: HDC::default(),
            bitmap: HBITMAP::default(),
            old: HGDIOBJ::default(),
            bits: std::ptr::null_mut(),
            w: 0,
            h: 0,
        }
    }

    pub fn raw(&self) -> isize {
        self.hwnd.0 as isize
    }

    fn resize(&mut self, w: i32, h: i32) {
        if w == self.w && h == self.h && !self.bits.is_null() {
            return;
        }
        unsafe {
            if !self.bitmap.is_invalid() {
                SelectObject(self.hdc, self.old);
                let _ = DeleteObject(self.bitmap.into());
            }
            let bi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: w,
                    biHeight: -h, // yukarıdan aşağı
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bits = std::ptr::null_mut();
            self.bitmap = CreateDIBSection(Some(self.hdc), &bi, DIB_RGB_COLORS, &mut bits, None, 0)
                .unwrap_or_default();
            self.old = SelectObject(self.hdc, self.bitmap.into());
            self.bits = bits as *mut u32;
        }
        self.w = w;
        self.h = h;
    }

    /// Yeni kareyi çizer ve pencereyi verilen konuma taşır.
    pub fn draw(&mut self, x: i32, y: i32, w: i32, h: i32, px: &[u32]) {
        self.resize(w, h);
        if self.bits.is_null() || px.len() != (w * h) as usize {
            return;
        }
        unsafe {
            // SAFETY: `bits` w*h piksellik DIB arabelleğini gösterir (resize ile ayrıldı).
            std::ptr::copy_nonoverlapping(px.as_ptr(), self.bits, px.len());
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            let screen = GetDC(None);
            let _ = UpdateLayeredWindow(
                self.hwnd,
                Some(screen),
                Some(&POINT { x, y }),
                Some(&SIZE { cx: w, cy: h }),
                Some(self.hdc),
                Some(&POINT { x: 0, y: 0 }),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );
            ReleaseDC(None, screen);
        }
    }

    pub fn move_to(&self, x: i32, y: i32) {
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                Some(HWND_TOPMOST),
                x,
                y,
                0,
                0,
                SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }

    pub fn show(&self, visible: bool) {
        unsafe {
            let _ = ShowWindow(self.hwnd, if visible { SW_SHOWNOACTIVATE } else { SW_HIDE });
        }
    }
}

impl Drop for Sprite {
    fn drop(&mut self) {
        if self.hwnd.is_invalid() {
            return;
        }
        unsafe {
            if !self.bitmap.is_invalid() {
                SelectObject(self.hdc, self.old);
                let _ = DeleteObject(self.bitmap.into());
            }
            let _ = DeleteDC(self.hdc);
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

/// Kedinin üstünde durabileceği bir pencerenin üst kenarı.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Platform {
    pub hwnd: isize,
    pub left: i32,
    pub top: i32,
    pub right: i32,
}

fn class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 64];
    let n = unsafe { GetClassNameW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

fn cloaked(hwnd: HWND) -> bool {
    let mut v: u32 = 0;
    unsafe {
        DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut v as *mut u32 as *mut _, 4).is_ok()
            && v != 0
    }
}

/// Görünür, normal uygulama pencereleri (büyütülmüş, simge durumunda, gizli, araç pencereleri,
/// kendi pencerelerimiz ve kabuk pencereleri hariç).
pub fn platforms() -> Vec<Platform> {
    let mut out: Vec<Platform> = Vec::new();
    unsafe extern "system" fn cb(hwnd: HWND, data: LPARAM) -> BOOL {
        // SAFETY: data, `platforms` içindeki yerel vektörü gösterir.
        let out = unsafe { &mut *(data.0 as *mut Vec<Platform>) };
        unsafe {
            if !IsWindowVisible(hwnd).as_bool()
                || IsIconic(hwnd).as_bool()
                || IsZoomed(hwnd).as_bool()
            {
                return BOOL(1);
            }
            let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
            if ex & WS_EX_TOOLWINDOW.0 != 0 || cloaked(hwnd) {
                return BOOL(1);
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == std::process::id() {
                return BOOL(1);
            }
            let cls = class_name(hwnd);
            if matches!(
                cls.as_str(),
                "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd"
            ) {
                return BOOL(1);
            }
            let mut r = RECT::default();
            if GetWindowRect(hwnd, &mut r).is_ok()
                && r.right - r.left > 200
                && r.bottom - r.top > 100
            {
                out.push(Platform {
                    hwnd: hwnd.0 as isize,
                    left: r.left,
                    top: r.top,
                    right: r.right,
                });
            }
        }
        BOOL(1)
    }
    unsafe {
        let _ = EnumWindows(Some(cb), LPARAM(&mut out as *mut _ as isize));
    }
    out
}

/// Pencere hâlâ platform olarak geçerliyse güncel üst kenarını döndürür.
pub fn platform_now(hwnd: isize) -> Option<Platform> {
    let h = HWND(hwnd as *mut _);
    unsafe {
        if !IsWindow(Some(h)).as_bool()
            || !IsWindowVisible(h).as_bool()
            || IsIconic(h).as_bool()
            || IsZoomed(h).as_bool()
        {
            return None;
        }
        let mut r = RECT::default();
        GetWindowRect(h, &mut r).ok()?;
        Some(Platform {
            hwnd,
            left: r.left,
            top: r.top,
            right: r.right,
        })
    }
}

/// Ön plandaki pencere bulunduğu monitörü tamamen kaplıyorsa (oyun, video, sunum) `true`.
pub fn fullscreen_app_active() -> bool {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.is_invalid() {
            return false;
        }
        let cls = class_name(fg);
        if matches!(cls.as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd") {
            return false;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(fg, Some(&mut pid));
        if pid == std::process::id() {
            return false;
        }
        let mut r = RECT::default();
        if GetWindowRect(fg, &mut r).is_err() {
            return false;
        }
        let rect = crate::desktop::Rect::from_ltrb(r.left, r.top, r.right, r.bottom);
        crate::desktop::monitors::all().iter().any(|m| {
            let b = m.bounds;
            rect.x <= b.x
                && rect.y <= b.y
                && rect.x + rect.width >= b.x + b.width
                && rect.y + rect.height >= b.y + b.height
        })
    }
}

/// Yerel sağ tık menüsü öğesi.
pub enum MenuItem {
    Item(u32, String, bool),
    Sub(String, Vec<MenuItem>),
    Separator,
}

/// Menüyü gösterir, seçilen öğenin kimliğini döndürür (iptalde 0).
pub fn popup_menu(owner: isize, x: i32, y: i32, items: &[MenuItem]) -> u32 {
    unsafe fn build(items: &[MenuItem]) -> HMENU {
        unsafe {
            let m = CreatePopupMenu().unwrap_or_default();
            for it in items {
                match it {
                    MenuItem::Item(id, text, enabled) => {
                        let flags = if *enabled {
                            MF_STRING
                        } else {
                            MF_STRING | MF_GRAYED
                        };
                        let _ = AppendMenuW(m, flags, *id as usize, &HSTRING::from(text.as_str()));
                    }
                    MenuItem::Sub(text, sub) => {
                        let s = build(sub);
                        let _ =
                            AppendMenuW(m, MF_POPUP, s.0 as usize, &HSTRING::from(text.as_str()));
                    }
                    MenuItem::Separator => {
                        let _ = AppendMenuW(m, MF_SEPARATOR, 0, PCWSTR::null());
                    }
                }
            }
            m
        }
    }
    unsafe {
        let hwnd = HWND(owner as *mut _);
        let menu = build(items);
        // Menünün dışına tıklanınca kapanabilmesi için sahibi ön plana alınır (belgelenmiş gereklilik).
        let _ = SetForegroundWindow(hwnd);
        let id = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_NONOTIFY,
            x,
            y,
            Some(0),
            hwnd,
            None,
        );
        let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
        let _ = DestroyMenu(menu);
        id.0 as u32
    }
}

/// Motor iş parçacığının mesaj kuyruğunu boşaltır; `false` dönerse WM_QUIT alınmıştır.
pub fn pump() -> bool {
    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            if msg.message == WM_QUIT {
                return false;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    true
}

/// Bir mesaj gelene ya da süre dolana kadar bekler (CPU kullanmadan).
pub fn wait(timeout_ms: Option<u32>) {
    use windows::Win32::System::Threading::INFINITE;
    unsafe {
        MsgWaitForMultipleObjects(None, false, timeout_ms.unwrap_or(INFINITE), QS_ALLINPUT);
    }
}

pub fn current_thread_id() -> u32 {
    unsafe { windows::Win32::System::Threading::GetCurrentThreadId() }
}

/// Motor iş parçacığını uyandırır.
pub fn wake_thread(thread_id: u32) {
    unsafe {
        let _ = PostThreadMessageW(thread_id, WM_APP, WPARAM(0), LPARAM(0));
    }
}
