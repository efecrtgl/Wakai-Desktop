//! Müzik Kedisi için Windows Global System Media Transport Controls (GSMTC) entegrasyonu.
//!
//! Spotify, tarayıcılar, Medya Oynatıcı gibi Windows medya oturumu yayınlayan her uygulama
//! desteklenir. Tamamen olay güdümlüdür: oturum, parça ve oynatma durumu değişince WinRT
//! olayları bir yenileme isteği gönderir; yoklama yapılmaz. Hizmet yalnızca bir Müzik Kedisi
//! (abone) varken çalışır.

use crate::events;
use serde::Serialize;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use windows::Foundation::TypedEventHandler;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as SessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
};
use windows::Storage::Streams::DataReader;
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};

/// Kapak görselinin en büyük boyutu; daha büyükse gösterilmez.
const MAX_THUMBNAIL_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MediaState {
    pub available: bool,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub app: String,
    /// playing | paused | stopped | none
    pub status: &'static str,
    pub can_play_pause: bool,
    pub can_next: bool,
    pub can_previous: bool,
    /// `data:` URL olarak kapak görseli.
    pub thumbnail: Option<String>,
}

impl Default for MediaState {
    fn default() -> Self {
        Self {
            available: false,
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            app: String::new(),
            status: "none",
            can_play_pause: false,
            can_next: false,
            can_previous: false,
            thumbnail: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    PlayPause,
    Next,
    Previous,
}

enum Msg {
    Refresh,
    Action(Action, Sender<bool>),
    Stop,
}

#[derive(Default)]
struct Inner {
    tx: Option<Sender<Msg>>,
    last: MediaState,
}

#[derive(Default, Clone)]
pub struct MediaService {
    inner: Arc<Mutex<Inner>>,
}

impl MediaService {
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn last(&self) -> MediaState {
        self.lock().last.clone()
    }

    pub fn set_active(&self, app: &AppHandle, active: bool) {
        let mut inner = self.lock();
        match (active, inner.tx.is_some()) {
            (true, false) => {
                let (tx, rx) = mpsc::channel();
                inner.tx = Some(tx.clone());
                let svc = self.clone();
                let app = app.clone();
                std::thread::Builder::new()
                    .name("wakai-media".into())
                    .spawn(move || worker(app, svc, tx, rx))
                    .expect("medya iş parçacığı oluşturulamadı");
            }
            (false, true) => {
                if let Some(tx) = inner.tx.take() {
                    let _ = tx.send(Msg::Stop);
                }
            }
            _ => {}
        }
    }

    /// Oynatma komutu gönderir; işlenip işlenmediğini döndürür.
    pub fn act(&self, action: Action) -> bool {
        let Some(tx) = self.lock().tx.clone() else {
            return false;
        };
        let (rtx, rrx) = mpsc::channel();
        if tx.send(Msg::Action(action, rtx)).is_err() {
            return false;
        }
        rrx.recv_timeout(std::time::Duration::from_secs(3))
            .unwrap_or(false)
    }
}

fn status_name(s: Status) -> &'static str {
    match s {
        Status::Playing => "playing",
        Status::Paused => "paused",
        Status::Stopped | Status::Closed => "stopped",
        _ => "none",
    }
}

/// "Spotify.exe" → "Spotify", "Microsoft.ZuneMusic_8wekyb3d8bbwe!Microsoft.ZuneMusic" → "ZuneMusic"
fn friendly_app(id: &str) -> String {
    let base = id.rsplit(['!', '\\']).next().unwrap_or(id);
    let base = base.strip_suffix(".exe").unwrap_or(base);
    base.rsplit('.').next().unwrap_or(base).to_string()
}

fn thumbnail(session: &Session) -> Option<String> {
    let props = session.TryGetMediaPropertiesAsync().ok()?.join().ok()?;
    let stream = props.Thumbnail().ok()?.OpenReadAsync().ok()?.join().ok()?;
    let size = stream.Size().ok()?;
    if size == 0 || size > MAX_THUMBNAIL_BYTES {
        return None;
    }
    let content_type = stream
        .ContentType()
        .map(|s| s.to_string())
        .unwrap_or_else(|_| "image/png".into());
    if !content_type.starts_with("image/") {
        return None;
    }
    let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0).ok()?).ok()?;
    let loaded = reader.LoadAsync(size as u32).ok()?.join().ok()?;
    let mut buf = vec![0u8; loaded as usize];
    reader.ReadBytes(&mut buf).ok()?;
    Some(format!("data:{content_type};base64,{}", base64(&buf)))
}

