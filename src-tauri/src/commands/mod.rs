//! Ön yüze açılan Tauri komutları. Tüm girdiler Rust tarafında doğrulanır.

pub mod app;
pub mod content;
pub mod focus;
pub mod pets;
pub mod profiles;
pub mod services;
pub mod settings;
pub mod shopping;
pub mod updates;
pub mod widgets;

/// Yıkıcı işlemler (geri yükleme, geri dönüş, güncelleme kurulumu) yalnızca Kontrol Merkezi'nden
/// istenebilir; widget pencereleri bu komutları çağıramaz.
pub(crate) fn require_main(window: &tauri::WebviewWindow) -> crate::error::AppResult<()> {
    if window.label() == crate::main_window::LABEL {
        Ok(())
    } else {
        Err(crate::error::AppError::invalid(
            "Bu işlem yalnızca Kontrol Merkezi'nden yapılabilir.",
        ))
    }
}
