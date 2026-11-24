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
