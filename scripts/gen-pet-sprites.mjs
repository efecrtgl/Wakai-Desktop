// Masaüstü kedilerinin özgün piksel sprite'larını üretir.
//   node scripts/gen-pet-sprites.mjs            → src-tauri/assets/pets/sprites.json
//   node scripts/gen-pet-sprites.mjs preview.png → ayrıca tüm kareleri PNG olarak önizler
//
// Kareler parçalardan (gövde, baş, kulaklar, bacaklar, kuyruk, göz) parametrik olarak kurulur;
// kontur, dolu piksellere komşu saydam piksellerden OTOMATİK çıkarılır. Böylece tüm pozlar ve
// cinsler aynı çizim diliyle tutarlı kalır. Kediler sağa bakar; sola bakış çizimde aynalanır.
//
// Karakterler:  . saydam  o kontur  p kürk  s ikincil kürk (çizgi/leke)  w açık bölge (karın)
//               k göz  n burun/kulak içi  h göz parıltısı
import { writeFileSync, mkdirSync } from 'node:fs';
import { deflateSync } from 'node:zlib';

const W = 24;
const H = 18;

function canvas() {
  return Array.from({ length: H }, () => Array(W).fill('.'));
}
const set = (c, x, y, ch) => {
  if (x >= 0 && y >= 0 && x < W && y < H) c[y][x] = ch;
};
const rect = (c, x0, y0, x1, y1, ch) => {
  for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) set(c, x, y, ch);
};
const get = (c, x, y) => (x >= 0 && y >= 0 && x < W && y < H ? c[y][x] : '.');

/** Dolu piksellere 4-komşu saydam pikselleri kontur yapar. */
function outline(c) {
  const out = c.map((r) => r.slice());
  for (let y = 0; y < H; y++)
    for (let x = 0; x < W; x++) {
      if (c[y][x] !== '.') continue;
      const n = [get(c, x - 1, y), get(c, x + 1, y), get(c, x, y - 1), get(c, x, y + 1)];
      if (n.some((v) => v !== '.' && v !== 'o')) out[y][x] = 'o';
    }
  return out;
}

/**
 * Bir kare çizer. Parametreler (hepsi isteğe bağlı):
 *  by: gövde dikey kayması, hy: baş dikey kayması, hx: baş yatay kayması
 *  legs: 4 bacağın [x, uzunluk] dizisi (arka→ön), tail: kuyruk noktaları
 *  eyes: 'open' | 'closed' | 'happy' | 'wide', body: 'stand' | 'sit' | 'loaf' | 'stretch' | 'hang'
 */
