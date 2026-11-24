// Piksel kedi bileşeni: sprite ızgarasını tek bir SVG'ye dönüştürür.
// Aynı renkteki yatay piksel dizileri tek bir <rect> olarak birleştirilir (daha az DOM düğümü).
import { memo, useMemo } from 'react';
import { SPRITES, PALETTE } from './sprites.js';

function buildRects(grid) {
  const rects = [];
  grid.forEach((row, y) => {
    let x = 0;
    while (x < row.length) {
      const ch = row[x];
      let run = 1;
      while (x + run < row.length && row[x + run] === ch) run++;
      if (PALETTE[ch]) rects.push({ x, y, w: run, ch });
      x += run;
    }
  });
  return rects;
}

const cache = new Map();
function rectsFor(name) {
  if (!cache.has(name)) cache.set(name, buildRects(SPRITES[name] ?? SPRITES.face));
  return cache.get(name);
}

export const PixelCat = memo(function PixelCat({ pose = 'face', size = 48, className = '', title }) {
  const rects = useMemo(() => rectsFor(pose), [pose]);
  const grid = SPRITES[pose] ?? SPRITES.face;
  const w = grid[0].length;
  const h = grid.length;
  return (
    <svg
      className={`pixel-cat pixel-cat--${pose} ${className}`}
      viewBox={`0 0 ${w} ${h}`}
      width={size}
      height={(size * h) / w}
      shapeRendering="crispEdges"
      role={title ? 'img' : undefined}
      aria-hidden={title ? undefined : true}
    >
      {title ? <title>{title}</title> : null}
      {rects.map((r, i) => (
        <rect key={i} x={r.x} y={r.y} width={r.w} height={1} fill={PALETTE[r.ch]} className={`px-${r.ch}`} />
      ))}
    </svg>
  );
});
