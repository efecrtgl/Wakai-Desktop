//! Sistem Kedisi için kaynak izleme. Yalnızca abone varken ve ayarlardaki aralıkla örnekler.
//!
//! - CPU: `GetSystemTimes` iki örnek arasındaki boşta/çekirdek/kullanıcı süresi farkı.
//! - Bellek: `GlobalMemoryStatusEx`.
//! - GPU: PDH `\GPU Engine(*engtype_3D)\Utilization Percentage` örneklerinin toplamı.
//!   Görev Yöneticisi'nin "3D" motoruna yaklaşık karşılık gelir. Sayaç yoksa/okunamazsa
//!   `None` döner ve arayüz "ölçülemiyor" gösterir — sahte değer üretilmez.
//! - Disk: sistem sürücüsünün doluluğu (`GetDiskFreeSpaceExW`).

use crate::events;
use serde::Serialize;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::FILETIME;
use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
use windows::Win32::System::Performance::{
    PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY, PDH_MORE_DATA,
    PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW,
    PdhOpenQueryW,
};
use windows::Win32::System::SystemInformation::{
    GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX,
};
use windows::Win32::System::Threading::GetSystemTimes;
use windows::core::w;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SystemStats {
    /// 0–100
    pub cpu: Option<f64>,
    pub memory_used: u64,
    pub memory_total: u64,
    pub gpu: Option<f64>,
    pub disk_used: u64,
    pub disk_total: u64,
    pub uptime_secs: u64,
    pub interval_secs: u32,
}

struct CpuSampler {
    last: Option<(u64, u64)>, // (boşta, toplam)
}

fn ft(f: FILETIME) -> u64 {
    (u64::from(f.dwHighDateTime) << 32) | u64::from(f.dwLowDateTime)
}

impl CpuSampler {
    fn sample(&mut self) -> Option<f64> {
        let (mut idle, mut kernel, mut user) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        unsafe { GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)).ok()? };
        // Çekirdek süresi boşta süresini de içerir.
        let (idle, total) = (ft(idle), ft(kernel) + ft(user));
        let result = self.last.and_then(|(pi, pt)| {
            let dt = total.saturating_sub(pt);
            (dt > 0)
                .then(|| (1.0 - idle.saturating_sub(pi) as f64 / dt as f64).clamp(0.0, 1.0) * 100.0)
        });
        self.last = Some((idle, total));
        result
    }
}

/// PDH ile GPU kullanımı. Oluşturulamazsa izleme GPU'suz devam eder.
struct GpuSampler {
    query: PDH_HQUERY,
    counter: PDH_HCOUNTER,
    buf: Vec<u8>,
}

impl GpuSampler {
    fn new() -> Option<Self> {
        let mut query = PDH_HQUERY::default();
        let mut counter = PDH_HCOUNTER::default();
        unsafe {
            if PdhOpenQueryW(None, 0, &mut query) != 0 {
                return None;
            }
            if PdhAddEnglishCounterW(
                query,
                w!("\\GPU Engine(*engtype_3D)\\Utilization Percentage"),
                0,
                &mut counter,
            ) != 0
            {
                PdhCloseQuery(query);
                return None;
            }
            // Oran sayaçları ilk toplamadan sonra değer üretir.
            PdhCollectQueryData(query);
        }
        Some(Self {
            query,
            counter,
            buf: Vec::new(),
        })
    }

    fn sample(&mut self) -> Option<f64> {
        unsafe {
            if PdhCollectQueryData(self.query) != 0 {
                return None;
            }
            let (mut size, mut count) = (0u32, 0u32);
            let status = PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &mut size,
                &mut count,
                None,
            );
            if status != PDH_MORE_DATA || size == 0 {
                return if count == 0 { Some(0.0) } else { None };
            }
            self.buf.resize(size as usize, 0);
            let items = self.buf.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
            if PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &mut size,
                &mut count,
                Some(items),
            ) != 0
            {
                return None;
            }
            // SAFETY: PDH, arabelleğe `count` adet öğe yazdı.
            let slice = std::slice::from_raw_parts(items, count as usize);
            let sum: f64 = slice
                .iter()
                .filter(|i| i.FmtValue.CStatus == 0)
                .map(|i| i.FmtValue.Anonymous.doubleValue)
                .sum();
            Some(sum.clamp(0.0, 100.0))
        }
    }
}

impl Drop for GpuSampler {
    fn drop(&mut self) {
        unsafe {
            PdhCloseQuery(self.query);
        }
    }
}