fn snapshot(session: Option<&Session>, previous: &MediaState) -> MediaState {
    let Some(session) = session else {
        return MediaState {
            status: "none",
            ..Default::default()
        };
    };
    let mut state = MediaState {
        available: true,
        status: "none",
        ..Default::default()
    };
    state.app = session
        .SourceAppUserModelId()
        .map(|s| friendly_app(&s.to_string()))
        .unwrap_or_default();
    if let Ok(props) = session
        .TryGetMediaPropertiesAsync()
        .and_then(|op| op.join())
    {
        state.title = props.Title().map(|s| s.to_string()).unwrap_or_default();
        state.artist = props.Artist().map(|s| s.to_string()).unwrap_or_default();
        state.album = props
            .AlbumTitle()
            .map(|s| s.to_string())
            .unwrap_or_default();
    }
    if let Ok(info) = session.GetPlaybackInfo() {
        state.status = info.PlaybackStatus().map(status_name).unwrap_or("none");
        if let Ok(c) = info.Controls() {
            state.can_play_pause = c.IsPlayPauseToggleEnabled().unwrap_or(false);
            state.can_next = c.IsNextEnabled().unwrap_or(false);
            state.can_previous = c.IsPreviousEnabled().unwrap_or(false);
        }
    }
    // Kapak yalnızca parça değiştiğinde yeniden okunur.
    state.thumbnail =
        if previous.available && previous.title == state.title && previous.artist == state.artist {
            previous.thumbnail.clone()
        } else {
            thumbnail(session)
        };
    state
}

struct Watch {
    session: Session,
    props_token: i64,
    playback_token: i64,
}

fn watch(session: &Session, tx: &Sender<Msg>) -> Option<Watch> {
    let t1 = tx.clone();
    let t2 = tx.clone();
    let props_token = session
        .MediaPropertiesChanged(&TypedEventHandler::new(move |_, _| {
            let _ = t1.send(Msg::Refresh);
            Ok(())
        }))
        .ok()?;
    let playback_token = session
        .PlaybackInfoChanged(&TypedEventHandler::new(move |_, _| {
            let _ = t2.send(Msg::Refresh);
            Ok(())
        }))
        .ok()?;
    Some(Watch {
        session: session.clone(),
        props_token,
        playback_token,
    })
}

fn unwatch(w: Watch) {
    let _ = w.session.RemoveMediaPropertiesChanged(w.props_token);
    let _ = w.session.RemovePlaybackInfoChanged(w.playback_token);
}

fn worker(app: AppHandle, svc: MediaService, tx: Sender<Msg>, rx: Receiver<Msg>) {
    // WinRT çağrıları için bu iş parçacığında COM (MTA) başlatılır.
    let com = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
    let manager = match SessionManager::RequestAsync().and_then(|op| op.join()) {
        Ok(m) => m,
        Err(e) => {
            log::warn!("medya oturum yöneticisine erişilemedi: {e}");
            if com {
                unsafe { CoUninitialize() };
            }
            return;
        }
    };
    let t = tx.clone();
    let session_token = manager
        .CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
            let _ = t.send(Msg::Refresh);
            Ok(())
        }))
        .ok();

    let mut current: Option<Watch> = None;
    let _ = tx.send(Msg::Refresh);
    while let Ok(msg) = rx.recv() {
        match msg {
            Msg::Refresh => {
                // Arka arkaya gelen olayları tek yenilemede birleştir.
                while let Ok(Msg::Refresh) = rx.try_recv() {}
                let session = manager.GetCurrentSession().ok();
                let same = matches!((&current, &session), (Some(w), Some(s)) if w.session == *s);
                if !same {
                    if let Some(w) = current.take() {
                        unwatch(w);
                    }
                    current = session.as_ref().and_then(|s| watch(s, &tx));
                }
                let previous = svc.last();
                let state = snapshot(session.as_ref(), &previous);
                if state != previous {
                    svc.lock().last = state.clone();
                    events::broadcast(&app, events::MEDIA_CHANGED, state);
                }
            }
            Msg::Action(action, reply) => {
                let ok = manager.GetCurrentSession().ok().is_some_and(|s| {
                    let op = match action {
                        Action::PlayPause => s.TryTogglePlayPauseAsync(),
                        Action::Next => s.TrySkipNextAsync(),
                        Action::Previous => s.TrySkipPreviousAsync(),
                    };
                    op.and_then(|op| op.join()).unwrap_or(false)
                });
                let _ = reply.send(ok);
            }
            Msg::Stop => break,
        }
    }
    if let Some(w) = current.take() {
        unwatch(w);
    }
    if let Some(token) = session_token {
        let _ = manager.RemoveCurrentSessionChanged(token);
    }
    drop(manager);
    if com {
        unsafe { CoUninitialize() };
    }
}

/// Standart base64 kodlaması (kapak görseli için; ek bağımlılık gerekmez).
pub fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            T[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc4648_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn friendly_app_names() {
        assert_eq!(friendly_app("Spotify.exe"), "Spotify");
        assert_eq!(
            friendly_app("Microsoft.ZuneMusic_8wekyb3d8bbwe!Microsoft.ZuneMusic"),
            "ZuneMusic"
        );
        assert_eq!(friendly_app("C:\\Apps\\foobar2000.exe"), "foobar2000");
    }
}
