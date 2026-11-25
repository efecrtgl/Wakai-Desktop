// Kedi Evi çizimi: tek <canvas>, tam sayı ölçek (keskin pikseller). Oda arka planı ve
// sprite'lar önbelleğe alınır; her karede yalnızca hareketli öğeler çizilir.
import { ITEMS, ROOM, ROOM_PX, SKY, TILE, footprint } from './catalog.js';
import { OUTLINE } from '../home/furniture.js';
import { CANVAS_H, CANVAS_W, petGrid } from '../sprite.js';
import { transformColor } from '../../theme/palette.js';

const cache = new Map();
function cached(key, w, h, draw) {
  let c = cache.get(key);
  if (!c) {
    if (cache.size > 400) cache.clear();
    c = document.createElement('canvas');
    c.width = w;
    c.height = h;
    draw(c.getContext('2d'));
    cache.set(key, c);
  }
  return c;
}
export const clearRenderCache = () => cache.clear();

function gridCanvas(key, grid, palette, line, sky) {
  return cached(key, grid[0].length, grid.length, (ctx) => {
    grid.forEach((row, y) =>
      row.forEach((c, x) => {
        if (!c) return;
        ctx.fillStyle = c === OUTLINE ? line : c === SKY ? sky : transformColor(c, palette);
        ctx.fillRect(x, y, 1, 1);
      }),
    );
  });
}

// ── Arka plan ───────────────────────────────────────────────────────────────
function drawFloor(ctx, kind, c) {
  const y0 = ROOM.wall * TILE;
  ctx.fillStyle = c.floor;
  ctx.fillRect(0, y0, ROOM_PX.w, ROOM_PX.h - y0);
  ctx.fillStyle = c['floor-alt'];
  for (let ty = ROOM.wall; ty < ROOM.h; ty++)
    for (let tx = 0; tx < ROOM.w; tx++) {
      const x = tx * TILE;
      const y = ty * TILE;
      if (kind === 'planks') {
        ctx.fillRect(x, y + TILE - 1, TILE, 1);
        if ((tx + ty * 3) % 4 === 0) ctx.fillRect(x, y, 1, TILE);
      } else if (kind === 'tiles') {
        ctx.fillRect(x, y, TILE, 1);
        ctx.fillRect(x, y, 1, TILE);
      } else if (kind === 'checker') {
        if ((tx + ty) % 2) ctx.fillRect(x, y, TILE, TILE);
      } else if (kind === 'carpet') {
        if ((tx * 7 + ty * 3) % 5 === 0) ctx.fillRect(x + 5, y + 7, 2, 1);
        if ((tx * 3 + ty * 5) % 7 === 0) ctx.fillRect(x + 11, y + 3, 1, 1);
      } else if (kind === 'grid') {
        ctx.globalAlpha = 0.5;
        ctx.fillRect(x, y, TILE, 1);
        ctx.fillRect(x, y, 1, TILE);
        ctx.globalAlpha = 1;
      }
    }
}

function drawWall(ctx, kind, c) {
  const h = ROOM.wall * TILE;
  ctx.fillStyle = c.wall;
  ctx.fillRect(0, 0, ROOM_PX.w, h);
  ctx.fillStyle = c['wall-alt'];
  if (kind === 'stripes') for (let x = 0; x < ROOM_PX.w; x += 12) ctx.fillRect(x, 0, 6, h);
  else if (kind === 'dots') {
    for (let y = 4; y < h; y += 10) for (let x = (y / 10) % 2 ? 9 : 4; x < ROOM_PX.w; x += 10) ctx.fillRect(x, y, 2, 2);
  } else if (kind === 'brick') {
    for (let y = 0; y < h; y += 8) {
      ctx.fillRect(0, y, ROOM_PX.w, 1);
      for (let x = (y / 8) % 2 ? 0 : 8; x < ROOM_PX.w; x += 16) ctx.fillRect(x, y, 1, 8);
    }
  } else if (kind === 'panel') ctx.fillRect(0, h - 22, ROOM_PX.w, 22);
  else if (kind === 'grid') {
    ctx.globalAlpha = 0.6;
    for (let x = 0; x < ROOM_PX.w; x += 16) ctx.fillRect(x, 0, 1, h);
    for (let y = 0; y < h; y += 16) ctx.fillRect(0, y, ROOM_PX.w, 1);
    ctx.globalAlpha = 1;
  }
  // Süpürgelik
  ctx.fillStyle = c.trim;
  ctx.fillRect(0, h - 3, ROOM_PX.w, 3);
  ctx.fillStyle = c.line;
  ctx.fillRect(0, h, ROOM_PX.w, 1);
}

