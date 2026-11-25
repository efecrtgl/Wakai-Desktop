// Kedi yapay zekâsı: fayda tabanlı (utility AI) karar verme. Her olası eylem ihtiyaçlar,
// kişilik, ikincil özellik, alışkanlıklar, günün saati, odadaki eşyalar, diğer kediler,
// ilişkiler ve son eylemlerden puan alır; en yüksek puanlı eylem seçilir. Seçimin nedeni
// (`reason`) saklanır; arayüz "neden bunu yapıyor" sorusunu yanıtlayabilir. Rastgelelik yalnızca
// küçük bir çeşitlilik payıdır ve tohumlanabilir (testlerde belirlenimci).
import { ITEMS, ROOM, TILE } from './catalog.js';
import { interactionPoint } from './world.js';

/** Kişilik ve ikincil özelliğin eylem çarpanları. */
const PERSONALITY = {
  playful: { play: 1.7, explore: 1.2, rest: 0.7, scratch: 1.3 },
  calm: { rest: 1.5, play: 0.8, observe: 1.2 },
  curious: { explore: 1.6, climb: 1.5, observe: 1.4 },
  sleepy: { sleep: 1.6, rest: 1.3, play: 0.7 },
  affectionate: { seek_user: 1.7, socialize: 1.4 },
  independent: { seek_user: 0.4, socialize: 0.6, explore: 1.2, climb: 1.2 },
};
const QUIRK = {
  none: {},
  mischievous: { play: 1.3, scratch: 1.6, explore: 1.3 },
  shy: { hide: 2.2, seek_user: 0.4, socialize: 0.7 },
  lazy: { sleep: 1.4, rest: 1.5, play: 0.6, climb: 0.6, explore: 0.7 },
};

export const ACTION_LABELS = {
  sleep: 'uyuyor',
  eat: 'yemek yiyor',
  drink: 'su içiyor',
  play: 'oynuyor',
  groom: 'tımarlanıyor',
  scratch: 'tırmalıyor',
  climb: 'tırmanıyor',
  observe: 'pencereden bakıyor',
  explore: 'etrafı keşfediyor',
  socialize: 'arkadaşının yanında',
  seek_user: 'ilgi istiyor',
  hide: 'saklanıyor',
  rest: 'dinleniyor',
  wander: 'dolaşıyor',
  beg: 'mama bekliyor',
};

const need = (v) => Math.max(0, Math.min(1, (100 - v) / 100));

function topHabit(habits, group) {
  const m = habits?.[group];
  if (!m || typeof m !== 'object') return null;
  let best = null;
  let bn = 0;
  for (const [k, n] of Object.entries(m)) if (n > bn) [best, bn] = [k, n];
  return best;
}

function nearestBy(cat, list, pick = (x) => x) {
  let best = null;
  let bd = Infinity;
  for (const it of list) {
    const p = pick(it);
    const d = Math.hypot(p.x - cat.x, p.y - cat.y);
    if (d < bd) [best, bd] = [it, d];
  }
  return best;
}

/**
 * Bir sonraki eylemi seçer.
 * @param {object} cat çalışma zamanı kedisi { id, x, y, recent: string[], fearUntil }
 * @param {object} ctx { pet, items, cats, toys, cursor, period, weather, now, random, affinity(a,b) }
 * @returns {{ type, target: {x,y}|null, item?, other?, secs, reason, scores }}
 */
