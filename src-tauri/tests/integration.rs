//! Modüller arası bütünleşme testleri: gerçek dosya üzerinde veritabanı, göçler,
//! içerik ve widget verilerinin bağımsızlığı, yedekleme ve odak geçmişi.

use rusqlite::Connection;
use std::path::PathBuf;
use wakai_desktop_lib::db::{self, Db, migrations};
use wakai_desktop_lib::{backup, focus, settings};

fn temp_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("wakai-it-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Eski sürümden (şema v1) yükseltme: veriler korunur, göç öncesi yedek alınır.
#[test]
fn upgrade_from_schema_v1_preserves_data() {
    let dir = temp_dir("upgrade");
    let path = dir.join("wakai.db");
    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(migrations::MIGRATIONS[0]).unwrap();
        conn.pragma_update(None, "user_version", 1).unwrap();
        conn.execute(
            "INSERT INTO widget_instances (id, kind, x, y, width, height, config) VALUES ('clock-1', 'clock', 10, 20, 300, 150, '{\"hour12\":true}')",
            [],
        )
        .unwrap();
    }
    let db = Db::open(&path).unwrap();
    assert_eq!(
        db.with(migrations::current_version).unwrap(),
        migrations::MIGRATIONS.len()
    );
    let w = db.with(|c| db::widgets::get(c, "clock-1")).unwrap();
    assert_eq!(w.rect.width, 300);
    assert_eq!(w.config["hour12"], serde_json::json!(true));
    assert!(
        path.with_extension("db.bak-v1").exists(),
        "göç öncesi yedek alınmalı"
    );
    // Yeni tablolar kullanılabilir.
    assert_eq!(db.with(db::events::categories).unwrap().len(), 4);
}

/// Daha yeni bir sürümün açtığı veritabanı eski sürümde bozulmaz (göç geri alınmaz).
#[test]
fn newer_schema_is_left_untouched() {
    let dir = temp_dir("newer");
    let path = dir.join("wakai.db");
    drop(Db::open(&path).unwrap());
    {
        let conn = Connection::open(&path).unwrap();
        conn.pragma_update(None, "user_version", 99).unwrap();
    }
    let db = Db::open(&path).unwrap();
    assert_eq!(db.with(migrations::current_version).unwrap(), 99);
}

/// Widget örneğini silmek bağlı notu silmez; notu silmek widget'ı bozmaz.
#[test]
fn widget_and_note_data_are_independent() {
    let db = Db::open_in_memory().unwrap();
    let note = db
        .with(|c| {
            db::notes::create(
                c,
                &db::notes::NoteInput {
                    title: "Liste".into(),
                    body: "- süt".into(),
                    color: "mint".into(),
                    pinned: false,
                },
            )
        })
        .unwrap();
    let w = db::widgets::WidgetInstance {
        id: "note-1".into(),
        kind: "note".into(),
        rect: wakai_desktop_lib::desktop::Rect::new(0, 0, 200, 200),
        visible: true,
        click_through: false,
        config: serde_json::json!({ "note_id": note.id, "font_size": 14 }),
    };
    db.with(|c| db::widgets::insert(c, &w)).unwrap();
    db.with(|c| db::widgets::delete(c, "note-1")).unwrap();
    assert_eq!(
        db.with(|c| db::notes::get(c, note.id)).unwrap().title,
        "Liste"
    );

    db.with(|c| db::widgets::insert(c, &w)).unwrap();
    db.with(|c| db::notes::delete(c, note.id)).unwrap();
    assert!(db.with(|c| db::widgets::get(c, "note-1")).is_ok());
}

/// Bozuk ayar kaydı uygulamayı düşürmez; varsayılanlara döner ve sonraki kayıt düzeltir.
#[test]
fn corrupt_settings_fall_back_to_defaults() {
    let db = Db::open_in_memory().unwrap();
    db.with(|c| {
        Ok(c.execute(
            "INSERT INTO settings (key, value) VALUES ('app', '{\"widget_opacity\": \"yarım\"')",
            [],
        )?)
    })
    .unwrap();
    assert_eq!(
        settings::load(&db).unwrap(),
        settings::AppSettings::default()
    );
    let s = settings::update(&db, serde_json::json!({ "shadows": false })).unwrap();
    assert!(!s.shadows);
    assert!(!settings::load(&db).unwrap().shadows);
}

/// Odak oturumları kalıcıdır: uygulama yeniden başlayınca "bugün" sayacı korunur.
#[test]
fn focus_sessions_survive_restart() {
    let dir = temp_dir("focus");
    let path = dir.join("wakai.db");
    let cfg = focus::FocusConfig {
        work_minutes: 1,
        ..Default::default()
    };
    {
        let db = Db::open(&path).unwrap();
        let t = focus::FocusTimer::new(&db);
        let start = focus::now_ms();
        t.start(&db, &cfg, start).unwrap();
        t.complete_if_due(&db, &cfg, start + 60_000)
            .unwrap()
            .expect("aşama tamamlanmalı");
    }
    let db = Db::open(&path).unwrap();
    let t = focus::FocusTimer::new(&db);
    assert_eq!(t.state(focus::now_ms()).completed_today, 1);
}

/// Yedekle → veriyi değiştir → geri yükle → eski veri geri gelir.
#[test]
fn backup_restore_end_to_end() {
    let dir = temp_dir("backup");
    let path = dir.join("wakai.db");
    let name = {
        let db = Db::open(&path).unwrap();
        db.with(|c| {
            db::tasks::create(
                c,
                &db::tasks::TaskInput {
                    title: "Yedekteki görev".into(),
                    notes: String::new(),
                    priority: 1,
                    due_at: None,
                    category: String::new(),
                    reminder_at: None,
                },
            )
        })
        .unwrap();
        let name = backup::create(&db, &dir, "").unwrap().name;
        db.with(|c| Ok(c.execute("DELETE FROM tasks", [])?))
            .unwrap();
        name
    };
    backup::schedule_restore(&dir, &name).unwrap();
    assert!(backup::apply_pending_restore(&dir, &path).unwrap());
    let db = Db::open(&path).unwrap();
    let tasks = db.with(|c| db::tasks::list(c, true)).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Yedekteki görev");
}
