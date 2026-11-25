// Dünya motoru: karo ızgarası, yerleşim kuralları (Rust world.rs ile aynı), çarpışma ızgarası,
// A* yol bulma ve eşyaların etkileşim noktaları. Saf mantık; test edilebilir.
import { ITEMS, ROOM, TILE, footprint } from './catalog.js';

/** Eşya oda sınırında ve doğru katmanda mı? */
export function fits(def, tx, ty, rot = 0) {
  if (!def || rot < 0 || rot > 3 || (def.wall && rot !== 0)) return false;
  const { w, h } = footprint(def, rot);
  const [y0, y1] = def.wall ? [0, ROOM.wall] : [ROOM.wall, ROOM.h];
  return tx >= 0 && tx + w <= ROOM.w && ty >= y0 && ty + h <= y1;
}

/** Engelleyici eşyalar üst üste binemez (halı ve oyuncaklar engel değildir). */
export function overlaps(items, def, tx, ty, rot = 0, ignoreId = null) {
  if (def.walk) return false;
  const { w, h } = footprint(def, rot);
  return items.some((it) => {
    if (it.id === ignoreId) return false;
    const o = ITEMS[it.kind];
    if (!o || o.walk) return false;
    const f = footprint(o, it.rot);
    return tx < it.tx + f.w && it.tx < tx + w && ty < it.ty + f.h && it.ty < ty + h;
  });
}

export const canPlace = (items, def, tx, ty, rot, ignoreId) => fits(def, tx, ty, rot) && !overlaps(items, def, tx, ty, rot, ignoreId);

/** Yürünemez karolar (1). Duvar satırları ve engelleyici eşyalar kapalıdır. */
export function blockedGrid(items) {
  const g = new Uint8Array(ROOM.w * ROOM.h);
  for (let y = 0; y < ROOM.wall; y++) for (let x = 0; x < ROOM.w; x++) g[y * ROOM.w + x] = 1;
  for (const it of items) {
    const d = ITEMS[it.kind];
    if (!d || d.walk || d.wall) continue;
    const { w, h } = footprint(d, it.rot);
    for (let y = it.ty; y < it.ty + h; y++) for (let x = it.tx; x < it.tx + w; x++) if (y < ROOM.h && x < ROOM.w) g[y * ROOM.w + x] = 1;
  }
  return g;
}

export const tileOf = (px, py) => ({ tx: Math.max(0, Math.min(ROOM.w - 1, Math.floor(px / TILE))), ty: Math.max(0, Math.min(ROOM.h - 1, Math.floor(py / TILE))) });
export const centerOf = (tx, ty) => ({ x: tx * TILE + TILE / 2, y: ty * TILE + TILE - 2 });
const free = (g, x, y) => x >= 0 && y >= ROOM.wall && x < ROOM.w && y < ROOM.h && !g[y * ROOM.w + x];

/** En yakın yürünebilir karo (kedi engelin içinde kaldıysa kurtarma). */
export function nearestFree(g, tx, ty) {
  if (free(g, tx, ty)) return { tx, ty };
  for (let r = 1; r < ROOM.w; r++)
    for (let dy = -r; dy <= r; dy++)
      for (let dx = -r; dx <= r; dx++) {
        if (Math.max(Math.abs(dx), Math.abs(dy)) !== r) continue;
        if (free(g, tx + dx, ty + dy)) return { tx: tx + dx, ty: ty + dy };
      }
  return { tx: Math.floor(ROOM.w / 2), ty: ROOM.h - 1 };
}

/**
 * A* yol bulma (8 yön, köşe kesmeden). Hedef kapalıysa en yakın açık karoya gidilir.
 * Yol yoksa null (yapay zekâ başka bir hedef seçer — kedi takılı kalmaz).
 * @returns {{tx:number, ty:number}[] | null} başlangıç hariç karo listesi
 */
