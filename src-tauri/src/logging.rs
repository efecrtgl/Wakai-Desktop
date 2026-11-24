//! Hafif günlük kaydı: hata ayıklama derlemesinde stderr'e, her zaman uyarı ve
//! üstünü `wakai.log` dosyasına yazar. Üretimde ayrıntılı günlük tutulmaz.

use log::{Level, LevelFilter, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

/// Günlük dosyası bu boyutu aşarsa başlangıçta sıfırlanır.
const MAX_LOG_BYTES: u64 = 1024 * 1024;

struct Logger {
    file: Mutex<Option<File>>,
    file_level: Level,
}

impl log::Log for Logger {
    fn enabled(&self, m: &Metadata) -> bool {
        m.target().starts_with("wakai") && (cfg!(debug_assertions) || m.level() <= self.file_level)
    }

    fn log(&self, r: &Record) {
        if !self.enabled(r.metadata()) {
            return;
        }
        let line = format!(
            "{} {:<5} {}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            r.level(),
            r.args()
        );
        if cfg!(debug_assertions) {
            eprint!("{line}");
        }
        if r.level() <= self.file_level
            && let Ok(mut guard) = self.file.lock()
            && let Some(f) = guard.as_mut()
        {
            let _ = f.write_all(line.as_bytes());
        }
    }

    fn flush(&self) {}
}

pub fn init(dir: &Path) {
    let path = dir.join("wakai.log");
    if std::fs::metadata(&path)
        .map(|m| m.len() > MAX_LOG_BYTES)
        .unwrap_or(false)
    {
        let _ = std::fs::remove_file(&path);
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok();
    let logger = Logger {
        file: Mutex::new(file),
        file_level: Level::Warn,
    };
    if log::set_boxed_logger(Box::new(logger)).is_ok() {
        log::set_max_level(if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Warn
        });
    }
}
