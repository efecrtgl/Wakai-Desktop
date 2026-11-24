//! Masaüstü kedi motoru: tek bir iş parçacığı tüm kedi pencerelerinin sahibidir.
//!
//! Zamanlama etkinliğe göre uyarlanır: yürürken/koşarken/düşerken ~30 Hz konum güncellemesi,
//! otururken birkaç saniyede bir kare (göz kırpma), uyurken ~1,6 sn'de bir nefes karesi.
//! Hiçbir kedi hareket etmiyorsa iş parçacığı bir sonraki kare zamanına kadar uyur; kedi yoksa
//! tamamen uyur. Ön yüze yüksek frekanslı IPC gönderilmez; yalnızca etkileşimlerde
//! `pets:changed` olayı yayınlanır.

use super::art::{self, Effect, Render};
use super::behavior::{self, Action, ActivityLevel, Context, Rng};
use super::model::{Activity, FOODS, TOYS};
use super::native::{self, Input, MenuItem, Platform, Sprite};
use super::service;
use super::store::PetStore;
use crate::db::pets::PetRecord;
use crate::events;
use crate::state::AppState;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineSettings {
    pub enabled: bool,
    pub level: ActivityLevel,
    /// Sprite pikseli başına ekran pikseli (monitör ölçeğiyle çarpılır), 2–5
    pub size: u32,
    pub max_active: usize,
    pub hide_fullscreen: bool,
}

pub enum Cmd {
    /// Kedi listesi/yerleri/aksesuarları değişti.
    Reload,
    Settings(EngineSettings),
    /// Kontrol Merkezi'nden gelen etkileşimin animasyonu.
    Animate(i64, Anim),
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum Anim {
    Eat,
    Petted,
    Play(String),
    /// Kediyi imlecin yanına çağır.
    Summon,
}

#[derive(Default)]
pub struct PetEngine {
    tx: Mutex<Option<Sender<Cmd>>>,
    thread: AtomicU32,
}

impl PetEngine {
    pub fn start(&self, app: AppHandle, settings: EngineSettings) {
        let (tx, rx) = mpsc::channel();
        *self.tx.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
        let (id_tx, id_rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("wakai-pets".into())
            .spawn(move || {
                let _ = id_tx.send(native::current_thread_id());
                Engine::new(app, settings).run(rx);
            })
            .expect("kedi motoru başlatılamadı");
        if let Ok(id) = id_rx.recv() {
            self.thread.store(id, Ordering::Release);
        }
    }

    pub fn send(&self, cmd: Cmd) {
        if let Some(tx) = self.tx.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            let _ = tx.send(cmd);
            native::wake_thread(self.thread.load(Ordering::Acquire));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Idle,
    Walk,
    Run,
    Sit,
    Look,
    Sleep,
    Play,
    Approach,
    Jump,
    Fall,
    Carried,
    Petted,
    Eat,
    Wake,
}

impl Mode {
    fn anim(self) -> &'static str {
        match self {
            Mode::Idle => "idle",
            Mode::Walk | Mode::Approach => "walk",
            Mode::Run => "run",
            Mode::Sit => "sit",
            Mode::Look => "look",
            Mode::Sleep => "sleep",
            Mode::Play => "play",
            Mode::Jump => "jump",
            Mode::Fall => "fall",
            Mode::Carried => "carried",
            Mode::Petted => "petted",
            Mode::Eat => "eat",
            Mode::Wake => "wake",
        }
    }

    fn frame_secs(self, frame: usize) -> f64 {
        match self {
            Mode::Walk | Mode::Approach => 0.12,
            Mode::Run => 0.08,
            Mode::Idle => 0.9,
            // Otururken uzun açık göz, kısa göz kırpma
            Mode::Sit => {
                if frame.is_multiple_of(2) {
                    4.0
                } else {
                    0.15
                }
            }
            Mode::Look => 1.2,
            Mode::Sleep => 1.6,
            Mode::Play => 0.25,
            Mode::Petted => 0.35,
            Mode::Eat => 0.3,
            _ => 10.0,
        }
    }

    fn moving(self) -> bool {
        matches!(
            self,
            Mode::Walk | Mode::Run | Mode::Approach | Mode::Jump | Mode::Fall
        )
    }

    fn intensity(self) -> Activity {
        match self {
            Mode::Sleep => Activity::Sleeping,
            Mode::Run | Mode::Play | Mode::Jump => Activity::Awake { intensity: 1.0 },
            Mode::Walk | Mode::Approach | Mode::Fall => Activity::Awake { intensity: 0.5 },
            _ => Activity::Awake { intensity: 0.1 },
        }
    }
}

struct Toy {
    sprite: Sprite,
    x: f64,
    y: f64,
    vx: f64,
    w: i32,
}

struct Pet {
    rec: PetRecord,
    sprite: Sprite,
    scale: i32,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    face_right: bool,
    mode: Mode,
    until: f64,
    frame: usize,
    next_frame: f64,
    platform: Option<Platform>,
    press: Option<(i32, i32)>,
    drag_off: Option<(f64, f64)>,
    last_drag: (f64, f64, f64),
    stroke: f64,
    stroke_last: (i32, f64),
    effect: Effect,
    effect_until: f64,
    toy: Option<Toy>,
    drawn_pos: (i32, i32),
    needs_render: bool,
    rng: Rng,
}

struct Engine {
    app: AppHandle,
    settings: EngineSettings,
    pets: Vec<Pet>,
    t0: Instant,
    hidden: bool,
    next_fullscreen_check: f64,
    next_flush: f64,
    platforms_cache: (f64, Vec<Platform>),
    seed: u64,
}

const GRAVITY: f64 = 2400.0;
const FLUSH_SECS: f64 = 300.0;

impl Engine {
    fn new(app: AppHandle, settings: EngineSettings) -> Self {
        let seed = chrono::Local::now().timestamp_nanos_opt().unwrap_or(1) as u64;
        Self {
            app,
            settings,
            pets: Vec::new(),
            t0: Instant::now(),
            hidden: false,
            next_fullscreen_check: 0.0,
            next_flush: FLUSH_SECS,
            platforms_cache: (-1.0, Vec::new()),
            seed,
        }
    }

