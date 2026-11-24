//! Wakai Desktop'un kendi kaynak kullanımı: ana süreç ve ondan türeyen tüm süreçler
//! (WebView2 tarayıcı, GPU ve renderer süreçleri). Performans sayfası ve ölçüm betiği kullanır.

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::ProcessStatus::{
    GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
};
use windows::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};

#[derive(Debug, Clone, Serialize)]
pub struct ResourceUsage {
    /// Tüm süreçlerin özel bellek (commit) toplamı, bayt.
    pub private_bytes: u64,
    /// Tüm süreçlerin çalışma kümesi toplamı, bayt.
    pub working_set: u64,
    /// Toplam CPU süresi (çekirdek + kullanıcı), milisaniye.
    pub cpu_time_ms: u64,
    pub processes: usize,
    pub widget_windows: usize,
}

/// Bu sürecin kendisi ve tüm alt süreçlerinin kimlikleri.
fn process_tree(root: u32) -> Vec<u32> {
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return vec![root];
        };
        let mut e = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut e).is_ok() {
            loop {
                children
                    .entry(e.th32ParentProcessID)
                    .or_default()
                    .push(e.th32ProcessID);
                if Process32NextW(snap, &mut e).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }
    let mut out = vec![root];
    let mut seen: HashSet<u32> = HashSet::from([root]);
    let mut i = 0;
    while i < out.len() {
        for &c in children.get(&out[i]).map(Vec::as_slice).unwrap_or(&[]) {
            if seen.insert(c) {
                out.push(c);
            }
        }
        i += 1;
    }
    out
}

fn filetime_ms(f: windows::Win32::Foundation::FILETIME) -> u64 {
    ((u64::from(f.dwHighDateTime) << 32) | u64::from(f.dwLowDateTime)) / 10_000
}

pub fn usage(widget_windows: usize) -> ResourceUsage {
    let pids = process_tree(std::process::id());
    let mut u = ResourceUsage {
        private_bytes: 0,
        working_set: 0,
        cpu_time_ms: 0,
        processes: 0,
        widget_windows,
    };
    for pid in pids {
        unsafe {
            let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
                continue;
            };
            let mut m = PROCESS_MEMORY_COUNTERS_EX {
                cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
                ..Default::default()
            };
            if GetProcessMemoryInfo(h, &mut m as *mut _ as *mut PROCESS_MEMORY_COUNTERS, m.cb)
                .is_ok()
            {
                u.private_bytes += m.PrivateUsage as u64;
                u.working_set += m.WorkingSetSize as u64;
                u.processes += 1;
            }
            let (mut c, mut x, mut k, mut us) = Default::default();
            if GetProcessTimes(h, &mut c, &mut x, &mut k, &mut us).is_ok() {
                u.cpu_time_ms += filetime_ms(k) + filetime_ms(us);
            }
            let _ = CloseHandle(h);
        }
    }
    u
}

#[cfg(test)]
mod tests {
    #[test]
    fn measures_current_process() {
        let u = super::usage(0);
        assert!(u.processes >= 1);
        assert!(u.private_bytes > 0 && u.working_set > 0);
    }
}
