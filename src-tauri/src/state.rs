//! Uygulama genelinde paylaşılan durum.

use crate::db::Db;

pub struct AppState {
    pub db: Db,
}

use crate::error::{AppError, AppResult};
use tauri::AppHandle;

/// Pencere ve masaüstü katmanı işlemlerini ana (UI) iş parçacığında çalıştırır ve sonucu bekler.
///
/// Neden gerekli: Windows'ta senkron bir komut içinde WebView penceresi oluşturmak kilitlenmeye
/// yol açar; ayrıca pencere işlemleri pencerenin sahibi olan iş parçacığına mesaj gönderir.
/// Bu yüzden pencere oluşturan/taşıyan komutlar `async` tanımlanır ve işi buraya devreder.
pub fn on_main<T: Send + 'static>(
    app: &AppHandle,
    f: impl FnOnce(&AppHandle) -> AppResult<T> + Send + 'static,
) -> AppResult<T> {
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = app.clone();
    app.run_on_main_thread(move || {
        let _ = tx.send(f(&handle));
    })?;
    rx.recv()
        .map_err(|_| AppError::Other("Uygulama yanıt vermedi.".into()))?
}
