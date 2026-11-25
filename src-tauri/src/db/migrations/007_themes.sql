-- 007: İçe aktarılan (özel) temalar. Yerleşik temalar uygulamayla gelir, burada saklanmaz.
-- Bildirim (manifest) içe aktarılırken doğrulanmış JSON metnidir; çalıştırılabilir içerik yoktur.
CREATE TABLE IF NOT EXISTS custom_themes (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    version     TEXT NOT NULL,
    manifest    TEXT NOT NULL,
    installed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime'))
);
