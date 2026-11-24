-- 005: Masaüstü profilleri. Yerleşik profillerin `builtin` anahtarı vardır (silinemez,
-- varsayılana döndürülebilir); özel profillerde NULL'dır. Yapılandırma JSON'dur.
CREATE TABLE desktop_profiles (
    id         INTEGER PRIMARY KEY,
    builtin    TEXT UNIQUE,
    name       TEXT NOT NULL CHECK (length(name) > 0),
    config     TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime'))
);
