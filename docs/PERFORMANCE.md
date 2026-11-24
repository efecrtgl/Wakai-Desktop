# Performance Log (technical)

> English technical log. The Turkish user-facing report is [PERFORMANS.md](PERFORMANS.md); its methodology applies here.
> Tool: `scripts/measure.ps1` (sums the whole process tree; CPU % is of **all** 32 logical cores).

## Machine

Windows 11 Home build 26200, 32 logical cores, 31.2 GB RAM, two monitors (125 % + 100 %), WebView2 158.0.4370.33.

## Baseline — 1.0.0 (installed release build), 24 Nov 2026

| Scenario | Processes | Private (avg) | Working set (avg) | CPU avg / max |
|---|---|---|---|---|
| 8 widgets, Control Center closed, 60 s idle | 11 | 375.3 MB | 377.7 MB | 0.05 % / 1 % |
| All widgets hidden, 30 s | 1 | 2.7 MB | 15.1 MB | 0 % |
| Startup to all WebViews ready (autostart, 8 widgets), 3 runs | — | — | — | 884 / 983 / 784 ms |

Test results at baseline: 12/12 frontend, 55 unit + 6 integration Rust tests, lint and clippy clean.

## 1.1.0 measurements

Appended per phase below.

### Phase 12 — 1.1.0 release build (pre-version-bump), 25 Nov 2026

Release binary (`tauri build --no-bundle`), isolated data dir (`WAKAI_DATA_DIR`) seeded with a copy of the real DB, `--autostart`, 15 s settle, `scripts/measure.ps1 -Duration 60`.

| Scenario | Processes | Private (avg) | Working set (avg) | CPU avg / max |
|---|---|---|---|---|
| A: 8 widgets, pets off, Control Center closed (= baseline scenario) | 11 | 362.2 MB (baseline 375.3) | 354.1 MB (baseline 377.7) | 0.05 % / 2 % |
| B: 8 widgets + 2 desktop pets | 11 | 355.1 MB | 349.6 MB | 0.03 % / 1 % |
| C: widgets hidden + 2 desktop pets (pet engine only) | 1 | 3.0 MB (baseline hidden 2.7) | 17.4 MB | ~0 % / 0 % |

The native pet engine adds about 0.3 MB and no measurable CPU; it creates no WebView (2 `WakaiPet` layered windows verified, 6 threads total). An earlier run of C showed 7 processes / 108 MB. That came from WebView2 children of a force-killed configuration instance that hadn't exited yet. Re-measured after confirming no leftover processes.
Startup timing was not re-measured.
