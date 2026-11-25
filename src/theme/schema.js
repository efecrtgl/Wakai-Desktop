// Tema bildirimi (manifest) doğrulama ve göç. Temalar bildirimseldir: yalnızca sözleşmede
// (contract.json) adı geçen anahtarlar ve güvenli değerler kabul edilir; kod, CSS dosyası ya da
// dış adres içeremez. Aynı sözleşmeyi Rust tarafı (themes.rs) içe aktarmada da uygular.
import CONTRACT from './contract.json';

export { CONTRACT };
export const SCHEMA_VERSION = CONTRACT.schema;

const TOKENS = new Set(CONTRACT.tokens);
const ID_RE = /^[a-z0-9][a-z0-9-]{1,39}$/;
const VERSION_RE = /^\d{1,4}\.\d{1,4}\.\d{1,4}$/;
// Belirteç değerleri: renk, uzunluk, gölge, gradyan. url(), ifade ve kaçış karakterleri yasak.
const VALUE_RE = /^[#a-zA-Z0-9 .,%()+\-'"/]*$/;
const FORBIDDEN_RE = /url\s*\(|image-set|expression|attr\s*\(|src\s*\(|element\s*\(|cross-fade/i;
const FONT_RE = /^[A-Za-z0-9 ,'-]*$/;
const COLOR_RE = /^(#[0-9a-fA-F]{3,8}|rgba?\(\s*[0-9.,%\s]+\))$/;
const ASSET_RE = /^data:image\/(png|webp|gif|jpeg);base64,[A-Za-z0-9+/]+=*$/;

/** "1.2.3" karşılaştırması: -1, 0, 1. */
export function compareVersions(a, b) {
  const pa = String(a).split('.').map((n) => parseInt(n, 10) || 0);
  const pb = String(b).split('.').map((n) => parseInt(n, 10) || 0);
  for (let i = 0; i < 3; i++) {
    if ((pa[i] ?? 0) !== (pb[i] ?? 0)) return (pa[i] ?? 0) < (pb[i] ?? 0) ? -1 : 1;
  }
  return 0;
}

export function safeValue(v) {
  return typeof v === 'string' && v.length > 0 && v.length <= CONTRACT.limits.string && VALUE_RE.test(v) && !FORBIDDEN_RE.test(v);
}

const isObj = (v) => v !== null && typeof v === 'object' && !Array.isArray(v);
const text = (v, max = 120) => (typeof v === 'string' ? v.trim().slice(0, max) : '');

/**
 * Eski biçimleri güncel şemaya taşır. Şema 0 (alan yok): `colors` → `tokens`.
 * Bilinmeyen gelecekteki şema sürümleri taşınmaz; doğrulama reddeder.
 */
export function migrateTheme(raw) {
  if (!isObj(raw)) return raw;
  if (raw.schema === undefined || raw.schema === 0) {
    const { colors, ...rest } = raw;
    return { ...rest, schema: 1, tokens: { ...(colors ?? {}), ...(raw.tokens ?? {}) } };
  }
  return raw;
}

function cleanTokens(src, warnings, where) {
  const out = {};
  if (!isObj(src)) return out;
  for (const [k, v] of Object.entries(src)) {
    if (!TOKENS.has(k)) warnings.push(`${where}: bilinmeyen belirteç "${k}" yok sayıldı.`);
    else if (!safeValue(v)) warnings.push(`${where}: "${k}" değeri geçersiz, yok sayıldı.`);
    else out[k] = v;
  }
  return out;
}

function cleanEnum(value, options, fallback, warnings, name) {
  if (value === undefined) return fallback;
  if (options.includes(value)) return value;
  warnings.push(`"${name}" için geçersiz değer "${value}"; "${fallback}" kullanıldı.`);
  return fallback;
}

export const DEFAULT_STYLE = {
  shape: 'rounded',
  surface: 'soft',
  border: 'thin',
  icons: 'line',
  density: 'normal',
  texture: 'none',
  headings: 'normal',
  motion: 'smooth',
  rendering: 'pixelated',
};

/**
 * Bir tema bildirimini doğrular ve normalleştirir.
 * Ölümcül hatalar (`errors`) temanın hiç uygulanmamasına yol açar; uyarılar (`warnings`)
 * yalnızca hatalı parçanın yok sayılmasına (tema yine de çalışır).
 * @returns {{ ok: boolean, theme: object|null, errors: string[], warnings: string[] }}
 */
export function validateTheme(input, { appVersion = CONTRACT.app } = {}) {
  const errors = [];
  const warnings = [];
  const raw = migrateTheme(input);
  if (!isObj(raw)) return { ok: false, theme: null, errors: ['Tema dosyası bir nesne değil.'], warnings };
  if (typeof raw.schema !== 'number' || raw.schema > SCHEMA_VERSION || raw.schema < 1) {
    errors.push(raw.schema > SCHEMA_VERSION ? 'Bu tema daha yeni bir Ivily Desktop sürümü için hazırlanmış.' : 'Tema şema sürümü geçersiz.');
  }
  if (!ID_RE.test(raw.id ?? '')) errors.push('Tema kimliği geçersiz (küçük harf, rakam ve tire; 2–40 karakter).');
  const name = text(raw.name, 40);
  if (!name) errors.push('Tema adı eksik.');
  const version = VERSION_RE.test(raw.version ?? '') ? raw.version : '1.0.0';
  if (raw.version !== undefined && version !== raw.version) warnings.push('Tema sürümü geçersiz; 1.0.0 kabul edildi.');
  const minApp = VERSION_RE.test(raw.min_app ?? '') ? raw.min_app : CONTRACT.app;
  if (compareVersions(minApp, appVersion) > 0) errors.push(`Bu tema Ivily Desktop ${minApp} veya üstünü gerektiriyor.`);
  if (errors.length) return { ok: false, theme: null, errors, warnings };

  const style = {};
  const rawStyle = isObj(raw.style) ? raw.style : {};
  for (const [key, options] of Object.entries(CONTRACT.style)) {
    style[key] = cleanEnum(rawStyle[key], options, DEFAULT_STYLE[key], warnings, `style.${key}`);
  }

  const fonts = {};
  if (isObj(raw.fonts)) {
    for (const k of CONTRACT.fonts) {
      const v = raw.fonts[k];
      if (v === undefined) continue;
      if (typeof v === 'string' && v.length <= CONTRACT.limits.string && FONT_RE.test(v)) fonts[k] = v;
      else warnings.push(`fonts.${k} geçersiz, yok sayıldı.`);
    }
  }

  const base = raw.base === 'dark' ? 'dark' : 'light';
  const variants = {};
  if (isObj(raw.variants)) {
    const entries = Object.entries(raw.variants);
    if (entries.length > CONTRACT.limits.variants) warnings.push(`En fazla ${CONTRACT.limits.variants} çeşit kullanılır.`);
    for (const [vid, v] of entries.slice(0, CONTRACT.limits.variants)) {
      if (!ID_RE.test(vid) || !isObj(v)) {
        warnings.push(`Geçersiz çeşit "${vid}" yok sayıldı.`);
        continue;
      }
      variants[vid] = {
        name: text(v.name, 30) || vid,
        base: v.base === 'dark' ? 'dark' : v.base === 'light' ? 'light' : base,
        tokens: cleanTokens(v.tokens, warnings, `variants.${vid}`),
      };
    }
  }

  const home = { floor: CONTRACT.home.floor[0], wall: CONTRACT.home.wall[0], colors: {}, assets: {} };
  if (isObj(raw.home)) {
    home.floor = cleanEnum(raw.home.floor, CONTRACT.home.floor, home.floor, warnings, 'home.floor');
    home.wall = cleanEnum(raw.home.wall, CONTRACT.home.wall, home.wall, warnings, 'home.wall');
    if (isObj(raw.home.colors)) {
      for (const [k, v] of Object.entries(raw.home.colors)) {
        if (['floor', 'floor-alt', 'wall', 'wall-alt', 'trim', 'line'].includes(k) && typeof v === 'string' && COLOR_RE.test(v)) home.colors[k] = v;
        else warnings.push(`home.colors.${k} geçersiz, yok sayıldı.`);
      }
    }
  }

  const assets = {};
  const rawAssets = isObj(raw.assets) ? raw.assets : {};
  for (const [k, v] of Object.entries(rawAssets)) {
    if (!CONTRACT.assets.includes(k)) warnings.push(`Bilinmeyen varlık "${k}" yok sayıldı.`);
    else if (typeof v !== 'string' || !ASSET_RE.test(v) || v.length > (CONTRACT.limits.asset_bytes * 4) / 3 + 64) warnings.push(`Varlık "${k}" geçersiz ya da çok büyük, yok sayıldı.`);
    else assets[k] = v;
  }

  const sounds = {};
  if (isObj(raw.sounds)) {
    for (const [k, s] of Object.entries(raw.sounds)) {
      if (!CONTRACT.sounds.includes(k) || !isObj(s) || !CONTRACT.waves.includes(s.wave)) {
        warnings.push(`Ses "${k}" geçersiz, yok sayıldı.`);
        continue;
      }
      const num = (v, lo, hi, d) => (Number.isFinite(v) ? Math.min(hi, Math.max(lo, v)) : d);
      sounds[k] = { wave: s.wave, freq: num(s.freq, 40, 4000, 440), ms: num(s.ms, 5, 600, 60), gain: num(s.gain, 0, 0.2, 0.03) };
    }
  }

  const theme = {
    schema: SCHEMA_VERSION,
    id: raw.id,
    name,
    version,
    author: text(raw.author, 60),
    description: text(raw.description, 240),
    min_app: minApp,
    base,
    allow_accent: raw.allow_accent !== false,
    fonts,
    style,
    tokens: cleanTokens(raw.tokens, warnings, 'tokens'),
    variants,
    pets: { palette: cleanEnum(raw.pets?.palette, CONTRACT.pets.palette, 'none', warnings, 'pets.palette') },
    home,
    ambient: cleanEnum(raw.ambient, CONTRACT.ambient, 'none', warnings, 'ambient'),
    sounds,
    assets,
  };
  return { ok: true, theme, errors, warnings };
}
