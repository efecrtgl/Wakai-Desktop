//! Kedi Evi dünyası: odalar, eşya yerleşimi, envanter, ilerleme, alışkanlıklar, ilişkiler,
//! anılar ve çevrim dışı ilerleme. Çizim ve anlık hareket ön yüzdedir (src/pets/world/);
//! burada yalnızca anlamlı durum değişiklikleri doğrulanır ve kalıcı olarak saklanır.
//!
//! Eşya kataloğu ön yüzle ortaktır: `src/pets/world/catalog.json`.

use crate::db::pets::PetRecord;
use crate::error::{AppError, AppResult};
use crate::pets::model::{FOODS, NeedsDelta, Personality};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::OnceLock;

const CATALOG_JSON: &str = include_str!("../../src/pets/world/catalog.json");

#[derive(Debug, Clone, Deserialize)]
pub struct ItemDef {
    pub name: String,
    pub cat: String,
    pub w: i64,
    pub h: i64,
    #[serde(rename = "use")]
    pub use_: Option<String>,
    #[serde(default)]
    pub walk: bool,
    #[serde(default)]
    pub wall: bool,
    #[serde(default)]
    pub start: i64,
    pub unlock: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct RoomSize {
    w: i64,
    h: i64,
    wall: i64,
}

#[derive(Debug, Deserialize)]
struct RoomDef {
    id: String,
}

#[derive(Debug, Deserialize)]
pub struct Catalog {
    room: RoomSize,
    rooms: Vec<RoomDef>,
    pub items: BTreeMap<String, ItemDef>,
}

pub fn catalog() -> &'static Catalog {
    static C: OnceLock<Catalog> = OnceLock::new();
    C.get_or_init(|| serde_json::from_str(CATALOG_JSON).expect("catalog.json"))
}

fn def(kind: &str) -> AppResult<&'static ItemDef> {
    catalog()
        .items
        .get(kind)
        .ok_or_else(|| AppError::invalid("Bilinmeyen eşya."))
}

fn check_room(room: &str) -> AppResult<()> {
    if catalog().rooms.iter().any(|r| r.id == room) {
        Ok(())
    } else {
        Err(AppError::invalid("Bilinmeyen oda."))
    }
}

/// Dönüşe göre kapladığı alan (tek dönüşlerde genişlik/yükseklik yer değiştirir).
pub fn footprint(d: &ItemDef, rot: i64) -> (i64, i64) {
    if rot % 2 == 1 { (d.h, d.w) } else { (d.w, d.h) }
}

/// Eşya oda sınırları içinde ve doğru katmanda mı? Duvar eşyaları duvar satırlarında,
/// diğerleri zeminde durur; duvar eşyaları döndürülmez.
pub fn fits(d: &ItemDef, tx: i64, ty: i64, rot: i64) -> bool {
    let c = &catalog().room;
    if !(0..=3).contains(&rot) || (d.wall && rot != 0) {
        return false;
    }
    let (w, h) = footprint(d, rot);
    let (y0, y1) = if d.wall { (0, c.wall) } else { (c.wall, c.h) };
    tx >= 0 && tx + w <= c.w && ty >= y0 && ty + h <= y1
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WorldItem {
    pub id: i64,
    pub room: String,
    pub kind: String,
    pub tx: i64,
    pub ty: i64,
    pub rot: i64,
    pub state: Value,
}

/// Engelleyici eşyalar üst üste binemez. Üzerinde yürünebilenler (halı, oyuncaklar) her yere
/// konabilir.
pub fn overlaps(
    items: &[WorldItem],
    d: &ItemDef,
    tx: i64,
    ty: i64,
    rot: i64,
    ignore: Option<i64>,
) -> bool {
    if d.walk {
        return false;
    }
    let (w, h) = footprint(d, rot);
    items.iter().filter(|it| Some(it.id) != ignore).any(|it| {
        let Ok(o) = def(&it.kind) else { return false };
        if o.walk {
            return false;
        }
        let (ow, oh) = footprint(o, it.rot);
        tx < it.tx + ow && it.tx < tx + w && ty < it.ty + oh && it.ty < ty + h
    })
}

// ── Veritabanı ──────────────────────────────────────────────────────────────

fn state_get(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM world_state WHERE key = ?1", [key], |r| {
            r.get(0)
        })
        .optional()?)
}

