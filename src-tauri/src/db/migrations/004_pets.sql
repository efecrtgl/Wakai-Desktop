-- 004: Masaüstü kedileri, ihtiyaçlar, envanter, başarımlar ve Kedi Evi eşyaları.
-- İhtiyaçlar zaman damgasıyla saklanır (needs_at, Unix ms) ve geçen süreden hesaplanır;
-- animasyon kareleri ve anlık ekran koordinatları veritabanına YAZILMAZ.
CREATE TABLE pets (
    id          INTEGER PRIMARY KEY,
    name        TEXT NOT NULL CHECK (length(name) > 0),
    breed       TEXT NOT NULL,
    personality TEXT NOT NULL,
    location    TEXT NOT NULL DEFAULT 'desktop' CHECK (location IN ('desktop', 'home')),
    paused      INTEGER NOT NULL DEFAULT 0 CHECK (paused IN (0, 1)),
    acc_head    TEXT,
    acc_neck    TEXT,
    satiety     REAL NOT NULL DEFAULT 80,
    energy      REAL NOT NULL DEFAULT 80,
    happiness   REAL NOT NULL DEFAULT 80,
    rest        REAL NOT NULL DEFAULT 80,
    needs_at    INTEGER NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime'))
);

CREATE TABLE pet_inventory (
    item        TEXT PRIMARY KEY NOT NULL,
    count       INTEGER NOT NULL CHECK (count >= 0),
    refilled_on TEXT NOT NULL
);

CREATE TABLE pet_stats (
    key   TEXT PRIMARY KEY NOT NULL,
    value INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE pet_achievements (
    id          TEXT PRIMARY KEY NOT NULL,
    unlocked_at TEXT NOT NULL
);

CREATE TABLE pet_home_items (
    id      INTEGER PRIMARY KEY,
    kind    TEXT NOT NULL,
    x       INTEGER NOT NULL,
    y       INTEGER NOT NULL,
    flipped INTEGER NOT NULL DEFAULT 0 CHECK (flipped IN (0, 1))
);
