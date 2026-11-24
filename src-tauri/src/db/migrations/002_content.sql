-- 002: Widget içerikleri (etkinlikler, görevler, notlar, odak oturumları).
-- İçerik tabloları widget_instances tablosuna BAĞLI DEĞİLDİR: bir widget'ı silmek veriyi silmez.

CREATE TABLE event_categories (
    id    INTEGER PRIMARY KEY,
    name  TEXT NOT NULL UNIQUE,
    color TEXT NOT NULL
);

INSERT INTO event_categories (name, color) VALUES
    ('Kişisel', 'pink'),
    ('İş', 'lavender'),
    ('Sağlık', 'mint'),
    ('Sosyal', 'peach');

CREATE TABLE events (
    id               INTEGER PRIMARY KEY,
    title            TEXT NOT NULL CHECK (length(title) > 0),
    description      TEXT NOT NULL DEFAULT '',
    start_at         TEXT NOT NULL,              -- yerel zaman: YYYY-MM-DDTHH:MM
    end_at           TEXT,
    all_day          INTEGER NOT NULL DEFAULT 0 CHECK (all_day IN (0, 1)),
    category_id      INTEGER REFERENCES event_categories (id) ON DELETE SET NULL,
    color            TEXT NOT NULL DEFAULT 'pink',
    reminder_minutes INTEGER CHECK (reminder_minutes IS NULL OR reminder_minutes >= 0),
    reminded_at      TEXT,
    created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime')),
    updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime'))
);

CREATE INDEX idx_events_start ON events (start_at);
CREATE INDEX idx_events_pending_reminder ON events (reminded_at) WHERE reminder_minutes IS NOT NULL;

CREATE TABLE tasks (
    id           INTEGER PRIMARY KEY,
    title        TEXT NOT NULL CHECK (length(title) > 0),
    notes        TEXT NOT NULL DEFAULT '',
    priority     INTEGER NOT NULL DEFAULT 1 CHECK (priority BETWEEN 0 AND 2),
    due_at       TEXT,
    category     TEXT NOT NULL DEFAULT '',
    completed    INTEGER NOT NULL DEFAULT 0 CHECK (completed IN (0, 1)),
    completed_at TEXT,
    reminder_at  TEXT,
    reminded_at  TEXT,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime'))
);

CREATE INDEX idx_tasks_open ON tasks (completed, due_at);

CREATE TABLE notes (
    id         INTEGER PRIMARY KEY,
    title      TEXT NOT NULL DEFAULT '',
    body       TEXT NOT NULL DEFAULT '',
    color      TEXT NOT NULL DEFAULT 'butter',
    pinned     INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime'))
);

CREATE INDEX idx_notes_order ON notes (pinned DESC, updated_at DESC);

CREATE TABLE focus_sessions (
    id              INTEGER PRIMARY KEY,
    kind            TEXT NOT NULL CHECK (kind IN ('work', 'short_break', 'long_break')),
    planned_seconds INTEGER NOT NULL CHECK (planned_seconds > 0),
    started_at      TEXT NOT NULL,
    ended_at        TEXT,
    completed       INTEGER NOT NULL DEFAULT 0 CHECK (completed IN (0, 1))
);

CREATE INDEX idx_focus_started ON focus_sessions (started_at);