/** Odalar arası kapılar: oturma odasının solu oyun odası, sağı yatak odası. */
export const DOORS = {
  living: [
    { side: 'left', to: 'playroom' },
    { side: 'right', to: 'bedroom' },
  ],
  bedroom: [{ side: 'left', to: 'living' }],
  playroom: [{ side: 'right', to: 'living' }],
};
export function doorRect(side) {
  const w = 14;
  const h = 34;
  const y = ROOM.wall * TILE - h + 1;
  return { x: side === 'left' ? 2 : ROOM_PX.w - w - 2, y, w, h };
}

function background(roomId, floor, wall, c) {
  return cached(`bg|${roomId}|${floor}|${wall}|${Object.values(c).join()}`, ROOM_PX.w, ROOM_PX.h, (ctx) => {
    drawWall(ctx, wall, c);
    drawFloor(ctx, floor, c);
    for (const d of DOORS[roomId] ?? []) {
      const r = doorRect(d.side);
      ctx.fillStyle = c.line;
      ctx.fillRect(r.x - 1, r.y - 1, r.w + 2, r.h + 1);
      ctx.fillStyle = c.trim;
      ctx.fillRect(r.x, r.y, r.w, r.h);
      ctx.fillStyle = c['floor-alt'];
      ctx.fillRect(r.x + 2, r.y + 2, r.w - 4, r.h - 2);
      ctx.fillStyle = c.line;
      ctx.fillRect(r.x + (d.side === 'left' ? r.w - 4 : 2), r.y + r.h / 2, 2, 2);
    }
  });
}

// ── Gökyüzü, ışık, hava ─────────────────────────────────────────────────────
export function skyColor(hour, weather) {
  let c;
  if (hour >= 6 && hour < 8) c = '#ffd6a8';
  else if (hour >= 8 && hour < 17) c = '#9fd0f0';
  else if (hour >= 17 && hour < 19.5) c = '#f7a77f';
  else if (hour >= 19.5 && hour < 21) c = '#5b5a9a';
  else c = '#1d2147';
  if (weather === 'rain' || weather === 'cloudy') c = mixHex(c, '#8a93a6', 0.45);
  if (weather === 'snow') c = mixHex(c, '#d8dde8', 0.4);
  return c;
}
function mixHex(a, b, t) {
  const p = (h) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16));
  const x = p(a);
  const y = p(b);
  return '#' + x.map((v, i) => Math.round(v + (y[i] - v) * t).toString(16).padStart(2, '0')).join('');
}

const EMOTES = {
  heart: ['.##.##.', '#######', '#######', '.#####.', '..###..', '...#...'],
  z: ['#####', '...#.', '..#..', '.#...', '#####'],
  note: ['..###', '..#.#', '..#..', '###..', '###..'],
  bowl: ['#.....#', '#######', '.#####.'],
  drop: ['..#..', '.###.', '#####', '.###.'],
  star: ['..#..', '#####', '.###.', '.#.#.'],
};
const EMOTE_COLORS = { heart: '#ef5f86', z: '#6f7cc8', note: '#4f9d69', bowl: '#c8734f', drop: '#5aa0d8', star: '#e2a03f' };

function drawEmote(ctx, kind, x, y) {
  const m = EMOTES[kind];
  if (!m) return;
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(x - 1, y - 1, m[0].length + 2, m.length + 2);
  ctx.fillStyle = EMOTE_COLORS[kind];
  m.forEach((row, ry) => [...row].forEach((ch, rx) => ch === '#' && ctx.fillRect(x + rx, y + ry, 1, 1)));
}

/**
 * Bir kareyi çizer.
 * @param {CanvasRenderingContext2D} ctx hedef (canvas boyutu = görünüm)
 * @param {object} s { room, style:{floor,wall}, colors, palette, items, cats(Map), pets(Map),
 *   toys(Map), hour, weather, period, view:{scale, ox, oy}, ghost, hover, particles, motion, ambient }
 */