function frame(o = {}) {
  const c = canvas();
  const by = o.by ?? 0;
  const hy = o.hy ?? 0;
  const hx = o.hx ?? 0;
  const body = o.body ?? 'stand';

  // Kuyruk (gövdenin arkasında çizilir)
  for (const [x, y] of o.tail ?? [[5, 10], [4, 9], [3, 8], [3, 7], [3, 6], [4, 5]]) {
    set(c, x, y + by, 'p');
    set(c, x + 1, y + by, 'p');
  }

  // Gövde
  if (body === 'stand' || body === 'stretch') {
    const x0 = body === 'stretch' ? 5 : 6;
    const x1 = body === 'stretch' ? 17 : 15;
    rect(c, x0, 9 + by, x1, 12 + by, 'p');
    set(c, x0, 9 + by, '.');
    rect(c, x0 + 2, 12 + by, x1 - 2, 12 + by, 'w');
    for (const sx of [x0 + 2, x0 + 5, x0 + 8]) set(c, sx, 9 + by, 's'), set(c, sx, 10 + by, 's');
  } else if (body === 'sit') {
    rect(c, 9, 7 + by, 15, 13 + by, 'p');
    rect(c, 8, 10 + by, 9, 13 + by, 'p');
    set(c, 9, 7 + by, '.');
    rect(c, 12, 9 + by, 15, 13 + by, 'w');
    set(c, 10, 8 + by, 's'), set(c, 10, 9 + by, 's'), set(c, 9, 11 + by, 's');
  } else if (body === 'loaf') {
    rect(c, 5, 11, 17, 14, 'p');
    set(c, 5, 11, '.'), set(c, 17, 11, '.');
    rect(c, 7, 14, 15, 14, 'w');
    for (const sx of [7, 10, 13]) set(c, sx, 11, 's'), set(c, sx, 12, 's');
  } else if (body === 'hang') {
    rect(c, 10, 6, 14, 14, 'p');
    rect(c, 11, 8, 13, 13, 'w');
    set(c, 10, 9, 's'), set(c, 14, 9, 's'), set(c, 10, 11, 's'), set(c, 14, 11, 's');
  }

  // Bacaklar
  for (const [lx, len] of o.legs ?? [[7, 2], [9, 2], [12, 2], [14, 2]]) {
    rect(c, lx, 13 + by, lx, 12 + by + len, 'p');
    set(c, lx, 12 + by + len, 'w');
  }

  // Baş (6×6 blok + kulaklar)
  const hx0 = (body === 'hang' ? 9 : body === 'sit' ? 13 : body === 'loaf' ? 14 : 14) + hx;
  const hy0 = (body === 'hang' ? 1 : body === 'loaf' ? 8 : 4) + hy + (body === 'stand' || body === 'stretch' ? by : 0) + (body === 'sit' ? by - 2 : 0);
  rect(c, hx0, hy0, hx0 + 6, hy0 + 5, 'p');
  set(c, hx0, hy0, '.'), set(c, hx0 + 6, hy0, '.');
  // kulaklar
  set(c, hx0 + 1, hy0 - 1, 'p'), set(c, hx0 + 5, hy0 - 1, 'p');
  set(c, hx0 + 1, hy0, 'n'), set(c, hx0 + 5, hy0, 'n');
  // alın çizgisi
  set(c, hx0 + 3, hy0, 's'), set(c, hx0 + 3, hy0 + 1, 's');
  // ağız bölgesi
  rect(c, hx0 + 4, hy0 + 3, hx0 + 6, hy0 + 5, 'w');
  set(c, hx0 + 6, hy0 + 3, 'n');
  // gözler
  const ex = hx0 + 4;
  const ey = hy0 + 2;
  const eyes = o.eyes ?? 'open';
  if (eyes === 'open') set(c, ex, ey, 'k'), set(c, ex, ey + 1, 'k');
  else if (eyes === 'wide') set(c, ex, ey, 'k'), set(c, ex, ey + 1, 'k'), set(c, ex - 1, ey, 'h');
  else if (eyes === 'happy') set(c, ex - 1, ey + 1, 'k'), set(c, ex, ey, 'k'), set(c, ex + 1, ey + 1, 'k');
  else set(c, ex - 1, ey + 1, 'k'), set(c, ex, ey + 1, 'k');

  // Pati kaldırma (oyun)
  if (o.paw) for (const [x, y] of o.paw) set(c, x, y, 'p');

  const done = outline(c);
  return {
    rows: done.map((r) => r.join('')),
    // aksesuar bağlantı noktaları: başın üstü ortası ve boyun
    anchors: { head: [hx0 + 3, hy0 - 1], neck: [hx0 + 1, hy0 + 5] },
  };
}

const TAIL_UP = [[5, 10], [4, 9], [3, 8], [3, 7], [3, 6], [4, 5]];
const TAIL_SWAY = [[5, 10], [4, 9], [3, 8], [2, 7], [2, 6], [3, 5]];
const TAIL_BACK = [[5, 10], [4, 10], [3, 9], [2, 9], [1, 8]];
const TAIL_SIT = [[8, 13], [7, 13], [6, 13], [5, 12], [5, 11]];
const TAIL_LOAF = [[5, 14], [4, 14], [3, 13], [3, 12]];

