// Kedi Evi eşyaları: kedilerle aynı çizim dili (şekiller + otomatik kontur).
// Her eşya bir ızgaradır; konum, eşyanın ALT ORTA noktasıdır (zemin üzerindeki ayak izi).

export function makeGrid(w, h, draw) {
  const g = Array.from({ length: h }, () => Array(w).fill(null));
  const set = (x, y, c) => {
    if (x >= 0 && y >= 0 && x < w && y < h) g[y][x] = c;
  };
  const rect = (x0, y0, x1, y1, c) => {
    for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) set(x, y, c);
  };
  const ellipse = (cx, cy, rx, ry, c) => {
    for (let y = -ry; y <= ry; y++)
      for (let x = -rx; x <= rx; x++) if ((x * x) / (rx * rx) + (y * y) / (ry * ry) <= 1) set(cx + x, cy + y, c);
  };
  draw({ set, rect, ellipse });
  // Otomatik kontur
  const out = g.map((r) => r.slice());
  for (let y = 0; y < h; y++)
    for (let x = 0; x < w; x++) {
      if (g[y][x]) continue;
      const n = [g[y][x - 1], g[y][x + 1], g[y - 1]?.[x], g[y + 1]?.[x]];
      if (n.some(Boolean)) out[y][x] = OUTLINE;
    }
  return out;
}

export const OUTLINE = '#5b4a4f';

export const FURNITURE = {
  bed: {
    name: 'Kedi yatağı',
    role: 'bed',
    grid: makeGrid(26, 11, ({ ellipse }) => {
      ellipse(13, 6, 12, 4, '#e88aa3');
      ellipse(13, 6, 9, 2, '#ffe4ec');
    }),
  },
  cushion: {
    name: 'Minder',
    role: 'bed',
    grid: makeGrid(18, 7, ({ rect, set }) => {
      rect(1, 1, 16, 5, '#ddd2f5');
      rect(1, 4, 16, 5, '#c9bbee');
      set(8, 3, '#b4a2e3');
      set(9, 3, '#b4a2e3');
    }),
  },
  scratch: {
    name: 'Tırmalama tahtası',
    role: 'scratch',
    grid: makeGrid(16, 26, ({ rect, set }) => {
      rect(1, 1, 14, 3, '#f6b08f');
      rect(6, 4, 9, 22, '#e8d3a8');
      for (let y = 5; y < 22; y += 3) rect(6, y, 9, y, '#cdb07c');
      rect(1, 22, 14, 24, '#b88a5e');
      set(3, 2, '#ffeee4');
    }),
  },
  bowl: {
    name: 'Mama kabı',
    role: 'food',
    grid: makeGrid(12, 6, ({ rect }) => {
      rect(2, 1, 9, 2, '#b88a5e');
      rect(1, 2, 10, 4, '#8fc3ee');
      rect(3, 5, 8, 5, '#6fa9d8');
    }),
  },
  yarn: {
    name: 'Yün yumağı',
    role: 'toy',
    grid: makeGrid(9, 9, ({ ellipse, set }) => {
      ellipse(4, 4, 3, 3, '#f49ab2');
      set(3, 3, '#ffe4ec');
      set(5, 5, '#cf6b88');
      set(2, 5, '#cf6b88');
    }),
  },
  ball: {
    name: 'Top',
    role: 'toy',
    grid: makeGrid(8, 8, ({ ellipse, set }) => {
      ellipse(3, 3, 3, 3, '#8fc3ee');
      set(2, 2, '#ffffff');
    }),
  },
  plant: {
    name: 'Saksı bitkisi',
    role: 'deco',
    grid: makeGrid(14, 22, ({ rect, ellipse }) => {
      ellipse(7, 7, 6, 6, '#7fcfa6');
      ellipse(4, 9, 3, 3, '#6fbf97');
      ellipse(10, 9, 3, 3, '#6fbf97');
      rect(3, 14, 10, 20, '#f6b08f');
      rect(2, 14, 11, 15, '#e2955f');
    }),
  },
  lamp: {
    name: 'Lamba',
    role: 'deco',
    grid: makeGrid(12, 30, ({ rect }) => {
      rect(1, 1, 10, 7, '#fff4cc');
      rect(1, 6, 10, 7, '#f3d27a');
      rect(5, 8, 6, 26, '#a8999f');
      rect(2, 27, 9, 28, '#7a6b71');
    }),
  },
  rug: {
    name: 'Halı',
    role: 'floor',
    grid: makeGrid(48, 13, ({ ellipse }) => {
      ellipse(24, 6, 23, 5, '#ffd9c4');
      ellipse(24, 6, 18, 3, '#ffeee4');
    }),
  },
  box: {
    name: 'Karton kutu',
    role: 'bed',
    grid: makeGrid(18, 14, ({ rect }) => {
      rect(1, 4, 16, 12, '#d9b382');
      rect(1, 4, 16, 5, '#c79e6b');
      rect(0, 1, 4, 3, '#e3c49a');
      rect(13, 1, 17, 3, '#e3c49a');
    }),
  },
};

export const WALLS = {
  pink: { name: 'Pembe', base: '#fde7ee', stripe: '#fbd9e4' },
  mint: { name: 'Nane', base: '#e3f5ec', stripe: '#d3eee1' },
  lavender: { name: 'Lavanta', base: '#efe9fb', stripe: '#e3dbf6' },
  cream: { name: 'Krem', base: '#fff4e6', stripe: '#fbe9d2' },
  night: { name: 'Gece', base: '#2e2a40', stripe: '#36314b' },
};

export const FLOORS = {
  wood: { name: 'Ahşap', base: '#e8c9a3', line: '#d6b38a' },
  tile: { name: 'Fayans', base: '#f1ece8', line: '#e0d8d2' },
  carpet: { name: 'Halı kaplama', base: '#e9d6ee', line: '#e0c9e7' },
};

export const ROOM_W = 320;
export const ROOM_H = 180;
export const FLOOR_TOP = 104;
export const FLOOR_BOTTOM = 176;
