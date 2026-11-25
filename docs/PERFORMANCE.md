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

### Phase 12 — startup and security hardening

Startup was measured the same way as the baseline: time from launch (`--autostart`) until the WebView2 process count for `com.wakai.desktop` stays stable for 2 s. Release build, isolated data dir, pets enabled.

| Run | 1.1.0 (6 widgets + 2 pets) | 1.0.0 baseline (8 widgets) |
|---|---|---|
| 1 | 677 ms | 884 ms |
| 2 | 687 ms | 983 ms |
| 3 | 753 ms | 784 ms |

These numbers aren't directly comparable: when this measurement was taken, the live database copied for the test had 6 widgets instead of 8. The startup work added in 1.1.0 (launch health file, version record, deferred update check on its own thread 20 s later, native pet engine thread) doesn't block window creation.

Security review:
- Capabilities stay minimal (`core:event`, a few window ops). The updater plugin's JS permissions are not granted; all update logic runs through Rust commands.
- Destructive commands (`install_update`, `rollback_to_version`, `restore_backup`, `restore_backup_selective`, `delete_backup`) now reject calls from any window other than the Control Center (`commands::require_main`).
- `WAKAI_UPDATE_ENDPOINT` is honored only in debug builds. In any build, signatures are always verified against the embedded public key.
- Version and backup names are validated against path traversal (`../`) in Rust (unit-tested). Selective restore only touches a fixed table list, and column names come from `PRAGMA table_info`.
- No private key in the repository (checked before each commit). The CI fails if the signing secret is missing instead of publishing an unsigned release.

## Ivily Desktop 1.1.5 (25 Nov 2026)

Method: 1.1.0 (installed) and 1.1.5 (release build) each ran on a **fresh copy of the same
database** (6 widgets, all made visible, desktop cats on), started with `--autostart` (Control
Center closed), no CDP during measurement, 15 s settle, then `scripts/measure.ps1 -Duration 40`.

| Scenario | 1.1.0 | 1.1.5 | Δ |
|---|---|---|---|
| Startup until WebView2 process count is stable | 736 ms | 699 ms | −5 % |
| WebView2 processes | 9 | 9 | 0 |
| Private memory, average | 316.9 MB | 319.2 MB | +2.3 MB (+0.7 %) |
| Private memory, peak | 318.7 MB | 323.4 MB | +4.7 MB |
| CPU average (32 logical cores) | 0.05 % | 0.025 % | ≈ 0 |

The +2–5 MB is within run-to-run noise of earlier measurements; plausible sources are the theme
engine module and the bundled Pixelify Sans subsets.

Control Center open (1.1.5, CDP enabled, so memory is not comparable with the table above; CPU is):

| Scenario | Private memory | CPU (of 32 cores) |
|---|---|---|
| Overview open | 379.0 MB | 0.00 % |
| Pet World open, 2 cats, 30 fps cap | 386.1 MB (+7.1 MB) | 0.23 % (≈ 7.5 % of one core) |

The Pet World loop stops when the tab is left or the window is hidden, so it costs nothing in the
background. Reduced motion lowers it to 12 fps, animations off to 6 fps.

Theme switching (settings write + atomic CSS application + two animation frames, 10 switches
across all five themes): **median 23 ms**, min 11 ms, max 150 ms (first switch to Pixel loads the
font subset).
