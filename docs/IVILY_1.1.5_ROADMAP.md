# Ivily Desktop 1.1.5 — Roadmap

Target: **Ivily Desktop 1.1.5 Pre-release** (formerly Wakai Desktop 1.1.0).
Statuses: NOT STARTED · IN PROGRESS · IMPLEMENTED · TESTED · COMMITTED · PUSHED · BLOCKED.
A phase is only marked TESTED when its tests were actually executed, and PUSHED only after
`git push` was verified (`git status` reports the branch up to date with `origin/main`).

## Ordering decision

The directive lists the UI redesign (phase 2) before the Theme Engine (phase 3). The redesign
is built **on top of** the theme primitives (shape, surface, border, icon set, density), so the
engine and its CSS contract are implemented first and the redesigned shell immediately uses
them. Redesigning first would mean styling every screen twice. Everything else follows the
directive's order; dependent phases are merged where they are one piece of code (e.g. World,
Interaction, Inventory and Pet Home share one renderer and one save path).

## Phases

| # | Phase | Scope | Status | Tests | Commit | Pushed |
|---|---|---|---|---|---|---|
| 0 | Audit & baseline | Repository, architecture, installer identity, updater, data paths, baseline tests | COMMITTED | baseline suite executed: all green (see TESTING) | see git log (docs: establish Ivily 1.1.5 roadmap) | yes |
| 1 | Brand migration | Product name, UI strings, tray, notifications, installer hooks for the Wakai→Ivily install move, legacy autostart cleanup, one-time notice | NOT STARTED | – | – | – |
| 3 | Theme Engine | Declarative manifest, validation, isolation, fallback, atomic switching, versioning, lazy assets | NOT STARTED | – | – | – |
| 4 | Five themes | Pixel-Art (default), Modern, Cozy, Monochrome, Y2K | NOT STARTED | – | – | – |
| 2 | UI redesign | Navigation, overview, widget management, settings categories, search | NOT STARTED | – | – | – |
| 5 | Theme Library | Browse, favorite, live preview (sandboxed), customization, import/export | NOT STARTED | – | – | – |
| 6 | Accessibility | Reduced motion, animation intensity, high contrast, text size, focus, shortcuts | NOT STARTED | – | – | – |
| 7–17 | Pet World | World/rooms/tiles/collision, A* navigation, utility AI, needs, animation state machine, drag & drop, toys, inventory/building, rooms, day/night, weather, offline progress, progression, save | NOT STARTED | – | – | – |
| 18 | Workspace profiles | Theme + animation + pet settings per profile | NOT STARTED | – | – | – |
| 19 | Performance | Measurements vs. 1.1.0 baseline | NOT STARTED | – | – | – |
| 21 | Updater | Stable / pre-release channels, release workflow | NOT STARTED | – | – | – |
| 22 | Installer | Ivily identity, upgrade from Wakai installs | NOT STARTED | – | – | – |
| 23 | Security | Theme packs, IPC, permissions review | NOT STARTED | – | – | – |
| 24 | Testing | Unit, integration, E2E, manual | NOT STARTED | – | – | – |
| 25/27 | Docs & release prep | Docs, version 1.1.5, Turkish release notes, artifacts | NOT STARTED | – | – | – |

## Known issues found during the audit

- The GitHub repository is not publicly reachable (`https://github.com/efecrtgl/Wakai-Desktop`
  returns 404 without authentication). The updater endpoint (`releases/latest/download/latest.json`)
  therefore cannot be reached by installed apps until the repository (or a separate release
  repository) is public. Owner decision; documented in `docs/GUNCELLEMELER.md`.
- The v1.1.0 release workflow needs the `TAURI_SIGNING_PRIVATE_KEY` secret (owner action from 1.1.0).
- Renaming `productName` changes the NSIS install directory, uninstall key and shortcut names
  (see `IVILY_BRAND_MIGRATION.md`); handled by installer hooks in phase 1.
