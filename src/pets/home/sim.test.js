import { describe, expect, it } from 'vitest';
import { chooseIntent, createCat, rng, stepCat } from './sim.js';
import { FURNITURE, FLOOR_BOTTOM, FLOOR_TOP, ROOM_W } from './furniture.js';

const items = [
  { id: 1, kind: 'bed', x: 60, y: 130 },
  { id: 2, kind: 'bowl', x: 200, y: 160 },
  { id: 3, kind: 'yarn', x: 150, y: 168 },
  { id: 4, kind: 'scratch', x: 270, y: 120 },
];
const fine = { satiety: 80, energy: 80, happiness: 80, rest: 80 };

describe('Kedi Evi simülasyonu', () => {
  it('uykulu kedi yatağa gidip uyur', () => {
    const r = rng(1);
    const cat = createCat({ id: 1 }, r);
    const ctx = { needs: { ...fine, rest: 10 }, personality: 'calm', items, cats: [cat], random: r };
    const intent = chooseIntent(cat, ctx);
    expect(intent.then).toBe('sleep');
    expect(intent.target.x).toBe(60);
    cat.until = 0;
    let t = 0;
    for (let i = 0; i < 400 && cat.state !== 'sleep'; i++) stepCat(cat, 0.125, (t += 0.125), ctx);
    expect(cat.state).toBe('sleep');
  });

  it('aç kedi mama kabına gider', () => {
    const r = rng(2);
    const cat = createCat({ id: 1 }, r);
    expect(chooseIntent(cat, { needs: { ...fine, satiety: 10 }, personality: 'calm', items, cats: [cat], random: r }).then).toBe('eat');
  });

  it('oyuncu kedi sakin kediden daha çok oynar', () => {
    const count = (personality) => {
      const r = rng(3);
      const cat = createCat({ id: 1 }, r);
      let n = 0;
      for (let i = 0; i < 3000; i++) if (chooseIntent(cat, { needs: fine, personality, items, cats: [cat], random: r }).then === 'play') n++;
      return n;
    };
    expect(count('playful')).toBeGreaterThan(count('calm') * 1.8);
  });

  it('kediler odanın zemininden çıkmaz', () => {
    const r = rng(4);
    const cats = [createCat({ id: 1 }, r), createCat({ id: 2 }, r)];
    let t = 0;
    for (let i = 0; i < 4000; i++) {
      t += 0.125;
      for (const c of cats) stepCat(c, 0.125, t, { needs: fine, personality: 'curious', items, cats, random: r });
    }
    for (const c of cats) {
      expect(c.x).toBeGreaterThanOrEqual(0);
      expect(c.x).toBeLessThanOrEqual(ROOM_W);
      expect(c.y).toBeGreaterThanOrEqual(FLOOR_TOP - 2);
      expect(c.y).toBeLessThanOrEqual(FLOOR_BOTTOM + 2);
    }
  });

  it('tüm eşyaların çizimi var', () => {
    for (const f of Object.values(FURNITURE)) expect(f.grid.flat().filter(Boolean).length).toBeGreaterThan(10);
  });
});
