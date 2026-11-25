-- 008: Kedi Evi dünyası. Yalnızca yeni tablolar ve varsayılanlı sütunlar eklenir; 1.1.0 bu
-- sütunları yok sayar (önceki sürüme dönüş güvenli). Eski pet_home_items tablosu korunur;
-- içeriği dünya ilk açıldığında oturma odasına bir kez kopyalanır.
ALTER TABLE pets ADD COLUMN social REAL NOT NULL DEFAULT 70;
ALTER TABLE pets ADD COLUMN trust REAL NOT NULL DEFAULT 50;
ALTER TABLE pets ADD COLUMN curiosity REAL NOT NULL DEFAULT 70;
ALTER TABLE pets ADD COLUMN cleanliness REAL NOT NULL DEFAULT 85;
ALTER TABLE pets ADD COLUMN quirk TEXT NOT NULL DEFAULT 'none';
ALTER TABLE pets ADD COLUMN xp INTEGER NOT NULL DEFAULT 0;
ALTER TABLE pets ADD COLUMN room TEXT NOT NULL DEFAULT 'living';
ALTER TABLE pets ADD COLUMN wx REAL;
ALTER TABLE pets ADD COLUMN wy REAL;
ALTER TABLE pets ADD COLUMN habits TEXT NOT NULL DEFAULT '{}';

CREATE TABLE IF NOT EXISTS world_rooms (
    id    TEXT PRIMARY KEY NOT NULL,
    floor TEXT,
    wall  TEXT
);

CREATE TABLE IF NOT EXISTS world_items (
    id    INTEGER PRIMARY KEY,
    room  TEXT NOT NULL,
    kind  TEXT NOT NULL,
    tx    INTEGER NOT NULL,
    ty    INTEGER NOT NULL,
    rot   INTEGER NOT NULL DEFAULT 0 CHECK (rot BETWEEN 0 AND 3),
    state TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX IF NOT EXISTS world_items_room ON world_items(room);

CREATE TABLE IF NOT EXISTS world_inventory (
    kind  TEXT PRIMARY KEY NOT NULL,
    count INTEGER NOT NULL CHECK (count >= 0)
);

CREATE TABLE IF NOT EXISTS pet_relations (
    a        INTEGER NOT NULL,
    b        INTEGER NOT NULL,
    affinity REAL NOT NULL DEFAULT 0,
    PRIMARY KEY (a, b),
    CHECK (a < b)
);

CREATE TABLE IF NOT EXISTS pet_memories (
    id     INTEGER PRIMARY KEY,
    pet_id INTEGER NOT NULL,
    at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime')),
    text   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS pet_memories_pet ON pet_memories(pet_id);

CREATE TABLE IF NOT EXISTS world_state (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
