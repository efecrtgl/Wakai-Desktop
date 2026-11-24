import { describe, expect, it } from 'vitest';
import { CANVAS_H, CANVAS_W, SPRITES, petGrid } from './sprite.js';

const filled = (g) => g.flat().filter(Boolean).length;

describe('kedi sprite ızgarası', () => {
  it('her animasyon ve cins için dolu bir ızgara üretir', () => {
    for (const anim of Object.keys(SPRITES.animations)) {
      for (const breed of Object.keys(SPRITES.breeds)) {
        const g = petGrid({ anim, breed });
        expect(g).toHaveLength(CANVAS_H);
        expect(g[0]).toHaveLength(CANVAS_W);
        expect(filled(g)).toBeGreaterThan(50);
      }
    }
  });
  it('aksesuar ızgarayı değiştirir ve tuvalin içinde kalır', () => {
    const plain = petGrid({ anim: 'sit', breed: 'white' });
    for (const acc of Object.keys(SPRITES.accessories)) {
      const g = petGrid({ anim: 'sit', breed: 'white', accessories: [acc] });
      expect(JSON.stringify(g)).not.toBe(JSON.stringify(plain));
    }
  });
});
