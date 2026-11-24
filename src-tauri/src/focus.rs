//! Odak Kedisi (Pomodoro) zamanlayıcısı.
//!
//! Zaman, saniyede bir sayaç azaltarak DEĞİL, bitiş anı (`ends_at_ms`, Unix ms) saklanarak
//! hesaplanır. Böylece arayüz güncellenmese, pencere gizlense ya da bilgisayar uyusa bile
//! kalan süre doğrudur. Aşama bitişini zamanlayıcı iş parçacığı (`scheduler`) yakalar.

use crate::db::Db;
use crate::error::AppResult;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, MutexGuard};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Idle,
    Work,
    ShortBreak,
    LongBreak,
}

impl Phase {
    fn db_kind(self) -> Option<&'static str> {
        match self {
            Phase::Idle => None,
            Phase::Work => Some("work"),
            Phase::ShortBreak => Some("short_break"),
            Phase::LongBreak => Some("long_break"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(default)]
pub struct FocusConfig {
    pub work_minutes: u32,
    pub short_break_minutes: u32,
    pub long_break_minutes: u32,
    pub long_break_every: u32,
    pub auto_start: bool,
    pub notify: bool,
}

impl Default for FocusConfig {
    fn default() -> Self {
        Self {
            work_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            long_break_every: 4,
            auto_start: false,
            notify: true,
        }
    }
}

impl FocusConfig {
    pub fn sanitized(mut self) -> Self {
        self.work_minutes = self.work_minutes.clamp(1, 180);
        self.short_break_minutes = self.short_break_minutes.clamp(1, 60);
        self.long_break_minutes = self.long_break_minutes.clamp(1, 120);
        self.long_break_every = self.long_break_every.clamp(1, 12);
        self
    }

    fn duration_ms(&self, phase: Phase) -> i64 {
        let minutes = match phase {
            Phase::Idle | Phase::Work => self.work_minutes,
            Phase::ShortBreak => self.short_break_minutes,
            Phase::LongBreak => self.long_break_minutes,
        };
        i64::from(minutes) * 60_000
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FocusState {
    pub phase: Phase,
    pub running: bool,
    /// Çalışırken bitiş anı (Unix ms).
    pub ends_at_ms: Option<i64>,
    /// Duraklatılmış ya da başlamaya hazır durumda kalan süre.
    pub remaining_ms: i64,
    pub planned_ms: i64,
    /// Son uzun moladan bu yana tamamlanan çalışma oturumu.
    pub cycle: u32,
    pub completed_today: i64,
    #[serde(skip)]
    session_id: Option<i64>,
}

/// Bir aşama bittiğinde ne olduğu (bildirim metni için).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Completion {
    pub finished: Phase,
    pub next: Phase,
}

pub struct FocusTimer(Mutex<FocusState>);

pub fn now_ms() -> i64 {
    chrono::Local::now().timestamp_millis()
}

fn local_from_ms(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .map(|d| {
            d.with_timezone(&chrono::Local)
                .format("%Y-%m-%dT%H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(crate::db::now_local)
}

impl FocusTimer {
    pub fn new(db: &Db) -> Self {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let completed_today = db
            .with(|c| {
                Ok(c.query_row(
                    "SELECT COUNT(*) FROM focus_sessions WHERE kind = 'work' AND completed = 1 AND started_at >= ?1",
                    [today],
                    |r| r.get(0),
                )?)
            })
            .unwrap_or(0);
        let cfg = FocusConfig::default();
        Self(Mutex::new(FocusState {
            phase: Phase::Idle,
            running: false,
            ends_at_ms: None,
            remaining_ms: cfg.duration_ms(Phase::Work),
            planned_ms: cfg.duration_ms(Phase::Work),
            cycle: 0,
            completed_today,
            session_id: None,
        }))
    }

    fn lock(&self) -> MutexGuard<'_, FocusState> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn state(&self, now: i64) -> FocusState {
        let mut s = self.lock().clone();
        if let Some(end) = s.ends_at_ms {
            s.remaining_ms = (end - now).max(0);
        }
        s
    }

    /// Çalışırken bir sonraki bitiş anı.
    pub fn deadline(&self) -> Option<i64> {
        let s = self.lock();
        if s.running { s.ends_at_ms } else { None }
    }

    fn end_session(db: &Db, s: &mut FocusState, completed: bool, at: i64) -> AppResult<()> {
        if let Some(id) = s.session_id.take() {
            db.with(|c| {
                c.execute(
                    "UPDATE focus_sessions SET ended_at = ?2, completed = ?3 WHERE id = ?1",
                    params![id, local_from_ms(at), completed],
                )?;
                Ok(())
            })?;
        }
        Ok(())
    }

    fn begin(db: &Db, s: &mut FocusState, phase: Phase, now: i64, planned: i64) -> AppResult<()> {
        let kind = phase.db_kind().unwrap_or("work");
        let started = local_from_ms(now);
        let id = db.with(|c| {
            c.execute(
                "INSERT INTO focus_sessions (kind, planned_seconds, started_at) VALUES (?1, ?2, ?3)",
                params![kind, (planned / 1000).max(1), started],
            )?;
            Ok(c.last_insert_rowid())
        })?;
        s.phase = phase;
        s.running = true;
        s.planned_ms = planned;
        s.remaining_ms = planned;
        s.ends_at_ms = Some(now + planned);
        s.session_id = Some(id);
        Ok(())
    }

    /// Başlat ya da devam ettir.
    pub fn start(&self, db: &Db, cfg: &FocusConfig, now: i64) -> AppResult<FocusState> {
        let mut s = self.lock();
        if s.running {
            return Ok(s.clone());
        }
        match (s.phase, s.session_id) {
            // Duraklatılmış bir oturum: kalan süreyle devam.
            (phase, Some(_)) if phase != Phase::Idle => {
                s.running = true;
                s.ends_at_ms = Some(now + s.remaining_ms);
            }
            // Hazır bekleyen aşama (ör. mola) ya da boşta: yeni oturum başlat.
            (phase, _) => {
                let phase = if phase == Phase::Idle {
                    Phase::Work
                } else {
                    phase
                };
                let planned = cfg.duration_ms(phase);
                Self::begin(db, &mut s, phase, now, planned)?;
            }
        }
        Ok(s.clone())
    }

    pub fn pause(&self, now: i64) -> FocusState {
        let mut s = self.lock();
        if let (true, Some(end)) = (s.running, s.ends_at_ms) {
            s.running = false;
            s.remaining_ms = (end - now).max(0);
            s.ends_at_ms = None;
        }
        s.clone()
    }

    pub fn reset(&self, db: &Db, cfg: &FocusConfig, now: i64) -> AppResult<FocusState> {
        let mut s = self.lock();
        Self::end_session(db, &mut s, false, now)?;
        s.phase = Phase::Idle;
        s.running = false;
        s.ends_at_ms = None;
        s.cycle = 0;
        s.planned_ms = cfg.duration_ms(Phase::Work);
        s.remaining_ms = s.planned_ms;
        Ok(s.clone())
    }

    fn advance(
        db: &Db,
        s: &mut FocusState,
        cfg: &FocusConfig,
        now: i64,
        completed: bool,
    ) -> AppResult<Completion> {
        let finished = s.phase;
        Self::end_session(db, s, completed, now)?;
        let next = match finished {
            Phase::Work => {
                if completed {
                    s.cycle += 1;
                    s.completed_today += 1;
                }
                if s.cycle > 0 && s.cycle.is_multiple_of(cfg.long_break_every) {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                }
            }
            Phase::LongBreak => {
                s.cycle = 0;
                Phase::Work
            }
            Phase::ShortBreak | Phase::Idle => Phase::Work,
        };
        let planned = cfg.duration_ms(next);
        if cfg.auto_start {
            Self::begin(db, s, next, now, planned)?;
        } else {
            s.phase = next;
            s.running = false;
            s.ends_at_ms = None;
            s.planned_ms = planned;
            s.remaining_ms = planned;
        }
        Ok(Completion { finished, next })
    }

    /// Mevcut aşamayı tamamlanmamış sayarak sonraki aşamaya geçer.
    pub fn skip(&self, db: &Db, cfg: &FocusConfig, now: i64) -> AppResult<FocusState> {
        let mut s = self.lock();
        if s.phase == Phase::Idle {
            return Ok(s.clone());
        }
        Self::advance(db, &mut s, cfg, now, false)?;
        Ok(s.clone())
    }

    /// Bitiş anı geçtiyse aşamayı tamamlar. Zamanlayıcı iş parçacığı çağırır.
    pub fn complete_if_due(
        &self,
        db: &Db,
        cfg: &FocusConfig,
        now: i64,
    ) -> AppResult<Option<Completion>> {
        let mut s = self.lock();
        match (s.running, s.ends_at_ms) {
            (true, Some(end)) if end <= now => Ok(Some(Self::advance(db, &mut s, cfg, end, true)?)),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> FocusConfig {
        FocusConfig {
            work_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            long_break_every: 2,
            auto_start: false,
            notify: false,
        }
    }

    #[test]
    fn remaining_time_is_computed_from_wall_clock() {
        let db = Db::open_in_memory().unwrap();
        let t = FocusTimer::new(&db);
        let s = t.start(&db, &cfg(), 1_000).unwrap();
        assert_eq!(s.phase, Phase::Work);
        assert_eq!(t.state(1_000 + 60_000).remaining_ms, 24 * 60_000);
    }

    #[test]
    fn pause_and_resume_keep_remaining_time() {
        let db = Db::open_in_memory().unwrap();
        let t = FocusTimer::new(&db);
        t.start(&db, &cfg(), 0).unwrap();
        let p = t.pause(10 * 60_000);
        assert!(!p.running);
        assert_eq!(p.remaining_ms, 15 * 60_000);
        // Duraklatılmışken süre akmaz.
        assert_eq!(t.state(50 * 60_000).remaining_ms, 15 * 60_000);
        let r = t.start(&db, &cfg(), 60 * 60_000).unwrap();
        assert_eq!(r.ends_at_ms, Some(75 * 60_000));
    }

    #[test]
    fn cycle_goes_work_short_work_long() {
        let db = Db::open_in_memory().unwrap();
        let t = FocusTimer::new(&db);
        let c = cfg();
        let mut now = 0;
        t.start(&db, &c, now).unwrap();
        now += 25 * 60_000;
        assert_eq!(
            t.complete_if_due(&db, &c, now).unwrap().unwrap().next,
            Phase::ShortBreak
        );
        t.start(&db, &c, now).unwrap();
        now += 5 * 60_000;
        assert_eq!(
            t.complete_if_due(&db, &c, now).unwrap().unwrap().next,
            Phase::Work
        );
        t.start(&db, &c, now).unwrap();
        now += 25 * 60_000;
        assert_eq!(
            t.complete_if_due(&db, &c, now).unwrap().unwrap().next,
            Phase::LongBreak
        );
        assert_eq!(t.state(now).completed_today, 2);

        let done: i64 = db
            .with(|c| {
                Ok(c.query_row(
                    "SELECT COUNT(*) FROM focus_sessions WHERE completed = 1",
                    [],
                    |r| r.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(done, 3);
    }

    #[test]
    fn not_due_yet_does_nothing_and_skip_is_incomplete() {
        let db = Db::open_in_memory().unwrap();
        let t = FocusTimer::new(&db);
        t.start(&db, &cfg(), 0).unwrap();
        assert!(t.complete_if_due(&db, &cfg(), 1000).unwrap().is_none());
        let s = t.skip(&db, &cfg(), 1000).unwrap();
        assert_eq!(s.phase, Phase::ShortBreak);
        assert_eq!(s.completed_today, 0);
        let r = t.reset(&db, &cfg(), 2000).unwrap();
        assert_eq!(r.phase, Phase::Idle);
    }

    #[test]
    fn auto_start_begins_next_phase() {
        let db = Db::open_in_memory().unwrap();
        let t = FocusTimer::new(&db);
        let mut c = cfg();
        c.auto_start = true;
        t.start(&db, &c, 0).unwrap();
        t.complete_if_due(&db, &c, 25 * 60_000).unwrap();
        let s = t.state(25 * 60_000);
        assert!(s.running);
        assert_eq!(s.phase, Phase::ShortBreak);
        assert_eq!(s.ends_at_ms, Some(30 * 60_000));
    }
}
