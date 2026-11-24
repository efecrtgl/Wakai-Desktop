// Piksel kedi bileşeni: sprite katmanlarını tek bir SVG'ye dönüştürür.
// Aynı renkteki yatay piksel dizileri tek bir <rect> olarak birleştirilir (daha az DOM düğümü).
// Sürekli animasyonlar seyrek JS zamanlayıcılarıyla yapılır (bkz. useCatMotion.js); CSS yalnızca
// üzerine gelince kuyruk sallama ve tek seferlik kutlama için kullanılır.
import { memo } from 'react';
import { useBlink, useFrame, useMotionAllowed } from './useCatMotion.js';
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

const has = (layers, name) => layers.some((l) => l.name === name);

/** Bir katmanın SVG grubu; içerik sabittir, yalnızca görünürlük değişir. */
const Layer = memo(function Layer({ layer, hidden, dy = 0 }) {
  return (
    <g className={`pc-layer pc-layer--${layer.name}`} opacity={hidden ? 0 : 1} transform={dy ? `translate(0 ${dy})` : undefined}>
      {layer.rects.map((r, i) => (
        <rect key={i} x={r.x} y={r.y} width={r.w} height={1} fill={r.fill} />
      ))}
    </g>
  );
});

/**
 * @param {object} props
 * @param {string} props.pose       face | sitting | happy | curious | relaxed | working | sleeping
 * @param {number} props.size       piksel cinsinden genişlik
 * @param {boolean} props.celebrate parıltı animasyonunu bir kez oynat
 * @param {boolean} props.wag       kuyruğu sürekli salla (varsayılan: yalnızca üzerine gelince)
 */
export const PixelCat = memo(function PixelCat({ pose = 'face', size = 48, celebrate = false, wag = false, className = '', title }) {
  const layers = layersFor(pose);
  const motion = useMotionAllowed();
  const blink = useBlink(motion && has(layers, 'blink'));
  const frame = useFrame(motion && (has(layers, 'zz') || has(layers, 'question')), 1500);
  const wagFrame = useFrame(motion && wag && has(layers, 'tailB'), 450);
  const classes = ['pixel-cat', `pixel-cat--${pose}`, celebrate && 'pixel-cat--celebrate', className].filter(Boolean).join(' ');

  return (
    <svg
      className={classes}
      viewBox="0 0 16 16"
      width={size}
      height={size}
      shapeRendering="crispEdges"
      role={title ? 'img' : undefined}
      aria-hidden={title ? undefined : true}
    >
      {title ? <title>{title}</title> : null}
      {layers.map((layer) => {
        switch (layer.name) {
          case 'blink':
            return <Layer key={layer.name} layer={layer} hidden={!blink} />;
          case 'tailA':
            return <Layer key={layer.name} layer={layer} hidden={wagFrame === 1} />;
          case 'tailB':
            return <Layer key={layer.name} layer={layer} hidden={wagFrame === 0} />;
          case 'sparkle':
            return <Layer key={layer.name} layer={layer} hidden={!celebrate} />;
          case 'zz':
          case 'question':
            return <Layer key={layer.name} layer={layer} dy={frame ? -1 : 0} />;
          default:
            return <Layer key={layer.name} layer={layer} />;
        }
      })}
    </svg>
  );
});
