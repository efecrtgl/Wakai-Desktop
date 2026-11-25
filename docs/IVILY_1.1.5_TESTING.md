# Ivily Desktop 1.1.5 — Testing

Only tests that were actually executed are reported as passed. Manual scenarios that were not
executed are listed as such.

## Baseline (Wakai Desktop 1.1.0, commit 55590c4, 2026-11-25)

| Suite | Command | Result |
|---|---|---|
| ESLint | `npm run lint` | clean |
| Frontend unit (Vitest) | `npm test` | 21 / 21 passed (6 files) |
| Rust unit | `cargo test` | 94 passed, 2 ignored (Windows runtime tests) |
| Rust integration | `cargo test --test integration` | 6 / 6 passed |
| Clippy | `cargo clippy --all-targets -- -D warnings` | clean |
| E2E (CDP) | `node scripts/e2e.mjs` | 19 / 19 (last run during 1.1.0, isolated data dir) |

Baseline performance (from `docs/PERFORMANCE.md`, measured during 1.1.0 on the development
machine): 8 widgets with the Control Center closed 362.2 MB private memory; 8 widgets + 2
desktop cats 355.1 MB; widgets hidden + 2 cats 3.0 MB, ≈0 % CPU; startup 677–753 ms (6 widgets).

No pre-existing failures.

## Ivily Desktop 1.1.5 results (25 Nov 2026, final commit of the release)

| Suite | Command | Result |
|---|---|---|
| ESLint | `npm run lint` | clean |
| Frontend unit (Vitest) | `npm test` | **52 / 52** passed (9 files; was 21) |
| Rust unit | `cargo test` | **122** passed, 2 ignored (was 94) |
| Rust integration | `cargo test --test integration` | 6 / 6 passed |
| Clippy | `cargo clippy --all-targets -- -D warnings` | clean |
| rustfmt | `cargo fmt --check` | clean |
| E2E (CDP, isolated copy of real data) | `node scripts/e2e.mjs` | **22 / 22** passed (was 19) |
| Installer migration (isolated variant) | `hooktest.ps1` (scratch, see IVILY_SECURITY.md) | update mode and install mode: all 13 checks passed |

### New automated coverage
- **Brand:** brand notice only for upgraders; legacy file removal conditions; unknown files kept.
- **Theme engine:** built-ins validate without warnings; CSS injection/URLs rejected; schema 0 →
  1 migration; fatal vs. tolerant errors; fallback; accent with readable text colour; density
  override; WCAG AA contrast for every built-in theme and variant; pack import gate (Rust) incl.
  image signatures, size limit, built-in id protection; store import/replace/delete; settings
  migration from 1.1.0 appearance; settings sanitization.
- **Navigation/search:** legacy routes, unknown routes, Turkish-insensitive search, every search
  entry resolves.
- **Pet World:** catalog consistency; layers/bounds/rotation; overlap rules; idempotent init with
  inventory conservation; legacy Kedi Evi migration; place/move/remove round trip; levels and
  unlocks once; bounded habits; personality-dependent deltas; bowls; offline progress (24 h cap);
  eat event consumes and rewards; friendship memory; A* around obstacles; unreachable → null and
  recovery; AI eats/begs, sleeps at night in favourite bed, shy cat hides, playful vs lazy play
  frequency, chases moving toys; animation lock and wake→stretch; toy physics stays in room;
  game clock, periods, deterministic weather (no summer snow); desktop-cat palette equals frontend.
- **Profiles:** theme applied only when set and valid.
- **Updater:** stable channel never points at pre-releases.

### Manual / CDP checks executed
- 1.1.0 database copy → migrations 007 and 008 applied, legacy appearance "dark" → Modern/Siyah,
  9 legacy Kedi Evi items moved into the living room.
- Screenshots of Pixel, Modern (dark), Y2K app-wide preview, theme library sandboxes, settings,
  widget window under Pixel, custom theme applied app-wide.
- Search "yazi boyu" → "Yazı boyutu" → navigated and highlighted.
- Pet World with synthetic mouse events: petting by rubbing (+social), carry and drop, throwing a
  ball (velocity applied), clicking the bowl (filled, cat chose "eat", persisted), placing a cat
  tree from the inventory.
- A render error during development was caught by ThemeBoundary and the fallback UI appeared.

### Bugs found by these tests and fixed
- Habit eviction removed the newest entry on ties (new habits never stuck).
- Bowl portions desynchronised when a reward was on cooldown.
- A deleted component import (Appearance.jsx) was still referenced (ESLint does not resolve imports; caught by the dev server).
- An effect returned a Promise (caught by ThemeBoundary).
- Pointer mapping was wrong when the canvas was CSS-scaled in a narrow window.

### Not executed
- A real NSIS **update through the updater** (needs a published, publicly reachable release).
- Upgrading the user's real Wakai Desktop 1.1.0 installation (deliberately not done; the hook was
  verified in an isolated variant instead).
- Real (physical) mouse and touch input in the Pet World; long-running (multi-hour) soak test.
- Different DPI scales and multi-monitor layouts for the new UI (the window adapts with CSS grid
  breakpoints; not measured on other displays).
