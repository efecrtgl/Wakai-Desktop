//! Tema deposu ve içe aktarma güvenlik kapısı.
//!
//! Temalar bildirimseldir (JSON). Yerleşik temalar ön yüzle birlikte gelir
//! (`src/theme/builtin/*.json`) ve buraya da derleme zamanında gömülür; özel temalar
//! `custom_themes` tablosunda saklanır. İçe aktarılan her paket burada katı kurallarla
//! doğrulanır: yalnızca sözleşmedeki (`src/theme/contract.json`) anahtarlar, güvenli CSS
//! değerleri (url()/ifade yok), yalnızca gömülü PNG/WebP/GIF/JPEG görseller. Paket hiçbir
//! dosya yolu içermez; bu yüzden yol geçişi (path traversal) mümkün değildir.

use crate::error::{AppError, AppResult};
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::sync::OnceLock;

const CONTRACT_JSON: &str = include_str!("../../src/theme/contract.json");
pub const BUILTIN_JSON: &[&str] = &[
    include_str!("../../src/theme/builtin/pixel.json"),
    include_str!("../../src/theme/builtin/modern.json"),
    include_str!("../../src/theme/builtin/cozy.json"),
    include_str!("../../src/theme/builtin/mono.json"),
    include_str!("../../src/theme/builtin/y2k.json"),
];
pub const DEFAULT_THEME: &str = "pixel";

struct Contract {
    schema: u64,
    tokens: HashSet<String>,
    fonts: HashSet<String>,
    style: Map<String, Value>,
    enums: Map<String, Value>,
    pack_bytes: usize,
    asset_bytes: usize,
    string_max: usize,
    variants_max: usize,
}

fn contract() -> &'static Contract {
    static C: OnceLock<Contract> = OnceLock::new();
    C.get_or_init(|| {
        let v: Value = serde_json::from_str(CONTRACT_JSON).expect("contract.json");
        let set = |k: &str| -> HashSet<String> {
            v[k].as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default()
        };
        let lim = |k: &str| v["limits"][k].as_u64().unwrap_or(0) as usize;
        let mut enums = Map::new();
        for k in ["ambient", "sounds", "waves", "assets"] {
            enums.insert(k.into(), v[k].clone());
        }
        enums.insert("palette".into(), v["pets"]["palette"].clone());
        enums.insert("floor".into(), v["home"]["floor"].clone());
        enums.insert("wall".into(), v["home"]["wall"].clone());
        Contract {
            schema: v["schema"].as_u64().unwrap_or(1),
            tokens: set("tokens"),
            fonts: set("fonts"),
            style: v["style"].as_object().cloned().unwrap_or_default(),
            enums,
            pack_bytes: lim("pack_bytes"),
            asset_bytes: lim("asset_bytes"),
            string_max: lim("string"),
            variants_max: lim("variants"),
        }
    })
}

fn bad(msg: impl Into<String>) -> AppError {
    AppError::invalid(msg.into())
}

fn in_enum(list: &str, value: &Value) -> bool {
    let c = contract();
    let options = c.enums.get(list).or_else(|| c.style.get(list));
    matches!((options, value.as_str()), (Some(Value::Array(a)), Some(s)) if a.iter().any(|o| o == s))
}

pub fn valid_id(id: &str) -> bool {
    let b = id.as_bytes();
    (2..=40).contains(&b.len())
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

fn valid_version(v: &str) -> bool {
    let parts: Vec<&str> = v.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.len() <= 4 && p.bytes().all(|c| c.is_ascii_digit()))
}

/// CSS belirteç değeri güvenli mi? (ön yüzdeki `safeValue` ile aynı kural)
pub fn safe_value(v: &str) -> bool {
    if v.is_empty() || v.chars().count() > contract().string_max {
        return false;
    }
    let charset = v
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || " #.,%()+-'\"/".contains(c));
    let lower = v.to_ascii_lowercase().replace(' ', "");
    let forbidden = [
        "url(",
        "image-set",
        "expression",
        "attr(",
        "src(",
        "element(",
        "cross-fade",
    ]
    .iter()
    .any(|f| lower.contains(f));
    charset && !forbidden
}

