// Simülasyon: animasyon durum makinesi, kedi hareketi (A* yolları), eylem yürütme, oyuncak
// fiziği, oyun saati ve hava. Çizimden bağımsızdır; adım fonksiyonu dt ile çağrılır.
import { ITEMS, ROOM, ROOM_PX, TILE } from './catalog.js';
import { animFor, chooseAction } from './ai.js';
import { blockedGrid, findPath, nearestFree, tileOf, centerOf } from './world.js';
import { SPRITES } from '../sprite.js';

// ── Animasyon motoru ────────────────────────────────────────────────────────
/** secs: kare süresi; once: bir kez oynar, sonra `next`. Kilitli animasyonlar (once) yalnızca
 *  INTERRUPT listesindekilerle kesilebilir: ani ve geçersiz geçişler olmaz. */
export const ANIMS = {
  idle: { secs: 0.9 },
  walk: { secs: 0.12 },
  run: { secs: 0.08 },
  sit: { secs: 2.6 },
  look: { secs: 1.1 },
  sleep: { secs: 1.6 },
  wake: { secs: 0.5, once: true, next: 'stretch' },
  stretch: { secs: 0.55, once: true, next: 'sit' },
  yawn: { secs: 0.9, once: true, next: 'sit' },
  jump: { secs: 0.3, once: true, next: 'sit' },
  fall: { secs: 0.3, once: true, next: 'sit' },
  carried: { secs: 1 },
  petted: { secs: 0.35 },
  play: { secs: 0.22 },
  eat: { secs: 0.3 },
  groom: { secs: 0.4 },
  scratch: { secs: 0.16 },
  crouch: { secs: 1.2 },
};
const INTERRUPT = new Set(['carried', 'fall', 'petted']);

export function setAnim(cat, name, now) {
  if (!ANIMS[name] || !SPRITES.animations[name]) name = 'sit';
  if (cat.anim === name) return;
  const cur = ANIMS[cat.anim];
  if (cur?.once && now < cat.animUntil && !INTERRUPT.has(name)) {
    cat.queued = name;
    return;
  }
  // Uyuyan kedi doğrudan koşmaz: önce uyanır ve gerinir.
  if (cat.anim === 'sleep' && !INTERRUPT.has(name) && name !== 'sleep') {
    cat.queued = name;
    name = 'wake';
  }
  cat.anim = name;
  cat.frame = 0;
  cat.frameAt = now + ANIMS[name].secs;
  const frames = SPRITES.animations[name].length;
  cat.animUntil = ANIMS[name].once ? now + ANIMS[name].secs * frames : Infinity;
}

function tickAnim(cat, now, motion) {
  const def = ANIMS[cat.anim];
  if (def.once && now >= cat.animUntil) {
    const next = cat.queued ?? def.next ?? 'sit';
    cat.queued = null;
    cat.anim = null;
    setAnim(cat, next, now);
    return;
  }
  if (motion === 'off') return;
  if (now >= cat.frameAt) {
    cat.frame += 1;
    cat.frameAt = now + def.secs * (motion === 'reduced' ? 2 : 1);
  }
}

// ── Oyun saati ve hava ──────────────────────────────────────────────────────
/** Oyun saati (0–24). "real": yerel saat; "fast": bir oyun saati = bir gerçek dakika. */
export function gameHour(now, mode = 'real') {
  if (mode === 'fast') return (now / 60000) % 24;
  const d = new Date(now);
  return d.getHours() + d.getMinutes() / 60;
}
/** 0 sabah, 1 öğleden sonra, 2 akşam, 3 gece (Rust `period` ile aynı sınırlar). */
export function periodOf(hour) {
  if (hour >= 6 && hour < 12) return 0;
  if (hour >= 12 && hour < 18) return 1;
  if (hour >= 18 && hour < 23) return 2;
  return 3;
}
export const PERIOD_NAMES = ['Sabah', 'Öğleden sonra', 'Akşam', 'Gece'];

/** Belirlenimci hava: gün ve 3 saatlik dilimden tohumlanır; kışın yağmur yerine kar. */
export function weatherAt(now) {
  const d = new Date(now);
  const seed = d.getFullYear() * 10000 + (d.getMonth() + 1) * 100 + d.getDate() * 8 + Math.floor(d.getHours() / 3);
  let x = (seed * 2654435761) >>> 0;
  x ^= x >>> 13;
  const r = (x % 1000) / 1000;
  const winter = [11, 0, 1].includes(d.getMonth());
  if (r < 0.55) return 'clear';
  if (r < 0.75) return 'cloudy';
  return winter ? 'snow' : 'rain';
}
export const WEATHER_NAMES = { clear: 'Açık', cloudy: 'Bulutlu', rain: 'Yağmurlu', snow: 'Karlı' };

