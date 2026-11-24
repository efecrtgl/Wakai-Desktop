//! Widget pencerelerinin oluşturulması. Her widget örneği kendi penceresinde çalışır;
//! tüm pencereler aynı WebView2 ortamını (tarayıcı + GPU süreci) paylaşır.

use crate::desktop::Rect;
use crate::error::AppResult;
use tauri::{AppHandle, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

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
        .title("Wakai Widget")
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .resizable(true)
        .focused(false)
        .visible(false)
        .inner_size(rect.width as f64, rect.height as f64)
        .build()?;
    Ok(window)
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
