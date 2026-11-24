//! Uygulama genelinde paylaşılan durum.

use crate::db::Db;

pub struct AppState {
    pub db: Db,
}