    fn now(&self) -> f64 {
        self.t0.elapsed().as_secs_f64()
    }

    fn store(&self) -> tauri::State<'_, PetStore> {
        self.app.state::<PetStore>()
    }

    fn run(mut self, rx: Receiver<Cmd>) {
        self.reload();
        loop {
            let now = self.now();
            self.tick(now);
            native::wait(self.next_wake(now));
            if !native::pump() {
                break;
            }
            let mut quit = false;
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    Cmd::Reload => self.reload(),
                    Cmd::Settings(s) => {
                        let resize = s.size != self.settings.size;
                        self.settings = s;
                        if resize {
                            self.pets.clear();
                        }
                        self.reload();
                    }
                    Cmd::Animate(id, a) => self.animate(id, a),
                    Cmd::Shutdown => quit = true,
                }
            }
            for input in native::drain_inputs() {
                self.input(input);
            }
            if quit {
                break;
            }
        }
        let app = self.app.clone();
        let _ = app.state::<PetStore>().flush(&app.state::<AppState>().db);
        self.pets.clear();
    }

    /// Masaüstünde olması gereken kedileri oluşturur/kaldırır.
    fn reload(&mut self) {
        let wanted: Vec<PetRecord> = if self.settings.enabled {
            self.store()
                .snapshot()
                .into_iter()
                .filter(|p| p.location == "desktop")
                .take(self.settings.max_active)
                .collect()
        } else {
            Vec::new()
        };
        self.pets
            .retain(|p| wanted.iter().any(|w| w.id == p.rec.id));
        let now = self.now();
        for rec in wanted {
            if let Some(p) = self.pets.iter_mut().find(|p| p.rec.id == rec.id) {
                let changed = p.rec.breed != rec.breed
                    || p.rec.acc_head != rec.acc_head
                    || p.rec.acc_neck != rec.acc_neck;
                p.rec = rec;
                if changed {
                    p.needs_render = true;
                }
                continue;
            }
            if let Some(p) = self.spawn(rec, now) {
                self.pets.push(p);
            }
        }
        for p in &self.pets {
            p.sprite.show(!self.hidden);
        }
    }

    fn spawn(&mut self, rec: PetRecord, now: f64) -> Option<Pet> {
        let sprite = Sprite::new(&rec.name)?;
        self.seed = self
            .seed
            .wrapping_add(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(rec.id as u64);
        let mut rng = Rng::new(self.seed);
        let area = primary_work_area();
        let scale = pixel_scale(self.settings.size, area);
        let (cw, ch) = art::canvas_size();
        let (w, _h) = (cw * scale, ch * scale);
        let x = rng.range(area.x as f64 + 40.0, (area.x + area.width - w - 40) as f64);
        // Yukarıdan düşerek gelir.
        let y = (area.y + area.height / 3) as f64;
        let mut p = Pet {
            rec,
            sprite,
            scale,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            face_right: rng.chance(0.5),
            mode: Mode::Fall,
            until: now + 10.0,
            frame: 0,
            next_frame: now,
            platform: None,
            press: None,
            drag_off: None,
            last_drag: (x, y, now),
            stroke: 0.0,
            stroke_last: (0, 0.0),
            effect: Effect::None,
            effect_until: 0.0,
            toy: None,
            drawn_pos: (i32::MIN, i32::MIN),
            needs_render: true,
            rng,
        };
        p.sprite.show(!self.hidden);
        self.render(&mut p);
        Some(p)
    }

    fn render(&self, p: &mut Pet) {
        let accs: Vec<&str> = [p.rec.acc_head.as_deref(), p.rec.acc_neck.as_deref()]
            .into_iter()
            .flatten()
            .collect();
        let (w, h, px) = art::render(&Render {
            anim: p.mode.anim(),
            frame: p.frame,
            breed: &p.rec.breed,
            accessories: &accs,
            effect: p.effect,
            flip: !p.face_right,
            scale: p.scale,
        });
        let (x, y) = (p.x.round() as i32, p.y.round() as i32);
        p.sprite.draw(x, y, w, h, &px);
        p.drawn_pos = (x, y);
        p.needs_render = false;
    }

    fn set_mode(&self, p: &mut Pet, mode: Mode, now: f64, secs: f64) {
        if p.mode != mode {
            self.store().set_activity(p.rec.id, mode.intensity());
        }
        p.mode = mode;
        p.until = now + secs;
        p.frame = 0;
        p.next_frame = now + mode.frame_secs(0);
        p.needs_render = true;
    }

    fn platforms(&mut self, now: f64) -> Vec<Platform> {
        if now - self.platforms_cache.0 > 0.25 {
            self.platforms_cache = (now, native::platforms());
        }
        self.platforms_cache.1.clone()
    }

    /// Kedinin ayağının bastığı yüzey (pencere üstü ya da çalışma alanının altı).
    fn ground_y(p: &Pet) -> f64 {
        let h = p.sprite.h.max(1) as f64;
        match p.platform {
            Some(pl) => pl.top as f64 - h,
            None => {
                let a = work_area_at(p.x as i32 + p.sprite.w / 2, p.y as i32);
                (a.y + a.height) as f64 - h
            }
        }
    }

    fn decide(&mut self, i: usize, now: f64) {
        let (cx, cy) = native::cursor_pos();
        let platforms = self.platforms(now);
        let p = &self.pets[i];
        let needs = self.store().needs(p.rec.id).unwrap_or_default();
        let level = if p.rec.paused {
            ActivityLevel::Paused
        } else {
            self.settings.level
        };
        let center = (p.x + p.sprite.w as f64 / 2.0, p.y + p.sprite.h as f64);
        let near = (cx as f64 - center.0).abs() < 260.0 && (cy as f64 - center.1).abs() < 200.0;
        let reach = climb_target(p, &platforms);
        let ctx = Context {
            personality: p.rec.personality,
            needs,
            level,
            cursor_near: near,
            platform_reachable: reach.is_some(),
        };
        let p = &mut self.pets[i];
        let action = behavior::choose(&ctx, &mut p.rng);
        let secs = behavior::duration(action, p.rec.personality, &mut p.rng);
        let mode = match action {
            Action::Walk => {
                p.face_right = p.rng.chance(0.5);
                Mode::Walk
            }
            Action::Run => {
                p.face_right = p.rng.chance(0.5);
                Mode::Run
            }
            Action::Sit => Mode::Sit,
            Action::Idle => Mode::Idle,
            Action::Look => Mode::Look,
            Action::Sleep => Mode::Sleep,
            Action::Play => Mode::Play,
            Action::Approach => Mode::Approach,
            Action::Climb => {
                if let Some((pl, tx)) = reach {
                    // Hedef pencerenin üstüne doğru parabolik zıplama.
                    let t = 0.6;
                    let target_y = pl.top as f64 - p.sprite.h as f64;
                    p.vx = (tx - p.x) / t;
                    p.vy = (target_y - p.y - 0.5 * GRAVITY * t * t) / t;
                    p.face_right = p.vx >= 0.0;
                    p.platform = None;
                    Mode::Jump
                } else {
                    Mode::Sit
                }
            }
        };
        let mut p = std::mem::replace(&mut self.pets[i], dummy_pet());
        self.set_mode(&mut p, mode, now, secs);
        if mode == Mode::Sleep {
            p.effect = Effect::Zz(0);
        }
        self.pets[i] = p;
    }

    fn tick(&mut self, now: f64) {
        if now >= self.next_flush {
            self.next_flush = now + FLUSH_SECS;
            let _ = self.store().flush(&self.app.state::<AppState>().db);
        }
        if !self.pets.is_empty() && now >= self.next_fullscreen_check {
            self.next_fullscreen_check = now + 2.0;
            let hide = self.settings.hide_fullscreen && native::fullscreen_app_active();
            if hide != self.hidden {
                self.hidden = hide;
                for p in &self.pets {
                    p.sprite.show(!hide);
                }
            }
        }
        if self.hidden {
            return;
        }
        let (cx, _) = native::cursor_pos();
        for i in 0..self.pets.len() {
            let mut p = std::mem::replace(&mut self.pets[i], dummy_pet());
            self.step(&mut p, now, cx);
            self.pets[i] = p;
            if self.pets[i].mode != Mode::Carried
                && self.pets[i].until <= now
                && !self.pets[i].mode.moving()
            {
                self.decide(i, now);
            }
        }
        for i in 0..self.pets.len() {
            let mut p = std::mem::replace(&mut self.pets[i], dummy_pet());
            let pos = (p.x.round() as i32, p.y.round() as i32);
            if p.needs_render {
                self.render(&mut p);
            } else if pos != p.drawn_pos {
                p.sprite.move_to(pos.0, pos.1);
                p.drawn_pos = pos;
            }
            if let Some(t) = &mut p.toy {
                t.sprite.move_to(t.x.round() as i32, t.y.round() as i32);
            }
            self.pets[i] = p;
        }
    }

    fn step(&mut self, p: &mut Pet, now: f64, cursor_x: i32) {
        let dt = 1.0 / 30.0;
        // Kare ilerletme
        if now >= p.next_frame && p.mode != Mode::Carried {
            let n = art::frame_count(p.mode.anim()).max(1);
            p.frame = (p.frame + 1) % n;
            p.next_frame = now + p.mode.frame_secs(p.frame);
            if let Effect::Zz(k) = p.effect {
                p.effect = Effect::Zz(k.wrapping_add(1));
            }
            p.needs_render = true;
        }
        if p.effect == Effect::Heart && now >= p.effect_until {
            p.effect = Effect::None;
            p.needs_render = true;
        }
        if p.mode != Mode::Sleep && matches!(p.effect, Effect::Zz(_)) {
            p.effect = Effect::None;
        }

        // Üstünde durduğu pencereyi takip et.
        if let Some(pl) = p.platform
            && !matches!(p.mode, Mode::Jump | Mode::Fall | Mode::Carried)
        {
            match native::platform_now(pl.hwnd) {
                Some(cur) => {
                    if cur != pl {
                        p.x += (cur.left - pl.left) as f64;
                        p.platform = Some(cur);
                        p.y = Self::ground_y(p);
                    }
                }
                None => {
                    p.platform = None;
                    self.set_mode(p, Mode::Fall, now, 10.0);
                }
            }
        }

        let w = p.sprite.w as f64;
        let speed = 11.0 * p.scale as f64;
        match p.mode {
            Mode::Walk | Mode::Run | Mode::Approach => {
                if p.mode == Mode::Approach {
                    let target = cursor_x as f64 - w / 2.0;
                    if (target - p.x).abs() < w * 0.6 {
                        p.face_right = cursor_x as f64 > p.x + w / 2.0;
                        let secs = p.rng.range(5.0, 12.0);
                        self.set_mode(p, Mode::Sit, now, secs);
                        return;
                    }
                    p.face_right = target > p.x;
                }
                let v = if p.mode == Mode::Run {
                    speed * 3.0
                } else {
                    speed
                };
                p.x += if p.face_right { v * dt } else { -v * dt };
                let (lo, hi) = match p.platform {
                    Some(pl) => (pl.left as f64 - w * 0.2, pl.right as f64 - w * 0.8),
                    None => {
                        let a = work_area_at(p.x as i32 + p.sprite.w / 2, p.y as i32);
                        (a.x as f64, (a.x + a.width) as f64 - w)
                    }
                };
                if p.x < lo || p.x > hi {
                    if p.platform.is_some() && p.rng.chance(0.35) {
                        // Pencerenin kenarından atla.
                        p.platform = None;
                        p.vx = if p.face_right {
                            speed * 2.0
                        } else {
                            -speed * 2.0
                        };
                        p.vy = -200.0;
                        self.set_mode(p, Mode::Fall, now, 10.0);
                    } else {
                        p.x = p.x.clamp(lo, hi);
                        p.face_right = !p.face_right;
                        p.needs_render = true;
                    }
                }
                if now >= p.until {
                    self.set_mode(p, Mode::Sit, now, 0.0);
                }
            }
            Mode::Jump | Mode::Fall => {
                let prev_feet = p.y + p.sprite.h as f64;
                p.vy += GRAVITY * dt;
                p.x += p.vx * dt;
                p.y += p.vy * dt;
                p.vx *= 0.995;
                if p.vy > 0.0 {
                    let feet = p.y + p.sprite.h as f64;
                    let cx = p.x + w / 2.0;
                    let platforms = self.platforms(now);
                    let landing = platforms
                        .iter()
                        .filter(|pl| {
                            (pl.left as f64) < cx
                                && cx < pl.right as f64
                                && (pl.top as f64) >= prev_feet - 1.0
                                && (pl.top as f64) <= feet
                        })
                        .min_by_key(|pl| pl.top)
                        .copied();
                    let a = work_area_at(cx as i32, p.y as i32);
                    let ground = (a.y + a.height) as f64;
                    if let Some(pl) = landing {
                        p.platform = Some(pl);
                    }
                    if landing.is_some() || feet >= ground {
                        if landing.is_none() {
                            p.platform = None;
                        }
                        p.y = Self::ground_y(p);
                        p.vx = 0.0;
                        p.vy = 0.0;
                        let new_scale = pixel_scale(self.settings.size, a);
                        if new_scale != p.scale {
                            p.scale = new_scale;
                        }
                        let secs = p.rng.range(1.0, 3.0);
                        self.set_mode(p, Mode::Sit, now, secs);
                    }
                }
                // Ekranın yanlarından çıkmasın.
                let a = work_area_at(p.x as i32 + p.sprite.w / 2, p.y as i32);
                p.x = p.x.clamp(a.x as f64, (a.x + a.width) as f64 - w);
            }
            Mode::Play => {
                if let Some(t) = &mut p.toy {
                    let toy_cx = t.x + t.w as f64 / 2.0;
                    let cat_front = if p.face_right {
                        p.x + w * 0.85
                    } else {
                        p.x + w * 0.15
                    };
                    // Oyuncağa yakınsa patiyle iter, uzaksa yanına gider.
                    if (toy_cx - cat_front).abs() > w * 0.35 {
                        p.face_right = toy_cx > p.x + w / 2.0;
                        p.x += if p.face_right {
                            speed * dt
                        } else {
                            -speed * dt
                        };
                    } else if p.rng.chance(0.04) {
                        t.vx = if p.face_right { 180.0 } else { -180.0 } * p.rng.range(0.6, 1.4);
                    }
                    t.x += t.vx * dt;
                    t.vx *= 0.94;
                    let a = work_area_at(t.x as i32, t.y as i32);
                    t.x = t.x.clamp(a.x as f64, (a.x + a.width - t.w) as f64);
                }
                if now >= p.until {
                    p.toy = None;
                    self.set_mode(p, Mode::Sit, now, 4.0);
                }
            }
            Mode::Eat | Mode::Petted | Mode::Wake if now >= p.until => {
                let secs = p.rng.range(3.0, 8.0);
                self.set_mode(p, Mode::Sit, now, secs);
            }
            _ => {}
        }
        if !matches!(p.mode, Mode::Jump | Mode::Fall | Mode::Carried) && p.platform.is_none() {
            p.y = Self::ground_y(p);
        }
    }

    /// Bir sonraki uyanma süresi (ms). Hiçbir kedi yoksa sonsuz bekler.
    fn next_wake(&self, now: f64) -> Option<u32> {
        if self.pets.is_empty() {
            return None;
        }
        if self.hidden {
            return Some(2000);
        }
        let mut next = self.next_fullscreen_check.min(self.next_flush);
        for p in &self.pets {
            if p.mode == Mode::Carried {
                continue;
            }
            if p.mode.moving() || p.mode == Mode::Play {
                next = next.min(now + 1.0 / 30.0);
            }
            next = next.min(p.next_frame).min(p.until);
            if p.effect == Effect::Heart {
                next = next.min(p.effect_until);
            }
        }
        Some(((next - now).max(0.005) * 1000.0) as u32)
    }

    fn find(&self, hwnd: isize) -> Option<usize> {
        self.pets.iter().position(|p| p.sprite.raw() == hwnd)
    }

    fn notify(&self) {
        events::broadcast(&self.app, events::PETS_CHANGED, ());
    }

    fn input(&mut self, input: Input) {
        let now = self.now();
        match input {
            Input::Down(h, x, y) => {
                if let Some(i) = self.find(h) {
                    self.pets[i].press = Some((x, y));
                }
            }
            Input::Move(h, x, y, pressed) => {
                let Some(i) = self.find(h) else { return };
                let mut p = std::mem::replace(&mut self.pets[i], dummy_pet());
                if pressed && let Some((px, py)) = p.press {
                    if p.drag_off.is_none() && ((x - px).abs() > 4 || (y - py).abs() > 4) {
                        p.drag_off = Some((px as f64 - p.x, py as f64 - p.y));
                        p.platform = None;
                        p.toy = None;
                        self.set_mode(&mut p, Mode::Carried, now, f64::MAX);
                    }
                    if let Some((ox, oy)) = p.drag_off {
                        p.last_drag = (p.x, p.y, now);
                        p.x = x as f64 - ox;
                        p.y = y as f64 - oy;
                        let pos = (p.x.round() as i32, p.y.round() as i32);
                        if p.needs_render {
                            self.render(&mut p);
                        } else {
                            p.sprite.move_to(pos.0, pos.1);
                            p.drawn_pos = pos;
                        }
                    }
                } else if !pressed {
                    // Okşama: fareyi kedinin üstünde ileri geri gezdirmek.
                    let (lx, lt) = p.stroke_last;
                    if now - lt < 0.25 {
                        p.stroke += (x - lx).abs() as f64;
                    } else {
                        p.stroke = 0.0;
                    }
                    p.stroke_last = (x, now);
                    if p.stroke > 40.0 * p.scale as f64 {
                        p.stroke = 0.0;
                        self.stroke(&mut p, now);
                    }
                }
                self.pets[i] = p;
            }
            Input::Up(h, x, _y) => {
                let Some(i) = self.find(h) else { return };
                let mut p = std::mem::replace(&mut self.pets[i], dummy_pet());
                if p.drag_off.take().is_some() {
                    // Bırakılınca düşer; hızla bırakılırsa biraz fırlar.
                    let (lx, ly, lt) = p.last_drag;
                    let dt = (now - lt).max(1.0 / 60.0);
                    p.vx = ((p.x - lx) / dt).clamp(-900.0, 900.0);
                    p.vy = ((p.y - ly) / dt).clamp(-900.0, 300.0);
                    self.set_mode(&mut p, Mode::Fall, now, 10.0);
                } else if p.press.is_some() {
                    // Kısa tıklama: uyuyorsa yavaşça uyanır, değilse bakar ve mutlu olur.
                    let _ = x;
                    if p.mode == Mode::Sleep {
                        self.set_mode(&mut p, Mode::Wake, now, 1.2);
                    } else {
                        self.stroke(&mut p, now);
                    }
                }
                p.press = None;
                self.pets[i] = p;
            }
            Input::Leave(h) => {
                if let Some(i) = self.find(h) {
                    self.pets[i].stroke = 0.0;
                }
            }
            Input::Menu(h, x, y) => {
                if let Some(i) = self.find(h) {
                    self.context_menu(i, h, x, y);
                }
            }
        }
    }

    fn stroke(&self, p: &mut Pet, now: f64) {
        let state = self.app.state::<AppState>();
        if service::stroke(&state.db, &self.store(), p.rec.id).is_ok() {
            self.notify();
        }
        p.effect = Effect::Heart;
        p.effect_until = now + 1.5;
        // Uyuyan kedi uyanmaz; yalnızca mutlu olur.
        if p.mode != Mode::Sleep && p.mode != Mode::Carried {
            self.set_mode(p, Mode::Petted, now, 2.0);
        }
        p.needs_render = true;
    }

    fn animate(&mut self, id: i64, anim: Anim) {
        let now = self.now();
        let Some(i) = self.pets.iter().position(|p| p.rec.id == id) else {
            return;
        };
        let mut p = std::mem::replace(&mut self.pets[i], dummy_pet());
        match anim {
            Anim::Eat => self.set_mode(&mut p, Mode::Eat, now, 3.0),
            Anim::Petted => {
                p.effect = Effect::Heart;
                p.effect_until = now + 1.5;
                if p.mode != Mode::Sleep {
                    self.set_mode(&mut p, Mode::Petted, now, 2.0);
                }
            }
            Anim::Play(toy) => {
                if let Some(sprite) = Sprite::new("Wakai oyuncak") {
                    let (w, h, px) = art::toy_pixels(&toy, p.scale);
                    let side = if p.face_right { 1.0 } else { -1.0 };
                    let tx = p.x + p.sprite.w as f64 / 2.0 + side * p.sprite.w as f64;
                    let ty = p.y + p.sprite.h as f64 - h as f64;
                    let mut sprite = sprite;
                    sprite.draw(tx as i32, ty as i32, w, h, &px);
                    sprite.show(true);
                    p.toy = Some(Toy {
                        sprite,
                        x: tx,
                        y: ty,
                        vx: 0.0,
                        w,
                    });
                }
                let secs = p.rng.range(8.0, 14.0);
                self.set_mode(&mut p, Mode::Play, now, secs);
            }
            Anim::Summon => {
                let (cx, cy) = native::cursor_pos();
                p.x = cx as f64 - p.sprite.w as f64 / 2.0;
                p.y = (cy - p.sprite.h - 40) as f64;
                p.platform = None;
                self.set_mode(&mut p, Mode::Fall, now, 10.0);
            }
        }
        self.pets[i] = p;
    }

    fn context_menu(&mut self, i: usize, hwnd: isize, x: i32, y: i32) {
        let app = self.app.clone();
        let state = app.state::<AppState>();
        let inv = state
            .db
            .with(|c| crate::db::pets::inventory(c, &service::today()))
            .unwrap_or_default();
        let p = &self.pets[i];
        let id = p.rec.id;
        let sleeping = p.mode == Mode::Sleep;
        let foods: Vec<MenuItem> = FOODS
            .iter()
            .enumerate()
            .map(|(k, f)| {
                let n = inv.get(f.id).copied().unwrap_or(0);
                MenuItem::Item(100 + k as u32, format!("{} ({n})", f.name), n > 0)
            })
            .collect();
        let toys: Vec<MenuItem> = TOYS
            .iter()
            .enumerate()
            .filter(|(_, t)| service::is_unlocked(&state.db, t.unlocked_by))
            .map(|(k, t)| MenuItem::Item(200 + k as u32, t.name.to_string(), true))
            .collect();
        let items = vec![
            MenuItem::Item(1, format!("{}: sev", p.rec.name), true),
            MenuItem::Sub("Besle".into(), foods),
            MenuItem::Sub("Oyna".into(), toys),
            MenuItem::Item(
                2,
                if sleeping {
                    "Uyandır".into()
                } else {
                    "Uyut".into()
                },
                true,
            ),
            MenuItem::Item(
                3,
                if p.rec.paused {
                    "Dolaşmaya devam et".into()
                } else {
                    "Yerinde dursun".into()
                },
                true,
            ),
            MenuItem::Separator,
            MenuItem::Item(4, "Kedi Evi'ne gönder".into(), true),
            MenuItem::Item(5, "Kedileri yönet…".into(), true),
        ];
        let choice = native::popup_menu(hwnd, x, y, &items);
        let now = self.now();
        let store = app.state::<PetStore>();
        let result: crate::error::AppResult<()> = match choice {
            1 => {
                let mut p = std::mem::replace(&mut self.pets[i], dummy_pet());
                self.stroke(&mut p, now);
                self.pets[i] = p;
                Ok(())
            }
            2 => {
                let mut p = std::mem::replace(&mut self.pets[i], dummy_pet());
                if sleeping {
                    self.set_mode(&mut p, Mode::Wake, now, 1.2);
                } else {
                    self.set_mode(&mut p, Mode::Sleep, now, 120.0);
                    p.effect = Effect::Zz(0);
                }
                self.pets[i] = p;
                Ok(())
            }
            3 => store
                .update(id, |r| r.paused = !r.paused)
                .and_then(|_| store.persist(&state.db, id))
                .map(|_| {
                    self.reload();
                }),
            4 => store
                .update(id, |r| r.location = "home".into())
                .and_then(|_| store.persist(&state.db, id))
                .map(|_| {
                    self.reload();
                }),
            5 => {
                crate::main_window::show(&self.app);
                events::broadcast(
                    &self.app,
                    events::NAVIGATE,
                    serde_json::json!({ "section": "pets" }),
                );
                Ok(())
            }
            c @ 100..=199 => FOODS.get((c - 100) as usize).map_or(Ok(()), |f| {
                service::feed(&state.db, &store, id, f.id).map(|_| self.animate(id, Anim::Eat))
            }),
            c @ 200..=299 => TOYS.get((c - 200) as usize).map_or(Ok(()), |t| {
                service::play(&state.db, &store, id, t.id)
                    .map(|_| self.animate(id, Anim::Play(t.id.into())))
            }),
            _ => Ok(()),
        };
        match result {
            Ok(()) => self.notify(),
            Err(e) => {
                // Örn. "Tarçın çok yorgun" — kedi uykuya dalar, kullanıcıya kısa bir bildirim.
                use tauri_plugin_notification::NotificationExt;
                let _ = self
                    .app
                    .notification()
                    .builder()
                    .title("Wakai Desktop")
                    .body(e.to_string())
                    .show();
            }
        }
    }
}