export function findPath(g, from, to, maxNodes = 600) {
  const start = nearestFree(g, from.tx, from.ty);
  const goal = nearestFree(g, to.tx, to.ty);
  const W = ROOM.w;
  const key = (x, y) => y * W + x;
  const sk = key(start.tx, start.ty);
  const gk = key(goal.tx, goal.ty);
  if (sk === gk) return [];
  const gScore = new Map([[sk, 0]]);
  const came = new Map();
  const h = (x, y) => Math.max(Math.abs(x - goal.tx), Math.abs(y - goal.ty));
  const open = [{ k: sk, x: start.tx, y: start.ty, f: h(start.tx, start.ty) }];
  const closed = new Set();
  let expanded = 0;
  while (open.length && expanded++ < maxNodes) {
    let bi = 0;
    for (let i = 1; i < open.length; i++) if (open[i].f < open[bi].f) bi = i;
    const cur = open.splice(bi, 1)[0];
    if (cur.k === gk) {
      const path = [];
      let k = gk;
      while (k !== sk) {
        path.push({ tx: k % W, ty: Math.floor(k / W) });
        k = came.get(k);
      }
      return path.reverse();
    }
    closed.add(cur.k);
    for (let dy = -1; dy <= 1; dy++)
      for (let dx = -1; dx <= 1; dx++) {
        if (!dx && !dy) continue;
        const nx = cur.x + dx;
        const ny = cur.y + dy;
        if (!free(g, nx, ny)) continue;
        if (dx && dy && (!free(g, cur.x + dx, cur.y) || !free(g, cur.x, cur.y + dy))) continue;
        const nk = key(nx, ny);
        if (closed.has(nk)) continue;
        const cost = (gScore.get(cur.k) ?? 0) + (dx && dy ? 1.414 : 1);
        if (cost < (gScore.get(nk) ?? Infinity)) {
          gScore.set(nk, cost);
          came.set(nk, cur.k);
          open.push({ k: nk, x: nx, y: ny, f: cost + h(nx, ny) });
        }
      }
  }
  return null;
}

/**
 * Eşyanın etkileşim noktası (dünya pikseli): kedinin duracağı yer ve oradaki duruş.
 * Yatak/minder/kutu: üstünde; kedi ağacı: tepesinde; kap: yanında; pencere: önündeki zemin.
 */
export function interactionPoint(it) {
  const d = ITEMS[it.kind];
  const { w, h } = footprint(d, it.rot);
  const cx = (it.tx + w / 2) * TILE;
  const bottom = (it.ty + h) * TILE;
  switch (d.use) {
    case 'bed':
    case 'hide':
      return { x: cx, y: bottom - 3, on: true };
    case 'climb':
      return { x: cx, y: it.ty * TILE + 6, on: true, high: true };
    case 'perch':
      return { x: cx, y: bottom - 8, on: true };
    case 'window':
      return { x: cx, y: (ROOM.wall + 1) * TILE - 2, on: false };
    case 'food':
    case 'water':
    case 'scratch':
      return { x: it.tx * TILE - 6, y: bottom - 2, on: false, face: true };
    default:
      return { x: cx, y: bottom - 2, on: false };
  }
}

/** Bir dünya noktasındaki en üstteki eşya (çizim sırasına göre ters: önce öndekiler). */
export function itemAt(items, px, py, filter = () => true) {
  const sorted = [...items].sort((a, b) => b.ty + footprint(ITEMS[b.kind], b.rot).h - (a.ty + footprint(ITEMS[a.kind], a.rot).h));
  return (
    sorted.find((it) => {
      const d = ITEMS[it.kind];
      if (!d || !filter(it, d)) return false;
      const { w, h } = footprint(d, it.rot);
      const gh = d.grid ? d.grid.length : h * TILE;
      const x0 = it.tx * TILE;
      const y1 = (it.ty + h) * TILE;
      return px >= x0 && px < x0 + w * TILE && py <= y1 && py >= y1 - Math.max(gh, h * TILE);
    }) ?? null
  );
}