fn state_set(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO world_state (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn items(conn: &Connection, room: Option<&str>) -> AppResult<Vec<WorldItem>> {
    let mut st = conn.prepare(
        "SELECT id, room, kind, tx, ty, rot, state FROM world_items
         WHERE ?1 IS NULL OR room = ?1 ORDER BY id",
    )?;
    let rows = st
        .query_map([room], |r| {
            let state: String = r.get(6)?;
            Ok(WorldItem {
                id: r.get(0)?,
                room: r.get(1)?,
                kind: r.get(2)?,
                tx: r.get(3)?,
                ty: r.get(4)?,
                rot: r.get(5)?,
                state: serde_json::from_str(&state).unwrap_or_else(|_| json!({})),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn item(conn: &Connection, id: i64) -> AppResult<WorldItem> {
    items(conn, None)?
        .into_iter()
        .find(|i| i.id == id)
        .ok_or(AppError::NotFound)
}

pub fn inventory(conn: &Connection) -> AppResult<BTreeMap<String, i64>> {
    let mut st = conn.prepare("SELECT kind, count FROM world_inventory WHERE count > 0")?;
    let rows = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    Ok(rows)
}

fn add_inventory(conn: &Connection, kind: &str, n: i64) -> AppResult<()> {
    conn.execute(
        "INSERT INTO world_inventory (kind, count) VALUES (?1, MAX(?2, 0))
         ON CONFLICT(kind) DO UPDATE SET count = MAX(count + ?2, 0)",
        params![kind, n],
    )?;
    Ok(())
}

fn insert_item(
    conn: &Connection,
    room: &str,
    kind: &str,
    tx: i64,
    ty: i64,
    rot: i64,
) -> AppResult<i64> {
    conn.execute(
        "INSERT INTO world_items (room, kind, tx, ty, rot) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![room, kind, tx, ty, rot],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Başlangıç düzeni: boş bir oturma odası yerine küçük, yaşanmış bir oda.
const DEFAULT_LAYOUT: &[(&str, i64, i64)] = &[
    ("window", 3, 1),
    ("painting", 13, 1),
    ("rug", 8, 7),
    ("bed", 2, 9),
    ("tree", 16, 6),
    ("scratch", 14, 5),
    ("bowl", 5, 5),
    ("water", 6, 5),
    ("plant", 0, 4),
    ("lamp", 19, 4),
    ("ball", 10, 8),
];

/// Dünyayı ilk kez hazırlar (tek seferlik, tek işlem). 1.1.0 Kedi Evi eşyaları oturma
/// odasına taşınır; eski tablo silinmez (önceki sürüme dönüşte o düzen kullanılır).
pub fn init(conn: &Connection) -> AppResult<()> {
    if state_get(conn, "initialized")?.is_some() {
        return Ok(());
    }
    let c = &catalog().room;
    let mut placed: BTreeMap<String, i64> = BTreeMap::new();
    let mut old = conn.prepare("SELECT kind, x, y FROM pet_home_items ORDER BY id")?;
    let legacy = old
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut existing = Vec::new();
    for (kind, x, y) in &legacy {
        let Ok(d) = def(kind) else { continue };
        // Eski oda 320×180 piksel, konum eşyanın alt ortası; yeni oda 16 px karolar.
        let tx = (x / 16 - d.w / 2).clamp(0, c.w - d.w);
        let ty = (y * c.h / 180 - d.h).clamp(c.wall, c.h - d.h);
        if overlaps(&existing, d, tx, ty, 0, None) {
            continue;
        }
        let id = insert_item(conn, "living", kind, tx, ty, 0)?;
        existing.push(WorldItem {
            id,
            room: "living".into(),
            kind: kind.clone(),
            tx,
            ty,
            rot: 0,
            state: json!({}),
        });
        *placed.entry(kind.clone()).or_default() += 1;
    }
    if legacy.is_empty() {
        for (kind, tx, ty) in DEFAULT_LAYOUT {
            let d = def(kind)?;
            if fits(d, *tx, *ty, 0) && !overlaps(&existing, d, *tx, *ty, 0, None) {
                let id = insert_item(conn, "living", kind, *tx, *ty, 0)?;
                existing.push(WorldItem {
                    id,
                    room: "living".into(),
                    kind: kind.to_string(),
                    tx: *tx,
                    ty: *ty,
                    rot: 0,
                    state: json!({}),
                });
                *placed.entry(kind.to_string()).or_default() += 1;
            }
        }
    }
    for (kind, d) in &catalog().items {
        let left = d.start - placed.get(kind).copied().unwrap_or(0);
        if left > 0 {
            add_inventory(conn, kind, left)?;
        }
    }
    state_set(conn, "initialized", "1")?;
    Ok(())
}

// ── İlerleme ────────────────────────────────────────────────────────────────

/// Ev seviyesi: 1 + ⌊√(xp / 40)⌋ (2: 40 xp, 3: 160, 4: 360, 5: 640 …).
pub fn level_for(xp: i64) -> i64 {
    1 + ((xp.max(0) as f64 / 40.0).sqrt().floor() as i64)
}

pub fn xp_for_level(level: i64) -> i64 {
    40 * (level - 1).max(0).pow(2)
}

fn household_xp(conn: &Connection) -> AppResult<i64> {
    Ok(state_get(conn, "xp")?
        .and_then(|v| v.parse().ok())
        .unwrap_or(0))
}

/// Seviyeyle açılan eşyaları envantere ekler (her eşya bir kez). Açılan türleri döndürür.
fn grant_unlocks(conn: &Connection, level: i64) -> AppResult<Vec<String>> {
    let granted: Vec<String> = state_get(conn, "granted")?
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default();
    let mut out = Vec::new();
    for (kind, d) in &catalog().items {
        if d.unlock.is_some_and(|l| l <= level) && !granted.contains(kind) {
            add_inventory(conn, kind, 1)?;
            out.push(kind.clone());
        }
    }
    if !out.is_empty() {
        let all: Vec<&String> = granted.iter().chain(out.iter()).collect();
        state_set(conn, "granted", &serde_json::to_string(&all)?)?;
    }
    Ok(out)
}

pub fn add_memory(conn: &Connection, pet_id: i64, text: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO pet_memories (pet_id, text) VALUES (?1, ?2)",
        params![pet_id, text],
    )?;
    // Kedi başına en fazla 50 anı.
    conn.execute(
        "DELETE FROM pet_memories WHERE pet_id = ?1 AND id NOT IN
           (SELECT id FROM pet_memories WHERE pet_id = ?1 ORDER BY id DESC LIMIT 50)",
        [pet_id],
    )?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct Memory {
    pub at: String,
    pub text: String,
}

pub fn memories(conn: &Connection, pet_id: i64) -> AppResult<Vec<Memory>> {
    let mut st = conn
        .prepare("SELECT at, text FROM pet_memories WHERE pet_id = ?1 ORDER BY id DESC LIMIT 30")?;
    let rows = st
        .query_map([pet_id], |r| {
            Ok(Memory {
                at: r.get(0)?,
                text: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[derive(Debug, Clone, Serialize)]
pub struct Relation {
    pub a: i64,
    pub b: i64,
    pub affinity: f64,
}

pub fn relations(conn: &Connection) -> AppResult<Vec<Relation>> {
    let mut st = conn.prepare(
        "SELECT a, b, affinity FROM pet_relations
         WHERE a IN (SELECT id FROM pets) AND b IN (SELECT id FROM pets)",
    )?;
    let rows = st
        .query_map([], |r| {
            Ok(Relation {
                a: r.get(0)?,
                b: r.get(1)?,
                affinity: r.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// İki kedinin yakınlığını değiştirir (−100…100); yeni değeri döndürür.
fn bump_relation(conn: &Connection, x: i64, y: i64, d: f64) -> AppResult<f64> {
    let (a, b) = if x < y { (x, y) } else { (y, x) };
    conn.execute(
        "INSERT INTO pet_relations (a, b, affinity) VALUES (?1, ?2, MAX(-100, MIN(100, ?3)))
         ON CONFLICT(a, b) DO UPDATE SET affinity = MAX(-100, MIN(100, affinity + ?3))",
        params![a, b, d],
    )?;
    Ok(conn.query_row(
        "SELECT affinity FROM pet_relations WHERE a = ?1 AND b = ?2",
        params![a, b],
        |r| r.get(0),
    )?)
}

/// Alışkanlık sayacını artırır; her grupta en sık 8 kayıt tutulur.
pub fn bump_habit(habits: &mut Value, group: &str, key: &str) {
    if !habits.is_object() {
        *habits = json!({});
    }
    let map = habits
        .as_object_mut()
        .expect("nesne")
        .entry(group)
        .or_insert_with(|| json!({}));
    if !map.is_object() {
        *map = json!({});
    }
    let m = map.as_object_mut().expect("nesne");
    let n = m.get(key).and_then(Value::as_i64).unwrap_or(0) + 1;
    m.insert(key.to_string(), json!(n));
    while m.len() > 8 {
        let Some(min) = m
            .iter()
            .filter(|(k, _)| k.as_str() != key)
            .min_by_key(|(_, v)| v.as_i64().unwrap_or(0))
            .map(|(k, _)| k.clone())
        else {
            break;
        };
        m.remove(&min);
    }
}

/// Saat dilimi: 0 sabah (6–12), 1 öğleden sonra (12–18), 2 akşam (18–23), 3 gece.
pub fn period(hour: u32) -> usize {
    match hour {
        6..=11 => 0,
        12..=17 => 1,
        18..=22 => 2,
        _ => 3,
    }
}

// ── Olaylar ─────────────────────────────────────────────────────────────────

/// Dünyada olan anlamlı bir şey. Ön yüz gönderir; etkileri burada hesaplanır.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Eat,
    Drink,
    Play,
    Pet,
    Brush,
    Carried,
    Groom,
    Scratch,
    Climb,
    Observe,
    Explore,
    Sleep,
    Socialize,
    Hide,
}

impl EventKind {
    /// Aynı kedi için aynı olay en fazla bu sıklıkla ödül verir (sn). Spam ile xp kazanılmaz.
    pub fn cooldown_secs(&self) -> i64 {
        match self {
            // Kap tüketen olaylar kendiliğinden sınırlıdır.
            Self::Eat | Self::Drink => 1,
            Self::Pet | Self::Carried => 3,
            Self::Brush => 6,
            Self::Play | Self::Groom | Self::Scratch | Self::Explore => 8,
            _ => 15,
        }
    }

    fn xp(&self) -> i64 {
        match self {
            Self::Play | Self::Brush | Self::Socialize => 3,
            Self::Pet | Self::Eat | Self::Climb | Self::Observe => 2,
            Self::Carried | Self::Sleep | Self::Hide => 0,
            _ => 1,
        }
    }
}

/// Olayın ihtiyaçlara etkisi. Kişilik ve ikincil özellik etkiyi değiştirir; hiçbir olay
/// sert ceza vermez (taşınmaktan hoşlanmayan kedi en fazla biraz güven kaybeder).
pub fn delta_for(kind: &EventKind, p: Personality, quirk: &str, food: Option<&str>) -> NeedsDelta {
    let shy = quirk == "shy";
    let affectionate = p == Personality::Affectionate;
    let independent = p == Personality::Independent;
    let mut d = NeedsDelta::default();
    match kind {
        EventKind::Eat => {
            let f = FOODS
                .iter()
                .find(|f| Some(f.id) == food)
                .unwrap_or(&FOODS[0]);
            d.satiety = f.satiety;
            d.happiness = f.happiness;
            d.trust = 0.5;
        }
        EventKind::Drink => {
            d.satiety = 4.0;
            d.happiness = 1.0;
        }
        EventKind::Play => {
            d.happiness = if p == Personality::Playful {
                14.0
            } else {
                10.0
            };
            d.energy = -8.0;
            d.social = 3.0;
            d.curiosity = 4.0;
        }
        EventKind::Pet => {
            d.happiness = if affectionate {
                8.0
            } else if independent {
                3.0
            } else {
                5.0
            };
            d.social = if affectionate { 8.0 } else { 5.0 };
            d.trust = if shy { 0.6 } else { 1.2 };
        }
        EventKind::Brush => {
            d.cleanliness = 25.0;
            d.trust = 1.0;
            d.social = 2.0;
            d.happiness = 2.0;
        }
        EventKind::Carried => {
            if shy || independent {
                d.trust = -0.5;
                d.happiness = -1.0;
            } else if affectionate {
                d.social = 3.0;
            }
        }
        EventKind::Groom => d.cleanliness = 15.0,
        EventKind::Scratch => {
            d.happiness = 4.0;
            d.energy = -2.0;
        }
        EventKind::Climb => {
            d.curiosity = 6.0;
            d.energy = -3.0;
            d.happiness = 2.0;
        }
        EventKind::Observe => {
            d.curiosity = 10.0;
            d.happiness = 1.0;
        }
        EventKind::Explore => d.curiosity = 6.0,
        EventKind::Sleep => {}
        EventKind::Socialize => {
            d.social = 8.0;
            d.happiness = 3.0;
        }
        EventKind::Hide => {
            d.happiness = if shy { 3.0 } else { 1.0 };
        }
    }
    d
}

#[derive(Debug, Clone, Serialize)]
pub struct EventOutcome {
    pub pet: PetRecord,
    /// Ödül verildi mi (bekleme süresi dolmadıysa false; etkiler yine de görsel olarak oynar).
    pub rewarded: bool,
    pub level: i64,
    pub household_level: i64,
    pub unlocked: Vec<String>,
    pub memory: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EventInput {
    pub pet_id: i64,
    pub kind: Option<EventKind>,
    pub item_id: Option<i64>,
    pub other_id: Option<i64>,
    /// Uyuma/oturma noktası (alışkanlık için): "oda:tx:ty"
    pub spot: Option<String>,
}

/// Olayın veritabanı tarafı (tek işlem içinde çağrılır). Kedi kaydı `pet` üzerinde değişir.
pub fn apply_event(
    conn: &Connection,
    pet: &mut PetRecord,
    kind: &EventKind,
    input: &EventInput,
    hour: u32,
) -> AppResult<(Vec<String>, Option<String>, i64)> {
    let mut food = None;
    if matches!(kind, EventKind::Eat | EventKind::Drink) {
        let id = input
            .item_id
            .ok_or_else(|| AppError::invalid("Kap belirtilmedi."))?;
        let mut it = item(conn, id)?;
        let key = if matches!(kind, EventKind::Eat) {
            "portions"
        } else {
            "water"
        };
        let left = it.state.get(key).and_then(Value::as_i64).unwrap_or(0);
        if left <= 0 {
            return Err(AppError::invalid("Kap boş."));
        }
        food = it
            .state
            .get("food")
            .and_then(Value::as_str)
            .map(String::from);
        it.state[key] = json!(left - 1);
        conn.execute(
            "UPDATE world_items SET state = ?2 WHERE id = ?1",
            params![id, it.state.to_string()],
        )?;
    }
    pet.needs.apply(delta_for(
        kind,
        pet.personality,
        &pet.quirk,
        food.as_deref(),
    ));

    // Alışkanlıklar
    match kind {
        EventKind::Play => {
            if let Some(id) = input.item_id
                && let Ok(it) = item(conn, id)
            {
                bump_habit(&mut pet.habits, "toys", &it.kind);
            }
        }
        EventKind::Scratch | EventKind::Climb | EventKind::Hide | EventKind::Observe => {
            if let Some(id) = input.item_id
                && let Ok(it) = item(conn, id)
            {
                bump_habit(&mut pet.habits, "furniture", &it.kind);
            }
        }
        EventKind::Sleep => {
            if let Some(spot) = &input.spot
                && spot.len() <= 40
            {
                bump_habit(&mut pet.habits, "spots", spot);
            }
        }
        _ => {}
    }
    if kind.xp() > 0 {
        bump_habit(
            &mut pet.habits,
            "periods",
            ["morning", "afternoon", "evening", "night"][period(hour)],
        );
    }

    // İlişkiler
    let mut memory = None;
    if let (EventKind::Socialize, Some(other)) = (kind, input.other_id)
        && other != pet.id
    {
        let before = relations(conn)?
            .iter()
            .find(|r| (r.a, r.b) == (pet.id.min(other), pet.id.max(other)))
            .map_or(0.0, |r| r.affinity);
        let after = bump_relation(conn, pet.id, other, 3.0)?;
        if before < 30.0 && after >= 30.0 {
            let other_name: String = conn
                .query_row("SELECT name FROM pets WHERE id = ?1", [other], |r| r.get(0))
                .unwrap_or_default();
            let text = format!("{} ile arkadaş oldu.", other_name);
            add_memory(conn, pet.id, &text)?;
            memory = Some(format!("{} {}", pet.name, text));
        }
    }

    // İlerleme
    let gain = kind.xp();
    let mut unlocked = Vec::new();
    let before_pet = level_for(pet.xp);
    pet.xp += gain;
    let after_pet = level_for(pet.xp);
    if after_pet > before_pet {
        let text = format!("{}. seviyeye ulaştı.", after_pet);
        add_memory(conn, pet.id, &text)?;
        memory = Some(format!("{} {}", pet.name, text));
    }
    let hx = household_xp(conn)?;
    let new_hx = hx + gain;
    state_set(conn, "xp", &new_hx.to_string())?;
    let hlevel = level_for(new_hx);
    if hlevel > level_for(hx) {
        unlocked = grant_unlocks(conn, hlevel)?;
    }
    if matches!(kind, EventKind::Play)
        && pet
            .habits
            .get("toys")
            .and_then(Value::as_object)
            .is_some_and(|m| m.values().filter_map(Value::as_i64).sum::<i64>() == 1)
    {
        add_memory(conn, pet.id, "İlk kez oyuncakla oynadı.")?;
    }
    Ok((unlocked, memory, hlevel))
}

// ── Kaplar ──────────────────────────────────────────────────────────────────

/// Mama kabını seçilen yiyecekle (günlük ücretsiz stoktan), su kabını sınırsız doldurur.
pub fn fill(conn: &Connection, id: i64, food: Option<&str>, today: &str) -> AppResult<WorldItem> {
    let mut it = item(conn, id)?;
    match it.kind.as_str() {
        "bowl" => {
            let food = food.unwrap_or("kibble");
            if !FOODS.iter().any(|f| f.id == food) {
                return Err(AppError::invalid("Bilinmeyen yiyecek."));
            }
            crate::db::pets::consume(conn, food, today)?;
            it.state = json!({ "food": food, "portions": 3 });
        }
        "water" => it.state = json!({ "water": 4 }),
        _ => return Err(AppError::invalid("Bu eşya doldurulamaz.")),
    }
    conn.execute(
        "UPDATE world_items SET state = ?2 WHERE id = ?1",
        params![id, it.state.to_string()],
    )?;
    Ok(it)
}

// ── Yerleşim ────────────────────────────────────────────────────────────────

pub fn place(
    conn: &Connection,
    kind: &str,
    room: &str,
    tx: i64,
    ty: i64,
    rot: i64,
) -> AppResult<WorldItem> {
    check_room(room)?;
    let d = def(kind)?;
    if inventory(conn)?.get(kind).copied().unwrap_or(0) <= 0 {
        return Err(AppError::invalid("Envanterde bu eşyadan kalmadı."));
    }
    if !fits(d, tx, ty, rot) {
        return Err(AppError::invalid("Eşya buraya sığmıyor."));
    }
    if overlaps(&items(conn, Some(room))?, d, tx, ty, rot, None) {
        return Err(AppError::invalid("Burada başka bir eşya var."));
    }
    let id = insert_item(conn, room, kind, tx, ty, rot)?;
    add_inventory(conn, kind, -1)?;
    item(conn, id)
}

pub fn move_item(
    conn: &Connection,
    id: i64,
    room: &str,
    tx: i64,
    ty: i64,
    rot: i64,
) -> AppResult<WorldItem> {
    check_room(room)?;
    let it = item(conn, id)?;
    let d = def(&it.kind)?;
    if !fits(d, tx, ty, rot) {
        return Err(AppError::invalid("Eşya buraya sığmıyor."));
    }
    if overlaps(&items(conn, Some(room))?, d, tx, ty, rot, Some(id)) {
        return Err(AppError::invalid("Burada başka bir eşya var."));
    }
    conn.execute(
        "UPDATE world_items SET room = ?2, tx = ?3, ty = ?4, rot = ?5 WHERE id = ?1",
        params![id, room, tx, ty, rot],
    )?;
    item(conn, id)
}

pub fn remove(conn: &Connection, id: i64) -> AppResult<()> {
    let it = item(conn, id)?;
    conn.execute("DELETE FROM world_items WHERE id = ?1", [id])?;
    add_inventory(conn, &it.kind, 1)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct RoomStyle {
    pub id: String,
    pub floor: Option<String>,
    pub wall: Option<String>,
}

pub fn rooms(conn: &Connection) -> AppResult<Vec<RoomStyle>> {
    let mut out = Vec::new();
    for r in &catalog().rooms {
        let (floor, wall) = conn
            .query_row(
                "SELECT floor, wall FROM world_rooms WHERE id = ?1",
                [&r.id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
            .unwrap_or((None, None));
        out.push(RoomStyle {
            id: r.id.clone(),
            floor,
            wall,
        });
    }
    Ok(out)
}

/// Odanın zemin/duvar deseni (boş = temanın varsayılanı). Desenler tema sözleşmesinden gelir.
pub fn set_room_style(
    conn: &Connection,
    room: &str,
    floor: Option<&str>,
    wall: Option<&str>,
) -> AppResult<()> {
    check_room(room)?;
    let contract: Value = serde_json::from_str(include_str!("../../src/theme/contract.json"))?;
    let ok = |list: &str, v: Option<&str>| {
        v.is_none_or(|v| {
            contract["home"][list]
                .as_array()
                .is_some_and(|a| a.iter().any(|o| o == v))
        })
    };
    if !ok("floor", floor) || !ok("wall", wall) {
        return Err(AppError::invalid("Bilinmeyen desen."));
    }
    conn.execute(
        "INSERT INTO world_rooms (id, floor, wall) VALUES (?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET floor = excluded.floor, wall = excluded.wall",
        params![room, floor, wall],
    )?;
    Ok(())
}

// ── Çevrim dışı ilerleme ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Default)]
pub struct Away {
    pub hours: f64,
    pub portions_eaten: i64,
    pub toys_moved: i64,
    pub notes: Vec<String>,
}

/// Dünya en son açıldığından beri geçen süre (en fazla 24 saat hesaba katılır). Her kare
/// ayrı ayrı simüle edilmez: evdeki kediler yaklaşık 6 saatte bir kaptan yer, oyuncu kediler
/// oyuncakları biraz dağıtır. Sonuç belirlenimci ve sınırlıdır.
pub fn offline_progress(
    conn: &Connection,
    now_ms: i64,
    home_pets: &[PetRecord],
) -> AppResult<Option<Away>> {
    let last: Option<i64> = state_get(conn, "last_visit")?.and_then(|v| v.parse().ok());
    state_set(conn, "last_visit", &now_ms.to_string())?;
    let Some(last) = last else { return Ok(None) };
    let hours = ((now_ms - last).max(0) as f64 / 3_600_000.0).min(24.0);
    if hours < 0.5 || home_pets.is_empty() {
        return Ok(None);
    }
    let mut away = Away {
        hours,
        ..Default::default()
    };
    let mut meals = (hours / 6.0 * home_pets.len() as f64).floor() as i64;
    for mut bowl in items(conn, None)?.into_iter().filter(|i| i.kind == "bowl") {
        let left = bowl
            .state
            .get("portions")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        let eat = left.min(meals);
        if eat > 0 {
            bowl.state["portions"] = json!(left - eat);
            conn.execute(
                "UPDATE world_items SET state = ?2 WHERE id = ?1",
                params![bowl.id, bowl.state.to_string()],
            )?;
            meals -= eat;
            away.portions_eaten += eat;
        }
    }
    let playful = home_pets
        .iter()
        .any(|p| p.personality == Personality::Playful || p.quirk == "mischievous");
    if playful && hours >= 1.0 {
        let c = &catalog().room;
        for (n, toy) in items(conn, None)?
            .into_iter()
            .filter(|i| def(&i.kind).is_ok_and(|d| d.cat == "toy"))
            .enumerate()
        {
            let seed = (now_ms / 60_000 + toy.id * 7 + n as i64 * 13).rem_euclid(97);
            let tx = (toy.tx + seed % 7 - 3).clamp(0, c.w - 1);
            let ty = (toy.ty + seed % 5 - 2).clamp(c.wall, c.h - 1);
            if (tx, ty) != (toy.tx, toy.ty) {
                conn.execute(
                    "UPDATE world_items SET tx = ?2, ty = ?3 WHERE id = ?1",
                    params![toy.id, tx, ty],
                )?;
                away.toys_moved += 1;
            }
        }
    }
    if away.portions_eaten > 0 {
        away.notes.push(format!(
            "Sen yokken kediler kaptan {} porsiyon yedi.",
            away.portions_eaten
        ));
    }
    if away.toys_moved > 0 {
        away.notes
            .push("Oyuncaklar biraz dağılmış: birileri oynamış.".into());
    }
    Ok(Some(away))
}

// ── Özet ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct WorldSnapshot {
    pub rooms: Vec<RoomStyle>,
    pub items: Vec<WorldItem>,
    pub inventory: BTreeMap<String, i64>,
    pub xp: i64,
    pub level: i64,
    pub next_level_xp: i64,
    pub relations: Vec<Relation>,
    pub away: Option<Away>,
}

pub fn snapshot(conn: &Connection, away: Option<Away>) -> AppResult<WorldSnapshot> {
    let xp = household_xp(conn)?;
    let level = level_for(xp);
    Ok(WorldSnapshot {
        rooms: rooms(conn)?,
        items: items(conn, None)?,
        inventory: inventory(conn)?,
        xp,
        level,
        next_level_xp: xp_for_level(level + 1),
        relations: relations(conn)?,
        away,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let mut c = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut c).unwrap();
        c
    }

    #[test]
    fn catalog_is_consistent() {
        let c = catalog();
        assert_eq!(c.rooms.len(), 3);
        for (k, d) in &c.items {
            assert!(d.w >= 1 && d.h >= 1, "{k}");
            assert!(
                d.w <= c.room.w
                    && (d.wall && d.h <= c.room.wall || !d.wall && d.h <= c.room.h - c.room.wall),
                "{k}"
            );
            assert!(d.start > 0 || d.unlock.is_some(), "{k} hiç edinilemiyor");
        }
    }

    #[test]
    fn fits_respects_layers_bounds_and_rotation() {
        let bed = def("bed").unwrap();
        assert!(fits(bed, 0, 4, 0));
        assert!(!fits(bed, 19, 4, 0), "sağdan taşar");
        assert!(fits(bed, 19, 4, 1), "dönünce 1×2");
        assert!(!fits(bed, 0, 3, 0), "duvar satırı");
        assert!(!fits(bed, 0, 11, 1), "aşağıdan taşar");
        let win = def("window").unwrap();
        assert!(fits(win, 0, 0, 0));
        assert!(!fits(win, 0, 4, 0), "pencere zemine konmaz");
        assert!(!fits(win, 0, 0, 1), "duvar eşyası dönmez");
        assert!(!fits(bed, 0, 4, 4));
    }

    #[test]
    fn overlap_rules() {
        let c = conn();
        init(&c).unwrap();
        let living = items(&c, Some("living")).unwrap();
        let bed = living.iter().find(|i| i.kind == "bed").unwrap();
        let d = def("cushion").unwrap();
        assert!(overlaps(&living, d, bed.tx, bed.ty, 0, None));
        assert!(!overlaps(&living, d, bed.tx, bed.ty, 0, Some(bed.id)));
        // Halı ve oyuncaklar engel değildir.
        assert!(!overlaps(
            &living,
            def("ball").unwrap(),
            bed.tx,
            bed.ty,
            0,
            None
        ));
    }

    #[test]
    fn init_is_idempotent_and_conserves_items() {
        let c = conn();
        init(&c).unwrap();
        init(&c).unwrap();
        let placed = items(&c, None).unwrap();
        let inv = inventory(&c).unwrap();
        for (k, d) in &catalog().items {
            let total = placed.iter().filter(|i| &i.kind == k).count() as i64
                + inv.get(k).copied().unwrap_or(0);
            assert_eq!(total, d.start, "{k}");
        }
        assert!(placed.len() >= 8, "başlangıç odası dolu olmalı");
    }

    #[test]
    fn legacy_home_items_are_migrated_once() {
        let c = conn();
        c.execute("INSERT INTO pet_home_items (kind, x, y) VALUES ('bed', 60, 150), ('plant', 300, 120), ('bogus', 10, 10)", []).unwrap();
        init(&c).unwrap();
        let placed = items(&c, None).unwrap();
        assert_eq!(placed.len(), 2);
        assert!(
            placed
                .iter()
                .all(|i| fits(def(&i.kind).unwrap(), i.tx, i.ty, i.rot))
        );
        // Eski tablo korunur (önceki sürüme dönüş).
        let old: i64 = c
            .query_row("SELECT COUNT(*) FROM pet_home_items", [], |r| r.get(0))
            .unwrap();
        assert_eq!(old, 3);
        assert_eq!(inventory(&c).unwrap().get("plant"), Some(&1));
    }

    #[test]
    fn place_move_remove_round_trip() {
        let c = conn();
        init(&c).unwrap();
        assert!(place(&c, "plant", "bedroom", 0, 4, 0).is_ok());
        assert!(
            place(&c, "plant", "bedroom", 0, 4, 0).is_err(),
            "envanter bitti ya da dolu"
        );
        assert!(
            place(&c, "sofa", "bedroom", 5, 5, 0).is_err(),
            "kilitli eşya envanterde yok"
        );
        assert!(place(&c, "plant", "mutfak", 0, 4, 0).is_err());
        let it = items(&c, Some("bedroom")).unwrap()[0].clone();
        assert!(move_item(&c, it.id, "playroom", 3, 6, 0).is_ok());
        assert!(move_item(&c, it.id, "playroom", 3, 0, 0).is_err());
        remove(&c, it.id).unwrap();
        assert_eq!(inventory(&c).unwrap().get("plant"), Some(&1));
    }

    #[test]
    fn levels_and_unlocks() {
        assert_eq!(level_for(0), 1);
        assert_eq!(level_for(39), 1);
        assert_eq!(level_for(40), 2);
        assert_eq!(level_for(160), 3);
        assert_eq!(xp_for_level(3), 160);
        let c = conn();
        init(&c).unwrap();
        let got = grant_unlocks(&c, 3).unwrap();
        assert!(got.contains(&"sofa".to_string()) && got.contains(&"mouse".to_string()));
        assert!(grant_unlocks(&c, 3).unwrap().is_empty(), "bir kez verilir");
        assert_eq!(inventory(&c).unwrap().get("sofa"), Some(&1));
    }

    #[test]
    fn habits_are_bounded() {
        let mut h = json!({});
        for i in 0..20 {
            bump_habit(&mut h, "spots", &format!("living:{i}:5"));
        }
        bump_habit(&mut h, "spots", "living:19:5");
        let m = h["spots"].as_object().unwrap();
        assert_eq!(m.len(), 8);
        assert_eq!(m["living:19:5"], 2);
    }

    #[test]
    fn deltas_reflect_personality_without_harsh_penalties() {
        let shy = delta_for(&EventKind::Carried, Personality::Calm, "shy", None);
        assert!(shy.trust < 0.0 && shy.trust >= -1.0);
        let aff = delta_for(&EventKind::Pet, Personality::Affectionate, "none", None);
        let ind = delta_for(&EventKind::Pet, Personality::Independent, "none", None);
        assert!(aff.happiness > ind.happiness);
        let fish = delta_for(&EventKind::Eat, Personality::Calm, "none", Some("fish"));
        assert_eq!(fish.satiety, 35.0);
    }

    #[test]
    fn bowls_and_offline_progress() {
        let c = conn();
        init(&c).unwrap();
        let bowl = items(&c, None)
            .unwrap()
            .into_iter()
            .find(|i| i.kind == "bowl")
            .unwrap();
        fill(&c, bowl.id, Some("fish"), "2026-11-25").unwrap();
        assert!(fill(&c, bowl.id, Some("taş"), "2026-11-25").is_err());
        let now = 1_000_000_000_000;
        assert!(
            offline_progress(&c, now, &[]).unwrap().is_none(),
            "ilk ziyaret"
        );
        let pet =
            crate::db::pets::insert(&c, "Tarçın", "orange", Personality::Playful, now).unwrap();
        // 100 saat sonra: 24 saatle sınırlı; tek kedi → 4 öğün, kapta 3 porsiyon var.
        let away = offline_progress(&c, now + 100 * 3_600_000, &[pet])
            .unwrap()
            .unwrap();
        assert_eq!(away.hours, 24.0);
        assert_eq!(away.portions_eaten, 3);
        let b = items(&c, None)
            .unwrap()
            .into_iter()
            .find(|i| i.id == bowl.id)
            .unwrap();
        assert_eq!(b.state["portions"], 0);
    }

    #[test]
    fn eat_event_consumes_portion_and_grants_xp() {
        let c = conn();
        init(&c).unwrap();
        let mut pet =
            crate::db::pets::insert(&c, "Gece", "black", Personality::Curious, 0).unwrap();
        let bowl = items(&c, None)
            .unwrap()
            .into_iter()
            .find(|i| i.kind == "bowl")
            .unwrap();
        let input = EventInput {
            pet_id: pet.id,
            item_id: Some(bowl.id),
            ..Default::default()
        };
        assert!(
            apply_event(&c, &mut pet, &EventKind::Eat, &input, 9).is_err(),
            "boş kap"
        );
        fill(&c, bowl.id, None, "2026-11-25").unwrap();
        pet.needs.satiety = 40.0;
        apply_event(&c, &mut pet, &EventKind::Eat, &input, 9).unwrap();
        assert_eq!(pet.needs.satiety, 65.0);
        assert_eq!(pet.xp, 2);
        assert_eq!(pet.habits["periods"]["morning"], 1);
        assert_eq!(household_xp(&c).unwrap(), 2);
    }

    #[test]
    fn socializing_builds_friendship_memory() {
        let c = conn();
        let mut a = crate::db::pets::insert(&c, "A", "pink", Personality::Affectionate, 0).unwrap();
        let b = crate::db::pets::insert(&c, "B", "gray", Personality::Calm, 0).unwrap();
        let input = EventInput {
            pet_id: a.id,
            other_id: Some(b.id),
            ..Default::default()
        };
        let mut memory = None;
        for _ in 0..10 {
            memory = apply_event(&c, &mut a, &EventKind::Socialize, &input, 20)
                .unwrap()
                .1
                .or(memory);
        }
        assert_eq!(memory.as_deref(), Some("A B ile arkadaş oldu."));
        assert_eq!(relations(&c).unwrap()[0].affinity, 30.0);
        assert_eq!(memories(&c, a.id).unwrap().len(), 1);
    }
}
