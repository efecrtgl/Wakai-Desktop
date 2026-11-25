# Ivily Desktop 1.1.5 — Architecture

This document describes the architecture as implemented. Sections marked *(planned)* describe
work that has not landed yet; they are updated as phases complete.

## 1. Existing system (audit of 1.1.0, commit 55590c4)

| Layer | Implementation |
|---|---|
| Shell | Tauri 2.12, single binary `wakai-desktop.exe`, identifier `com.wakai.desktop` |
| Windows | `main` (Control Center, `index.html`) + one WebView window per widget (`widget.html?id=`) |
| Desktop layer | `desktop/` — WorkerW/Progman hierarchy, placement, watcher for Explorer restarts |
| Desktop cats | `pets/engine.rs` + `pets/native.rs` — native layered windows (no WebView), 15 fps, hidden on fullscreen |
| Data | SQLite (`%APPDATA%\com.wakai.desktop\wakai.db`), `PRAGMA user_version` migrations 001–006 |
| Settings | One JSON object (`settings.app`), serde with `#[serde(default)]`; unknown fields are ignored, so older builds tolerate new fields |
| Updates | tauri-plugin-updater, minisign public key in `tauri.conf.json`, endpoint `releases/latest/download/latest.json` |
| Recovery | `recovery.rs` — previous executables kept under `previous/<ver>/`, launch health, auto-rollback |
| Backups | `backup.rs` — `VACUUM INTO`, verification, full/selective restore |
| Frontend | React 19, plain JS, Vite 8; design tokens in `src/ui/tokens.css`; `data-theme` (wakai/dark/light), `data-font`, `data-motion` on `<html>` |
| Pet Home | `PetHome.jsx` single-room canvas, `pets/home/sim.js` weighted-random intents, `pet_home_items` table |

Measured baseline: see `IVILY_1.1.5_TESTING.md`.

## 2. Key decisions

### 2.1 Identifier and data directory stay unchanged
`identifier` remains `com.wakai.desktop`. It determines the data directory
(`%APPDATA%\com.wakai.desktop`), the WebView2 profile, the notification AppUserModelID and the
uninstaller's "delete app data" target. Keeping it means **no user data has to move**; the brand
migration is purely presentational plus an installer-level move of the program files. See
`IVILY_BRAND_MIGRATION.md`.

### 2.2 Binary name stays `wakai-desktop.exe`
The executable name is a technical identifier: the recovery module stores previous versions
under that name, the NSIS "app is running" check and the updater use it, and keeping it makes a
rollback to 1.1.0 a plain file swap. Users see the product name (file description, window
titles, shortcuts, Apps list), which becomes "Ivily Desktop".

### 2.3 Settings compatibility
New settings are added as **new fields** (e.g. `theme`) instead of changing the meaning of
existing enum fields (`appearance`). A rollback to 1.1.0 then still parses the settings object
(it ignores the unknown fields) instead of failing and falling back to defaults.

### 2.4 Schema compatibility
New migrations only add tables and columns with defaults. Older builds keep working against a
newer schema (they never write the new columns), which keeps version rollback safe.