export function chooseAction(cat, ctx) {
  const { pet, items, cats, cursor, period, weather, now, random } = ctx;
  const n = pet.needs;
  const pm = { ...(PERSONALITY[pet.personality] ?? {}), __q: 0 };
  const qm = QUIRK[pet.quirk] ?? {};
  const mult = (type) => (pm[type] ?? 1) * (qm[type] ?? 1);
  const byUse = (use) => items.filter((it) => ITEMS[it.kind]?.use === use);
  const favFurniture = topHabit(pet.habits, 'furniture');
  const favToy = topHabit(pet.habits, 'toys');
  const night = period === 3;
  const options = [];
  const add = (type, score, extra, reason) => {
    if (score <= 0) return;
    let s = score * mult(type);
    if (cat.recent?.[0] === type) s *= 0.45;
    else if (cat.recent?.includes(type)) s *= 0.75;
    if (extra?.item && favFurniture === extra.item.kind) s *= 1.3;
    s *= 0.85 + random() * 0.3;
    options.push({ type, score: s, reason, ...extra });
  };

  // Uyku: uyku ihtiyacı, gece, sevilen yatak
  const beds = byUse('bed');
  const spot = topHabit(pet.habits, 'spots');
  const favBed = beds.find((b) => spot && spot === `${b.room ?? ''}:${b.tx}:${b.ty}`) ?? beds.find((b) => b.kind === favFurniture);
  const bed = favBed ?? nearestBy(cat, beds, interactionPoint);
  const sleepNeed = need(n.rest) * 1.4 + need(n.energy) * 0.5 + (night ? 0.35 : 0);
  add('sleep', sleepNeed * (bed ? 1 : 0.6), { item: bed, target: bed ? interactionPoint(bed) : null, secs: 45 + random() * 90 }, night ? 'Gece oldu, uykusu geldi' : 'Uykusu geldi');

  // Yemek ve su
  const bowls = byUse('food').filter((b) => (b.state?.portions ?? 0) > 0);
  const hunger = need(n.satiety);
  if (bowls.length) add('eat', hunger ** 1.3 * 2.1, { item: nearestBy(cat, bowls, interactionPoint), secs: 5 }, 'Acıkmış');
  else if (hunger > 0.55) add('beg', hunger * 1.2, { target: cursor?.active ? { x: cursor.x, y: Math.max(cursor.y, (ROOM.wall + 1) * TILE) } : null, secs: 6 }, 'Acıkmış ama kap boş');
  const water = byUse('water').filter((b) => (b.state?.water ?? 0) > 0);
  if (water.length) add('drink', hunger * 0.6 + 0.08, { item: nearestBy(cat, water, interactionPoint), secs: 4 }, 'Susamış');

  // Oyun: hareket eden oyuncak çok çekicidir
  const toys = items.filter((it) => ITEMS[it.kind]?.cat === 'toy');
  const moving = (ctx.toys ?? []).filter((t) => Math.hypot(t.vx, t.vy) > 8);
  const playDrive = (n.energy / 100) * (0.4 + need(n.happiness)) * (night ? 0.5 : 1);
  if (moving.length) add('play', playDrive * 3 + 0.4, { item: nearestBy(cat, moving.map((t) => t.item)), chase: true, secs: 6 + random() * 4 }, 'Hareket eden oyuncağı gördü');
  else if (toys.length) {
    const toy = toys.find((t) => t.kind === favToy) ?? toys[Math.floor(random() * toys.length)];
    add('play', playDrive, { item: toy, secs: 5 + random() * 5 }, favToy === toy.kind ? 'En sevdiği oyuncak' : 'Oyun havasında');
  }

  // Bakım ve eşyalar
  add('groom', need(n.cleanliness) * 1.3 + 0.04, { secs: 4 + random() * 3 }, 'Tüylerini temizliyor');
  const scratch = byUse('scratch');
  if (scratch.length) add('scratch', 0.12 + (n.energy / 100) * 0.1, { item: nearestBy(cat, scratch, interactionPoint), secs: 3 + random() * 2 }, 'Pençelerini bilemek istiyor');
  const trees = [...byUse('climb'), ...byUse('perch')];
  const cur = need(n.curiosity);
  if (trees.length) add('climb', cur * 0.8 + 0.05, { item: trees[Math.floor(random() * trees.length)], secs: 12 + random() * 20 }, 'Yükseklerden bakmak istiyor');
  const windows = byUse('window');
  if (windows.length)
    add('observe', cur * 0.7 * (weather === 'rain' || weather === 'snow' ? 1.4 : 1) * (night ? 0.6 : 1) + 0.04, { item: windows[0], secs: 8 + random() * 10 }, weather === 'rain' ? 'Yağmuru izliyor' : weather === 'snow' ? 'Karı izliyor' : 'Dışarıyı merak ediyor');
  add('explore', cur * 0.6 + 0.05, { target: randomFloor(random), secs: 2 + random() * 2 }, 'Merak ediyor');

  // Sosyal: diğer kediler ve kullanıcı
  const others = cats.filter((c) => c.id !== cat.id && !c.carried);
  if (others.length) {
    const other = others.reduce((best, c) => ((ctx.affinity?.(cat.id, c.id) ?? 0) > (ctx.affinity?.(cat.id, best.id) ?? 0) ? c : best), others[0]);
    const aff = ctx.affinity?.(cat.id, other.id) ?? 0;
    add('socialize', need(n.social) * 0.9 * (1 + Math.max(-0.5, aff / 100)), { other, secs: 6 + random() * 6 }, aff >= 30 ? 'Arkadaşını özledi' : 'Yalnız kalmak istemiyor');
  }
  if (cursor?.active) add('seek_user', need(n.social) * (n.trust / 100) * 1.2, { target: { x: cursor.x, y: Math.max(cursor.y, (ROOM.wall + 1) * TILE) }, secs: 6 }, 'İlgi istiyor');
  const hides = byUse('hide');
  const scared = (cat.fearUntil ?? 0) > now;
  add('hide', (scared ? 1.6 : 0.06) * (hides.length ? 1 : 0.6), { item: nearestBy(cat, hides, interactionPoint), secs: 6 + random() * 10 }, scared ? 'Ürktü, saklanıyor' : 'Kuytu bir köşe arıyor');

  add('rest', 0.22, { secs: 4 + random() * 7 }, 'Biraz soluklanıyor');
  add('wander', 0.16, { target: randomFloor(random), secs: 2 }, 'Dolaşıyor');

  options.sort((a, b) => b.score - a.score);
  const best = options[0];
  if (best.item && !best.target) best.target = interactionPoint(best.item);
  if (best.other && !best.target) best.target = { x: best.other.x + (best.other.x > cat.x ? -18 : 18), y: best.other.y };
  return { ...best, scores: options.slice(0, 4).map((o) => [o.type, Math.round(o.score * 100) / 100]) };
}

export function randomFloor(random) {
  const tx = 1 + Math.floor(random() * (ROOM.w - 2));
  const ty = ROOM.wall + Math.floor(random() * (ROOM.h - ROOM.wall));
  return { x: tx * TILE + TILE / 2, y: ty * TILE + TILE - 2 };
}

/** Eylemin oynatılacağı animasyon (varışta). */
export function animFor(type, item) {
  switch (type) {
    case 'sleep':
      return 'sleep';
    case 'eat':
    case 'drink':
      return 'eat';
    case 'play':
      return 'play';
    case 'groom':
      return 'groom';
    case 'scratch':
      return 'scratch';
    case 'observe':
      return 'look';
    case 'hide':
      return item ? 'crouch' : 'crouch';
    case 'climb':
      return 'sit';
    case 'beg':
    case 'seek_user':
      return 'look';
    case 'socialize':
      return 'sit';
    default:
      return 'sit';
  }
}
