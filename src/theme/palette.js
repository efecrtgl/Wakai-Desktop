// Kedi ve Kedi Evi eşyası pikselleri için tema paleti dönüşümleri. Masaüstü kedileri aynı
// dönüşümleri Rust tarafında uygular (src-tauri/src/pets/art.rs, `palette_transform`).
const cache = new Map();

function rgb(hex) {
  const n = parseInt(hex.slice(1, 7), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}
const hex = (c) => '#' + c.map((v) => Math.round(Math.max(0, Math.min(255, v))).toString(16).padStart(2, '0')).join('');
const mixc = (a, b, t) => a.map((v, i) => v + (b[i] - v) * t);

/** Tek bir rengi dönüştürür. Tanınmayan palet ya da biçim → aynen. */
export function transformColor(color, palette) {
  if (!palette || palette === 'none' || typeof color !== 'string' || !/^#[0-9a-fA-F]{6}$/.test(color)) return color;
  const key = palette + color;
  let out = cache.get(key);
  if (out) return out;
  const c = rgb(color);
  const l = 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
  switch (palette) {
    case 'grayscale':
      out = hex([l, l, l]);
      break;
    case 'warm':
      out = hex(mixc(mixc(c, [l, l, l], 0.15), [240, 160, 96], 0.14));
      break;
    case 'cool':
      out = hex(mixc(c, [111, 160, 255], 0.14));
      break;
    case 'neon': {
      const s = c.map((v) => l + (v - l) * 1.8);
      out = hex(mixc(s, l > 150 ? [230, 140, 255] : [40, 210, 255], 0.28));
      break;
    }
    case 'pastel':
      out = hex(mixc(c, [255, 255, 255], 0.35));
      break;
    default:
      out = color;
  }
  cache.set(key, out);
  return out;
}
