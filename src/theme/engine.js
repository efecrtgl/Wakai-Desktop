// Tema motoru: doğrulanmış bir temayı CSS değişkenlerine ve yapı özniteliklerine derler ve
// tek adımda uygular. Uygulama sırasında hata olursa önceki çalışan tema korunur; hiçbir şey
// çalışmazsa tokens.css içindeki varsayılanlar (Modern) devrede kalır.
import pixel from './builtin/pixel.json';
import modern from './builtin/modern.json';
import cozy from './builtin/cozy.json';
import mono from './builtin/mono.json';
import y2k from './builtin/y2k.json';
import { validateTheme } from './schema.js';

export const DEFAULT_THEME_ID = 'pixel';
export const FALLBACK_THEME_ID = 'modern';

/** Yerleşik temalar (doğrulanmış). Bozuk bir yerleşik tema listeye girmez. */
export const BUILTIN_THEMES = [pixel, modern, cozy, mono, y2k]
  .map((raw) => {
    const r = validateTheme(raw);
    if (!r.ok) console.error('yerleşik tema geçersiz', raw.id, r.errors);
    return r.ok ? { ...r.theme, builtin: true } : null;
  })
  .filter(Boolean);

export const builtinTheme = (id) => BUILTIN_THEMES.find((t) => t.id === id) ?? null;

