# Ivily Desktop 1.1.5 — Security review

Scope: changes made for 1.1.5 (brand migration, theme engine and import, Pet World, updater
channels, installer hooks). Reviewed against the existing 1.1.0 baseline (docs/PERFORMANCE.md,
"Security review").

## Permissions and IPC

- **Capabilities unchanged.** `src-tauri/capabilities/default.json` still grants only event and
  window show/hide/focus/minimize/close/drag permissions. No filesystem, shell, dialog or HTTP
  plugin was added. All new features use custom Rust commands.
- **Window guard.** Commands that change persistent state beyond simple content are restricted to
  the Control Center window (`require_main`): `import_theme`, `delete_custom_theme`,
  `place_world_item`, `move_world_item`, `remove_world_item`, `fill_world_bowl`,
  `set_world_room_style`, `save_world_positions`, `world_event` (in addition to the 1.1.0 guards
  on restore, rollback, update installation and backup deletion). E2E test: a widget window calling
  a guarded command gets "Bu işlem yalnızca Kontrol Merkezi'nden yapılabilir."
- **Input validation.** All world commands validate room ids, item kinds (catalog), bounds,
  layers, overlap, inventory counts and finite coordinates (clamped). Reward events have per-pet
  cooldowns, so scripted calls cannot farm progression.

## Theme packs

- **Declarative only.** A pack is JSON; it cannot contain CSS, JavaScript, HTML or file paths. The
  frontend never injects pack text as markup; only validated values are written into CSS custom
  properties.
- **Strict import gate (Rust, `themes::validate_pack`).** Size ≤ 2 MB; unknown top-level keys
  rejected; id `[a-z0-9-]{2,40}`; built-in ids cannot be overwritten; schema and `min_app`
  checked; every token value restricted to `[#a-zA-Z0-9 .,%()+-'"/]` with `url(`, `image-set`,
  `expression`, `attr(`, `src(`, `element(`, `cross-fade` rejected (case/space-insensitive);
  fonts, colours, enums and sound numbers checked; images only as `data:image/(png|webp|gif|jpeg)`
  with matching file signature and ≤ 512 KB. Any violation rejects the whole pack.
- **No path traversal.** Import receives file *contents* read in the WebView (no path crosses
  IPC). Export writes `<id>.ivilytheme` (id already validated) into the Downloads known folder and
  never overwrites an existing file.
- **CSP unchanged** (`img-src 'self' data:`, `font-src 'self'`, `connect-src ipc:`): even a value
  slipping past validation could not load remote content.
- Tests: `themes::tests::rejects_css_injection_and_urls`, `rejects_unknown_keys_tokens_and_enums`,
  `assets_must_be_real_images`, `size_limit`, frontend `CSS enjeksiyonunu ve dış adresleri reddeder`.

## Updater

- Signature verification unchanged (minisign public key embedded, plugin verifies before install).
- Channel endpoints are compile-time HTTPS constants; the debug-only endpoint override is still
  compiled out of release builds.
- Stable channel uses GitHub's `releases/latest` which never resolves to a pre-release or draft.
- The release workflow now creates **draft** releases; nothing reaches users until the owner
  publishes. The signing key is read only from the `TAURI_SIGNING_PRIVATE_KEY` secret and never
  printed; no key material is in the repository (`git ls-files` checked).

## Installer hooks

- The hooks only touch: `HKCU\…\Uninstall\Wakai Desktop`, `HKCU\Software\Wakai Desktop\Wakai Desktop`,
  the `Run\Wakai Desktop` value, `Wakai Desktop.lnk` in Start menu/Desktop, and three known files
  in the legacy folder. `RMDir` is non-recursive. The data directory is never touched and the old
  uninstaller (which can delete app data) is never run.
- Verified in an isolated variant (product "Ivily Hook Test", identifier `com.ivily.hooktest`)
  against a fake legacy install in update and install modes; a user file in the legacy folder was
  kept; the real installation was untouched.

## Dependencies

- Added: `@fontsource/pixelify-sans` 5.3.0 (font files only, SIL OFL 1.1). Rust: `windows` crate
  feature `Win32_System_Registry` (registry value deletion for the legacy autostart entry).
- `npm install` reports 0 vulnerabilities.

## Residual risks

- The repository is private; installed apps cannot reach update metadata until it is public.
- Theme author text (name, description) is rendered as React text (escaped), never as HTML.