// ── Dünya durumu ────────────────────────────────────────────────────────────
export function createWorldCat(pet, random) {
  const g = { x: pet.wx ?? 60 + random() * (ROOM_PX.w - 120), y: pet.wy ?? (ROOM.wall + 2 + random() * (ROOM.h - ROOM.wall - 3)) * TILE };
  return {
    id: pet.id,
    x: g.x,
    y: g.y,
    z: 0,
    face: random() < 0.5,
    anim: 'sit',
    frame: 0,
    frameAt: 0,
    animUntil: Infinity,
    queued: null,
    action: null,
    path: [],
    recent: [],
    stuck: 0,
    carried: false,
    emote: null,
    reason: 'Yeni geldi, etrafa bakıyor',
    sleeping: false,
    fearUntil: 0,
    rubbed: 0,
    idleAt: 0,
  };
}

/** Kediyi yürünebilir bir karoya bırakır (eşya ya da duvar içinde kalmaz). */
export function settle(cat, grid) {
  const t = tileOf(cat.x, cat.y);
  const f = nearestFree(grid, t.tx, t.ty);
  if (f.tx !== t.tx || f.ty !== t.ty) Object.assign(cat, centerOf(f.tx, f.ty));
  cat.z = 0;
}

function planPath(cat, target, grid) {
  if (!target) return [];
  const path = findPath(grid, tileOf(cat.x, cat.y), tileOf(target.x, target.y));
  if (path === null) return null;
  const pts = path.map((p) => centerOf(p.tx, p.ty));
  // Son nokta hedefin kendisi (yatağın üstü gibi kapalı karolar dahil)
  pts.push({ x: target.x, y: target.y });
  return pts;
}

export function emote(cat, kind, now, secs = 1.6) {
  cat.emote = { kind, until: now + secs * 1000 };
}

/**
 * Dünyayı dt saniye ilerletir.
 * @param {object} w { items, grid, cats: Map, toys: Map, random }
 * @param {object} env { pets: Map(id→rec), now(ms), period, weather, cursor, motion,
 *   affinity(a,b), onEvent(cat, action), onSleep(cat, bool), onToyRest(item) }
 */
export function stepWorld(w, dt, env) {
  const now = env.now;
  const nowS = now / 1000;
  const cats = [...w.cats.values()];
  for (const cat of cats) {
    const pet = env.pets.get(cat.id);
    if (!pet) continue;
    if (cat.emote && now > cat.emote.until) cat.emote = null;
    tickAnim(cat, nowS, env.motion);
    if (cat.carried) {
      setAnim(cat, 'carried', nowS);
      continue;
    }
    if (cat.z > 0) {
      cat.z = Math.max(0, cat.z - 90 * dt);
      if (cat.z === 0) setAnim(cat, 'fall', nowS);
      continue;
    }
    const a = cat.action;
    if (!a || (a.phase === 'doing' && now >= a.until)) {
      if (a) finish(cat, a, env);
      decide(cat, w, env, cats, pet);
      continue;
    }
    if (a.phase === 'moving') move(cat, a, dt, w, env, nowS);
    else idleVariation(cat, a, env, nowS, pet);
  }
  stepToys(w, dt, env);
}

function decide(cat, w, env, cats, pet) {
  const nowS = env.now / 1000;
  const choice = chooseAction(cat, {
    pet,
    items: w.items,
    cats,
    toys: [...w.toys.values()],
    cursor: env.cursor,
    period: env.period,
    weather: env.weather,
    now: env.now,
    random: w.random,
    affinity: env.affinity,
  });
  cat.recent = [choice.type, ...cat.recent].slice(0, 3);
  cat.reason = choice.reason;
  const path = choice.target ? planPath(cat, choice.target, w.grid) : [];
  if (path === null) {
    // Ulaşılamayan hedef: kısa bir mola, sonra yeniden karar (takılıp kalmaz).
    cat.action = { type: 'rest', phase: 'doing', until: env.now + 1500 };
    setAnim(cat, 'sit', nowS);
    return;
  }
  cat.action = { ...choice, phase: path.length ? 'moving' : 'doing', until: env.now + choice.secs * 1000, retarget: 0, progressAt: env.now };
  cat.path = path;
  if (!path.length) arrive(cat, cat.action, env);
  else if (cat.sleeping) {
    cat.sleeping = false;
    env.onSleep?.(cat, false);
  }
}

