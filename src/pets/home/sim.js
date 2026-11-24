// Kedi Evi davranış simülasyonu (saf mantık, test edilebilir).
// Kediler ihtiyaçlarına ve kişiliklerine göre yatağa, mama kabına, oyuncaklara ve
// tırmalama tahtasına gider; birbirlerinin yanına oturur. Yalnızca oda açıkken çalışır.
import { FURNITURE, FLOOR_BOTTOM, FLOOR_TOP, ROOM_W } from './furniture.js';

const SPEED = 22; // birim/sn

/** Tohumlanabilir rastgele sayı üreteci (mulberry32). */
export function rng(seed) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export function createCat(pet, random) {
  return {
    id: pet.id,
    x: 40 + random() * (ROOM_W - 80),
    y: FLOOR_TOP + 20 + random() * (FLOOR_BOTTOM - FLOOR_TOP - 20),
    face: random() < 0.5,
    state: 'sit',
    until: 1 + random() * 3,
    target: null,
    then: null,
    frame: 0,
    frameAt: 0,
  };
}

const ANIM = { walk: 'walk', sit: 'sit', sleep: 'sleep', eat: 'eat', play: 'play', idle: 'idle', look: 'look', petted: 'petted' };
const FRAME_SECS = { walk: 0.12, sit: 3, sleep: 1.6, eat: 0.3, play: 0.25, idle: 0.9, look: 1.2, petted: 0.35 };

export function animOf(cat) {
  return ANIM[cat.state] ?? 'sit';
}

const byRole = (items, role) => items.filter((it) => FURNITURE[it.kind]?.role === role);

function nearest(cat, list) {
  let best = null;
  let bd = Infinity;
  for (const it of list) {
    const d = Math.hypot(it.x - cat.x, it.y - cat.y);
    if (d < bd) {
      bd = d;
      best = it;
    }
  }
  return best;
}

/**
 * Bir sonraki niyeti seçer: { state, target: {x, y} | null, then, secs }.
 * Önce ihtiyaçlar (uyku → yatak, açlık → mama), sonra kişilik ağırlıkları.
 */
export function chooseIntent(cat, { needs, personality, items, cats, random }) {
  const beds = byRole(items, 'bed');
  const bowl = nearest(cat, byRole(items, 'food'));
  const toys = byRole(items, 'toy');
  const scratch = nearest(cat, byRole(items, 'scratch'));
  if (needs.rest < 35 && beds.length) {
    const bed = nearest(cat, beds);
    return { state: 'walk', target: { x: bed.x, y: bed.y - 1 }, then: 'sleep', secs: 90 + random() * 120 };
  }
  if (needs.satiety < 35 && bowl) {
    return { state: 'walk', target: { x: bowl.x - 12, y: bowl.y }, then: 'eat', secs: 5 };
  }
  const playful = personality === 'playful' ? 3 : personality === 'curious' ? 2 : 1;
  const sleepy = personality === 'sleepy' ? 3 : personality === 'calm' ? 2 : 1;
  const social = personality === 'affectionate' ? 3 : personality === 'independent' ? 0.2 : 1;
  const tired = needs.energy < 30 ? 0.2 : 1;
  const options = [
    ['wander', 3],
    ['sit', 2 * sleepy],
    ['look', 1],
    ['toy', toys.length ? 2 * playful * tired : 0],
    ['scratch', scratch ? 1 * playful * tired : 0],
    ['nap', beds.length ? 0.6 * sleepy : 0],
    ['friend', cats.length > 1 ? 1.2 * social : 0],
  ];
  const total = options.reduce((s, [, w]) => s + w, 0);
  let r = random() * total;
  let pick = 'sit';
  for (const [k, w] of options) {
    if (r < w) {
      pick = k;
      break;
    }
    r -= w;
  }
  const clampX = (x) => Math.max(14, Math.min(ROOM_W - 14, x));
  const clampY = (y) => Math.max(FLOOR_TOP + 6, Math.min(FLOOR_BOTTOM, y));
  switch (pick) {
    case 'wander':
      return { state: 'walk', target: { x: clampX(cat.x + (random() - 0.5) * 160), y: clampY(cat.y + (random() - 0.5) * 40) }, then: 'sit', secs: 3 + random() * 6 };
    case 'toy': {
      const t = toys[Math.floor(random() * toys.length)];
      return { state: 'walk', target: { x: t.x - 10, y: t.y }, then: 'play', secs: 4 + random() * 5 };
    }
    case 'scratch':
      return { state: 'walk', target: { x: scratch.x - 10, y: scratch.y }, then: 'play', secs: 3 + random() * 3 };
    case 'nap': {
      const bed = nearest(cat, beds);
      return { state: 'walk', target: { x: bed.x, y: bed.y - 1 }, then: 'sleep', secs: 30 + random() * 60 };
    }
    case 'friend': {
      const others = cats.filter((c) => c.id !== cat.id);
      const o = others[Math.floor(random() * others.length)];
      return { state: 'walk', target: { x: clampX(o.x + (o.x > cat.x ? -22 : 22)), y: o.y }, then: 'sit', secs: 5 + random() * 8 };
    }
    case 'look':
      return { state: 'look', target: null, then: null, secs: 2 + random() * 3 };
    default:
      return { state: 'sit', target: null, then: null, secs: 4 + random() * 8 };
  }
}

/** Kediyi `dt` saniye ilerletir (yerinde günceller). */
export function stepCat(cat, dt, now, ctx) {
  if (cat.state === 'walk' && cat.target) {
    const dx = cat.target.x - cat.x;
    const dy = cat.target.y - cat.y;
    const d = Math.hypot(dx, dy);
    if (d < 1.5) {
      cat.x = cat.target.x;
      cat.y = cat.target.y;
      cat.state = cat.then ?? 'sit';
      cat.target = null;
      cat.until = now + (cat.pendingSecs ?? 5);
      cat.frame = 0;
    } else {
      const step = Math.min(d, SPEED * dt);
      cat.x += (dx / d) * step;
      cat.y += (dy / d) * step;
      cat.face = dx >= 0;
    }
  } else if (now >= cat.until) {
    const intent = chooseIntent(cat, ctx);
    cat.state = intent.state;
    cat.target = intent.target;
    cat.then = intent.then;
    cat.pendingSecs = intent.secs;
    cat.until = now + intent.secs;
    cat.frame = 0;
  }
  if (now >= cat.frameAt) {
    cat.frame += 1;
    cat.frameAt = now + (FRAME_SECS[cat.state] ?? 1);
  }
  return cat;
}
