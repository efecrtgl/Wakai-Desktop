use serde::{Serialize, Serializer};

/// Uygulama genelinde kullanılan hata türü. Ön yüze Türkçe mesaj olarak iletilir.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Veritabanı hatası: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Veri biçimi hatalı: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Dosya hatası: {0}")]
    Io(#[from] std::io::Error),
    #[error("Kayıt bulunamadı.")]
    NotFound,
    #[error("{0}")]
    Invalid(String),
    #[error("{0}")]
    Other(String),
}

impl AppError {
    pub fn invalid(msg: impl Into<String>) -> Self {
        Self::Invalid(msg.into())
    }
}

impl From<tauri::Error> for AppError {
    fn from(e: tauri::Error) -> Self {
        Self::Other(format!("Pencere hatası: {e}"))
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