export function drawWorld(ctx, s) {
  const { scale, ox, oy } = s.view;
  ctx.imageSmoothingEnabled = false;
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.fillStyle = s.colors.line;
  ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
  ctx.setTransform(scale, 0, 0, scale, -ox * scale, -oy * scale);
  ctx.drawImage(background(s.room, s.style.floor, s.style.wall, s.colors), 0, 0);
  const sky = skyColor(s.hour, s.weather);
  const night = s.period === 3;

  // Çizim listesi: duvar eşyaları önce; zemindekiler alt kenarlarına göre sıralanır.
  const list = [];
  for (const it of s.items) {
    const d = ITEMS[it.kind];
    if (!d?.grid) continue;
    const { w, h } = footprint(d, it.rot);
    const toy = s.toys.get(it.id);
    const bx = toy ? toy.x : (it.tx + w / 2) * TILE;
    const by = toy ? toy.y + 2 : (it.ty + h) * TILE;
    list.push({ kind: 'item', it, d, bx, by: d.wall ? -1 : d.walk && d.cat !== 'toy' ? 0 : by, drawY: by });
  }
  for (const cat of s.cats.values()) list.push({ kind: 'cat', cat, by: cat.carried ? 9999 : cat.y + (cat.action?.item && cat.action.phase === 'doing' ? 1 : 0) });
  list.sort((a, b) => a.by - b.by);

  for (const e of list) {
    if (e.kind === 'item') {
      const { it, d } = e;
      const g = d.grid;
      const key = `it|${it.kind}|${s.palette}|${s.colors.line}|${d.use === 'window' ? sky : ''}`;
      const img = gridCanvas(key, g, s.palette, s.colors.line, sky);
      const gw = g[0].length;
      const gh = g.length;
      ctx.save();
      if (it.rot % 2 === 1) {
        // Dönük eşya: kapladığı alanın ortasında 90° döndürülmüş çizilir.
        const f = footprint(d, it.rot);
        ctx.translate(Math.round((it.tx + f.w / 2) * TILE), Math.round((it.ty + f.h / 2) * TILE));
        ctx.rotate((it.rot === 1 ? 1 : -1) * (Math.PI / 2));
        ctx.drawImage(img, -Math.floor(gw / 2), -Math.floor(gh / 2));
      } else {
        ctx.translate(Math.round(e.bx), Math.round(e.drawY));
        if (it.rot === 2) ctx.scale(-1, 1);
        ctx.drawImage(img, -Math.floor(gw / 2), -gh);
      }
      ctx.restore();
      // Kap doluluğu
      if (d.use === 'food' || d.use === 'water') {
        const n = it.state?.portions ?? it.state?.water ?? 0;
        if (n > 0) {
          ctx.fillStyle = d.use === 'food' ? '#c8734f' : '#5aa0d8';
          for (let i = 0; i < Math.min(n, 4); i++) ctx.fillRect(Math.round(e.bx) - 4 + i * 2, Math.round(e.drawY) - 5, 1, 1);
        }
      }
      if (d.use === 'window') drawWindowWeather(ctx, it, s, night);
      if (s.hover?.id === it.id) {
        const { w, h } = footprint(d, it.rot);
        ctx.strokeStyle = s.colors.line;
        ctx.lineWidth = 1 / scale;
        ctx.setLineDash([2 / scale, 2 / scale]);
        ctx.strokeRect(it.tx * TILE + 0.5, (it.ty + h) * TILE - Math.max(gh, h * TILE) + 0.5, w * TILE - 1, Math.max(gh, h * TILE) - 1);
        ctx.setLineDash([]);
      }
    } else drawCat(ctx, e.cat, s);
  }

  // Işık: akşam sıcak, gece karanlık; lambalar gece yanar.
  if (s.period >= 2) {
    ctx.fillStyle = night ? 'rgba(16, 18, 52, 0.42)' : 'rgba(255, 140, 60, 0.10)';
    ctx.fillRect(0, 0, ROOM_PX.w, ROOM_PX.h);
    if (night) {
      ctx.globalCompositeOperation = 'lighter';
      for (const it of s.items) {
        if (it.kind !== 'lamp' && it.kind !== 'lantern') continue;
        const x = (it.tx + 0.5) * TILE;
        const y = (it.ty + 1) * TILE - (it.kind === 'lamp' ? 26 : 12);
        const g = ctx.createRadialGradient(x, y, 2, x, y, 46);
        g.addColorStop(0, 'rgba(255, 210, 140, 0.45)');
        g.addColorStop(1, 'rgba(255, 210, 140, 0)');
        ctx.fillStyle = g;
        ctx.fillRect(x - 46, y - 46, 92, 92);
      }
      ctx.globalCompositeOperation = 'source-over';
    }
  }

  // Ortam parçacıkları (temaya göre; hareket azaltıldıysa yok)
  if (s.particles?.length) {
    for (const p of s.particles) {
      ctx.globalAlpha = p.a;
      ctx.fillStyle = p.c;
      ctx.fillRect(Math.round(p.x), Math.round(p.y), p.s, p.s);
    }
    ctx.globalAlpha = 1;
  }

  // Duygu baloncukları
  for (const cat of s.cats.values()) if (cat.emote) drawEmote(ctx, cat.emote.kind, Math.round(cat.x) - 3, Math.round(cat.y - cat.z) - CANVAS_H - 6);

  // Yerleştirme önizlemesi
  if (s.ghost) {
    const { def, tx, ty, rot, ok } = s.ghost;
    const { w, h } = footprint(def, rot);
    ctx.fillStyle = ok ? 'rgba(80, 200, 120, 0.35)' : 'rgba(230, 70, 70, 0.4)';
    ctx.fillRect(tx * TILE, ty * TILE, w * TILE, h * TILE);
    if (def.grid) {
      ctx.globalAlpha = 0.75;
      const img = gridCanvas(`it|${def.kind}|${s.palette}|${s.colors.line}|${def.use === 'window' ? sky : ''}`, def.grid, s.palette, s.colors.line, sky);
      ctx.drawImage(img, Math.round((tx + w / 2) * TILE - def.grid[0].length / 2), (ty + h) * TILE - def.grid.length);
      ctx.globalAlpha = 1;
    }
  }
  ctx.setTransform(1, 0, 0, 1, 0, 0);
}

