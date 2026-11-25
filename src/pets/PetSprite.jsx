// Kediyi <canvas> üzerine çizer. `animate` verilirse animasyon kareleri seyrek bir
// zamanlayıcıyla döner (yalnızca hareket izinliyse ve sayfa görünürse).
import { memo, useEffect, useRef, useState } from 'react';
import { CANVAS_H, CANVAS_W, SPRITES, drawGrid, petGrid } from './sprite.js';
import { useMotionAllowed } from '../ui/cats/useCatMotion.js';
import { useActiveTheme } from '../theme/context.js';

export const PetSprite = memo(function PetSprite({ breed = 'pink', anim = 'sit', accessories = [], scale = 3, flip = false, animate = false, interval = 450, className = '', title, palette }) {
  const ref = useRef(null);
  const theme = useActiveTheme();
  // Açıkça verilmezse etkin temanın kedi paleti (önizleme kutuları kendi paletini verir).
  const pal = palette ?? theme?.pets?.palette ?? 'none';
  const motion = useMotionAllowed();
  const [frame, setFrame] = useState(0);
  const count = SPRITES.animations[anim]?.length ?? 1;
  const accKey = accessories.join(',');

  useEffect(() => {
    if (!animate || !motion || count < 2) return undefined;
    const t = setInterval(() => setFrame((f) => (f + 1) % count), interval);
    return () => clearInterval(t);
  }, [animate, motion, count, interval]);

  useEffect(() => {
    const ctx = ref.current?.getContext('2d');
    if (!ctx) return;
    ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
    drawGrid(ctx, petGrid({ anim, frame, breed, accessories: accKey ? accKey.split(',') : [] }), { scale, flip, palette: pal });
  }, [anim, frame, breed, accKey, scale, flip, pal]);

  return (
    <canvas
      ref={ref}
      className={`pet-sprite ${className}`}
      width={CANVAS_W * scale}
      height={CANVAS_H * scale}
      role={title ? 'img' : undefined}
      aria-label={title}
      aria-hidden={title ? undefined : true}
    />
  );
});