const ANIMATIONS = {
  idle: [frame({ tail: TAIL_UP }), frame({ tail: TAIL_SWAY })],
  blink: [frame({ tail: TAIL_UP, eyes: 'closed' })],
  walk: [
    frame({ legs: [[6, 2], [9, 2], [12, 2], [15, 2]], tail: TAIL_UP }),
    frame({ legs: [[7, 2], [9, 1], [13, 2], [14, 1]], tail: TAIL_UP, by: -1 }),
    frame({ legs: [[8, 2], [9, 2], [13, 2], [14, 2]], tail: TAIL_SWAY }),
    frame({ legs: [[7, 1], [10, 2], [12, 1], [15, 2]], tail: TAIL_SWAY, by: -1 }),
  ],
  run: [
    frame({ body: 'stretch', legs: [[4, 2], [6, 1], [16, 1], [18, 2]], tail: TAIL_BACK }),
    frame({ body: 'stretch', legs: [[8, 2], [9, 2], [13, 2], [14, 2]], tail: TAIL_BACK, by: -1 }),
  ],
  sit: [frame({ body: 'sit', legs: [[12, 2], [14, 2]], tail: TAIL_SIT }), frame({ body: 'sit', legs: [[12, 2], [14, 2]], tail: TAIL_SIT, eyes: 'closed' })],
  look: [frame({ body: 'sit', legs: [[12, 2], [14, 2]], tail: TAIL_SIT, eyes: 'wide', hy: -1 }), frame({ body: 'sit', legs: [[12, 2], [14, 2]], tail: TAIL_SIT, eyes: 'wide', hx: -1 })],
  sleep: [frame({ body: 'loaf', legs: [], tail: TAIL_LOAF, eyes: 'closed' }), frame({ body: 'loaf', legs: [], tail: TAIL_LOAF, eyes: 'closed', hy: 1 })],
  wake: [frame({ body: 'loaf', legs: [], tail: TAIL_LOAF, eyes: 'open', hy: -1 })],
  jump: [frame({ body: 'stretch', legs: [[5, 1], [6, 1], [17, 1], [18, 1]], tail: TAIL_BACK, by: -2, hy: -1 })],
  fall: [frame({ legs: [[5, 3], [8, 3], [13, 3], [16, 3]], tail: TAIL_UP, eyes: 'wide' })],
  carried: [frame({ body: 'hang', legs: [[10, 2], [14, 2]], tail: [[12, 15], [12, 16], [13, 16]], eyes: 'wide' })],
  petted: [frame({ body: 'sit', legs: [[12, 2], [14, 2]], tail: TAIL_SIT, eyes: 'happy' }), frame({ body: 'sit', legs: [[12, 2], [14, 2]], tail: TAIL_SIT, eyes: 'happy', hy: 1 })],
  play: [
    frame({ body: 'sit', legs: [[12, 2]], tail: TAIL_SIT, eyes: 'wide', paw: [[19, 10], [20, 9]] }),
    frame({ body: 'sit', legs: [[12, 2]], tail: TAIL_SIT, eyes: 'wide', paw: [[19, 12], [20, 12]] }),
  ],
  eat: [frame({ legs: [[7, 2], [9, 2], [12, 2], [14, 2]], tail: TAIL_UP, hy: 3, hx: 1, eyes: 'closed' }), frame({ legs: [[7, 2], [9, 2], [12, 2], [14, 2]], tail: TAIL_SWAY, hy: 4, hx: 1, eyes: 'closed' })],
};

// Cinsler: aynı sprite'lar, farklı palet. s = çizgiler/lekeler, w = açık bölgeler.
const BREEDS = {
  pink: { name: 'Pembiş', o: '#5b4a4f', p: '#f7b8c8', s: '#f49ab2', w: '#fff1f4', k: '#3d3034', n: '#e88aa3', h: '#ffffff' },
  white: { name: 'Pamuk', o: '#5b4f52', p: '#f6f3ef', s: '#ece6df', w: '#ffffff', k: '#5c8fd6', n: '#f2a7b6', h: '#ffffff' },
  black: { name: 'Gece', o: '#0d0a0b', p: '#2b2628', s: '#24201f', w: '#2b2628', k: '#f3d27a', n: '#5a4a50', h: '#ffffff' },
  orange: { name: 'Tarçın', o: '#5a3b25', p: '#f4a259', s: '#d9773a', w: '#ffe6cc', k: '#3d2a1c', n: '#e98a7c', h: '#ffffff' },
  gray: { name: 'Duman', o: '#3f3d44', p: '#a3a1aa', s: '#8a8892', w: '#dddce1', k: '#6fae6a', n: '#d79aa5', h: '#ffffff' },
  tabby: { name: 'Benekli', o: '#3e2c1d', p: '#b48d62', s: '#6b4f33', w: '#ecdcc4', k: '#4f8f3a', n: '#d9927f', h: '#ffffff' },
  tuxedo: { name: 'Frak', o: '#0d0a0b', p: '#2b2628', s: '#2b2628', w: '#fbf8f4', k: '#8fd18f', n: '#f2a7b6', h: '#ffffff' },
  cream: { name: 'Krem', o: '#6a5640', p: '#f3dfc0', s: '#e3c69a', w: '#fff7ea', k: '#7a5a3a', n: '#efb1a1', h: '#ffffff' },
};

