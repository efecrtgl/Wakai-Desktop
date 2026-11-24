//! Merkezi widget kayıt defteri. Widget TANIMLARI burada statik olarak tutulur;
//! widget ÖRNEKLERİ veritabanındadır. Yeni bir widget türü eklemek için:
//! 1. Buraya bir `WidgetDefinition` ekleyin,
//! 2. `src/widgets/registry.js` içine ön yüz bileşenini kaydedin.

use crate::error::{AppError, AppResult};
use serde::Serialize;
use serde_json::{Map, Value, json};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Time,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct WidgetDefinition {
    /// Benzersiz tür kimliği (veritabanında `kind`).
    pub kind: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: Category,
    /// Mantıksal piksel (monitör ölçeğiyle çarpılır).
    pub default_size: Size,
    pub min_size: Size,
    /// Aynı türden birden fazla örnek oluşturulabilir mi?
    pub multi_instance: bool,
    /// Desteklenen etkileşimler (Kontrol Merkezi'nde gösterilir).
    pub interactions: &'static [&'static str],
    /// Kedi pozu (widget kartlarında ve mağazada).
    pub cat: &'static str,
    #[serde(skip)]
    default_config: fn() -> Value,
}

impl WidgetDefinition {
    pub fn default_config(&self) -> Value {
        (self.default_config)()
    }

    /// Bir yapılandırma yamasını doğrular ve mevcut yapılandırmayla birleştirir.
    /// Yalnızca varsayılan yapılandırmada bulunan anahtarlar, aynı JSON türüyle kabul edilir.
    pub fn merge_config(&self, current: &Value, patch: &Value) -> AppResult<Value> {
        let defaults = self.default_config();
        let Value::Object(patch) = patch else {
            return Err(AppError::invalid("Widget ayarı bir nesne olmalıdır."));
        };
        let mut merged: Map<String, Value> = match (defaults.clone(), current) {
            (Value::Object(mut d), Value::Object(c)) => {
                for (k, v) in c {
                    if d.get(k).is_some_and(|dv| same_type(dv, v)) {
                        d.insert(k.clone(), v.clone());
                    }
                }
                d
            }
            (Value::Object(d), _) => d,
            _ => Map::new(),
        };
        for (k, v) in patch {
            match defaults.get(k) {
                Some(dv) if same_type(dv, v) => {
                    merged.insert(k.clone(), v.clone());
                }
                Some(_) => {
                    return Err(AppError::invalid(format!(
                        "'{k}' ayarı için geçersiz değer."
                    )));
                }
                None => {
                    return Err(AppError::invalid(format!(
                        "Bilinmeyen widget ayarı: '{k}'."
                    )));
                }
            }
        }
        Ok(Value::Object(merged))
    }
}

fn same_type(a: &Value, b: &Value) -> bool {
    matches!(
        (a, b),
        (Value::Bool(_), Value::Bool(_))
            | (Value::Number(_), Value::Number(_))
            | (Value::String(_), Value::String(_))
            | (Value::Array(_), Value::Array(_))
            | (Value::Object(_), Value::Object(_))
            | (Value::Null, _)
            | (_, Value::Null)
    )
}

const fn size(width: i32, height: i32) -> Size {
    Size { width, height }
}

pub static DEFINITIONS: &[WidgetDefinition] = &[WidgetDefinition {
    kind: "clock",
    name: "Saat Kedisi",
    description: "Dijital ya da analog saat, tarih ve ek saat dilimleri.",
    category: Category::Time,
    default_size: size(280, 150),
    min_size: size(180, 100),
    multi_instance: true,
    interactions: &["sürükle", "boyutlandır", "ayarlar"],
    cat: "sitting",
    default_config: || {
        json!({
            "mode": "digital",
            "show_seconds": false,
            "hour12": false,
            "show_date": true,
            "time_zones": []
        })
    },
}];

pub fn find(kind: &str) -> Option<&'static WidgetDefinition> {
    DEFINITIONS.iter().find(|d| d.kind == kind)
}

pub fn require(kind: &str) -> AppResult<&'static WidgetDefinition> {
    find(kind).ok_or_else(|| AppError::invalid(format!("Bilinmeyen widget türü: '{kind}'.")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_are_unique_and_sizes_sane() {
        let mut kinds: Vec<_> = DEFINITIONS.iter().map(|d| d.kind).collect();
        kinds.sort();
        kinds.dedup();
        assert_eq!(kinds.len(), DEFINITIONS.len());
        for d in DEFINITIONS {
            assert!(
                d.default_size.width >= d.min_size.width
                    && d.default_size.height >= d.min_size.height
            );
            assert!(d.default_config().is_object());
        }
    }

    #[test]
    fn merge_accepts_known_keys_and_rejects_others() {
        let d = require("clock").unwrap();
        let cur = d.default_config();
        let next = d
            .merge_config(&cur, &json!({ "show_seconds": true }))
            .unwrap();
        assert_eq!(next["show_seconds"], json!(true));
        assert_eq!(next["mode"], json!("digital"));
        assert!(
            d.merge_config(&cur, &json!({ "show_seconds": "evet" }))
                .is_err()
        );
        assert!(d.merge_config(&cur, &json!({ "hack": 1 })).is_err());
        assert!(d.merge_config(&cur, &json!(5)).is_err());
    }

    #[test]
    fn merge_drops_stale_keys_from_current() {
        let d = require("clock").unwrap();
        let cur = json!({ "old_key": 1, "hour12": true });
        let next = d.merge_config(&cur, &json!({})).unwrap();
        assert!(next.get("old_key").is_none());
        assert_eq!(next["hour12"], json!(true));
    }
}