function move(cat, a, dt, w, env, nowS) {
  // Hareket eden oyuncağı kovalama: hedef sık sık güncellenir.
  if (a.chase && a.item) {
    const toy = w.toys.get(a.item.id);
    if (toy && env.now > a.retarget) {
      a.retarget = env.now + 400;
      cat.path = planPath(cat, { x: toy.x, y: toy.y }, w.grid) ?? [];
    }
  }
  const p = cat.path[0];
  if (!p) {
    arrive(cat, a, env);
    return;
  }
  const speed = a.chase || a.type === 'hide' ? 62 : 28;
  const dx = p.x - cat.x;
  const dy = p.y - cat.y;
  const d = Math.hypot(dx, dy);
  setAnim(cat, speed > 40 ? 'run' : 'walk', nowS);
  if (Math.abs(dx) > 0.5) cat.face = dx > 0;
  if (d <= speed * dt) {
    cat.x = p.x;
    cat.y = p.y;
    cat.path.shift();
    a.progressAt = env.now;
  } else {
    cat.x += (dx / d) * speed * dt;
    cat.y += (dy / d) * speed * dt;
  }
  // Takılma koruması: 4 sn ilerleme yoksa yeniden planla.
  if (env.now - a.progressAt > 4000) {
    settle(cat, w.grid);
    cat.path = [];
    a.phase = 'doing';
    a.until = env.now + 800;
  }
}

function arrive(cat, a, env) {
  const nowS = env.now / 1000;
  a.phase = 'doing';
  a.until = env.now + (a.secs ?? 4) * 1000;
  if (a.item) {
    const it = a.item;
    const d = ITEMS[it.kind];
    if (d?.use === 'food' || d?.use === 'water' || d?.use === 'scratch') cat.face = true;
    if (d?.use === 'climb' && a.type === 'climb') {
      setAnim(cat, 'jump', nowS);
      env.onEvent?.(cat, a);
      a.reported = true;
    }
  }
  if (a.type === 'socialize' && a.other) cat.face = a.other.x > cat.x;
  setAnim(cat, animFor(a.type, a.item), nowS);
  if (a.type === 'sleep') {
    cat.sleeping = true;
    env.onSleep?.(cat, true);
    emote(cat, 'z', env.now, 3);
  }
  if (a.type === 'play') emote(cat, 'note', env.now);
  if (a.type === 'beg' || a.type === 'seek_user') emote(cat, a.type === 'beg' ? 'bowl' : 'heart', env.now, 2.5);
  if (a.type === 'hide') emote(cat, 'drop', env.now);
}

function finish(cat, a, env) {
  if (a.type === 'sleep' && cat.sleeping) {
    cat.sleeping = false;
    env.onSleep?.(cat, false);
  }
  if (!a.reported && ['eat', 'drink', 'play', 'groom', 'scratch', 'observe', 'explore', 'socialize', 'hide', 'sleep'].includes(a.type)) env.onEvent?.(cat, a);
}

function idleVariation(cat, a, env, nowS, pet) {
  if (env.motion !== 'full' || !['rest', 'socialize', 'climb'].includes(a.type)) return;
  if (nowS < cat.idleAt) return;
  cat.idleAt = nowS + 4 + Math.random() * 5;
  const r = Math.random();
  const sleepy = pet.needs.rest < 40;
  if (r < (sleepy ? 0.5 : 0.2)) setAnim(cat, 'yawn', nowS);
  else if (r < 0.6) setAnim(cat, 'look', nowS);
  else setAnim(cat, 'sit', nowS);
}

// ── Oyuncak fiziği ──────────────────────────────────────────────────────────
export function stepToys(w, dt, env) {
  for (const t of w.toys.values()) {
    if (t.held) continue;
    const sp = Math.hypot(t.vx, t.vy);
    if (sp < 4) {
      if (t.moving) {
        t.moving = false;
        t.vx = t.vy = 0;
        env.onToyRest?.(t);
      }
      // Kediler oynarken oyuncağı hafifçe iter.
      for (const c of w.cats.values()) {
        if (c.action?.type === 'play' && c.action.phase === 'doing' && c.action.item?.id === t.item.id && Math.random() < dt * 0.8) {
          t.vx = (c.face ? 1 : -1) * (40 + Math.random() * 40);
          t.vy = (Math.random() - 0.5) * 40;
          t.moving = true;
        }
      }
      continue;
    }
    t.moving = true;
    const f = Math.exp(-2.4 * dt);
    t.vx *= f;
    t.vy *= f;
    t.x += t.vx * dt;
    t.y += t.vy * dt;
    const minY = (ROOM.wall + 0.6) * TILE;
    if (t.x < 6 || t.x > ROOM_PX.w - 6) {
      t.vx = -t.vx * 0.6;
      t.x = Math.max(6, Math.min(ROOM_PX.w - 6, t.x));
    }
    if (t.y < minY || t.y > ROOM_PX.h - 2) {
      t.vy = -t.vy * 0.6;
      t.y = Math.max(minY, Math.min(ROOM_PX.h - 2, t.y));
    }
  }
}

/** Oyuncak öğesinden fizik varlığı (konum: karonun alt ortası). */
export function toyEntity(item) {
  return { item, x: item.tx * TILE + TILE / 2, y: item.ty * TILE + TILE - 2, vx: 0, vy: 0, moving: false, held: false };
}

export function rebuildGrid(w) {
  w.grid = blockedGrid(w.items);
}