/// `std::mem::replace` için geçici yer tutucu (penceresiz). Hemen geri konur.
fn dummy_pet() -> Pet {
    Pet {
        rec: PetRecord {
            id: -1,
            name: String::new(),
            breed: String::new(),
            personality: super::model::Personality::Calm,
            location: String::new(),
            paused: false,
            acc_head: None,
            acc_neck: None,
            needs: Default::default(),
            needs_at: 0,
        },
        sprite: Sprite::placeholder(),
        scale: 1,
        x: 0.0,
        y: 0.0,
        vx: 0.0,
        vy: 0.0,
        face_right: true,
        mode: Mode::Idle,
        until: 0.0,
        frame: 0,
        next_frame: 0.0,
        platform: None,
        press: None,
        drag_off: None,
        last_drag: (0.0, 0.0, 0.0),
        stroke: 0.0,
        stroke_last: (0, 0.0),
        effect: Effect::None,
        effect_until: 0.0,
        toy: None,
        drawn_pos: (0, 0),
        needs_render: false,
        rng: Rng::new(1),
    }
}

fn primary_work_area() -> crate::desktop::Rect {
    let mons = crate::desktop::monitors::all();
    mons.iter()
        .find(|m| m.primary)
        .or(mons.first())
        .map(|m| m.work_area)
        .unwrap_or(crate::desktop::Rect::new(0, 0, 1920, 1040))
}

