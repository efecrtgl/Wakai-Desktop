//! Hava Durumu Kedisi. Sağlayıcılar `WeatherProvider` arayüzünü uygular; varsayılan
//! sağlayıcı API anahtarı gerektirmeyen Open-Meteo'dur (https://open-meteo.com).
//!
//! - HTTP istekleri Windows'un yerleşik WinHTTP yığınıyla yapılır (`crate::http`; ek TLS kütüphanesi yok).
//! - Sonuçlar SQLite'ta önbelleğe alınır; önbellek tazeyse ağa çıkılmaz.
//! - Çevrim dışıyken önbellekteki son veri "eski" işaretiyle döndürülür.

use crate::db::{self, Db};
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Place {
    pub name: String,
    pub admin: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Current {
    pub temperature: f64,
    pub apparent: f64,
    pub humidity: f64,
    pub wind_speed: f64,
    /// WMO hava durumu kodu.
    pub code: i64,
    pub is_day: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Day {
    pub date: String,
    pub code: i64,
    pub max: f64,
    pub min: f64,
    pub precipitation_probability: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Report {
    pub current: Current,
    pub daily: Vec<Day>,
    /// Unix ms
    pub fetched_at: i64,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeatherResult {
    pub report: Report,
    /// Ağdan alınamadı; önbellekteki eski veri gösteriliyor.
    pub stale: bool,
    pub error: Option<String>,
}

pub trait WeatherProvider: Send + Sync {
    fn forecast(&self, latitude: f64, longitude: f64) -> AppResult<Report>;
    fn search(&self, query: &str) -> AppResult<Vec<Place>>;
}

fn http_get(url: String) -> AppResult<String> {
    crate::http::get(&url)
        .map_err(|e| AppError::Other(format!("Hava durumu servisine ulaşılamadı: {e}")))
}

/// URL sorgu parametresi için yüzde kodlama.
fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

pub struct OpenMeteo;

#[derive(Deserialize)]
struct OmForecast {
    current: OmCurrent,
    daily: OmDaily,
}
#[derive(Deserialize)]
struct OmCurrent {
    temperature_2m: f64,
    apparent_temperature: f64,
    relative_humidity_2m: f64,
    wind_speed_10m: f64,
    weather_code: i64,
    is_day: i64,
}
#[derive(Deserialize)]
struct OmDaily {
    time: Vec<String>,
    weather_code: Vec<i64>,
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
    #[serde(default)]
    precipitation_probability_max: Vec<Option<f64>>,
}
#[derive(Deserialize)]
struct OmSearch {
    #[serde(default)]
    results: Vec<OmPlace>,
}
#[derive(Deserialize)]
struct OmPlace {
    name: String,
    #[serde(default)]
    admin1: String,
    #[serde(default)]
    country: String,
    latitude: f64,
    longitude: f64,
}

impl OpenMeteo {
    fn parse_forecast(json: &str, now_ms: i64) -> AppResult<Report> {
        let f: OmForecast = serde_json::from_str(json)
            .map_err(|e| AppError::Other(format!("Hava durumu yanıtı okunamadı: {e}")))?;
        let d = &f.daily;
        let daily = (0..d.time.len())
            .filter_map(|i| {
                Some(Day {
                    date: d.time.get(i)?.clone(),
                    code: *d.weather_code.get(i)?,
                    max: *d.temperature_2m_max.get(i)?,
                    min: *d.temperature_2m_min.get(i)?,
                    precipitation_probability: d
                        .precipitation_probability_max
                        .get(i)
                        .copied()
                        .flatten(),
                })
            })
            .collect();
        Ok(Report {
            current: Current {
                temperature: f.current.temperature_2m,
                apparent: f.current.apparent_temperature,
                humidity: f.current.relative_humidity_2m,
                wind_speed: f.current.wind_speed_10m,
                code: f.current.weather_code,
                is_day: f.current.is_day == 1,
            },
            daily,
            fetched_at: now_ms,
            provider: "Open-Meteo".into(),
        })
    }

    fn parse_search(json: &str) -> AppResult<Vec<Place>> {
        let s: OmSearch = serde_json::from_str(json)
            .map_err(|e| AppError::Other(format!("Konum yanıtı okunamadı: {e}")))?;
        Ok(s.results
            .into_iter()
            .map(|p| Place {
                name: p.name,
                admin: p.admin1,
                country: p.country,
                latitude: p.latitude,
                longitude: p.longitude,
            })
            .collect())
    }
}

impl WeatherProvider for OpenMeteo {
    fn forecast(&self, latitude: f64, longitude: f64) -> AppResult<Report> {
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={latitude:.4}&longitude={longitude:.4}\
             &current=temperature_2m,apparent_temperature,relative_humidity_2m,wind_speed_10m,weather_code,is_day\
             &daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max\
             &timezone=auto&forecast_days=5"
        );
        Self::parse_forecast(&http_get(url)?, chrono::Local::now().timestamp_millis())
    }

    fn search(&self, query: &str) -> AppResult<Vec<Place>> {
        let url = format!(
            "https://geocoding-api.open-meteo.com/v1/search?name={}&count=6&language=tr&format=json",
            encode(query)
        );
        Self::parse_search(&http_get(url)?)
    }
}

fn cache_key(lat: f64, lon: f64) -> String {
    format!("weather:{lat:.2},{lon:.2}")
}

pub fn check_coords(lat: f64, lon: f64) -> AppResult<()> {
    if lat.is_finite()
        && lon.is_finite()
        && (-90.0..=90.0).contains(&lat)
        && (-180.0..=180.0).contains(&lon)
    {
        Ok(())
    } else {
        Err(AppError::invalid("Geçersiz konum."))
    }
}

/// Önbellek tazeyse onu, değilse sağlayıcıdan yenisini döndürür; ağ hatasında eski veriye düşer.
pub fn get(
    db: &Db,
    provider: &dyn WeatherProvider,
    lat: f64,
    lon: f64,
    max_age_minutes: u32,
    force: bool,
) -> AppResult<WeatherResult> {
    check_coords(lat, lon)?;
    let key = cache_key(lat, lon);
    let cached: Option<Report> = db
        .with(|c| db::settings::get(c, &key))?
        .and_then(|v| serde_json::from_value(v).ok());
    let now = chrono::Local::now().timestamp_millis();
    if let Some(r) = &cached
        && !force
        && now - r.fetched_at < i64::from(max_age_minutes) * 60_000
    {
        return Ok(WeatherResult {
            report: r.clone(),
            stale: false,
            error: None,
        });
    }
    match provider.forecast(lat, lon) {
        Ok(report) => {
            let value = serde_json::to_value(&report)?;
            db.with(|c| db::settings::set(c, &key, &value))?;
            Ok(WeatherResult {
                report,
                stale: false,
                error: None,
            })
        }
        Err(e) => match cached {
            Some(report) => Ok(WeatherResult {
                report,
                stale: true,
                error: Some(e.to_string()),
            }),
            None => Err(e),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const SAMPLE: &str = r#"{"current":{"temperature_2m":12.3,"apparent_temperature":10.1,"relative_humidity_2m":71,"wind_speed_10m":9.4,"weather_code":3,"is_day":1},
        "daily":{"time":["2026-11-24","2026-11-25"],"weather_code":[3,61],"temperature_2m_max":[14.0,11.2],"temperature_2m_min":[7.5,6.1],"precipitation_probability_max":[10,null]}}"#;

    struct Fake {
        calls: AtomicUsize,
        fail: bool,
    }
    impl WeatherProvider for Fake {
        fn forecast(&self, _: f64, _: f64) -> AppResult<Report> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                Err(AppError::Other("çevrim dışı".into()))
            } else {
                OpenMeteo::parse_forecast(SAMPLE, chrono::Local::now().timestamp_millis())
            }
        }
        fn search(&self, _: &str) -> AppResult<Vec<Place>> {
            Ok(vec![])
        }
    }

    #[test]
    fn parses_open_meteo_forecast() {
        let r = OpenMeteo::parse_forecast(SAMPLE, 0).unwrap();
        assert_eq!(r.current.code, 3);
        assert!(r.current.is_day);
        assert_eq!(r.daily.len(), 2);
        assert_eq!(r.daily[1].precipitation_probability, None);
    }

    #[test]
    fn parses_search_results_and_empty_response() {
        let p = OpenMeteo::parse_search(r#"{"results":[{"name":"İstanbul","admin1":"İstanbul","country":"Türkiye","latitude":41.01,"longitude":28.97}]}"#).unwrap();
        assert_eq!(p[0].name, "İstanbul");
        assert!(OpenMeteo::parse_search("{}").unwrap().is_empty());
    }

    #[test]
    fn cache_prevents_repeated_requests() {
        let db = Db::open_in_memory().unwrap();
        let p = Fake {
            calls: AtomicUsize::new(0),
            fail: false,
        };
        get(&db, &p, 41.0, 29.0, 30, false).unwrap();
        get(&db, &p, 41.0, 29.0, 30, false).unwrap();
        assert_eq!(p.calls.load(Ordering::SeqCst), 1);
        get(&db, &p, 41.0, 29.0, 30, true).unwrap();
        assert_eq!(p.calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn offline_falls_back_to_stale_cache() {
        let db = Db::open_in_memory().unwrap();
        get(
            &db,
            &Fake {
                calls: AtomicUsize::new(0),
                fail: false,
            },
            41.0,
            29.0,
            30,
            false,
        )
        .unwrap();
        let r = get(
            &db,
            &Fake {
                calls: AtomicUsize::new(0),
                fail: true,
            },
            41.0,
            29.0,
            30,
            true,
        )
        .unwrap();
        assert!(r.stale && r.error.is_some());
        assert!(
            get(
                &db,
                &Fake {
                    calls: AtomicUsize::new(0),
                    fail: true
                },
                10.0,
                10.0,
                30,
                false
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_bad_coordinates_and_encodes_queries() {
        assert!(check_coords(91.0, 0.0).is_err());
        assert!(check_coords(f64::NAN, 0.0).is_err());
        assert_eq!(encode("İzmir Konak"), "%C4%B0zmir%20Konak");
    }
}
