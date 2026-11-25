/* eslint-disable react-hooks/refs, react-hooks/purity, react-hooks/immutability --
   Oyun döngüsü bilinçli olarak React dışında (ref) çalışır: simülasyon her karede durum
   güncellemesi yapmaz; arayüz saniyede bir (tick) ref içeriğini okuyarak yenilenir. */
// Kedi Evi dünyası: doğrudan fareyle oynanan, kalıcı küçük bir yaşam simülasyonu.
// Kediyi sev (üzerinde gezdir ya da tıkla), sürükleyip taşı, oyuncağı fırlat, mamayı kaba
// sürükle, tarakla tara; Düzenle kipinde mobilyaları taşı, döndür, envantere kaldır.
// Simülasyon yalnızca bu sayfa açık ve görünürken çalışır.
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Badge, Button, ErrorNote, Segmented } from '../../ui/components.jsx';
import { ProgressBar } from '../../ui/widget.jsx';
import { PetSprite } from '../../pets/PetSprite.jsx';
import { PERSONALITIES, mood } from '../../pets/labels.js';
import { call, useTauriEvent } from '../../lib/ipc.js';
import { useActiveTheme } from '../../theme/context.js';
import { CONTRACT } from '../../theme/schema.js';
import { CATEGORIES, ITEMS, ROOM, ROOMS, ROOM_PX, TILE, footprint } from '../../pets/world/catalog.js';
import { canPlace, tileOf } from '../../pets/world/world.js';
import { rng } from '../../pets/home/sim.js';
import { ACTION_LABELS } from '../../pets/world/ai.js';
import { createWorldCat, emote, gameHour, periodOf, PERIOD_NAMES, rebuildGrid, setAnim, settle, stepWorld, toyEntity, weatherAt, WEATHER_NAMES } from '../../pets/world/sim.js';
import { DOORS, catAt, clearRenderCache, doorRect, drawWorld } from '../../pets/world/render.js';
import { itemAt } from '../../pets/world/world.js';
import { motionMode } from '../../lib/motion.js';

const QUIRKS = { none: null, mischievous: 'Yaramaz', shy: 'Utangaç', lazy: 'Tembel' };
const NEED_LABELS = [
  ['satiety', 'Tokluk'],
  ['energy', 'Enerji'],
  ['happiness', 'Mutluluk'],
  ['rest', 'Uyku'],
  ['social', 'Sosyallik'],
  ['trust', 'Güven'],
  ['curiosity', 'Merak'],
  ['cleanliness', 'Temizlik'],
];
const EVENT_KIND = { eat: 'eat', drink: 'drink', play: 'play', groom: 'groom', scratch: 'scratch', climb: 'climb', observe: 'observe', explore: 'explore', socialize: 'socialize', hide: 'hide', sleep: 'sleep' };
const levelOf = (xp) => 1 + Math.floor(Math.sqrt(Math.max(0, xp) / 40));
const PARTICLE_COLORS = { dust: '#fff6d8', leaves: '#d98b3a', stars: '#fff8b0', sparkles: '#f0c6ff' };

function topKey(map) {
  if (!map || typeof map !== 'object') return null;
  return Object.entries(map).sort((a, b) => b[1] - a[1])[0]?.[0] ?? null;
}

