# Wakai Desktop → Ivily Desktop brand migration

## What changes

| Item | Wakai Desktop 1.1.0 | Ivily Desktop 1.1.5 |
|---|---|---|
| Product name (window titles, tray, Apps list, shortcuts, notifications) | Wakai Desktop | Ivily Desktop |
| Install directory (per-user NSIS) | `%LOCALAPPDATA%\Wakai Desktop` | `%LOCALAPPDATA%\Ivily Desktop` |
| Uninstall registry key | `HKCU\…\Uninstall\Wakai Desktop` | `HKCU\…\Uninstall\Ivily Desktop` |
| Start menu / desktop shortcut | `Wakai Desktop.lnk` | `Ivily Desktop.lnk` |
| Autostart value (`HKCU\…\Run`) | `Wakai Desktop` | `Ivily Desktop` |

## What does not change

| Item | Value | Why |
|---|---|---|
| Tauri identifier | `com.wakai.desktop` | Data directory, WebView2 profile and notification identity derive from it |
| Data directory | `%APPDATA%\com.wakai.desktop` | All user data (database, backups, previous versions) stays in place — nothing is copied or moved |
| Database file | `wakai.db` | Same reason; renaming a live SQLite file buys nothing and risks loss |
| Executable | `wakai-desktop.exe` | Technical identifier used by recovery/rollback and the installer's running-app check |
| Updater key and endpoint | unchanged | 1.1.0 installs must accept the signed 1.1.5 update |

Because the data directory is untouched, the migration cannot lose, duplicate or reset user
data: there is exactly one database before and after.

## Why the installer needs a migration step

Tauri's NSIS template derives the install directory, uninstall key and shortcut names from
`productName`. Installing "Ivily Desktop" over "Wakai Desktop" without help would leave:
two entries in *Apps*, the old program folder, the old shortcuts (pointing at the old 1.1.0
executable), and — in update mode — **no** new shortcuts, because the template only refreshes
existing shortcuts during updates.

The installer hook (`src-tauri/windows/installer-hooks.nsh`) handles this. The status is
recorded in the roadmap.
