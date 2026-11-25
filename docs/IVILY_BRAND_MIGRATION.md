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

## Implementation

### Installer hook — `src-tauri/windows/installer-hooks.nsh`
Registered through `bundle.windows.nsis.installerHooks`.

**PREINSTALL**
1. Locate the legacy folder: `InstallLocation` of `HKCU\…\Uninstall\Wakai Desktop`
   (quotes stripped), otherwise `%LOCALAPPDATA%\Wakai Desktop` if it contains
   `wakai-desktop.exe`. If it equals the new `$INSTDIR`, nothing is migrated.
2. If a Wakai instance runs from the legacy folder, the template's Restart-Manager based
   `CheckIfAppIsRunning` macro closes it (asks first in interactive mode).

**POSTINSTALL** (only when a legacy folder was found)
1. Each legacy shortcut (`$SMPROGRAMS\Wakai Desktop.lnk`, `$DESKTOP\Wakai Desktop.lnk`) is
   deleted and an Ivily shortcut with the same AppUserModelID is created in its place. Users who
   had no desktop shortcut do not get one.
2. The legacy autostart value `HKCU\…\Run\Wakai Desktop` is deleted.
3. Only the files the Wakai installer created are deleted (`wakai-desktop.exe`,
   `wakai-desktop.exe.old`, `uninstall.exe`); the folder is removed only if it is then empty
   (`RMDir` without `/r`).
4. `HKCU\…\Uninstall\Wakai Desktop` and `HKCU\Software\Wakai Desktop\Wakai Desktop` are deleted,
   so *Apps* shows a single "Ivily Desktop" entry.

The old uninstaller is deliberately **not** run: its "delete app data" option targets the
shared data directory.

### Application side — `src-tauri/src/legacy.rs` (release builds only)
On every start, in a background thread:
- Deletes the legacy `Run\Wakai Desktop` value if it is still there.
- If "Windows ile başlat" is enabled, re-registers autostart under the new name and path.
- If the legacy uninstall key is gone (the hook ran) but `wakai-desktop.exe` is still in the
  legacy folder (it was locked while the updater's process exited), deletes the same known
  files and the empty folder. It never deletes anything while the uninstall key still exists
  or when the app itself runs from the legacy folder.

### One-time notice
`brand_notice` returns true when `app_versions` contains a version below 1.1.5 and the
`brand_notice_dismissed` setting is absent. The Control Center then shows a small banner
"Wakai Desktop artık Ivily Desktop." with a single "Tamam" button. Fresh installs never see it.

## Interrupted or failed migration
- The data directory is never part of the migration, so an interrupted install cannot affect
  user data.
- If the hook stops before step 3, the legacy files and uninstall entry remain; the next
  Ivily install or update finds them again and repeats the steps (every step is idempotent).
- If the legacy executable cannot be deleted, it is deleted by Ivily on its next start
  (see above).
- Rolling back to 1.1.0 from Settings swaps the executable inside the Ivily folder; the 1.1.0
  binary keeps working with the same data directory.

## Verification
See `IVILY_1.1.5_TESTING.md` (installer upgrade scenario).