// ── Renk yardımcıları (vurgu rengi özelleştirmesi) ──────────────────────────
function hexToRgb(hex) {
  let h = hex.replace('#', '');
  if (h.length === 3) h = [...h].map((c) => c + c).join('');
  const n = parseInt(h.slice(0, 6), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}
const toHex = (rgb) => '#' + rgb.map((v) => Math.round(Math.max(0, Math.min(255, v))).toString(16).padStart(2, '0')).join('');
export function mix(a, b, t) {
  const x = hexToRgb(a);
  const y = hexToRgb(b);
  return toHex(x.map((v, i) => v + (y[i] - v) * t));
}
/** WCAG göreli parlaklık (0–1). */
export function luminance(hex) {
  const [r, g, b] = hexToRgb(hex).map((v) => {
    const c = v / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}
export function contrast(a, b) {
  const [x, y] = [luminance(a), luminance(b)].sort((p, q) => q - p);
  return (x + 0.05) / (y + 0.05);
}
export const isHex = (v) => typeof v === 'string' && /^#[0-9a-fA-F]{6}$/.test(v);

/**
 * Temayı çalışma zamanı biçimine derler.
 * @param {object} theme doğrulanmış tema
 * @param {{variant?: string, accent?: string|null, density?: string}} opts kullanıcı özelleştirmeleri
 */
export function compileTheme(theme, { variant = '', accent = null, density = 'auto' } = {}) {
  const v = variant && theme.variants?.[variant] ? theme.variants[variant] : null;
  const base = v?.base ?? theme.base;
  const vars = { ...theme.tokens, ...(v?.tokens ?? {}) };
  for (const [k, f] of Object.entries(theme.fonts ?? {})) vars[`font-${k}`] = f;
  if (theme.fonts?.sans && !theme.fonts.display) vars['font-display'] = theme.fonts.sans;
  if (accent && isHex(accent) && theme.allow_accent !== false) {
    const surface = isHex(vars['color-surface']) ? vars['color-surface'] : base === 'dark' ? '#202020' : '#ffffff';
    const dark = base === 'dark';
    vars['color-primary'] = accent;
    vars['color-primary-deep'] = mix(accent, dark ? '#ffffff' : '#000000', 0.1);
    vars['color-primary-strong'] = mix(accent, dark ? '#ffffff' : '#000000', dark ? 0.3 : 0.28);
    vars['color-primary-soft'] = mix(accent, surface, dark ? 0.78 : 0.82);
    vars['color-text-on-primary'] = contrast(accent, '#ffffff') >= contrast(accent, '#1a1a1a') ? '#ffffff' : '#1a1a1a';
    if (theme.style.shape !== 'pixel') {
      vars['color-nav-active'] = vars['color-primary-soft'];
      vars['color-nav-active-text'] = vars['color-primary-strong'];
    }
  }
  const style = { ...theme.style };
  if (['compact', 'normal', 'roomy'].includes(density)) style.density = density;
  return {
    id: theme.id,
    name: theme.name,
    variant: v ? variant : '',
    base,
    style,
    vars,
    pets: theme.pets,
    home: theme.home,
    ambient: theme.ambient,
    sounds: theme.sounds,
    assets: theme.assets ?? {},
  };
}

/** Derlenmiş temanın CSS metni. Değerler doğrulamadan geçtiği için doğrudan yazılır. */
export function cssFor(compiled, selector = ':root') {
  const body = Object.entries(compiled.vars)
    .map(([k, val]) => `--${k}:${val};`)
    .join('');
  return `${selector}{color-scheme:${compiled.base};${body}}`;
}

/** Bir öğeye uygulanacak data-* öznitelikleri. */
export function attrsFor(compiled) {
  return {
    themeId: compiled.id,
    base: compiled.base,
    shape: compiled.style.shape,
    surface: compiled.style.surface,
    border: compiled.style.border,
    icons: compiled.style.icons,
    density: compiled.style.density,
    texture: compiled.style.texture,
    headings: compiled.style.headings,
    motionStyle: compiled.style.motion,
    rendering: compiled.style.rendering,
  };
}

let current = null;

// Son uygulama hatası (dış depo: React useSyncExternalStore ile okunur).
let applyError = null;
const errorListeners = new Set();
export function subscribeThemeError(fn) {
  errorListeners.add(fn);
  return () => errorListeners.delete(fn);
}
export const getThemeError = () => applyError;
export function setThemeError(msg) {
  if (applyError === msg) return;
  applyError = msg;
  for (const fn of errorListeners) fn();
}

/**
 * Temayı belge köküne tek adımda uygular: önce CSS metni ve öznitelikler hesaplanır, sonra
 * aynı görevde yazılır (tarayıcı tek bir stil yeniden hesaplaması yapar; yarım tema görünmez).
 * Hata olursa önceki tema yerinde kalır ve hata fırlatılır.
 */
export function applyTheme(compiled, doc = document) {
  const css = cssFor(compiled);
  const attrs = attrsFor(compiled);
  let el = doc.getElementById('ivily-theme');
  if (!el) {
    el = doc.createElement('style');
    el.id = 'ivily-theme';
    doc.head.appendChild(el);
  }
  const previous = { css: el.textContent, attrs: { ...doc.documentElement.dataset } };
  try {
    el.textContent = css;
    Object.assign(doc.documentElement.dataset, attrs);
    current = compiled;
  } catch (e) {
    el.textContent = previous.css;
    Object.assign(doc.documentElement.dataset, previous.attrs);
    throw e;
  }
  return compiled;
}

export const currentTheme = () => current;

/**
 * Tema kimliğinden derlenmiş temaya: yerleşik ya da (verilmişse) özel tema; bulunamaz veya
 * doğrulanamazsa yedek tema. Dönen nesnede `fallbackReason` doluysa yedeğe düşülmüştür.
 */
export function resolveTheme(id, { custom = null, variant, accent, density } = {}) {
  let theme = builtinTheme(id);
  let fallbackReason = null;
  if (!theme && custom) {
    const r = validateTheme(custom);
    if (r.ok) theme = r.theme;
    else fallbackReason = r.errors.join(' ');
  }
  if (!theme) {
    fallbackReason ??= `"${id}" teması bulunamadı.`;
    theme = builtinTheme(FALLBACK_THEME_ID);
  }
  const compiled = compileTheme(theme, { variant, accent, density });
  return { compiled, theme, fallbackReason };
}

// ── Tembel varlık yükleme ───────────────────────────────────────────────────
// Görsel varlıklar yalnızca ilk istendiklerinde çözülür; tema değişince eski önbellek bırakılır.
const assetCache = new Map();
export function themeAsset(compiled, key) {
  const src = compiled?.assets?.[key];
  if (!src) return null;
  const cacheKey = `${compiled.id}:${key}`;
  if (!assetCache.has(cacheKey)) {
    for (const k of assetCache.keys()) if (!k.startsWith(`${compiled.id}:`)) assetCache.delete(k);
    const img = new Image();
    img.decoding = 'async';
    img.src = src;
    assetCache.set(cacheKey, img);
  }
  return assetCache.get(cacheKey);
}

// ── Arayüz sesleri (isteğe bağlı, varsayılan kapalı) ─────────────────────────
let audio = null;
export function playThemeSound(compiled, name, enabled) {
  if (!enabled) return;
  const s = compiled?.sounds?.[name];
  if (!s) return;
  try {
    audio ??= new AudioContext();
    const osc = audio.createOscillator();
    const gain = audio.createGain();
    osc.type = s.wave;
    osc.frequency.value = s.freq;
    const t = audio.currentTime;
    gain.gain.setValueAtTime(s.gain, t);
    gain.gain.exponentialRampToValueAtTime(0.0001, t + s.ms / 1000);
    osc.connect(gain).connect(audio.destination);
    osc.start(t);
    osc.stop(t + s.ms / 1000 + 0.02);
  } catch {
    // Ses aygıtı yoksa sessiz kal.
  }
}
