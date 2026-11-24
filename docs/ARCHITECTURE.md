# Wakai Desktop — Technical Architecture

> Technical reference (English). The Turkish overview lives in [MIMARI.md](MIMARI.md).
> State: audited at the start of 1.1.0 development (commit `eb97fe2`, version 1.0.0).

## 1. Process model

```
wakai-desktop.exe (Rust, Tauri 2)
 ├─ main/UI thread ........ tao event loop, all window + desktop-layer operations
 ├─ wakai-desktop-watcher . hidden top-level window: Raw Input (hover-lift), TaskbarCreated,
 │                          WM_DISPLAYCHANGE, WM_ENDSESSION
 ├─ wakai-scheduler ....... condvar sleeper: event/task reminders, focus phase ends
 ├─ wakai-monitor ......... CPU/RAM/GPU/disk sampler (only while subscribed)
 ├─ wakai-media ........... GSMTC MTA thread (only while subscribed)
 └─ WebView2 (shared environment, --renderer-process-limit=2)
      ├─ main  (index.html)  Control Center — destroyed when closed to tray
      └─ widget-<id> (widget.html?id=...) one window per visible widget
```

## 2. Rust modules (`src-tauri/src`)

| Module | Responsibility |
|---|---|
| `lib.rs` | Builder, plugin setup, startup order, window/run events, shutdown |
| `state.rs` | `AppState { db, data_dir }`, `on_main()` helper (run window work on UI thread) |
| `error.rs` | `AppError` → serialized as Turkish message |
| `events.rs` | Event names + `broadcast` |
| `logging.rs` | stderr (debug) + `wakai.log` (warn+) |
| `settings.rs` | Typed `AppSettings` stored as JSON under `settings.app`, merge-patch + clamp |
| `validate.rs` | Input validation helpers (Turkish messages) |
| `db/` | `Db` (Mutex<Connection>, WAL), migrations (`PRAGMA user_version`), repositories: settings, widgets, events, tasks, notes |
| `desktop/` | **All unsafe Win32 for widgets**: hierarchy (Progman/WorkerW), placement (sink/lift), watcher thread, monitors, DPI guard |
| `layer.rs` | Glue between `DesktopLayer` and Tauri windows; raster-scale sync |
| `widgets/` | registry (static definitions), manager (lifecycle, debounced rect persistence, respawn), window creation |
| `scheduler.rs` | Reminder + focus deadline thread |
| `focus.rs` | Wall-clock Pomodoro state |
| `monitor.rs`, `media.rs`, `weather.rs`, `http.rs` | System stats, GSMTC, Open-Meteo via WinHTTP |
| `subscriptions.rs` | Window-scoped topic subscriptions that start/stop services |
| `backup.rs` | `VACUUM INTO` backups, restore-on-next-start |
| `tray.rs`, `main_window.rs` | Tray menu; Control Center window lifecycle |
| `procstats.rs` | Process-tree memory/CPU |
| `commands/` | IPC surface (see §4) |

## 3. Frontend (`src`)

- Two Vite entries: `index.html` → `control-center/`, `widget.html` → `widget-host/`.
- `widgets/<kind>/` — one lazily loaded component per widget kind (`widgets/registry.js`).
- `ui/` — tokens.css (design tokens), base.css, components (Button, Card, Toggle, Slider, Select…), widget primitives, `PixelIcon`, `cats/` (sprite grids + `PixelCat` + sparse-timer motion).
- `lib/` — `ipc.js` (`call`, `useTauriEvent`), data hooks, dates (Turkish), formatting, settings hook, motion attribute.

## 4. IPC surface (1.0.0)

**Commands (50):** app_info, create/list/delete/restore_backup, send_test_notification, app_resource_usage, get/update_settings, list_widget_definitions, list_widgets, get_widget, add/remove_widget, set_widget_visible, set_widget_click_through, update_widget_config, set_widget_rect, reset_widget(_position), open_control_center, list_event_categories, list/upcoming/create/update/delete_event, list/create/update/delete_task, set_task_completed, clear_completed_tasks, list/get/create/update/delete_note, focus_state/start/pause/reset/skip, subscribe/unsubscribe, system_stats, media_state, media_control, weather_get, weather_search.

**Events (10):** `settings:changed`, `widgets:changed`, `widget:updated`, `navigate`, `events:changed`, `tasks:changed`, `notes:changed`, `focus:changed`, `system:stats`, `media:changed`.

Window-creating commands are `async` and run on the UI thread via `on_main` (sync commands that create WebViews deadlock on Windows).

## 5. Database (schema v2)

| Table | Notes |
|---|---|
| `settings(key, value JSON)` | `app` key holds `AppSettings` |
| `widget_instances` | id, kind, rect, visible, click_through, config JSON |
| `event_categories`, `events` | calendar; reminder bookkeeping (`reminded_at`) |
| `tasks` | priority, due, category, reminder |
| `notes` | bound to note widgets via `config.note_id` (no FK on purpose) |
| `focus_sessions` | Pomodoro history |
| weather cache | stored in `settings` under `weather:<lat>,<lon>` |

Migrations are additive and run in one transaction each; a pre-migration copy (`wakai.db.bak-v<N>`) is written when an existing database is upgraded. A newer schema is left untouched by an older binary.

## 6. Audit findings relevant to 1.1.0

| Finding | Impact | Plan |
|---|---|---|
| No Shopping widget exists, although the 1.1.0 brief refers to one | Brief assumption ≠ code | Build "Alışveriş Kedisi" as a new widget (Phase 2); no legacy data to migrate |
| Only one appearance (pastel); tokens are mostly centralized but a few components use literal colours (`#fdebef`, `#fff`, rgba shadows) | Dark/light modes would break | Phase 1: move every literal to tokens, add `data-theme` variants |
| Typography sizes are tokens but weights/line-heights are ad hoc | Inconsistent hierarchy | Phase 1: type scale tokens |
| No CI/CD, no release automation, no updater | Manual releases only | Phase 9: GitHub Actions + Tauri updater |
| Widget windows are WebViews (~26 MB each) | Pets must NOT reuse this model | Phase 3: native layered-window pet renderer in Rust (no WebView) |
| `settings` JSON is a single blob | Profiles need per-profile overrides | Phase 8: profiles table; profile application layer on top of settings |
| Backup = full DB copy, restore = whole DB | No validation of schema compatibility, no selective restore | Phase 11 |
| E2E tests require CDP env var, which replaces app browser args | Perf must be measured without CDP | Keep documented |

No memory leaks were found by inspection: event listeners are removed on unmount (`useTauriEvent`), timers are cleared, background services stop on last unsubscribe, widget windows are destroyed (not hidden). Duplicate code is limited (two small `edit`-style date helpers); not worth churn.
