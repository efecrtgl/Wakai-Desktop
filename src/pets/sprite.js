// Masaüstü motoruyla (src-tauri/src/pets/art.rs) aynı sprite verisinden kedi pikselleri üretir.
// Tek kaynak: src-tauri/assets/pets/sprites.json (scripts/gen-pet-sprites.mjs üretir).
import SPRITES from '../../src-tauri/assets/pets/sprites.json';
import { transformColor } from '../theme/palette.js';

export const PAD_TOP = 5;
export const PAD_SIDE = 2;
export const CANVAS_W = SPRITES.width + PAD_SIDE * 2;
export const CANVAS_H = SPRITES.height + PAD_TOP;
export { SPRITES };

/**
 * Tuval ızgarası: her hücre '#rrggbb' ya da null.
 * @param {{anim?: string, frame?: number, breed?: string, accessories?: string[]}} o
 */
export function petGrid({ anim = 'sit', frame = 0, breed = 'pink', accessories = [] } = {}) {
  const frames = SPRITES.animations[anim] ?? SPRITES.animations.idle;
  const f = frames[frame % frames.length];
  const pal = SPRITES.breeds[breed] ?? SPRITES.breeds.pink;
  const g = Array.from({ length: CANVAS_H }, () => Array(CANVAS_W).fill(null));
  const put = (x, y, c) => {
    const gx = x + PAD_SIDE;
    const gy = y + PAD_TOP;
    if (c && gx >= 0 && gy >= 0 && gx < CANVAS_W && gy < CANVAS_H) g[gy][gx] = c;
  };
  f.rows.forEach((row, y) => [...row].forEach((ch, x) => ch !== '.' && put(x, y, pal[ch])));
  for (const id of accessories) {
    const a = SPRITES.accessories[id];
    if (!a) continue;
    const [ax, ay] = a.anchor === 'neck' ? f.anchors.neck : f.anchors.head;
    a.rows.forEach((row, y) => [...row].forEach((ch, x) => put(ax + a.dx + x, ay + a.dy + y, a.palette[ch])));
  }
  return g;
}

/** Izgarayı 2B tuvale çizer (en yakın komşu, isteğe bağlı aynalama ve tema paleti). */
export function drawGrid(ctx, grid, { x = 0, y = 0, scale = 3, flip = false, palette = 'none' } = {}) {
  grid.forEach((row, gy) =>
    row.forEach((c, gx) => {
      if (!c) return;
      ctx.fillStyle = transformColor(c, palette);
      const px = flip ? CANVAS_W - 1 - gx : gx;
      ctx.fillRect(x + px * scale, y + gy * scale, scale, scale);
    }),
  );
}