function drawWindowWeather(ctx, it, s, night) {
  const x0 = it.tx * TILE + 2;
  const y0 = it.ty * TILE + 2 + (32 - 30);
  const t = s.t ?? 0;
  if (night && s.weather !== 'rain') {
    ctx.fillStyle = '#fff8d6';
    for (let i = 0; i < 6; i++) {
      if (s.motion === 'full' && Math.sin(t * 2 + i * 1.7) < -0.6) continue;
      ctx.fillRect(x0 + ((i * 7) % 24) + 1, y0 + ((i * 5) % 20) + 1, 1, 1);
    }
  }
  if (s.weather === 'rain') {
    ctx.fillStyle = 'rgba(220, 235, 255, 0.8)';
    for (let i = 0; i < 8; i++) {
      const yy = (i * 9 + (s.motion === 'off' ? 0 : t * 40)) % 22;
      ctx.fillRect(x0 + ((i * 5) % 26), y0 + yy, 1, 3);
    }
  } else if (s.weather === 'snow') {
    ctx.fillStyle = '#ffffff';
    for (let i = 0; i < 8; i++) {
      const yy = (i * 7 + (s.motion === 'off' ? 0 : t * 8)) % 23;
      ctx.fillRect(x0 + ((i * 5 + Math.round(Math.sin(t + i) * 1.5)) % 26), y0 + yy, 1, 1);
    }
  }
}

function drawCat(ctx, cat, s) {
  const pet = s.pets.get(cat.id);
  if (!pet) return;
  const acc = [pet.acc_head, pet.acc_neck].filter(Boolean);
  const anim = cat.anim ?? 'sit';
  const key = `cat|${pet.breed}|${anim}|${cat.frame % 8}|${acc.join(',')}|${s.palette}`;
  const img = cached(key, CANVAS_W, CANVAS_H, (c) => {
    const g = petGrid({ anim, frame: cat.frame, breed: pet.breed, accessories: acc });
    g.forEach((row, y) =>
      row.forEach((col, x) => {
        if (!col) return;
        c.fillStyle = transformColor(col, s.palette);
        c.fillRect(x, y, 1, 1);
      }),
    );
  });
  const x = Math.round(cat.x - CANVAS_W / 2);
  const y = Math.round(cat.y - cat.z - CANVAS_H + 1);
  // Gölge
  if (!cat.carried) {
    ctx.fillStyle = 'rgba(0, 0, 0, 0.14)';
    ctx.fillRect(Math.round(cat.x) - 7, Math.round(cat.y), 14, 2);
  }
  ctx.save();
  if (!cat.face) {
    ctx.translate(x + CANVAS_W, y);
    ctx.scale(-1, 1);
    ctx.drawImage(img, 0, 0);
  } else ctx.drawImage(img, x, y);
  ctx.restore();
  if (s.hoverCat === cat.id) {
    ctx.fillStyle = s.colors.line;
    ctx.fillRect(Math.round(cat.x) - 1, y - 3, 3, 1);
  }
}

/** Bir dünya noktasındaki kedi (öndeki önce). */
export function catAt(cats, px, py) {
  return (
    [...cats.values()]
      .sort((a, b) => b.y - a.y)
      .find((c) => Math.abs(px - c.x) <= CANVAS_W / 2 - 2 && py <= c.y + 2 - c.z && py >= c.y - c.z - CANVAS_H + 6) ?? null
  );
}
