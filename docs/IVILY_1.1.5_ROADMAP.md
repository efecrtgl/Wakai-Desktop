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
| 0 | Audit & baseline | Repository, architecture, installer identity, updater, data paths, baseline tests | COMMITTED | baseline suite executed: all green (see TESTING) | 90d5cd4 | yes |
| 1 | Brand migration | Product name, UI strings, tray, notifications, installer hooks for the Wakai→Ivily install move, legacy autostart cleanup, one-time notice | PUSHED | cargo test (97 + 6), clippy, lint; NSIS build with hooks compiles. Installer upgrade run: see phase 22 | 6c8ee22 | yes |
| 3 | Theme Engine | Declarative manifest, validation, isolation, fallback, atomic switching, versioning, lazy assets | PUSHED | 12 Vitest (schema, injection, migration, fallback, WCAG contrast), 9 Rust (pack validation, store); manual: legacy dark → Modern/Siyah, switch to Pixel in 117 ms incl. IPC | 48686ff | yes |
| 4 | Five themes | Pixel-Art (default), Modern, Cozy, Monochrome, Y2K | PUSHED | validated + contrast-tested; screenshots of Modern/Pixel | 48686ff | yes |
| 2 | UI redesign | Navigation, overview, widget management, settings categories, search | PUSHED | 6 Vitest (routes, legacy ids, Turkish search); CDP: overview, settings, search "yazi boyu" → anchor, widget window themed; a render error during development was caught by ThemeBoundary (isolation verified) | 2852786 | yes |
| 5 | Theme Library | Browse, favorite, live preview (sandboxed), customization, import/export | PUSHED (same commit as phase 2) | CDP: sandbox isolation (Modern card inside Pixel app), app-wide preview not persisted, import → conflict → unsafe pack rejected → apply → export | 2852786 | yes |
| 6 | Accessibility | Reduced motion, animation intensity, high contrast, text size, focus, shortcuts | PUSHED | Settings page (2852786), skip link + focusable main (8b607a8); WCAG contrast test on all themes; Pet World respects reduced/off motion (lower fps, no particles, static frames) | 2852786, 8b607a8 | yes |
| 7–17 | Pet World | World/rooms/tiles/collision, A* navigation, utility AI, needs, animation state machine, drag & drop, toys, inventory/building, rooms, day/night, weather, offline progress, progression, save | PUSHED | Rust: 13 world tests (catalog, layers/rotation, overlap, idempotent init + legacy migration, inventory conservation, levels/unlocks, habits, deltas, bowls, offline progress, friendship memory) + palette test; Vitest: 13 (placement, A*, unreachable/recovery, AI needs/night/fear/personality/chase, animation state machine, toy physics, clock/weather). CDP on a copy of real data: migration 006→008, legacy items moved, petting by rubbing (+social), carry and drop, throw ball, click bowl → cat eats (persisted), place tree from inventory. Found and fixed: habit eviction bug, bowl desync with event cooldown | e4c5d55 | yes |
| 18 | Workspace profiles | Theme + animation + pet settings per profile | PUSHED | Rust: profile_theme_is_applied_only_when_set; profiles already existed (1.1.0) and keep Pet World state untouched (only settings fields are patched) | b277f8b | yes |
| 19 | Performance | Measurements vs. 1.1.0 baseline | COMMITTED | same-data comparison 1.1.0 vs 1.1.5, Pet World CPU, theme switch timing (docs/PERFORMANCE.md) | see git log | see git log |
| 21 | Updater | Stable / pre-release channels, release workflow | PUSHED | Rust: stable_channel_never_points_at_prereleases; release workflow now creates drafts (no publishing without the owner); prerelease-channel workflow copies latest.json on publish. Not executed: a real GitHub release (repository private, publishing not authorized) | 56b7c7f | yes |
| 22 | Installer | Ivily identity, upgrade from Wakai installs | COMMITTED | isolated hook variant vs fake legacy install, update + install modes, 13/13 checks each; signed 1.1.5 installer built | see git log | see git log |
| 23 | Security | Theme packs, IPC, permissions review | COMMITTED | docs/IVILY_SECURITY.md; window guard added to world commands | see git log | see git log |
| 24 | Testing | Unit, integration, E2E, manual | COMMITTED | 52 Vitest, 122 Rust unit, 6 integration, 22 E2E (docs/IVILY_1.1.5_TESTING.md) | see git log | see git log |
| 25/27 | Docs & release prep | Docs, version 1.1.5, Turkish release notes, artifacts | COMMITTED | README, THEME_API, KEDI_EVI, release notes, architecture; signed installer + .sig + latest-1.1.5.json built locally. GitHub Release NOT published (not authorized) | see git log | see git log |

## Known issues found during the audit

- The GitHub repository is not publicly reachable (`https://github.com/efecrtgl/Wakai-Desktop`
  returns 404 without authentication). The updater endpoint (`releases/latest/download/latest.json`)
  therefore cannot be reached by installed apps until the repository (or a separate release
  repository) is public. Owner decision; documented in `docs/GUNCELLEMELER.md`.
- The v1.1.0 release workflow needs the `TAURI_SIGNING_PRIVATE_KEY` secret (owner action from 1.1.0).
- Renaming `productName` changes the NSIS install directory, uninstall key and shortcut names
  (see `IVILY_BRAND_MIGRATION.md`); handled by installer hooks in phase 1.