fn safe_font(v: &str) -> bool {
    v.chars().count() <= contract().string_max
        && v.chars()
            .all(|c| c.is_ascii_alphanumeric() || " ,'-".contains(c))
}

fn safe_color(v: &str) -> bool {
    if let Some(hex) = v.strip_prefix('#') {
        return (3..=8).contains(&hex.len()) && hex.bytes().all(|c| c.is_ascii_hexdigit());
    }
    (v.starts_with("rgb(") || v.starts_with("rgba("))
        && v.ends_with(')')
        && v.len() <= 40
        && v.chars()
            .all(|c| c.is_ascii_alphanumeric() || " .,%()".contains(c))
}

fn decode_b64_prefix(s: &str, n: usize) -> Vec<u8> {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0;
    for c in s.bytes() {
        let Some(v) = T.iter().position(|&t| t == c) else {
            break;
        };
        buf = (buf << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            if out.len() >= n {
                break;
            }
        }
    }
    out
}

/// Gömülü görsel: data URI, izinli biçim, boyut sınırı ve dosya imzası (magic bytes).
fn safe_asset(v: &str) -> bool {
    let Some(rest) = v.strip_prefix("data:image/") else {
        return false;
    };
    let Some((kind, data)) = rest.split_once(";base64,") else {
        return false;
    };
    if data.len() > contract().asset_bytes * 4 / 3 + 4
        || !data
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'+' || c == b'/' || c == b'=')
    {
        return false;
    }
    let head = decode_b64_prefix(data, 12);
    match kind {
        "png" => head.starts_with(&[0x89, b'P', b'N', b'G']),
        "gif" => head.starts_with(b"GIF8"),
        "jpeg" => head.starts_with(&[0xFF, 0xD8, 0xFF]),
        "webp" => head.len() >= 12 && &head[0..4] == b"RIFF" && &head[8..12] == b"WEBP",
        _ => false,
    }
}

fn check_tokens(v: &Value, where_: &str) -> AppResult<()> {
    let Some(map) = v.as_object() else {
        return Err(bad(format!("{where_} bir nesne olmalı.")));
    };
    for (k, val) in map {
        if !contract().tokens.contains(k) {
            return Err(bad(format!("{where_}: tanınmayan belirteç \"{k}\".")));
        }
        if !val.as_str().is_some_and(safe_value) {
            return Err(bad(format!("{where_}: \"{k}\" değeri güvenli değil.")));
        }
    }
    Ok(())
}

