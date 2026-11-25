# Ivily Theme API (schema 1)

Themes change the visual language of the whole application — Control Center, desktop widgets,
desktop cats, the Pet World — **without code changes**. A theme is a declarative JSON document;
it contains no CSS, no JavaScript and no file paths. Widgets and the Pet World only consume the
semantic tokens and structural options described here, so a new theme never requires rewriting
a widget, the pet engine or the world engine.

Single source of truth for allowed names and limits: [`src/theme/contract.json`](../src/theme/contract.json)
(read by the frontend validator and embedded in the Rust import gate).

## 1. Theme pack (`.ivilytheme`)

A theme pack is one UTF-8 JSON file with the extension `.ivilytheme` (max 2 MB). Assets are
embedded as data URIs, so a pack never references files. Example:
[`docs/themes/gece-mavisi.ivilytheme`](themes/gece-mavisi.ivilytheme).

```json
{
  "schema": 1,
  "id": "gece-mavisi",
  "name": "Gece Mavisi",
  "version": "1.0.0",
  "author": "Ivily örnek",
  "description": "Koyu lacivert, yumuşak köşeler.",
  "min_app": "1.1.5",
  "base": "dark",
  "allow_accent": true,
  "fonts": { "display": "Bahnschrift, Segoe UI, sans-serif" },
  "style": { "shape": "soft", "surface": "soft", "icons": "line", "texture": "dots" },
  "tokens": { "color-bg": "#0f1730", "color-primary": "#7aa2ff" },
  "variants": { "light": { "name": "Gündüz", "base": "light", "tokens": { "color-bg": "#eef2ff" } } },
  "pets": { "palette": "cool" },
  "home": { "floor": "carpet", "wall": "plain", "colors": { "floor": "#2b3a6b" } },
  "ambient": "stars",
  "sounds": { "click": { "wave": "sine", "freq": 740, "ms": 35, "gain": 0.03 } },
  "assets": { "preview": "data:image/png;base64,…" }
}
```

## 2. Fields

| Field | Required | Meaning |
|---|---|---|
| `schema` | yes | Contract version. Currently `1`. Missing → treated as schema 0 and migrated. Higher than supported → rejected. |
| `id` | yes | `[a-z0-9][a-z0-9-]{1,39}`. Must not equal a built-in id (`pixel`, `modern`, `cozy`, `mono`, `y2k`). |
| `name` | yes | ≤ 40 characters, shown in the library. |
| `version` | no | Theme's own semver `x.y.z` (default `1.0.0`). Importing the same `id` again offers "Güncelle". |
| `author`, `description` | no | ≤ 60 / ≤ 240 characters. |
| `min_app` | no | Minimum Ivily version (default `1.1.5`). Newer than the running app → rejected with a Turkish message. |
| `base` | no | `light` or `dark`: `color-scheme` and window title bar. |
| `allow_accent` | no | `false` disables the user's accent colour (e.g. Monochrome). |
| `fonts` | no | `sans`, `display`, `mono`: font stacks of installed font names (`[A-Za-z0-9 ,'-]`). `Pixelify Sans` is bundled. |
| `style` | no | Structural options, see §3. Invalid values fall back to the defaults. |
| `tokens` | no | Semantic CSS tokens, see §4. |
| `variants` | no | Up to 4 variants: `{ name, base, tokens }`; tokens are applied on top of the theme. |
| `pets` | no | `palette`: `none`, `grayscale`, `warm`, `cool`, `neon`, `pastel` — applied to cats in the Control Center, the Pet World and the native desktop cats. |
| `home` | no | Pet World defaults: `floor` (`planks`, `tiles`, `checker`, `carpet`, `plain`, `grid`), `wall` (`stripes`, `dots`, `plain`, `brick`, `panel`, `grid`), `colors` (`floor`, `floor-alt`, `wall`, `wall-alt`, `trim`, `line` as `#hex` or `rgb()`). Users can override floor/wall per room; their choice survives theme changes. |
| `ambient` | no | Pet World particles: `none`, `dust`, `leaves`, `stars`, `sparkles`. Off with reduced motion or when the user disables effects. |
| `sounds` | no | `click`, `success`, `pet`, `drop`, `error`: `{ wave: sine|square|triangle|sawtooth, freq 40–4000, ms 5–600, gain 0–0.2 }`. Synthesized; only played when the user enables UI sounds. |
| `assets` | no | `floor`, `wall`, `preview`: `data:image/(png|webp|gif|jpeg);base64,…`, ≤ 512 KB each. The file signature must match the declared type. Decoded lazily. |

