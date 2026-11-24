-- 003: Alışveriş listeleri. Alışveriş Kedisi widget'ı yapılandırmasında list_id tutar;
-- widget silinse de liste kalır. Liste açıkça silinirse maddeleri de silinir.
CREATE TABLE shopping_lists (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL CHECK (length(name) > 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime'))
);

CREATE TABLE shopping_items (
    id         INTEGER PRIMARY KEY,
    list_id    INTEGER NOT NULL REFERENCES shopping_lists (id) ON DELETE CASCADE,
    name       TEXT NOT NULL CHECK (length(name) > 0),
    quantity   TEXT NOT NULL DEFAULT '',
    checked    INTEGER NOT NULL DEFAULT 0 CHECK (checked IN (0, 1)),
    checked_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime'))
);

CREATE INDEX idx_shopping_items_list ON shopping_items (list_id, checked);
