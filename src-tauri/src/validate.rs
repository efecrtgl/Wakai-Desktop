//! Ön yüzden gelen girdilerin doğrulanması. Hata mesajları Türkçedir.

use crate::error::{AppError, AppResult};
use chrono::{NaiveDate, NaiveDateTime};

pub const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M";
pub const DATE_FORMAT: &str = "%Y-%m-%d";
pub const LABEL_COLORS: &[&str] = &["pink", "lavender", "peach", "mint", "sky", "butter"];

/// Metni kırpar ve uzunluğunu doğrular.
pub fn text(field: &str, value: &str, max: usize, required: bool) -> AppResult<String> {
    let v = value.trim();
    if required && v.is_empty() {
        return Err(AppError::invalid(format!("{field} boş olamaz.")));
    }
    if v.chars().count() > max {
        return Err(AppError::invalid(format!(
            "{field} en fazla {max} karakter olabilir."
        )));
    }
    Ok(v.to_string())
}

/// Uzun gövde metni (not içeriği gibi): kırpılmaz, yalnızca uzunluğu sınırlanır.
pub fn body(field: &str, value: &str, max: usize) -> AppResult<String> {
    if value.chars().count() > max {
        return Err(AppError::invalid(format!(
            "{field} çok uzun (en fazla {max} karakter)."
        )));
    }
    Ok(value.to_string())
}

pub fn datetime(field: &str, value: &str) -> AppResult<NaiveDateTime> {
    NaiveDateTime::parse_from_str(value, DATETIME_FORMAT)
        .map_err(|_| AppError::invalid(format!("{field} geçerli bir tarih ve saat değil.")))
}

pub fn opt_datetime(field: &str, value: Option<&str>) -> AppResult<Option<String>> {
    match value.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => Ok(Some(
            datetime(field, s)?.format(DATETIME_FORMAT).to_string(),
        )),
        None => Ok(None),
    }
}

pub fn date(field: &str, value: &str) -> AppResult<NaiveDate> {
    NaiveDate::parse_from_str(value, DATE_FORMAT)
        .map_err(|_| AppError::invalid(format!("{field} geçerli bir tarih değil.")))
}

pub fn color(value: &str) -> AppResult<String> {
    if LABEL_COLORS.contains(&value) {
        Ok(value.to_string())
    } else {
        Err(AppError::invalid("Geçersiz renk etiketi."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_rules() {
        assert_eq!(text("Başlık", "  Kedi  ", 10, true).unwrap(), "Kedi");
        assert!(text("Başlık", "   ", 10, true).is_err());
        assert!(text("Başlık", "çok uzun bir başlık", 5, false).is_err());
        // Türkçe karakterler bayt değil karakter olarak sayılır.
        assert!(text("Başlık", "ğüşıöç", 6, true).is_ok());
    }

    #[test]
    fn datetime_rules() {
        assert!(datetime("Başlangıç", "2026-11-24T09:30").is_ok());
        assert!(datetime("Başlangıç", "2026-02-30T09:30").is_err());
        assert!(datetime("Başlangıç", "24.11.2026 09:30").is_err());
        assert_eq!(opt_datetime("x", Some("")).unwrap(), None);
        assert_eq!(
            opt_datetime("x", Some("2026-01-01T08:05"))
                .unwrap()
                .as_deref(),
            Some("2026-01-01T08:05")
        );
    }

    #[test]
    fn color_rules() {
        assert!(color("mint").is_ok());
        assert!(color("#ff0000").is_err());
    }
}