## 3. Structural options (`style`)

| Option | Values | Effect |
|---|---|---|
| `shape` | `rounded`, `soft`, `sharp`, `pixel` | Corner language; `pixel` adds 2 px hard borders, offset shadows, press-down buttons, dashed focus. |
| `surface` | `flat`, `soft`, `paper`, `glossy` | Card and button surfaces (paper grain, chrome highlights). |
| `border` | `none`, `thin`, `pixel`, `double` | Card and widget borders. |
| `icons` | `pixel`, `line` | Icon set (12×12 pixel icons or 24 px vector line icons). |
| `density` | `compact`, `normal`, `roomy` | Spacing scale (the user can override it). |
| `texture` | `none`, `paper`, `grid`, `scanline`, `dots` | Background texture of the main area. |
| `headings` | `normal`, `upper` | Uppercase, tracked headings. |
| `motion` | `smooth`, `stepped`, `bouncy` | Easing character of transitions. |
| `rendering` | `pixelated`, `smooth` | Image rendering of embedded images. |

These map to `data-*` attributes; the rules live in `src/ui/theme-styles.css` and use
`@scope (…) to (…)` so the nearest theme root wins (needed for isolated previews).

## 4. Tokens

All names are listed in `contract.json → tokens`. Groups:

- Surfaces: `color-bg`, `color-surface`, `color-surface-soft`, `color-surface-sunken`, `color-highlight`
- Brand: `color-primary`, `color-primary-strong`, `color-primary-deep`, `color-primary-soft`, `color-accent-soft`
- Text: `color-text`, `color-text-strong`, `color-text-muted`, `color-text-faint`, `color-text-on-primary`
- Lines and controls: `color-border`, `color-border-strong`, `color-focus`, `color-track`, `color-switch-thumb`
- States: `color-success(-soft)`, `color-danger(-soft)`, `color-weekend`, badges, weather colours
- Navigation and chrome: `color-nav-bg`, `color-nav-text`, `color-nav-active`, `color-nav-active-text`, `color-chrome`
- Shape: `shadow-sm|md|lg|widget`, `radius-sm|md|lg`, `border-width`
- Labels: `label-pink|lavender|peach|mint|sky|butter`
- Vector cat (sidebar logo): `cat-outline`, `cat-fur`, …

Values may be colours, lengths, shadows and gradients. Rejected: `url(`, `image-set`,
`expression`, `attr(`, `src(`, `element(`, `cross-fade`, and any of `; { } < > \ @ !`. Unset
tokens inherit the fallback (Modern) values from `src/ui/tokens.css`.

**Accessibility:** built-in themes are tested for text/background contrast ≥ 4.5:1 and muted
text ≥ 3:1 (`src/theme/engine.test.js`). Theme authors should meet the same targets; the
accessibility settings (text size, high contrast) apply on top of any theme.

## 5. Validation, isolation and recovery

1. **Import (Rust, strict):** size, JSON, unknown top-level keys, id, schema, `min_app`, every
   token value, enums, fonts, colours, sounds, assets (type + signature + size). Any violation
   rejects the whole pack with a Turkish message; nothing is written.
2. **Runtime (frontend, tolerant):** `validateTheme` drops invalid parts with warnings, so a
   theme saved by an older/newer build still renders.
3. **Fallback:** missing, incompatible or broken themes resolve to Modern; `tokens.css` keeps the
   UI usable even if no theme CSS applies; `ThemeBoundary` catches render errors.
4. **Removal:** deleting a custom theme never touches user data; if it was active, Pixel is applied.

## 6. Versioning and migration

- `schema` changes only when the contract changes incompatibly. `migrateTheme()` upgrades older
  schemas (schema 0: `colors` → `tokens`).
- `version` is the theme's own release number; re-importing a pack with the same id replaces it
  after confirmation.
- `min_app` lets a theme require newer contract features.

## 7. Creating a theme

1. Start from a built-in (`Temalar → Dışa aktar` writes `<id>.ivilytheme` to Downloads) or from
   the example pack.
2. Change `id`, `name`, tokens and style options.
3. `Temalar → Tema içe aktar`, check the sandbox preview and the warning count, apply.
4. Optional: `npm test` validates built-ins; for packs, the import dialog shows every problem.

Performance budget: themes are a single `<style>` element with ~60 variables; switching costs one
style recalculation (measured ~117 ms including the settings round-trip on the dev machine).
Embedded images are capped at 512 KB each and decoded only when used.
