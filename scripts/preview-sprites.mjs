// Piksel kedi sprite'larını PNG olarak önizler (bağımlılıksız):
//   node scripts/preview-sprites.mjs <çıktı.png> [katman=base,tailA]
import { writeFileSync } from 'node:fs';
import { deflateSync } from 'node:zlib';
import { SPRITES } from '../src/ui/cats/sprites.js';

const COLORS = {
  o: [91, 74, 79], p: [247, 184, 200], s: [244, 154, 178], w: [255, 241, 244], k: [61, 48, 52],
  n: [232, 138, 163], c: [244, 154, 178], y: [180, 162, 227], g: [221, 210, 245], m: [246, 176, 143], h: [255, 255, 255],
};
const out = process.argv[2] ?? 'sprites.png';
const layers = (process.argv[3] ?? 'base').split(',');
const SCALE = 10, GAP = 2;
const names = Object.keys(SPRITES);
const cellW = 20, cellH = 18;
const W = (cellW + GAP) * names.length * SCALE, H = cellH * SCALE;
const px = Buffer.alloc(W * H * 3, 255);
names.forEach((name, i) => {
  const s = SPRITES[name];
  for (const layer of layers) {
    const grid = s[layer];
    if (!grid) continue;
    grid.forEach((row, y) => [...row].forEach((ch, x) => {
      const c = COLORS[ch]; if (!c) return;
      for (let dy = 0; dy < SCALE; dy++) for (let dx = 0; dx < SCALE; dx++) {
        const X = ((cellW + GAP) * i + x) * SCALE + dx, Y = y * SCALE + dy;
        if (X < W && Y < H) px.set(c, (Y * W + X) * 3);
      }
    }));
  }
});
const crc = (b) => { let c = ~0; for (const v of b) { c ^= v; for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1)); } return ~c >>> 0; };
const chunk = (t, d) => { const l = Buffer.alloc(4); l.writeUInt32BE(d.length); const td = Buffer.concat([Buffer.from(t), d]); const c = Buffer.alloc(4); c.writeUInt32BE(crc(td)); return Buffer.concat([l, td, c]); };
const raw = Buffer.alloc((W * 3 + 1) * H); for (let y = 0; y < H; y++) px.copy(raw, y * (W * 3 + 1) + 1, y * W * 3, (y + 1) * W * 3);
const ihdr = Buffer.alloc(13); ihdr.writeUInt32BE(W, 0); ihdr.writeUInt32BE(H, 4); ihdr[8] = 8; ihdr[9] = 2;
writeFileSync(out, Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk('IHDR', ihdr), chunk('IDAT', deflateSync(raw)), chunk('IEND', Buffer.alloc(0))]));
console.log(`${out}: ${names.join(', ')}`);
