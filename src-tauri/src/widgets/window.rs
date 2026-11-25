//! Widget pencerelerinin oluşturulması. Her widget örneği kendi penceresinde çalışır;
//! tüm pencereler aynı WebView2 ortamını (tarayıcı + GPU süreci) paylaşır.

use crate::desktop::Rect;
use crate::error::AppResult;
use tauri::{AppHandle, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Tüm pencerelerde AYNI olmalıdır (WebView2 ortamı paylaşılır).
/// İlk kısım Tauri/wry'nin varsayılan bayraklarıdır. `--renderer-process-limit` aynı kökenli
/// sayfaların renderer süreçlerini paylaşmasını sağlar; her widget için ayrı bir renderer süreci
/// (~25–40 MB) açılmaz. Ölçüm sonuçları: docs/PERFORMANS.md.
pub const BROWSER_ARGS: &str =
    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --renderer-process-limit=2";

pub fn label_for(id: &str) -> String {
    format!("widget-{id}")
}

pub fn id_from_label(label: &str) -> Option<&str> {
    label.strip_prefix("widget-")
}

/// Gizli bir widget penceresi oluşturur. Pencere masaüstü katmanına bağlandıktan sonra gösterilir.
pub fn create(app: &AppHandle, id: &str, rect: Rect) -> AppResult<WebviewWindow> {
    let url = WebviewUrl::App(format!("widget.html?id={id}").into());
    let window = WebviewWindowBuilder::new(app, label_for(id), url)
        .title("Ivily Widget")
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .resizable(true)
        .focused(false)
        .visible(false)
        .additional_browser_args(BROWSER_ARGS)
        .inner_size(rect.width as f64, rect.height as f64)
        .build()?;
    set_low_memory_target(&window);
    Ok(window)
}

/// Widget WebView'larına düşük bellek hedefi verir: Chromium önbellekleri daha agresif
/// boşaltır. Widget'lar nadiren değiştiği için performans etkisi hissedilmez.
fn set_low_memory_target(window: &WebviewWindow) {
    let result = window.with_webview(|wv| {
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW, ICoreWebView2_19,
        };
        use windows::core::Interface;
        unsafe {
            if let Ok(core) = wv.controller().CoreWebView2()
                && let Ok(v19) = core.cast::<ICoreWebView2_19>()
            {
                let _ = v19.SetMemoryUsageTargetLevel(COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW);
            }
        }
    });
    if let Err(e) = result {
        log::warn!("bellek hedefi ayarlanamadı: {e}");
    }
}

pub fn hwnd_of(window: &WebviewWindow) -> Option<isize> {
    window.hwnd().ok().map(|h| h.0 as isize)
}

/// WebView içeriğinin piksel ölçeğini sabitler. Masaüstü katmanındaki pencere ebeveyninin
/// DPI'ını aldığı için WebView2'nin otomatik algılaması yanlış ölçek seçer; bunun yerine
/// widget'ın bulunduğu monitörün ölçeği elle uygulanır.
pub fn set_raster_scale(window: &WebviewWindow, scale: f64) {
    let result = window.with_webview(move |wv| {
        use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller3;
        use windows::core::Interface;
        unsafe {
            if let Ok(c3) = wv.controller().cast::<ICoreWebView2Controller3>() {
                let _ = c3.SetShouldDetectMonitorScaleChanges(false);
                let _ = c3.SetRasterizationScale(scale);
            }
        }
    });
    if let Err(e) = result {
        log::warn!("WebView ölçeği ayarlanamadı: {e}");
    }
}
