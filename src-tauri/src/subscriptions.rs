//! Pahalı arka plan hizmetleri (sistem izleme, medya oturumu) yalnızca onları gösteren
//! bir pencere varken çalışır. Pencereler bir "konuya" abone olur; pencere kapanınca
//! aboneliği otomatik olarak silinir. Abone sayısı 0 ↔ 1 geçişlerinde hizmet başlar/durur.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Topic {
    System,
    Media,
}

impl Topic {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "system" => Some(Self::System),
            "media" => Some(Self::Media),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct Subscriptions(Mutex<HashMap<Topic, HashSet<String>>>);

impl Subscriptions {
    /// Aboneyi ekler; konu bu çağrıyla etkinleştiyse `true` döner.
    pub fn add(&self, topic: Topic, label: &str) -> bool {
        let mut map = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let set = map.entry(topic).or_default();
        let was_empty = set.is_empty();
        set.insert(label.to_string());
        was_empty
    }

    /// Aboneyi siler; konu bu çağrıyla boşaldıysa `true` döner.
    pub fn remove(&self, topic: Topic, label: &str) -> bool {
        let mut map = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let Some(set) = map.get_mut(&topic) else {
            return false;
        };
        set.remove(label) && set.is_empty()
    }

    /// Pencerenin tüm aboneliklerini siler; boşalan konuları döndürür.
    pub fn remove_window(&self, label: &str) -> Vec<Topic> {
        let mut map = self.0.lock().unwrap_or_else(|e| e.into_inner());
        map.iter_mut()
            .filter_map(|(t, set)| (set.remove(label) && set.is_empty()).then_some(*t))
            .collect()
    }

    #[cfg(test)]
    pub fn count(&self, topic: Topic) -> usize {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&topic)
            .map_or(0, HashSet::len)
    }
}

/// Konunun hizmetini başlatır ya da durdurur.
pub fn set_active(app: &AppHandle, topic: Topic, active: bool) {
    match topic {
        Topic::System => app
            .state::<crate::monitor::SystemMonitor>()
            .set_active(app, active),
        Topic::Media => app
            .state::<crate::media::MediaService>()
            .set_active(app, active),
    }
}

pub fn on_window_destroyed(app: &AppHandle, label: &str) {
    if let Some(subs) = app.try_state::<Subscriptions>() {
        for topic in subs.remove_window(label) {
            set_active(app, topic, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_edges() {
        let s = Subscriptions::default();
        assert!(s.add(Topic::System, "a"));
        assert!(!s.add(Topic::System, "b"));
        assert!(!s.add(Topic::System, "b"));
        assert_eq!(s.count(Topic::System), 2);
        assert!(!s.remove(Topic::System, "a"));
        assert_eq!(s.remove_window("b"), vec![Topic::System]);
        assert_eq!(s.count(Topic::System), 0);
        assert!(!s.remove(Topic::Media, "x"));
    }
}
