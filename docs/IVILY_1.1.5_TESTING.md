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
