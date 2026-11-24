//! Aşama 2 geçici bağlantı noktası: Aşama 3'te widget yöneticisi bu çağrıları devralır.
use crate::desktop::Rect;
use tauri::AppHandle;

pub fn on_windows_lost(_app: &AppHandle, dead: Vec<isize>) {
    log::info!("kaybolan pencere sayısı: {}", dead.len());
}

pub fn on_rects_changed(_app: &AppHandle, changed: Vec<(isize, Rect)>) {
    log::debug!("konumu değişen pencere sayısı: {}", changed.len());
}