fn work_area_at(x: i32, y: i32) -> crate::desktop::Rect {
    crate::desktop::monitors::nearest_work_area(crate::desktop::Rect::new(x, y, 1, 1))
        .unwrap_or_else(primary_work_area)
}

fn pixel_scale(size: u32, area: crate::desktop::Rect) -> i32 {
    let mscale = crate::desktop::monitors::scale_for(area);
    ((size.clamp(2, 5) as f64) * mscale).round().max(1.0) as i32
}

/// Zıplanabilir yükseklikte, yatayda yakın bir pencerenin üstü ve hedef x.
fn climb_target(p: &Pet, platforms: &[Platform]) -> Option<(Platform, f64)> {
    let feet = p.y + p.sprite.h as f64;
    let w = p.sprite.w as f64;
    let max_jump = 70.0 * p.scale as f64;
    platforms
        .iter()
        .filter(|pl| {
            let rise = feet - pl.top as f64;
            rise > 30.0
                && rise < max_jump
                && (pl.right - pl.left) as f64 > w * 2.0
                && Some(pl.hwnd) != p.platform.map(|x| x.hwnd)
        })
        .filter_map(|pl| {
            let tx = (p.x).clamp(pl.left as f64 + 4.0, pl.right as f64 - w - 4.0);
            ((tx - p.x).abs() < 320.0).then_some((*pl, tx))
        })
        .min_by(|a, b| (a.1 - p.x).abs().total_cmp(&(b.1 - p.x).abs()))
}