fn short_text(v: Option<&Value>, max: usize) -> AppResult<String> {
    match v {
        None | Some(Value::Null) => Ok(String::new()),
        Some(Value::String(s)) if s.chars().count() <= max => Ok(s.trim().to_string()),
        _ => Err(bad("Tema metin alanlarından biri çok uzun ya da geçersiz.")),
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ThemeSummary {
    pub id: String,
    pub name: String,
    pub version: String,
}

const TOP_KEYS: &[&str] = &[
    "schema",
    "id",
    "name",
    "version",
    "author",
    "description",
    "min_app",
    "base",
    "allow_accent",
    "fonts",
    "style",
    "tokens",
    "colors",
    "variants",
    "pets",
    "home",
    "ambient",
    "sounds",
    "assets",
];

/// Bir tema paketini doğrular. Herhangi bir güvenlik kuralı ihlalinde paketin tamamı reddedilir.
pub fn validate_pack(text: &str, app_version: &str) -> AppResult<ThemeSummary> {
    let c = contract();
    if text.len() > c.pack_bytes {
        return Err(bad(format!(
            "Tema paketi çok büyük (en fazla {} MB).",
            c.pack_bytes / (1024 * 1024)
        )));
    }
    let v: Value =
        serde_json::from_str(text).map_err(|_| bad("Tema dosyası geçerli bir JSON değil."))?;
    let Some(obj) = v.as_object() else {
        return Err(bad("Tema dosyası bir nesne değil."));
    };
    if let Some(k) = obj.keys().find(|k| !TOP_KEYS.contains(&k.as_str())) {
        return Err(bad(format!("Tema dosyasında tanınmayan alan: \"{k}\".")));
    }
    let schema = obj.get("schema").map_or(Some(0), Value::as_u64);
    match schema {
        Some(s) if s <= c.schema => {}
        Some(_) => {
            return Err(bad(
                "Bu tema daha yeni bir Ivily Desktop sürümü için hazırlanmış.",
            ));
        }
        None => return Err(bad("Tema şema sürümü geçersiz.")),
    }
    let id = obj.get("id").and_then(Value::as_str).unwrap_or_default();
    if !valid_id(id) {
        return Err(bad(
            "Tema kimliği geçersiz (küçük harf, rakam ve tire; 2–40 karakter).",
        ));
    }
    let name = short_text(obj.get("name"), 40)?;
    if name.is_empty() {
        return Err(bad("Tema adı eksik."));
    }
    short_text(obj.get("author"), 60)?;
    short_text(obj.get("description"), 240)?;
    let version = match obj.get("version").and_then(Value::as_str) {
        Some(s) if valid_version(s) => s.to_string(),
        None => "1.0.0".into(),
        _ => return Err(bad("Tema sürümü geçersiz (ör. 1.0.0).")),
    };
    if let Some(min) = obj.get("min_app") {
        let min = min
            .as_str()
            .filter(|s| valid_version(s))
            .ok_or_else(|| bad("min_app geçersiz."))?;
        if crate::recovery::compare(min, app_version).is_gt() {
            return Err(bad(format!(
                "Bu tema Ivily Desktop {min} veya üstünü gerektiriyor."
            )));
        }
    }
    if let Some(b) = obj.get("base")
        && !matches!(b.as_str(), Some("light" | "dark"))
    {
        return Err(bad("base yalnızca light ya da dark olabilir."));
    }
    if let Some(a) = obj.get("allow_accent")
        && !a.is_boolean()
    {
        return Err(bad("allow_accent true ya da false olmalı."));
    }
    for key in ["tokens", "colors"] {
        if let Some(t) = obj.get(key) {
            check_tokens(t, key)?;
        }
    }
    if let Some(f) = obj.get("fonts") {
        let map = f
            .as_object()
            .ok_or_else(|| bad("fonts bir nesne olmalı."))?;
        for (k, val) in map {
            if !c.fonts.contains(k) || !val.as_str().is_some_and(safe_font) {
                return Err(bad(format!("fonts.{k} geçersiz.")));
            }
        }
    }
    if let Some(s) = obj.get("style") {
        let map = s
            .as_object()
            .ok_or_else(|| bad("style bir nesne olmalı."))?;
        for (k, val) in map {
            if !c.style.contains_key(k) || !in_enum(k, val) {
                return Err(bad(format!("style.{k} geçersiz.")));
            }
        }
    }
    if let Some(vs) = obj.get("variants") {
        let map = vs
            .as_object()
            .ok_or_else(|| bad("variants bir nesne olmalı."))?;
        if map.len() > c.variants_max {
            return Err(bad(format!("En fazla {} çeşit olabilir.", c.variants_max)));
        }
        for (vid, var) in map {
            let vo = var
                .as_object()
                .filter(|_| valid_id(vid))
                .ok_or_else(|| bad(format!("Çeşit \"{vid}\" geçersiz.")))?;
            if let Some(k) = vo
                .keys()
                .find(|k| !["name", "base", "tokens"].contains(&k.as_str()))
            {
                return Err(bad(format!("Çeşit \"{vid}\": tanınmayan alan \"{k}\".")));
            }
            short_text(vo.get("name"), 30)?;
            if let Some(b) = vo.get("base")
                && !matches!(b.as_str(), Some("light" | "dark"))
            {
                return Err(bad(format!("Çeşit \"{vid}\": base geçersiz.")));
            }
            if let Some(t) = vo.get("tokens") {
                check_tokens(t, &format!("variants.{vid}"))?;
            }
        }
    }
    if let Some(p) = obj.get("pets") {
        let ok = p.as_object().is_some_and(|m| {
            m.iter()
                .all(|(k, v)| k == "palette" && in_enum("palette", v))
        });
        if !ok {
            return Err(bad("pets alanı geçersiz."));
        }
    }
    if let Some(h) = obj.get("home") {
        let map = h.as_object().ok_or_else(|| bad("home bir nesne olmalı."))?;
        for (k, val) in map {
            let ok = match k.as_str() {
                "floor" | "wall" => in_enum(k, val),
                "colors" => val.as_object().is_some_and(|m| {
                    m.iter().all(|(ck, cv)| {
                        ["floor", "floor-alt", "wall", "wall-alt", "trim", "line"]
                            .contains(&ck.as_str())
                            && cv.as_str().is_some_and(safe_color)
                    })
                }),
                _ => false,
            };
            if !ok {
                return Err(bad(format!("home.{k} geçersiz.")));
            }
        }
    }
    if let Some(a) = obj.get("ambient")
        && !in_enum("ambient", a)
    {
        return Err(bad("ambient geçersiz."));
    }
    if let Some(s) = obj.get("sounds") {
        let map = s
            .as_object()
            .ok_or_else(|| bad("sounds bir nesne olmalı."))?;
        for (k, val) in map {
            let ok = in_enum("sounds", &Value::String(k.clone()))
                && val.as_object().is_some_and(|m| {
                    m.iter().all(|(sk, sv)| match sk.as_str() {
                        "wave" => in_enum("waves", sv),
                        "freq" | "ms" | "gain" => sv.is_number(),
                        _ => false,
                    })
                });
            if !ok {
                return Err(bad(format!("sounds.{k} geçersiz.")));
            }
        }
    }
    if let Some(a) = obj.get("assets") {
        let map = a
            .as_object()
            .ok_or_else(|| bad("assets bir nesne olmalı."))?;
        for (k, val) in map {
            if !in_enum("assets", &Value::String(k.clone()))
                || !val.as_str().is_some_and(safe_asset)
            {
                return Err(bad(format!(
                    "Varlık \"{k}\" geçersiz ya da çok büyük (yalnızca gömülü PNG/WebP/GIF/JPEG)."
                )));
            }
        }
    }
    Ok(ThemeSummary {
        id: id.to_string(),
        name,
        version,
    })
}

pub fn builtin_ids() -> Vec<String> {
    BUILTIN_JSON
        .iter()
        .filter_map(|j| serde_json::from_str::<Value>(j).ok())
        .filter_map(|v| v["id"].as_str().map(String::from))
        .collect()
}

fn builtin_manifest(id: &str) -> Option<Value> {
    BUILTIN_JSON
        .iter()
        .filter_map(|j| serde_json::from_str::<Value>(j).ok())
        .find(|v| v["id"] == id)
}

// ── Depo ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct CustomTheme {
    pub id: String,
    pub name: String,
    pub version: String,
    pub installed_at: String,
    pub manifest: String,
}

pub fn list(conn: &rusqlite::Connection) -> AppResult<Vec<CustomTheme>> {
    let mut st = conn.prepare(
        "SELECT id, name, version, installed_at, manifest FROM custom_themes ORDER BY name COLLATE NOCASE",
    )?;
    let rows = st
        .query_map([], |r| {
            Ok(CustomTheme {
                id: r.get(0)?,
                name: r.get(1)?,
                version: r.get(2)?,
                installed_at: r.get(3)?,
                manifest: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Paketi doğrular ve kaydeder. Aynı kimlikte özel tema varsa ancak `replace` ile güncellenir;
/// yerleşik temaların üzerine asla yazılmaz.
pub fn import(
    conn: &rusqlite::Connection,
    text: &str,
    app_version: &str,
    replace: bool,
) -> AppResult<ThemeSummary> {
    let s = validate_pack(text, app_version)?;
    if builtin_ids().contains(&s.id) {
        return Err(bad(format!(
            "\"{}\" yerleşik bir tema kimliği; içe aktarılan tema farklı bir kimlik kullanmalı.",
            s.id
        )));
    }
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM custom_themes WHERE id = ?1)",
        [&s.id],
        |r| r.get(0),
    )?;
    if exists && !replace {
        return Err(AppError::invalid(format!(
            "\"{}\" kimlikli bir tema zaten yüklü.",
            s.id
        )));
    }
    conn.execute(
        "INSERT INTO custom_themes (id, name, version, manifest) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, version = excluded.version,
           manifest = excluded.manifest",
        rusqlite::params![s.id, s.name, s.version, text],
    )?;
    Ok(s)
}

pub fn delete(conn: &rusqlite::Connection, id: &str) -> AppResult<()> {
    let n = conn.execute("DELETE FROM custom_themes WHERE id = ?1", [id])?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// Temanın bildirimi (yerleşik ya da özel).
pub fn manifest(conn: &rusqlite::Connection, id: &str) -> Option<Value> {
    builtin_manifest(id).or_else(|| {
        conn.query_row(
            "SELECT manifest FROM custom_themes WHERE id = ?1",
            [id],
            |r| r.get::<_, String>(0),
        )
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
    })
}

/// Temanın (ve çeşidinin) koyu olup olmadığı: pencere çerçevesi için.
pub fn is_dark(conn: &rusqlite::Connection, id: &str, variant: &str) -> bool {
    let Some(m) = manifest(conn, id).or_else(|| builtin_manifest(DEFAULT_THEME)) else {
        return false;
    };
    let base = m["variants"][variant]["base"]
        .as_str()
        .or(m["base"].as_str());
    base == Some("dark")
}

/// Masaüstü kedileri için tema paleti dönüşümü.
pub fn pet_palette(conn: &rusqlite::Connection, id: &str) -> String {
    manifest(conn, id)
        .and_then(|m| m["pets"]["palette"].as_str().map(String::from))
        .filter(|p| in_enum("palette", &Value::String(p.clone())))
        .unwrap_or_else(|| "none".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG_1PX: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

    fn pack(extra: &str) -> String {
        format!(r#"{{"schema":1,"id":"deneme","name":"Deneme","version":"1.0.0"{extra}}}"#)
    }

    #[test]
    fn builtins_pass_validation() {
        for j in BUILTIN_JSON {
            validate_pack(j, "1.1.5").unwrap();
        }
        assert_eq!(builtin_ids(), ["pixel", "modern", "cozy", "mono", "y2k"]);
    }

    #[test]
    fn rejects_css_injection_and_urls() {
        for bad_value in [
            r#""red;} body{display:none""#,
            r#""url(https://x)""#,
            r#""URL (x)""#,
            r#""expression(alert(1))""#,
            r#""<script>""#,
            r#""image-set('a.png' 1x)""#,
        ] {
            let p = pack(&format!(r#","tokens":{{"color-bg":{bad_value}}}"#));
            assert!(validate_pack(&p, "1.1.5").is_err(), "{bad_value}");
        }
        assert!(
            validate_pack(
                &pack(r#","tokens":{"color-bg":"linear-gradient(160deg, #fff, rgba(0,0,0,0.5))"}"#),
                "1.1.5"
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_unknown_keys_tokens_and_enums() {
        assert!(validate_pack(&pack(r#","script":"x""#), "1.1.5").is_err());
        assert!(validate_pack(&pack(r#","tokens":{"position":"fixed"}"#), "1.1.5").is_err());
        assert!(validate_pack(&pack(r#","style":{"shape":"blob"}"#), "1.1.5").is_err());
        assert!(validate_pack(&pack(r#","fonts":{"sans":"x;y"}"#), "1.1.5").is_err());
        assert!(validate_pack(&pack(r#","home":{"colors":{"floor":"url(x)"}}"#), "1.1.5").is_err());
    }

    #[test]
    fn version_and_compat_checks() {
        assert!(validate_pack(r#"{"schema":2,"id":"aa","name":"A"}"#, "1.1.5").is_err());
        assert!(validate_pack(&pack(r#","min_app":"9.0.0""#), "1.1.5").is_err());
        assert!(validate_pack(&pack(r#","min_app":"1.1.5""#), "1.1.5").is_ok());
        assert!(validate_pack(r#"{"schema":1,"id":"../x","name":"A"}"#, "1.1.5").is_err());
        assert!(validate_pack(r#"{"schema":1,"id":"Aa","name":"A"}"#, "1.1.5").is_err());
        // Şema 0 (eski biçim, `colors`) kabul edilir; ön yüz taşır.
        assert!(
            validate_pack(
                r##"{"id":"eski","name":"Eski","colors":{"color-bg":"#fff"}}"##,
                "1.1.5"
            )
            .is_ok()
        );
        assert!(validate_pack("[]", "1.1.5").is_err());
        assert!(validate_pack("{bozuk", "1.1.5").is_err());
    }

    #[test]
    fn assets_must_be_real_images() {
        assert!(
            validate_pack(
                &pack(&format!(r#","assets":{{"floor":"{PNG_1PX}"}}"#)),
                "1.1.5"
            )
            .is_ok()
        );
        // PNG diye bildirilmiş ama imzası farklı
        let fake = "data:image/png;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciLz4=";
        assert!(
            validate_pack(
                &pack(&format!(r#","assets":{{"floor":"{fake}"}}"#)),
                "1.1.5"
            )
            .is_err()
        );
        assert!(
            validate_pack(
                &pack(r#","assets":{"floor":"data:image/svg+xml;base64,AAAA"}"#),
                "1.1.5"
            )
            .is_err()
        );
        assert!(
            validate_pack(
                &pack(&format!(r#","assets":{{"evil":"{PNG_1PX}"}}"#)),
                "1.1.5"
            )
            .is_err()
        );
    }

    #[test]
    fn size_limit() {
        let big = format!(
            r#"{{"schema":1,"id":"aa","name":"A","description":"{}"}}"#,
            "x".repeat(3 * 1024 * 1024)
        );
        assert!(validate_pack(&big, "1.1.5").is_err());
    }

    #[test]
    fn store_import_replace_delete() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let p = pack(r#","pets":{"palette":"warm"},"base":"dark""#);
        import(&conn, &p, "1.1.5", false).unwrap();
        assert!(
            import(&conn, &p, "1.1.5", false)
                .unwrap_err()
                .to_string()
                .contains("zaten yüklü")
        );
        import(&conn, &p, "1.1.5", true).unwrap();
        assert_eq!(list(&conn).unwrap().len(), 1);
        assert_eq!(pet_palette(&conn, "deneme"), "warm");
        assert!(is_dark(&conn, "deneme", ""));
        assert!(!is_dark(&conn, "modern", ""));
        assert!(is_dark(&conn, "modern", "dark"));
        // Yerleşik kimlikle içe aktarılamaz.
        let builtin = r#"{"schema":1,"id":"pixel","name":"Sahte"}"#;
        assert!(import(&conn, builtin, "1.1.5", true).is_err());
        delete(&conn, "deneme").unwrap();
        assert!(delete(&conn, "deneme").is_err());
        assert_eq!(pet_palette(&conn, "yok"), "none");
    }
}