// Aksesuarlar: baş/boyun bağlantı noktasına göre çizilen küçük ızgaralar.
// Her satır ortalanır; '.' saydam, diğer karakterler aksesuarın kendi paletinden.
const ACCESSORIES = {
  bow: { name: 'Fiyonk', anchor: 'head', dx: -1, dy: -1, rows: ['r.r', 'rRr', 'r.r'], palette: { r: '#f07aa0', R: '#c94f78' } },
  party_hat: { name: 'Parti şapkası', anchor: 'head', dx: -1, dy: -3, rows: ['.y.', 'yby', 'byb'], palette: { y: '#f3d27a', b: '#8fc3ee' } },
  flower: { name: 'Çiçek', anchor: 'head', dx: 1, dy: -1, rows: ['.f.', 'fcf', '.f.'], palette: { f: '#f7f1f5', c: '#f3d27a' } },
  crown: { name: 'Taç', anchor: 'head', dx: -2, dy: -2, rows: ['g.g.g', 'ggggg'], palette: { g: '#f2c14e' } },
  collar: { name: 'Tasma', anchor: 'neck', dx: 0, dy: 0, rows: ['rrrr', '.b..'], palette: { r: '#d9607c', b: '#f2c14e' } },
  scarf: { name: 'Atkı', anchor: 'neck', dx: -1, dy: 0, rows: ['lllll', '.l...'], palette: { l: '#b4a2e3' } },
};

const data = { width: W, height: H, animations: {}, breeds: BREEDS, accessories: ACCESSORIES };
for (const [name, frames] of Object.entries(ANIMATIONS)) data.animations[name] = frames;

mkdirSync('src-tauri/assets/pets', { recursive: true });
writeFileSync('src-tauri/assets/pets/sprites.json', JSON.stringify(data));
console.log(`sprites.json: ${Object.keys(ANIMATIONS).length} animasyon, ${Object.values(ANIMATIONS).flat().length} kare, ${Object.keys(BREEDS).length} cins`);

// ── İsteğe bağlı PNG önizleme ─────────────────────────────────────────────
const out = process.argv[2];
if (out) {
  const S = 4;
  const frames = Object.entries(ANIMATIONS).flatMap(([n, fs]) => fs.map((f) => ({ n, f })));
  const breeds = Object.values(BREEDS);
  const cols = frames.length;
  const IW = cols * (W + 2) * S;
  const IH = breeds.length * (H + 2) * S;
  const px = Buffer.alloc(IW * IH * 3, 255);
  const hex = (h) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16));
  breeds.forEach((pal, bi) =>
    frames.forEach(({ f }, fi) =>
      f.rows.forEach((row, y) =>
        [...row].forEach((ch, x) => {
          if (!pal[ch]) return;
          const rgb = hex(pal[ch]);
          for (let dy = 0; dy < S; dy++)
            for (let dx = 0; dx < S; dx++) {
              const X = (fi * (W + 2) + x) * S + dx;
              const Y = (bi * (H + 2) + y) * S + dy;
              px.set(rgb, (Y * IW + X) * 3);
            }
        }),
      ),
    ),
  );
  const crc = (b) => {
    let c = ~0;
    for (const v of b) {
      c ^= v;
      for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
    }
    return ~c >>> 0;
  };
  const chunk = (t, d) => {
    const l = Buffer.alloc(4);
    l.writeUInt32BE(d.length);
    const td = Buffer.concat([Buffer.from(t), d]);
    const c = Buffer.alloc(4);
    c.writeUInt32BE(crc(td));
    return Buffer.concat([l, td, c]);
  };
  const raw = Buffer.alloc((IW * 3 + 1) * IH);
  for (let y = 0; y < IH; y++) px.copy(raw, y * (IW * 3 + 1) + 1, y * IW * 3, (y + 1) * IW * 3);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(IW, 0);
  ihdr.writeUInt32BE(IH, 4);
  ihdr[8] = 8;
  ihdr[9] = 2;
  writeFileSync(out, Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk('IHDR', ihdr), chunk('IDAT', deflateSync(raw)), chunk('IEND', Buffer.alloc(0))]));
  console.log('önizleme:', out);
}
