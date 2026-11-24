// Piksel kedi bileşeni: sprite katmanlarını tek bir SVG'ye dönüştürür.
// Aynı renkteki yatay piksel dizileri tek bir <rect> olarak birleştirilir (daha az DOM düğümü).
// Animasyonlar yalnızca CSS ile ve düşük kare hızında (steps) yapılır; hareket azaltıldığında durur.
import { memo, useMemo } from 'react';
import { PALETTE, SPRITES } from './sprites.js';
import './cats.css';

function buildRects(grid) {
  const rects = [];
  grid.forEach((row, y) => {
    let x = 0;
    while (x < row.length) {
      const ch = row[x];
      let run = 1;
      while (x + run < row.length && row[x + run] === ch) run++;
      if (PALETTE[ch]) rects.push({ x, y, w: run, fill: PALETTE[ch] });
      x += run;
    }
  });
  return rects;
}

// Katmanlar modül düzeyinde bir kez hesaplanır ve tüm örnekler tarafından paylaşılır.
const cache = new Map();
function layersFor(pose) {
  if (!cache.has(pose)) {
    const sprite = SPRITES[pose] ?? SPRITES.face;
    cache.set(
      pose,
      Object.entries(sprite).map(([name, grid]) => ({ name, rects: buildRects(grid) })),
    );
  }
  return cache.get(pose);
}

/**
 * @param {object} props
 * @param {string} props.pose       face | sitting | happy | curious | relaxed | working | sleeping
 * @param {number} props.size       piksel cinsinden genişlik
 * @param {boolean} props.celebrate parıltı animasyonunu bir kez oynat
 * @param {boolean} props.wag       kuyruğu sürekli salla (varsayılan: yalnızca üzerine gelince)
 */
export const PixelCat = memo(function PixelCat({ pose = 'face', size = 48, celebrate = false, wag = false, className = '', title }) {
  const layers = layersFor(pose);
  // Her kedi farklı anda göz kırpsın diye rastgele bir gecikme (bileşen ömrü boyunca sabit).
  const blinkDelay = useMemo(() => `${(Math.random() * 5).toFixed(2)}s`, []);
  const classes = ['pixel-cat', `pixel-cat--${pose}`, celebrate && 'pixel-cat--celebrate', wag && 'pixel-cat--wag', className]
    .filter(Boolean)
    .join(' ');

  return (
    <svg
      className={classes}
      viewBox="0 0 16 16"
      width={size}
      height={size}
      shapeRendering="crispEdges"
      role={title ? 'img' : undefined}
      aria-hidden={title ? undefined : true}
      style={{ '--pc-blink-delay': blinkDelay }}
    >
      {title ? <title>{title}</title> : null}
      {layers.map((layer) => (
        <g key={layer.name} className={`pc-layer pc-layer--${layer.name}`}>
          {layer.rects.map((r, i) => (
            <rect key={i} x={r.x} y={r.y} width={r.w} height={1} fill={r.fill} />
          ))}
        </g>
      ))}
    </svg>
  );
});