fn memory() -> (u64, u64) {
    let mut m = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    if unsafe { GlobalMemoryStatusEx(&mut m) }.is_ok() {
        (m.ullTotalPhys - m.ullAvailPhys, m.ullTotalPhys)
    } else {
        (0, 0)
    }
}

fn disk() -> (u64, u64) {
    let (mut free, mut total) = (0u64, 0u64);
    let drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
    let path: Vec<u16> = format!("{drive}\\").encode_utf16().chain(Some(0)).collect();
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            windows::core::PCWSTR(path.as_ptr()),
            Some(&mut free),
            Some(&mut total),
            None,
        )
        .is_ok()
    };
    if ok { (total - free, total) } else { (0, 0) }
}

#[derive(Default)]
struct Shared {
    active: bool,
    running: bool,
    last: Option<SystemStats>,
}

#[derive(Default, Clone)]
pub struct SystemMonitor {
    shared: Arc<(Mutex<Shared>, Condvar)>,
}

impl SystemMonitor {
    pub fn last(&self) -> Option<SystemStats> {
        self.shared
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .last
            .clone()
    }

    /// Abone varken iş parçacığını başlatır/uyandırır; abone kalmayınca örneklemeyi durdurur.
    pub fn set_active(&self, app: &AppHandle, active: bool) {
        let (lock, cv) = &*self.shared;
        let mut s = lock.lock().unwrap_or_else(|e| e.into_inner());
        s.active = active;
        if active && !s.running {
            s.running = true;
            let shared = self.shared.clone();
            let app = app.clone();
            std::thread::Builder::new()
                .name("wakai-monitor".into())
                .spawn(move || run(app, shared))
                .expect("izleme iş parçacığı oluşturulamadı");
        }
        cv.notify_all();
    }
}

fn interval(app: &AppHandle) -> u32 {
    let s = app.state::<crate::state::AppState>();
    crate::settings::load(&s.db)
        .map(|s| {
            if s.reduced_resource_mode {
                (s.monitor_interval_secs * 2).min(60)
            } else {
                s.monitor_interval_secs
            }
        })
        .unwrap_or(3)
}

fn run(app: AppHandle, shared: Arc<(Mutex<Shared>, Condvar)>) {
    let mut cpu = CpuSampler { last: None };
    let mut gpu = GpuSampler::new();
    if gpu.is_none() {
        log::warn!("GPU sayaçları kullanılamıyor; GPU kullanımı gösterilmeyecek");
    }
    cpu.sample();
    let (lock, cv) = &*shared;
    // Etkinleştikten hemen sonra kısa bir CPU ölçüm aralığıyla ilk değeri gönder.
    let mut fresh = true;
    loop {
        // Etkin değilken uyu (CPU kullanımı yok).
        {
            let mut s = lock.lock().unwrap_or_else(|e| e.into_inner());
            while !s.active {
                s = cv.wait(s).unwrap_or_else(|e| e.into_inner());
                cpu.sample(); // uzun aradan sonra ilk değer bayat olmasın
                fresh = true;
            }
        }
        let secs = interval(&app);
        let wait = if fresh {
            Duration::from_millis(500)
        } else {
            Duration::from_secs(u64::from(secs))
        };
        fresh = false;
        {
            let s = lock.lock().unwrap_or_else(|e| e.into_inner());
            let _ = cv
                .wait_timeout_while(s, wait, |s| s.active)
                .unwrap_or_else(|e| e.into_inner());
        }
        if !lock.lock().unwrap_or_else(|e| e.into_inner()).active {
            continue;
        }
        let (memory_used, memory_total) = memory();
        let (disk_used, disk_total) = disk();
        let stats = SystemStats {
            cpu: cpu.sample(),
            memory_used,
            memory_total,
            gpu: gpu.as_mut().and_then(GpuSampler::sample),
            disk_used,
            disk_total,
            uptime_secs: unsafe { GetTickCount64() } / 1000,
            interval_secs: secs,
        };
        lock.lock().unwrap_or_else(|e| e.into_inner()).last = Some(stats.clone());
        events::broadcast(&app, events::SYSTEM_STATS, stats);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_sampler_produces_percentage_on_second_sample() {
        let mut c = CpuSampler { last: None };
        assert!(c.sample().is_none());
        std::thread::sleep(Duration::from_millis(50));
        let v = c.sample();
        assert!(v.is_none_or(|v| (0.0..=100.0).contains(&v)));
    }

    #[test]
    fn memory_and_disk_are_real_values() {
        let (used, total) = memory();
        assert!(total > 0 && used <= total);
        let (du, dt) = disk();
        assert!(dt > 0 && du <= dt);
    }
}