export function PetWorldSection({ settingsApi, onNavigate }) {
  const settings = settingsApi.settings;
  const theme = useActiveTheme();
  const canvasRef = useRef(null);
  const wrapRef = useRef(null);
  const [data, setData] = useState(null);
  const [error, setError] = useState(null);
  const [room, setRoom] = useState('living');
  const [edit, setEdit] = useState(false);
  const [category, setCategory] = useState('furniture');
  const [toast, setToast] = useState(null);
  const [focusId, setFocusId] = useState(null);
  const [hover, setHover] = useState(null); // { type:'cat'|'item', id }
  const [foods, setFoods] = useState([]);
  const [tick, setTick] = useState(0); // HUD yenileme (saniyede bir)
  const [view, setView] = useState({ scale: 3, zoom: 1, ox: 0, oy: 0 });

  // Simülasyon durumu React dışında (her karede render yok).
  const sim = useRef({ items: [], cats: new Map(), toys: new Map(), grid: null, random: rng(0x1f15), pets: new Map(), ptr: null, cursor: { x: 0, y: 0, active: false }, particles: [], selected: null, t: 0 });
  const live = useRef({});
  // Geliştirme derlemesinde uçtan uca testler için simülasyona erişim.
  if (import.meta.env.DEV) window.__ivilyWorld = { sim: sim.current, canvas: () => canvasRef.current, view: () => viewRef.current };

  const motion = motionMode(settings);
  const palette = theme?.pets?.palette ?? 'none';
  const home = theme?.home ?? { floor: 'planks', wall: 'stripes', colors: {} };
  const colors = useMemo(() => ({ floor: '#d9a066', 'floor-alt': '#c88a52', wall: '#f7c6d4', 'wall-alt': '#f4b6c7', trim: '#8a5a44', line: '#3b2a30', ...home.colors }), [home.colors]);
  const roomStyle = data?.rooms.find((r) => r.id === room);
  const style = { floor: roomStyle?.floor ?? home.floor, wall: roomStyle?.wall ?? home.wall };

  const showToast = useCallback((text) => {
    setToast({ text, at: Date.now() });
  }, []);
  useEffect(() => {
    if (!toast) return undefined;
    const t = setTimeout(() => setToast(null), 4200);
    return () => clearTimeout(t);
  }, [toast]);

  const loadFoods = useCallback(() => {
    call('pet_catalog').then((c) => setFoods(c.foods), () => {});
  }, []);

  const load = useCallback(() => {
    call('get_world').then(
      (d) => {
        setData(d);
        if (d.away?.notes?.length) showToast(d.away.notes.join(' '));
      },
      (e) => setError(e.message),
    );
    loadFoods();
  }, [showToast, loadFoods]);
  useEffect(load, [load]);
  useTauriEvent('pets:changed', () => {
    call('list_pets').then((pets) => setData((d) => (d ? { ...d, pets } : d)), () => {});
  });

  // Oda değişince ya da veri gelince simülasyonu kur.
  useEffect(() => {
    if (!data) return;
    const s = sim.current;
    s.pets = new Map(data.pets.map((p) => [p.id, p]));
    s.items = data.items.filter((i) => i.room === room);
    rebuildGrid(s);
    const homePets = data.pets.filter((p) => p.location === 'home' && (p.room ?? 'living') === room);
    const next = new Map();
    for (const p of homePets) {
      const existing = s.cats.get(p.id);
      const cat = existing ?? createWorldCat(p, s.random);
      settle(cat, s.grid);
      next.set(p.id, cat);
    }
    s.cats = next;
    const toys = new Map();
    for (const it of s.items) if (ITEMS[it.kind]?.cat === 'toy') toys.set(it.id, s.toys.get(it.id) ?? toyEntity(it));
    s.toys = toys;
  }, [data, room]);

  useEffect(() => {
    live.current = { settings, colors, style, palette, room, edit, motion, theme };
  });
  useEffect(() => clearRenderCache, [palette, colors]);

  // Konumları kaydet: aralıklarla ve sayfadan çıkarken (her karede değil).
  const savePositions = useCallback(() => {
    const s = sim.current;
    const positions = [...s.cats.values()].filter((c) => !c.carried).map((c) => ({ id: c.id, room: live.current.room, x: c.x, y: c.y }));
    if (positions.length) call('save_world_positions', { positions }).catch(() => {});
  }, []);
  useEffect(() => {
    const t = setInterval(savePositions, 30000);
    return () => {
      clearInterval(t);
      savePositions();
    };
  }, [savePositions]);

  // Ölçek: kabın genişliğine göre tam sayı (keskin pikseller).
  useEffect(() => {
    const el = wrapRef.current;
    if (!el) return undefined;
    const ro = new ResizeObserver(([e]) => {
      const scale = Math.max(2, Math.ceil(e.contentRect.width / ROOM_PX.w));
      setView((v) => ({ ...v, scale }));
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const onEvent = useCallback(
    (cat, action) => {
      const kind = EVENT_KIND[action.type];
      if (!kind) return;
      const input = { pet_id: cat.id, kind, item_id: action.item?.id ?? null, other_id: action.other?.id ?? null };
      if (kind === 'sleep' && action.item) input.spot = `${live.current.room}:${action.item.tx}:${action.item.ty}`;
      call('world_event', { input }).then(
        (r) => {
          sim.current.pets.set(r.pet.id, r.pet);
          // Kap yalnızca sunucu tükettiyse azalır (istemci ve veritabanı eş kalır).
          if (r.rewarded && (kind === 'eat' || kind === 'drink') && action.item) {
            const key = kind === 'eat' ? 'portions' : 'water';
            const it = sim.current.items.find((i) => i.id === action.item.id);
            if (it) it.state = { ...it.state, [key]: Math.max(0, (it.state?.[key] ?? 0) - 1) };
          }
          if (r.memory) showToast(r.memory);
          if (r.unlocked?.length) showToast(`Yeni eşya açıldı: ${r.unlocked.map((k) => ITEMS[k]?.name ?? k).join(', ')}. Envantere eklendi.`);
          if (r.unlocked?.length || r.memory) load();
        },
        () => {},
      );
    },
    [showToast, load],
  );

  // Ana döngü: görünürken en fazla 30 kare/sn; hareket azaltıldığında 12, kapalıyken 6.
  useEffect(() => {
    let raf = 0;
    let last = performance.now();
    let acc = 0;
    let hud = 0;
    const loop = (t) => {
      raf = requestAnimationFrame(loop);
      if (document.hidden) {
        last = t;
        return;
      }
      const L = live.current;
      const fps = L.motion === 'full' ? 30 : L.motion === 'reduced' ? 12 : 6;
      acc += t - last;
      last = t;
      if (acc < 1000 / fps) return;
      const dt = Math.min(0.1, acc / 1000);
      acc = 0;
      const s = sim.current;
      const canvas = canvasRef.current;
      if (!canvas || !s.grid) return;
      const now = Date.now();
      const hour = gameHour(now, L.settings?.world_time);
      const period = periodOf(hour);
      const weather = L.settings?.world_weather === false ? 'clear' : weatherAt(now);
      s.t += dt;
      stepWorld(s, dt, {
        pets: s.pets,
        now,
        period,
        weather,
        cursor: s.cursor,
        motion: L.motion,
        affinity: (a, b) => {
          const r = live.current.relations?.find((x) => x.a === Math.min(a, b) && x.b === Math.max(a, b));
          return r?.affinity ?? 0;
        },
        onEvent,
        onSleep: (cat, sleeping) => call('set_world_activity', { petId: cat.id, sleeping }).catch(() => {}),
        onToyRest: (toy) => {
          const { tx, ty } = tileOf(toy.x, toy.y);
          if (tx === toy.item.tx && ty === toy.item.ty) return;
          call('move_world_item', { id: toy.item.id, room: L.room, tx, ty, rot: 0 }).then(
            (it) => Object.assign(toy.item, it),
            () => {},
          );
        },
      });
      stepParticles(s, dt, L);
      const v = viewRef.current;
      drawWorld(canvas.getContext('2d'), {
        room: L.room,
        style: L.style,
        colors: L.colors,
        palette: L.palette,
        items: s.items,
        cats: s.cats,
        pets: s.pets,
        toys: s.toys,
        hour,
        period,
        weather,
        view: { scale: v.scale * v.zoom, ox: v.ox, oy: v.oy },
        ghost: s.ghost,
        hover: hoverRef.current?.type === 'item' ? { id: hoverRef.current.id } : s.selected ? { id: s.selected } : null,
        hoverCat: hoverRef.current?.type === 'cat' ? hoverRef.current.id : null,
        particles: s.particles,
        motion: L.motion,
        t: s.t,
      });
      hud += dt;
      if (hud > 1) {
        hud = 0;
        setTick((x) => x + 1);
      }
    };
    raf = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(raf);
  }, [onEvent]);

  const viewRef = useRef(view);
  const hoverRef = useRef(hover);
  useEffect(() => {
    viewRef.current = view;
    hoverRef.current = hover;
    live.current.relations = data?.relations;
  });

  // ── Girdi ────────────────────────────────────────────────────────────────
  const toWorld = (e) => {
    const c = canvasRef.current;
    const r = c.getBoundingClientRect();
    const v = viewRef.current;
    // Görüntülenen boyut tuval boyutundan küçük olabilir (dar pencere): gerçek ölçeği kullan.
    const k = (r.width / c.width) * v.scale * v.zoom;
    return { x: (e.clientX - r.left) / k + v.ox, y: (e.clientY - r.top) / k + v.oy };
  };
  const doorAt = (p) => (DOORS[room] ?? []).find((d) => {
    const r = doorRect(d.side);
    return p.x >= r.x && p.x <= r.x + r.w && p.y >= r.y && p.y <= r.y + r.h + 10;
  });

  const placeFromInventory = async (kind, p) => {
    const def = ITEMS[kind];
    const { w, h } = footprint(def, 0);
    const tx = Math.round(p.x / TILE - w / 2);
    const ty = Math.round(p.y / TILE - h);
    try {
      await call('place_world_item', { kind, room, tx, ty, rot: 0 });
      load();
    } catch (e) {
      showToast(e.message);
    }
  };

  const ghostFor = (def, p, grab, rot, ignore) => {
    const { w, h } = footprint(def, rot);
    const tx = Math.round(p.x / TILE - (grab?.dx ?? w / 2));
    const ty = Math.round(p.y / TILE - (grab?.dy ?? h));
    return { def, tx, ty, rot, ok: canPlace(sim.current.items, def, tx, ty, rot, ignore) };
  };

  const onPointerDown = (e) => {
    if (e.button === 1 || (e.button === 0 && e.altKey)) {
      sim.current.ptr = { mode: 'pan', sx: e.clientX, sy: e.clientY, ox: view.ox, oy: view.oy };
      return;
    }
    if (e.button !== 0) return;
    canvasRef.current.setPointerCapture(e.pointerId);
    const p = toWorld(e);
    const s = sim.current;
    const cat = catAt(s.cats, p.x, p.y);
    if (cat && !edit) {
      s.ptr = { mode: 'catPress', cat, sx: p.x, sy: p.y };
      setFocusId(cat.id);
      return;
    }
    const it = itemAt(s.items, p.x, p.y);
    if (it) {
      const def = ITEMS[it.kind];
      const toy = s.toys.get(it.id);
      if (edit) {
        const { w, h } = footprint(def, it.rot);
        s.selected = it.id;
        s.ptr = { mode: 'itemDrag', it, grab: { dx: p.x / TILE - it.tx, dy: p.y / TILE - it.ty }, moved: false };
        s.ghost = { def, tx: it.tx, ty: it.ty, rot: it.rot, ok: true, w, h };
        return;
      }
      if (toy) {
        toy.held = true;
        s.ptr = { mode: 'toyDrag', toy, samples: [{ x: p.x, y: p.y, t: performance.now() }] };
        return;
      }
      if (def.use === 'food' || def.use === 'water') {
        s.ptr = { mode: 'bowlPress', it, sx: p.x, sy: p.y };
        return;
      }
    }
    const door = doorAt(p);
    if (door && !edit) {
      savePositions();
      setRoom(door.to);
      return;
    }
    if (view.zoom > 1) s.ptr = { mode: 'pan', sx: e.clientX, sy: e.clientY, ox: view.ox, oy: view.oy };
  };

  const onPointerMove = (e) => {
    const s = sim.current;
    const p = toWorld(e);
    s.cursor = { x: p.x, y: p.y, active: true };
    const ptr = s.ptr;
    if (!ptr) {
      const cat = catAt(s.cats, p.x, p.y);
      if (cat) {
        // Sevme: imleci kedinin üzerinde gezdirmek.
        const prev = s.rub;
        if (prev?.id === cat.id) {
          cat.rubbed += Math.hypot(p.x - prev.x, p.y - prev.y);
          if (cat.rubbed > 90) {
            cat.rubbed = 0;
            petCat(cat);
          }
        }
        s.rub = { id: cat.id, x: p.x, y: p.y };
        if (hoverRef.current?.id !== cat.id) setHover({ type: 'cat', id: cat.id });
      } else {
        s.rub = null;
        const it = itemAt(s.items, p.x, p.y);
        const h = it ? { type: 'item', id: it.id } : null;
        if (h?.id !== hoverRef.current?.id || h?.type !== hoverRef.current?.type) setHover(h);
      }
      canvasRef.current.style.cursor = catAt(s.cats, p.x, p.y) ? 'grab' : itemAt(s.items, p.x, p.y) ? 'pointer' : doorAt(p) ? 'pointer' : 'default';
      return;
    }
    if (ptr.mode === 'catPress' && Math.hypot(p.x - ptr.sx, p.y - ptr.sy) > 5) {
      const cat = ptr.cat;
      cat.carried = true;
      cat.action = null;
      cat.path = [];
      if (cat.sleeping) {
        cat.sleeping = false;
        call('set_world_activity', { petId: cat.id, sleeping: false }).catch(() => {});
      }
      call('world_event', { input: { pet_id: cat.id, kind: 'carried' } }).catch(() => {});
      ptr.mode = 'carry';
    }
    if (ptr.mode === 'carry') {
      ptr.cat.x = Math.max(8, Math.min(ROOM_PX.w - 8, p.x));
      ptr.cat.y = Math.max(ROOM.wall * TILE, Math.min(ROOM_PX.h - 1, p.y + 10));
      ptr.cat.z = 8;
      canvasRef.current.style.cursor = 'grabbing';
    } else if (ptr.mode === 'toyDrag') {
      ptr.toy.x = Math.max(6, Math.min(ROOM_PX.w - 6, p.x));
      ptr.toy.y = Math.max((ROOM.wall + 0.6) * TILE, Math.min(ROOM_PX.h - 2, p.y));
      ptr.samples.push({ x: p.x, y: p.y, t: performance.now() });
      if (ptr.samples.length > 6) ptr.samples.shift();
    } else if (ptr.mode === 'itemDrag') {
      ptr.moved = true;
      s.ghost = ghostFor(ITEMS[ptr.it.kind], p, ptr.grab, ptr.it.rot, ptr.it.id);
    } else if (ptr.mode === 'pan') {
      const v = viewRef.current;
      const k = v.scale * v.zoom;
      const maxX = ROOM_PX.w - ROOM_PX.w / v.zoom;
      const maxY = ROOM_PX.h - ROOM_PX.h / v.zoom;
      setView({ ...v, ox: Math.max(0, Math.min(maxX, ptr.ox - (e.clientX - ptr.sx) / k)), oy: Math.max(0, Math.min(maxY, ptr.oy - (e.clientY - ptr.sy) / k)) });
    }
  };

  const petCat = (cat) => {
    const nowS = Date.now() / 1000;
    if (!cat.sleeping) {
      setAnim(cat, 'petted', nowS);
      if (cat.action?.phase === 'doing') cat.action.until = Math.max(cat.action.until, Date.now() + 1500);
    }
    emote(cat, 'heart', Date.now());
    playSound('pet');
    call('world_event', { input: { pet_id: cat.id, kind: 'pet' } }).then((r) => sim.current.pets.set(r.pet.id, r.pet), () => {});
  };

  const playSound = (name) => {
    if (!live.current.settings?.ui_sounds) return;
    import('../../theme/engine.js').then((m) => m.playThemeSound(live.current.theme, name, true));
  };

  const onPointerUp = async (e) => {
    const s = sim.current;
    const ptr = s.ptr;
    s.ptr = null;
    if (!ptr) return;
    const p = toWorld(e);
    if (ptr.mode === 'catPress') petCat(ptr.cat);
    else if (ptr.mode === 'carry') {
      const cat = ptr.cat;
      cat.carried = false;
      const door = doorAt(p);
      if (door) {
        // Kediyi kapıdan diğer odaya taşı.
        const x = door.side === 'left' ? ROOM_PX.w - 30 : 30;
        await call('save_world_positions', { positions: [{ id: cat.id, room: door.to, x, y: (ROOM.wall + 2) * TILE }] }).catch(() => {});
        s.cats.delete(cat.id);
        showToast(`${s.pets.get(cat.id)?.name ?? 'Kedi'} ${ROOMS.find((r) => r.id === door.to)?.name.toLowerCase()} odasına geçti.`);
        load();
        return;
      }
      settle(cat, s.grid);
      cat.z = 10;
      cat.action = null;
      const pet = s.pets.get(cat.id);
      if (pet?.quirk === 'shy' || pet?.personality === 'independent') cat.fearUntil = Date.now() + 6000;
      playSound('drop');
    } else if (ptr.mode === 'toyDrag') {
      const t = ptr.toy;
      t.held = false;
      const a = ptr.samples[0];
      const b = ptr.samples[ptr.samples.length - 1];
      const dts = Math.max(0.016, (b.t - a.t) / 1000);
      t.vx = Math.max(-260, Math.min(260, (b.x - a.x) / dts));
      t.vy = Math.max(-200, Math.min(200, (b.y - a.y) / dts));
      t.moving = true;
      if (Math.hypot(t.vx, t.vy) < 10) t.vx = t.vy = 0.1;
    } else if (ptr.mode === 'itemDrag') {
      const g = s.ghost;
      s.ghost = null;
      if (ptr.moved && g?.ok && (g.tx !== ptr.it.tx || g.ty !== ptr.it.ty)) {
        try {
          const it = await call('move_world_item', { id: ptr.it.id, room, tx: g.tx, ty: g.ty, rot: ptr.it.rot });
          Object.assign(ptr.it, it);
          rebuildGrid(s);
          for (const c of s.cats.values()) settle(c, s.grid);
          playSound('drop');
        } catch (err) {
          showToast(err.message);
        }
      } else if (ptr.moved && !g?.ok) showToast('Eşya oraya sığmıyor; eski yerinde kaldı.');
    } else if (ptr.mode === 'bowlPress' && Math.hypot(p.x - ptr.sx, p.y - ptr.sy) < 5) {
      await fillBowl(ptr.it, ptr.it.kind === 'bowl' ? (foods.find((f) => f.count > 0)?.id ?? 'kibble') : null);
    }
  };

  const fillBowl = async (it, food) => {
    try {
      const updated = await call('fill_world_bowl', { id: it.id, food });
      const local = sim.current.items.find((i) => i.id === it.id);
      if (local) local.state = updated.state;
      loadFoods();
      playSound('success');
      showToast(it.kind === 'bowl' ? `Mama kabı dolduruldu (${foods.find((f) => f.id === updated.state.food)?.name ?? 'mama'}).` : 'Su kabı tazelendi.');
    } catch (err) {
      showToast(err.message);
    }
  };

  // Envanter ve araçlardan sürükleyip bırakma (dünyaya bırakılır).
  const startToolDrag = (e, tool) => {
    e.preventDefault();
    const s = sim.current;
    const move = (ev) => {
      const r = canvasRef.current.getBoundingClientRect();
      const inside = ev.clientX >= r.left && ev.clientX <= r.right && ev.clientY >= r.top && ev.clientY <= r.bottom;
      if (tool.type === 'item' && inside) s.ghost = ghostFor(ITEMS[tool.kind], toWorld(ev), null, 0, null);
      else s.ghost = null;
      s.dragTool = { ...tool, x: ev.clientX, y: ev.clientY };
      setDragTool(s.dragTool);
    };
    const up = async (ev) => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
      s.ghost = null;
      setDragTool(null);
      const r = canvasRef.current.getBoundingClientRect();
      if (ev.clientX < r.left || ev.clientX > r.right || ev.clientY < r.top || ev.clientY > r.bottom) return;
      const p = toWorld(ev);
      if (tool.type === 'item') return placeFromInventory(tool.kind, p);
      const cat = catAt(s.cats, p.x, p.y);
      const it = itemAt(s.items, p.x, p.y);
      if (tool.type === 'food') {
        if (it?.kind === 'bowl') return fillBowl(it, tool.id);
        if (cat) {
          try {
            const pet = await call('feed_pet', { id: cat.id, food: tool.id });
            s.pets.set(pet.id, pet);
            setAnim(cat, 'eat', Date.now() / 1000);
            if (cat.action) cat.action.until = Date.now() + 3000;
            emote(cat, 'heart', Date.now());
            loadFoods();
          } catch (err) {
            showToast(err.message);
          }
          return;
        }
        showToast('Yiyeceği bir kediye ya da mama kabına bırakın.');
      } else if (tool.type === 'brush') {
        if (!cat) return showToast('Tarağı bir kedinin üzerine bırakın.');
        setAnim(cat, 'petted', Date.now() / 1000);
        emote(cat, 'star', Date.now());
        call('world_event', { input: { pet_id: cat.id, kind: 'brush' } }).then((r) => s.pets.set(r.pet.id, r.pet), () => {});
      } else if (tool.type === 'invite') {
        try {
          await call('set_pet_location', { id: tool.id, location: 'home' });
          await call('save_world_positions', { positions: [{ id: tool.id, room, x: p.x, y: Math.max(p.y, (ROOM.wall + 1) * TILE) }] });
          load();
        } catch (err) {
          showToast(err.message);
        }
      }
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  };
  const [dragTool, setDragTool] = useState(null);

  // Klavye
  useEffect(() => {
    const onKey = async (e) => {
      if (e.target.closest?.('input, textarea, select')) return;
      const s = sim.current;
      if (e.key === 'e' || e.key === 'E') setEdit((x) => !x);
      else if (['1', '2', '3'].includes(e.key) && !e.ctrlKey) {
        savePositions();
        setRoom(ROOMS[Number(e.key) - 1].id);
      } else if (e.key === 'Escape') {
        s.selected = null;
        s.ghost = null;
        setFocusId(null);
      } else if ((e.key === 'r' || e.key === 'R') && edit && s.selected) {
        const it = s.items.find((i) => i.id === s.selected);
        if (!it) return;
        const def = ITEMS[it.kind];
        if (def.wall) return showToast('Duvar eşyaları döndürülemez.');
        const rot = (it.rot + 1) % 4;
        if (!canPlace(s.items, def, it.tx, it.ty, rot, it.id)) return showToast('Döndürmek için yer yok.');
        try {
          Object.assign(it, await call('move_world_item', { id: it.id, room, tx: it.tx, ty: it.ty, rot }));
          rebuildGrid(s);
        } catch (err) {
          showToast(err.message);
        }
      } else if ((e.key === 'Delete' || e.key === 'Backspace') && edit && s.selected) {
        try {
          await call('remove_world_item', { id: s.selected });
          s.selected = null;
          load();
        } catch (err) {
          showToast(err.message);
        }
      } else if (e.key === '+' || e.key === '-') setView((v) => ({ ...v, zoom: e.key === '+' ? 2 : 1, ox: e.key === '+' ? v.ox : 0, oy: e.key === '+' ? v.oy : 0 }));
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [edit, room, load, savePositions, showToast]);

  const onWheel = (e) => {
    const zoom = e.deltaY < 0 ? 2 : 1;
    if (zoom === view.zoom) return;
    const p = toWorld(e);
    setView((v) => ({ ...v, zoom, ox: zoom === 1 ? 0 : Math.max(0, Math.min(ROOM_PX.w / 2, p.x - ROOM_PX.w / 4)), oy: zoom === 1 ? 0 : Math.max(0, Math.min(ROOM_PX.h / 2, p.y - ROOM_PX.h / 4)) }));
  };

  if (error) return <ErrorNote error={error} />;
  if (!data || !settings) return <p className="cc-muted">Kedi Evi hazırlanıyor…</p>;

  void tick;
  const s = sim.current;
  const now = Date.now();
  const hour = gameHour(now, settings.world_time);
  const weather = settings.world_weather === false ? 'clear' : weatherAt(now);
  const homeCats = data.pets.filter((p) => p.location === 'home');
  const desktopCats = data.pets.filter((p) => p.location !== 'home');
  const focusPet = s.pets.get(focusId ?? (hover?.type === 'cat' ? hover.id : null));
  const focusCat = focusPet ? s.cats.get(focusPet.id) : null;
  const hoverItem = hover?.type === 'item' ? s.items.find((i) => i.id === hover.id) : null;
  const inv = Object.entries(data.inventory).filter(([k]) => ITEMS[k]?.cat === category);
  const hh = Math.floor(hour);
  const mm = Math.floor((hour % 1) * 60);
  const k = view.scale * view.zoom;

  return (
    <div className="pw">
      <div className="pw-hud">
        <div className="cc-tabs pw-rooms" role="tablist" aria-label="Odalar">
          {ROOMS.map((r, i) => (
            <button
              key={r.id}
              type="button"
              role="tab"
              className="cc-tabs__tab"
              aria-selected={room === r.id}
              title={`${r.name} (${i + 1})`}
              onClick={() => {
                savePositions();
                setRoom(r.id);
              }}
            >
              {r.name}
              <span className="pw-count">{homeCats.filter((p) => (p.room ?? 'living') === r.id).length || ''}</span>
            </button>
          ))}
        </div>
        <span className="pw-clock" title="Oyun saati ve hava">
          {String(hh).padStart(2, '0')}:{String(mm).padStart(2, '0')} · {PERIOD_NAMES[periodOf(hour)]} · {WEATHER_NAMES[weather]}
        </span>
        <span className="pw-level" title={`${data.xp} / ${data.next_level_xp} deneyim`}>
          Ev seviyesi {data.level}
          <ProgressBar value={Math.min(1, data.xp / Math.max(1, data.next_level_xp))} label="Ev seviyesi" />
        </span>
        <Button size="sm" variant={edit ? 'primary' : 'secondary'} onClick={() => setEdit((x) => !x)} aria-pressed={edit} title="Düzenle (E)">
          {edit ? 'Düzenlemeyi bitir' : 'Düzenle'}
        </Button>
      </div>

      <div className="pw-main">
        <div className="pw-stage" ref={wrapRef}>
          <canvas
            ref={canvasRef}
            className="pw-canvas"
            width={ROOM_PX.w * view.scale}
            height={ROOM_PX.h * view.scale}
            onPointerDown={onPointerDown}
            onPointerMove={onPointerMove}
            onPointerUp={onPointerUp}
            onPointerLeave={() => {
              sim.current.cursor = { ...sim.current.cursor, active: false };
              setHover(null);
            }}
            onWheel={onWheel}
            role="application"
            aria-label={`Kedi Evi, ${ROOMS.find((r) => r.id === room)?.name}. Kedileri sevmek için imleci üzerlerinde gezdirin, taşımak için sürükleyin.`}
            tabIndex={0}
          />
          {hoverItem && !edit ? (
            <div className="pw-tip" style={{ left: Math.min((hoverItem.tx + 1) * TILE * k - view.ox * k, ROOM_PX.w * view.scale - 160), top: Math.max(0, hoverItem.ty * TILE * k - view.oy * k - 34) }}>
              {itemTip(hoverItem)}
            </div>
          ) : null}
          {toast ? (
            <div className="pw-toast" role="status" key={toast.at}>
              {toast.text}
            </div>
          ) : null}
          {!homeCats.length ? (
            <div className="pw-empty">
              <p>Kedi Evi boş. Masaüstündeki bir kediyi aşağıdan odaya sürükleyin{desktopCats.length ? '' : ' ya da önce bir kedi sahiplenin'}.</p>
              {!desktopCats.length ? (
                <Button size="sm" variant="primary" onClick={() => onNavigate('pethome', 'cats')}>
                  Kedi sahiplen
                </Button>
              ) : null}
            </div>
          ) : null}
        </div>

        <aside className="pw-side" aria-live="polite">
          {focusPet ? (
            <div className="pw-cat">
              <div className="pw-cat__head">
                <PetSprite breed={focusPet.breed} anim={focusCat?.anim === 'sleep' ? 'sleep' : 'sit'} scale={2} accessories={[focusPet.acc_head, focusPet.acc_neck].filter(Boolean)} />
                <div>
                  <p className="pw-cat__name">{focusPet.name}</p>
                  <p className="cc-muted">
                    {PERSONALITIES[focusPet.personality]?.name}
                    {QUIRKS[focusPet.quirk] ? ` · ${QUIRKS[focusPet.quirk]}` : ''} · Seviye {levelOf(focusPet.xp)}
                  </p>
                </div>
              </div>
              <p className="pw-cat__doing">
                <strong>{mood(focusPet.needs)}</strong>
                {focusCat?.action ? ` — ${ACTION_LABELS[focusCat.action.type] ?? ''}` : ''}
                {focusCat?.reason ? <span className="cc-muted"> ({focusCat.reason})</span> : null}
              </p>
              <dl className="pw-needs">
                {NEED_LABELS.map(([key, label]) => (
                  <div key={key}>
                    <dt>{label}</dt>
                    <dd>
                      <ProgressBar value={(focusPet.needs[key] ?? 0) / 100} label={label} />
                    </dd>
                  </div>
                ))}
              </dl>
              <Habits pet={focusPet} />
            </div>
          ) : (
            <div className="pw-help">
              <p className="pw-help__title">Nasıl oynanır?</p>
              <ul>
                <li>İmleci bir kedinin üzerinde gezdir: sevilir.</li>
                <li>Kediyi sürükle: kucağına alıp taşı; kapıya bırakırsan diğer odaya geçer.</li>
                <li>Oyuncağı sürükleyip bırak: fırlatılır, kediler kovalar.</li>
                <li>Mama kabına tıkla ya da yiyeceği kaba sürükle.</li>
                <li>Tarağı bir kediye bırak: taranır.</li>
                <li>
                  <kbd>E</kbd> Düzenle · <kbd>R</kbd> döndür · <kbd>Delete</kbd> kaldır · <kbd>1–3</kbd> odalar · tekerlek: yakınlaştır
                </li>
              </ul>
            </div>
          )}
          {edit ? <RoomStyle room={room} style={roomStyle} onChange={load} /> : null}
        </aside>
      </div>

      <div className="pw-drawer">
        <div className="pw-tools" aria-label="Araçlar">
          {foods.map((f) => (
            <button key={f.id} type="button" className="pw-tool" disabled={!f.count} onPointerDown={(e) => f.count && startToolDrag(e, { type: 'food', id: f.id, label: f.name })} title={`${f.name}: bugün ${f.count} kaldı. Kediye ya da mama kabına sürükleyin.`}>
              <span className={`pw-food pw-food--${f.id}`} aria-hidden="true" />
              {f.name} <Badge tone="gray">{f.count}</Badge>
            </button>
          ))}
          <button type="button" className="pw-tool" onPointerDown={(e) => startToolDrag(e, { type: 'brush', label: 'Tarak' })} title="Bir kedinin üzerine sürükleyin">
            <span className="pw-food pw-food--brush" aria-hidden="true" /> Tarak
          </button>
          {desktopCats.map((p) => (
            <button key={p.id} type="button" className="pw-tool pw-tool--cat" onPointerDown={(e) => startToolDrag(e, { type: 'invite', id: p.id, label: p.name })} title={`${p.name} masaüstünde. Kedi Evi'ne getirmek için odaya sürükleyin.`}>
              <PetSprite breed={p.breed} anim="sit" scale={1} /> {p.name}
            </button>
          ))}
        </div>
        <div className="pw-inv">
          <Segmented label="Envanter" value={category} onChange={setCategory} options={CATEGORIES.map((c) => ({ value: c.id, label: c.label }))} />
          <ul className="pw-inv__list">
            {inv.length ? (
              inv.map(([kind, count]) => (
                <li key={kind}>
                  <button type="button" className="pw-inv__item" onPointerDown={(e) => startToolDrag(e, { type: 'item', kind, label: ITEMS[kind].name })} title={`${ITEMS[kind].name} — odaya sürükleyip bırakın`}>
                    <ItemIcon kind={kind} />
                    <span>{ITEMS[kind].name}</span>
                    <Badge tone="gray">{count}</Badge>
                  </button>
                </li>
              ))
            ) : (
              <li className="cc-muted">Bu kategoride envanterde eşya yok. Eşyalar seviye atladıkça açılır; odadaki bir eşyayı Düzenle kipinde seçip Delete ile envantere kaldırabilirsiniz.</li>
            )}
          </ul>
        </div>
      </div>
      {dragTool ? (
        <div className="pw-dragging" style={{ left: dragTool.x + 12, top: dragTool.y + 12 }} aria-hidden="true">
          {dragTool.label}
        </div>
      ) : null}
    </div>
  );
}

function itemTip(it) {
  const d = ITEMS[it.kind];
  if (d.use === 'food') {
    const n = it.state?.portions ?? 0;
    return n ? `Mama kabı: ${n} porsiyon` : 'Mama kabı boş — doldurmak için tıklayın';
  }
  if (d.use === 'water') return (it.state?.water ?? 0) ? `Su kabı: ${it.state.water} yudum` : 'Su kabı boş — tazelemek için tıklayın';
  if (d.cat === 'toy') return `${d.name} — sürükleyip fırlatın`;
  return d.name;
}

function Habits({ pet }) {
  const h = pet.habits ?? {};
  const toy = topKey(h.toys);
  const furn = topKey(h.furniture);
  const per = topKey(h.periods);
  const PER = { morning: 'sabahları', afternoon: 'öğleden sonraları', evening: 'akşamları', night: 'geceleri' };
  const items = [toy && `En sevdiği oyuncak: ${ITEMS[toy]?.name ?? toy}`, furn && `Sık kullandığı eşya: ${ITEMS[furn]?.name ?? furn}`, per && `En hareketli olduğu zaman: ${PER[per]}`].filter(Boolean);
  if (!items.length) return <p className="cc-muted pw-habits">Alışkanlıkları zamanla oluşacak.</p>;
  return (
    <ul className="pw-habits">
      {items.map((x) => (
        <li key={x}>{x}</li>
      ))}
    </ul>
  );
}

function ItemIcon({ kind }) {
  const ref = useRef(null);
  const theme = useActiveTheme();
  const palette = theme?.pets?.palette ?? 'none';
  useEffect(() => {
    const g = ITEMS[kind]?.grid;
    const c = ref.current;
    if (!g || !c) return;
    const s = Math.max(1, Math.floor(28 / Math.max(g.length, g[0].length)));
    c.width = g[0].length * s;
    c.height = g.length * s;
    const ctx = c.getContext('2d');
    import('../../theme/palette.js').then(({ transformColor }) =>
      g.forEach((row, y) =>
        row.forEach((col, x) => {
          if (!col) return;
          ctx.fillStyle = transformColor(col, palette);
          ctx.fillRect(x * s, y * s, s, s);
        }),
      ),
    );
  }, [kind, palette]);
  return <canvas ref={ref} className="pw-inv__icon" aria-hidden="true" />;
}

function RoomStyle({ room, style, onChange }) {
  const set = (patch) =>
    call('set_world_room_style', { room, floor: style?.floor ?? null, wall: style?.wall ?? null, ...patch })
      .then(onChange)
      .catch(() => {});
  const FLOOR = { planks: 'Tahta', tiles: 'Fayans', checker: 'Dama', carpet: 'Halı kaplama', plain: 'Düz', grid: 'Izgara' };
  const WALL = { stripes: 'Çizgili', dots: 'Puantiyeli', plain: 'Düz', brick: 'Tuğla', panel: 'Lambri', grid: 'Izgara' };
  return (
    <div className="pw-style">
      <p className="pw-help__title">Oda görünümü</p>
      <label className="wk-field">
        <span className="wk-field__label">Zemin</span>
        <select className="wk-select" value={style?.floor ?? ''} onChange={(e) => set({ floor: e.target.value || null })}>
          <option value="">Temanın zemini</option>
          {CONTRACT.home.floor.map((f) => (
            <option key={f} value={f}>
              {FLOOR[f] ?? f}
            </option>
          ))}
        </select>
      </label>
      <label className="wk-field">
        <span className="wk-field__label">Duvar</span>
        <select className="wk-select" value={style?.wall ?? ''} onChange={(e) => set({ wall: e.target.value || null })}>
          <option value="">Temanın duvarı</option>
          {CONTRACT.home.wall.map((w) => (
            <option key={w} value={w}>
              {WALL[w] ?? w}
            </option>
          ))}
        </select>
      </label>
      <p className="cc-muted">Düzenle kipinde eşyayı sürükleyerek taşıyın; seçiliyken R döndürür, Delete envantere kaldırır. Yeşil alan uygun, kırmızı alan dolu.</p>
    </div>
  );
}

/** Temaya göre ortam parçacıkları (toz, yaprak, yıldız, parıltı). Hareket kısıtlıysa yok. */
function stepParticles(s, dt, L) {
  const kind = L.theme?.ambient;
  if (L.motion !== 'full' || L.settings?.ambient_effects === false || !kind || kind === 'none') {
    s.particles = [];
    return;
  }
  const color = PARTICLE_COLORS[kind] ?? '#ffffff';
  while (s.particles.length < 16) {
    s.particles.push({ x: Math.random() * ROOM_PX.w, y: Math.random() * ROOM_PX.h, vx: (Math.random() - 0.5) * 6, vy: kind === 'leaves' ? 8 + Math.random() * 6 : -2 - Math.random() * 3, a: 0, life: 4 + Math.random() * 6, age: 0, s: kind === 'leaves' ? 2 : 1, c: color });
  }
  for (const p of s.particles) {
    p.age += dt;
    p.x += (p.vx + (kind === 'leaves' ? Math.sin(p.age * 2) * 6 : 0)) * dt;
    p.y += p.vy * dt;
    const f = p.age / p.life;
    p.a = (kind === 'stars' || kind === 'sparkles' ? Math.abs(Math.sin(p.age * 3)) : 1) * Math.min(1, f * 4, (1 - f) * 4) * 0.7;
  }
  s.particles = s.particles.filter((p) => p.age < p.life && p.y > -4 && p.y < ROOM_PX.h + 4);
}
