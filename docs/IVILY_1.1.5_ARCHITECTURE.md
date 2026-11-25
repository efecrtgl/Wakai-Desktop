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

## 3. Theme Engine (phase 3)

```
src/theme/contract.json   ← single source of truth: token names, style enums, limits
   │                         (read by the frontend AND include_str!'d by src-tauri/src/themes.rs)
   ├─ src/theme/builtin/*.json   five declarative manifests (also embedded in the Rust binary)
   ├─ src/theme/schema.js        validateTheme / migrateTheme (runtime normalization)
   ├─ src/theme/engine.js        compileTheme → cssFor + attrsFor → applyTheme (atomic)
   ├─ src/theme/useTheme.js      React hook: settings → resolved theme, custom theme loading
   ├─ src/theme/ThemeBoundary.jsx error boundary → fallback theme
   └─ src/ui/theme-styles.css    structural rules keyed on data-* attributes
src-tauri/src/themes.rs   import gate (strict validation), custom_themes table, window chrome
                          (is_dark), desktop-cat palette (pet_palette)
```

**Manifest → runtime.** A manifest only *selects* from the contract: semantic tokens (colors,
shadows, radii, border width), font stacks, structural options (`shape`, `surface`, `border`,
`icons`, `density`, `texture`, `headings`, `motion`, `rendering`), pet palette, Pet Home floor
and wall patterns/colors, ambient effect, synthesized UI sounds and embedded images. Themes
contain no CSS, no JavaScript and no file paths, so a theme cannot change behaviour, read files
or load remote content.

**Atomic switching.** `applyTheme` computes the full CSS text (`:root{--token:value;…}`) and the
attribute set first, then writes the single `<style id="ivily-theme">` element and the root
`data-*` attributes in the same task, so the browser performs one style recalculation and a
half-applied theme is never visible. If writing throws, the previous CSS and attributes are
restored. Switching touches only presentation; widget positions, widget data, pets, the Pet
World and profiles live in SQLite and are never read or written by the theme engine.

**Isolation and fallback.**
1. Invalid parts (unknown token, unsafe value, unknown enum value) are dropped with a warning;
   the rest of the theme still applies.
2. Fatal problems (bad id/name, unsupported schema, `min_app` newer than the app, missing theme)
   resolve to the fallback theme (Modern) and `fallbackReason` is reported to the UI.
3. `tokens.css` itself contains the Modern values, so the UI is usable even if no theme CSS is
   ever applied.
4. `ThemeBoundary` catches render errors, re-applies the fallback theme and offers "Yeniden dene".

**Compatibility and versioning.** `schema` (contract version, currently 1), `version` (theme's
own semver) and `min_app` (minimum Ivily version). Schema 0 manifests (`colors` instead of
`tokens`) are migrated by `migrateTheme`. Manifests with a newer schema are rejected with a
Turkish explanation. Widgets and the Pet World consume only semantic tokens and the compiled
`pets`/`home` sections, so new themes need no code changes.

**Rollback safety.** The new settings (`theme`, `theme_variant`, `theme_accent`, …) are new
fields; `appearance` is kept in sync for the Modern variants, so a rollback to 1.1.0 shows the
closest appearance instead of resetting settings. Settings stored by 1.1.0 (no `theme` key) are
mapped to Modern + the matching variant, so existing users keep their look; new installs start
with Pixel.

**Lazy assets.** Embedded images are decoded only when first requested (`themeAsset`) and the
cache is dropped when the theme changes. The Pixelify Sans font (OFL, bundled locally because
the CSP forbids remote fonts) is declared with `unicode-range`, so only the needed subsets load.

## 4. Control Center information architecture (phases 2 and 5)

`src/control-center/nav.js` is the single description of the navigation: seven sections, their
tabs, a map from 1.1.0 section ids (still sent by the widget context menu, tray and desktop cat
menu) to the new routes, and the search index. It contains no components and is unit-tested.

| Section | Tabs | Contents |
|---|---|---|
| Genel Bakış | – | Status tiles (widgets, theme, profile, cats), Pet Home summary, today, quick actions, app status |
| Widget'lar | Masaüstümde · Widget ekle | Existing widget management and store |
| Planlayıcı | Takvim · Görevler · Notlar | Existing data pages |
| Kedi Evi | Dünya · Kediler | Pet World and the cat collection |
| Temalar | – | Theme Library |
| Kişiselleştirme | Görünüm · Profiller | Accent, density, font, widget look; desktop profiles |
| Ayarlar | Genel · Erişilebilirlik · Performans · Bildirimler · Güncellemeler · Yedekleme · Hakkında | Vertical category list |

Existing section components are reused inside tabs; `SectionTabs` provides an `EmbedContext`
so their own page headers collapse into a toolbar (no duplicated titles).

**Search** (`CommandPalette`, Ctrl+K): Turkish-insensitive normalization (ı/i, ş/s, ğ/g, ü/u,
ö/o, ç/c, diacritics), every query word must match the label or keywords, results ranked by
label prefix. Choosing a result navigates and, when the entry has an `anchor`, scrolls to the
control, focuses it and flashes it. The index is a static array, so search costs nothing until
the palette is opened.

**Theme Library.** Cards render a `ThemeSandbox`: the compiled theme's CSS is scoped to
`[data-sandbox="<id>"]` and the structural attributes are written on the sandbox element. The
structural rules in `theme-styles.css` use `@scope (…) to (…)`, so the nearest theme root wins
and the app's active theme never leaks into a preview (and vice versa). "Uygulamada önizle" sets
an in-memory preview in the theme hook (`PreviewBar` offers Uygula / Vazgeç); nothing is written
to settings until "Uygula". Import reads the file in the WebView (no path crosses IPC), validates
it client-side for the preview dialog and then server-side in Rust (`import_theme`). Export
writes `<id>.ivilytheme` to the Downloads known folder without overwriting existing files.
