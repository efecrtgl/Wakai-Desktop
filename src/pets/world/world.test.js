import { describe, expect, it } from 'vitest';
import { ITEMS, ROOM, footprint } from './catalog.js';
import { blockedGrid, canPlace, findPath, fits, nearestFree, overlaps, interactionPoint } from './world.js';
import { chooseAction } from './ai.js';
import { ANIMS, createWorldCat, gameHour, periodOf, setAnim, stepToys, toyEntity, weatherAt } from './sim.js';
import { rng } from '../home/sim.js';

const needs = (o = {}) => ({ satiety: 80, energy: 80, happiness: 80, rest: 80, social: 70, trust: 50, curiosity: 70, cleanliness: 85, ...o });
const pet = (o = {}) => ({ id: 1, personality: 'calm', quirk: 'none', habits: {}, needs: needs(o.needs), ...o, needs: needs(o.needs) });
const item = (id, kind, tx, ty, extra = {}) => ({ id, kind, tx, ty, rot: 0, room: 'living', state: {}, ...extra });

describe('katalog ve yerleşim (Rust world.rs ile aynı kurallar)', () => {
  it('her eşyanın çizimi var ve odaya sığıyor', () => {
    for (const d of Object.values(ITEMS)) {
      expect(d.grid, d.kind).toBeTruthy();
      expect(fits(d, 0, d.wall ? 0 : ROOM.wall, 0), d.kind).toBe(true);
    }
  });

  it('katman, sınır ve dönüş kuralları', () => {
    expect(fits(ITEMS.bed, 19, 4, 0)).toBe(false);
    expect(fits(ITEMS.bed, 19, 4, 1)).toBe(true);
    expect(fits(ITEMS.window, 0, 4, 0)).toBe(false);
    expect(fits(ITEMS.window, 0, 0, 1)).toBe(false);
    expect(footprint(ITEMS.sofa, 1)).toEqual({ w: 1, h: 3 });
  });

  it('engelleyici eşyalar çakışmaz; halı ve oyuncaklar engel değildir', () => {
    const items = [item(1, 'bed', 2, 6), item(2, 'rug', 8, 7)];
    expect(overlaps(items, ITEMS.cushion, 3, 6)).toBe(true);
    expect(overlaps(items, ITEMS.cushion, 3, 6, 0, 1)).toBe(false);
    expect(canPlace(items, ITEMS.tree, 8, 7, 0)).toBe(true);
    expect(overlaps(items, ITEMS.ball, 2, 6)).toBe(false);
  });
});

describe('yol bulma', () => {
  it('engellerin etrafından dolaşır', () => {
    // Odanın ortasında dikey bir duvar (bir boşluk bırak)
    const items = [];
    for (let ty = ROOM.wall; ty < ROOM.h - 1; ty++) items.push(item(100 + ty, 'cushion', 10, ty));
    const g = blockedGrid(items);
    const path = findPath(g, { tx: 2, ty: 6 }, { tx: 17, ty: 6 });
    expect(path).not.toBeNull();
    expect(path.some((p) => p.ty === ROOM.h - 1 && p.tx === 10)).toBe(true);
    for (const p of path) expect(g[p.ty * ROOM.w + p.tx]).toBe(0);
  });

  it('ulaşılamayan hedefte null döner; engelin içindeki kedi en yakın boş karoya kurtarılır', () => {
    const items = [];
    for (let ty = ROOM.wall; ty < ROOM.h; ty++) items.push(item(200 + ty, 'cushion', 10, ty));
    const g = blockedGrid(items);
    expect(findPath(g, { tx: 2, ty: 6 }, { tx: 17, ty: 6 })).toBeNull();
    expect(nearestFree(g, 10, 6).tx).not.toBe(10);
    expect(nearestFree(g, 3, 1).ty).toBeGreaterThanOrEqual(ROOM.wall);
  });
});

