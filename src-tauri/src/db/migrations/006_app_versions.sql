-- Sürüm geçmişi: bu bilgisayarda çalışmış her Wakai Desktop sürümü.
-- Yalnızca ekleme yapar; eski sürümler bu tabloyu yok sayar.
CREATE TABLE IF NOT EXISTS app_versions (
    version     TEXT PRIMARY KEY,
    first_seen  INTEGER NOT NULL,
    last_seen   INTEGER NOT NULL,
    launches    INTEGER NOT NULL DEFAULT 0
);
