-- 001: Uygulama ayarları ve widget örnekleri
CREATE TABLE settings (
    key        TEXT PRIMARY KEY NOT NULL,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime'))
);

CREATE TABLE widget_instances (
    id            TEXT PRIMARY KEY NOT NULL,
    kind          TEXT NOT NULL,
    x             INTEGER NOT NULL,
    y             INTEGER NOT NULL,
    width         INTEGER NOT NULL CHECK (width > 0),
    height        INTEGER NOT NULL CHECK (height > 0),
    visible       INTEGER NOT NULL DEFAULT 1 CHECK (visible IN (0, 1)),
    click_through INTEGER NOT NULL DEFAULT 0 CHECK (click_through IN (0, 1)),
    config        TEXT NOT NULL DEFAULT '{}',
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime'))
);

CREATE INDEX idx_widget_instances_kind ON widget_instances (kind);