describe('kedi yapay zekâsı', () => {
  const ctx = (p, items = [], extra = {}) => ({ pet: p, items, cats: [], toys: [], cursor: null, period: 1, weather: 'clear', now: 0, random: rng(7), ...extra });
  const cat = () => ({ id: 1, x: 100, y: 120, recent: [] });

  it('acıkınca dolu kaba gider; kap boşsa mama ister', () => {
    const bowl = item(1, 'bowl', 5, 6, { state: { portions: 2 } });
    expect(chooseAction(cat(), ctx(pet({ needs: { satiety: 10 } }), [bowl])).type).toBe('eat');
    const empty = item(1, 'bowl', 5, 6, { state: { portions: 0 } });
    const r = chooseAction(cat(), ctx(pet({ needs: { satiety: 10 } }), [empty]));
    expect(r.type).toBe('beg');
    expect(r.reason).toMatch(/kap boş/);
  });

  it('gece ve uykusuzken yatağa gider; sevdiği yatağı seçer', () => {
    const a = item(1, 'bed', 2, 6);
    const b = item(2, 'cushion', 15, 9);
    const p = pet({ needs: { rest: 15, energy: 30 }, habits: { spots: { 'living:15:9': 4 } } });
    const r = chooseAction(cat(), ctx(p, [a, b], { period: 3 }));
    expect(r.type).toBe('sleep');
    expect(r.item.id).toBe(2);
    expect(r.target).toEqual(interactionPoint(b));
  });

  it('ürken utangaç kedi saklanır', () => {
    const box = item(1, 'box', 3, 7);
    const r = chooseAction({ ...cat(), fearUntil: 10 }, ctx(pet({ quirk: 'shy' }), [box], { now: 0 }));
    expect(r.type).toBe('hide');
  });

  it('kişilik davranışı gerçekten değiştirir (oyuncu vs tembel)', () => {
    const toys = [item(1, 'ball', 8, 8), item(2, 'yarn', 12, 9)];
    const count = (personality, quirk) => {
      const random = rng(11);
      let n = 0;
      for (let i = 0; i < 300; i++) if (chooseAction(cat(), { ...ctx(pet({ personality, quirk, needs: { happiness: 50 } }), toys), random }).type === 'play') n++;
      return n;
    };
    expect(count('playful', 'mischievous')).toBeGreaterThan(count('sleepy', 'lazy') * 2);
  });

  it('hareket eden oyuncak kovalanır; aynı eylem art arda tekrarlanmaya daha az eğilimli', () => {
    const ball = item(1, 'ball', 8, 8);
    const toy = { ...toyEntity(ball), vx: 120, vy: 0 };
    const r = chooseAction(cat(), ctx(pet(), [ball], { toys: [toy] }));
    expect(r.type).toBe('play');
    expect(r.chase).toBe(true);
    expect(r.reason).toMatch(/Hareket/);
  });
});

describe('animasyon ve simülasyon', () => {
  it('uyuyan kedi önce uyanır ve gerinir; kilitli animasyon kesilmez ama taşınma keser', () => {
    const c = createWorldCat({ id: 1 }, rng(1));
    setAnim(c, 'sleep', 0);
    setAnim(c, 'walk', 1);
    expect(c.anim).toBe('wake');
    expect(c.queued).toBe('walk');
    setAnim(c, 'sit', 1.1);
    expect(c.anim).toBe('wake');
    setAnim(c, 'carried', 1.2);
    expect(c.anim).toBe('carried');
    expect(ANIMS.wake.next).toBe('stretch');
  });

  it('fırlatılan oyuncak sürtünmeyle durur ve oda içinde kalır', () => {
    const t = { ...toyEntity(item(1, 'ball', 10, 8)), vx: 400, vy: -300, moving: true };
    let rested = 0;
    const w = { toys: new Map([[1, t]]), cats: new Map() };
    for (let i = 0; i < 200; i++) stepToys(w, 0.05, { onToyRest: () => rested++ });
    expect(t.moving).toBe(false);
    expect(rested).toBe(1);
    expect(t.x).toBeGreaterThanOrEqual(6);
    expect(t.x).toBeLessThanOrEqual(ROOM.w * 16 - 6);
    expect(t.y).toBeGreaterThanOrEqual((ROOM.wall + 0.6) * 16);
  });

  it('oyun saati, gün dilimleri ve belirlenimci hava', () => {
    expect(periodOf(7)).toBe(0);
    expect(periodOf(13)).toBe(1);
    expect(periodOf(20)).toBe(2);
    expect(periodOf(2)).toBe(3);
    expect(gameHour(90 * 60000, 'fast')).toBeCloseTo(18, 5);
    const t = new Date(2026, 6, 10, 14).getTime();
    expect(weatherAt(t)).toBe(weatherAt(t + 30 * 60000));
    expect(['clear', 'cloudy', 'rain', 'snow']).toContain(weatherAt(t));
    for (let d = 0; d < 60; d++) expect(weatherAt(new Date(2026, 6, 1 + d, 9).getTime())).not.toBe('snow');
  });
});
