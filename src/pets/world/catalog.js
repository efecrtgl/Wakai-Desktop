// Kedi Evi eşya kataloğu: kurallar catalog.json'dadır (Rust ile ortak), çizimler burada.
// Her çizim bir piksel ızgarasıdır; eşyanın kapladığı karoların ALT ORTASINA yerleştirilir.
import CATALOG from './catalog.json';
import { FURNITURE, makeGrid } from '../home/furniture.js';

export { CATALOG };
export const TILE = 16;
export const ROOM = CATALOG.room; // { w: 20, h: 12, wall: 4 } karo
export const ROOM_PX = { w: ROOM.w * TILE, h: ROOM.h * TILE };
export const ROOMS = CATALOG.rooms;
/** Pencere camı: çizimde bu renk, günün saatine göre gökyüzü rengiyle değiştirilir. */
export const SKY = '#9fd0f0';

const grids = {
  bed: FURNITURE.bed.grid,
  cushion: FURNITURE.cushion.grid,
  box: FURNITURE.box.grid,
  scratch: FURNITURE.scratch.grid,
  bowl: FURNITURE.bowl.grid,
  yarn: FURNITURE.yarn.grid,
  ball: FURNITURE.ball.grid,
  plant: FURNITURE.plant.grid,
  lamp: FURNITURE.lamp.grid,
  rug: makeGrid(46, 28, ({ ellipse }) => {
    ellipse(23, 14, 22, 13, '#ffd9c4');
    ellipse(23, 14, 17, 9, '#ffeee4');
    ellipse(23, 14, 9, 4, '#ffd9c4');
  }),
  cathouse: makeGrid(30, 30, ({ rect, set }) => {
    for (let y = 2; y <= 12; y++) rect(15 - y, y, 15 + y - 1, y, '#e88aa3');
    rect(4, 13, 25, 28, '#fbe0c8');
    rect(4, 13, 25, 14, '#e2c2a4');
    rect(10, 18, 19, 28, '#5b4a4f');
    rect(11, 19, 18, 28, '#3d3034');
    set(15, 7, '#ffe4ec');
  }),
  tree: makeGrid(30, 40, ({ rect, ellipse }) => {
    rect(13, 6, 16, 37, '#e8d3a8');
    for (let y = 8; y < 36; y += 3) rect(13, y, 16, y, '#cdb07c');
    ellipse(8, 22, 7, 3, '#ddd2f5');
    ellipse(21, 12, 7, 3, '#ddd2f5');
    ellipse(15, 4, 8, 3, '#b4a2e3');
    rect(4, 36, 25, 38, '#b88a5e');
  }),
  sofa: makeGrid(46, 22, ({ rect }) => {
    rect(3, 2, 42, 10, '#8fb7d9');
    rect(1, 8, 44, 17, '#a7c8e6');
    rect(0, 7, 4, 17, '#8fb7d9');
    rect(41, 7, 45, 17, '#8fb7d9');
    rect(6, 10, 21, 12, '#c3dbef');
    rect(24, 10, 39, 12, '#c3dbef');
    rect(2, 18, 4, 20, '#7a6b71');
    rect(41, 18, 43, 20, '#7a6b71');
  }),
  water: makeGrid(12, 6, ({ rect }) => {
    rect(2, 1, 9, 2, '#a8999f');
    rect(1, 2, 10, 4, '#cfe8f7');
    rect(3, 5, 8, 5, '#9fcde8');
  }),
  mouse: makeGrid(11, 6, ({ rect, set }) => {
    rect(2, 1, 7, 4, '#a8999f');
    set(8, 2, '#a8999f');
    set(3, 0, '#f2a7b6');
    set(9, 3, '#7a6b71'), set(10, 4, '#7a6b71');
    set(7, 2, '#3d3034');
  }),
  feather: makeGrid(7, 14, ({ rect, set }) => {
    rect(3, 5, 3, 12, '#7a6b71');
    rect(1, 0, 5, 4, '#b4a2e3');
    set(3, 1, '#efe9fb'), set(2, 3, '#efe9fb');
  }),
  window: makeGrid(30, 30, ({ rect }) => {
    rect(0, 0, 29, 29, '#ffffff');
    rect(2, 2, 13, 25, SKY);
    rect(16, 2, 27, 25, SKY);
    rect(0, 26, 29, 29, '#e2c6cf');
  }),
  painting: makeGrid(26, 14, ({ rect, ellipse }) => {
    rect(0, 0, 25, 13, '#cdb07c');
    rect(2, 2, 23, 11, '#fff4ec');
    ellipse(9, 8, 4, 3, '#7fcfa6');
    ellipse(17, 6, 3, 2, '#f6b08f');
  }),
  shelf: makeGrid(30, 10, ({ rect }) => {
    rect(0, 6, 29, 8, '#b88a5e');
    rect(3, 1, 7, 5, '#8fc3ee');
    rect(10, 2, 13, 5, '#f49ab2');
    rect(18, 0, 26, 5, '#f3d27a');
  }),
  lantern: makeGrid(10, 20, ({ rect }) => {
    rect(3, 0, 6, 1, '#7a6b71');
    rect(1, 2, 8, 12, '#fff4cc');
    rect(1, 2, 8, 3, '#e2955f');
    rect(1, 11, 8, 12, '#e2955f');
    rect(4, 13, 5, 17, '#7a6b71');
    rect(2, 18, 7, 19, '#7a6b71');
  }),
};

/** Eşya tanımı: kurallar + çizim. */
export const ITEMS = Object.fromEntries(Object.entries(CATALOG.items).map(([kind, d]) => [kind, { kind, ...d, grid: grids[kind] }]));

export const CATEGORIES = [
  { id: 'furniture', label: 'Mobilya' },
  { id: 'equipment', label: 'Kedi eşyası' },
  { id: 'toy', label: 'Oyuncak' },
  { id: 'decor', label: 'Dekorasyon' },
];

/** Dönüşe göre kapladığı alan (Rust `footprint` ile aynı). */
export function footprint(def, rot = 0) {
  return rot % 2 === 1 ? { w: def.h, h: def.w } : { w: def.w, h: def.h };
}
