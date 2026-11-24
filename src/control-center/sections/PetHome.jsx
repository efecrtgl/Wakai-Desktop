// Kedi Evi: tek bir <canvas> üzerinde piksel sanatı oda. Simülasyon yalnızca bu sayfa açıkken
// ve pencere görünürken ~8 kare/sn çalışır. Eşyalar "Düzenle" modunda sürüklenip kaydedilir.
import { useCallback, useEffect, useRef, useState } from 'react';
import { Badge, Button, Card, EmptyState, ErrorNote } from '../../ui/components.jsx';
import { PixelCat } from '../../ui/cats/PixelCat.jsx';
import { call, useTauriEvent } from '../../lib/ipc.js';
import { CANVAS_H, CANVAS_W, drawGrid, petGrid } from '../../pets/sprite.js';
import { FLOORS, FLOOR_BOTTOM, FLOOR_TOP, FURNITURE, ROOM_H, ROOM_W, WALLS } from '../../pets/home/furniture.js';
import { animOf, createCat, rng, stepCat } from '../../pets/home/sim.js';
import { PageHeader } from './PageHeader.jsx';

const TICK = 0.125;
/** Oturum başına bir kez seçilen tohum (render dışında; saf olmayan çağrı burada). */
const SEED = Date.now() % 100000;
const CAT_SCALE = 1;

function itemBox(it) {
  const f = FURNITURE[it.kind];
  const w = f.grid[0].length;
  const h = f.grid.length;
  return { x: it.x - w / 2, y: it.y - h, w, h };
}

function drawRoom(ctx, style) {
  const wall = WALLS[style.wall] ?? WALLS.pink;
  const floor = FLOORS[style.floor] ?? FLOORS.wood;
  ctx.fillStyle = wall.base;
  ctx.fillRect(0, 0, ROOM_W, FLOOR_TOP);
  ctx.fillStyle = wall.stripe;
  for (let x = 0; x < ROOM_W; x += 16) ctx.fillRect(x, 0, 8, FLOOR_TOP);
  // Pencere
  ctx.fillStyle = '#5b4a4f';
  ctx.fillRect(127, 17, 66, 50);
  ctx.fillStyle = style.wall === 'night' ? '#26334f' : '#cfe8fa';
  ctx.fillRect(129, 19, 62, 46);
  ctx.fillStyle = style.wall === 'night' ? '#f3d27a' : '#ffffff';
  ctx.fillRect(style.wall === 'night' ? 172 : 140, style.wall === 'night' ? 25 : 30, 8, 8);
  ctx.fillStyle = '#5b4a4f';
  ctx.fillRect(159, 19, 2, 46);
  ctx.fillRect(129, 41, 62, 2);
  // Süpürgelik ve zemin
  ctx.fillStyle = '#5b4a4f';
  ctx.fillRect(0, FLOOR_TOP - 3, ROOM_W, 3);
  ctx.fillStyle = floor.base;
  ctx.fillRect(0, FLOOR_TOP, ROOM_W, ROOM_H - FLOOR_TOP);
  ctx.fillStyle = floor.line;
  for (let y = FLOOR_TOP + 12; y < ROOM_H; y += 14) ctx.fillRect(0, y, ROOM_W, 1);
}

export function PetHomeSection({ onNavigate }) {
  const canvasRef = useRef(null);
  const wrapRef = useRef(null);
  const [items, setItems] = useState([]);
  const [pets, setPets] = useState([]);
  const [style, setStyle] = useState({ wall: 'pink', floor: 'wood' });
  const [editing, setEditing] = useState(false);
  const [selected, setSelected] = useState(null);
  const [scale, setScale] = useState(3);
  const [error, setError] = useState(null);
  const sim = useRef({ cats: new Map(), hearts: new Map(), drag: null, t: 0, random: rng(SEED) });
  const live = useRef({ items, pets, style, editing, selected });
  useEffect(() => {
    live.current = { items, pets, style, editing, selected };
  });

  const load = useCallback(() => {
    Promise.all([call('list_home_items'), call('list_pets'), call('get_home_style')]).then(
      ([i, p, s]) => {
        setItems(i);
        setPets(p);
        setStyle(s);
      },
      (e) => setError(e.message),
    );
  }, []);
  useEffect(load, [load]);
  useTauriEvent('pets:changed', () => call('list_pets').then(setPets, () => {}));

  // Kabın genişliğine göre tam sayı ölçek (keskin pikseller).
  useEffect(() => {
    const el = wrapRef.current;
    if (!el) return undefined;
    const ro = new ResizeObserver(() => setScale(Math.max(2, Math.floor(el.clientWidth / ROOM_W))));
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const homePets = pets.filter((p) => p.location === 'home');

  // Simülasyon + çizim döngüsü (yalnızca sayfa görünürken).
  useEffect(() => {
    const s = sim.current;
    const draw = () => {
      const ctx = canvasRef.current?.getContext('2d');
      if (!ctx) return;
      const { items: its, pets: ps, style: st, editing: ed, selected: sel } = live.current;
      ctx.setTransform(scale, 0, 0, scale, 0, 0);
      ctx.imageSmoothingEnabled = false;
      drawRoom(ctx, st);
      const home = ps.filter((p) => p.location === 'home');
      // Kedileri senkronla
      for (const p of home) if (!s.cats.has(p.id)) s.cats.set(p.id, createCat(p, s.random));
      for (const id of [...s.cats.keys()]) if (!home.some((p) => p.id === id)) s.cats.delete(id);
      const drawables = [
        ...its.map((it) => ({ y: FURNITURE[it.kind]?.role === 'floor' ? -1 : it.y, kind: 'item', it })),
        ...[...s.cats.values()].map((c) => ({ y: c.y + (c.state === 'sleep' ? 0.5 : 0), kind: 'cat', c })),
      ].sort((a, b) => a.y - b.y);
      for (const d of drawables) {
        if (d.kind === 'item') {
          const f = FURNITURE[d.it.kind];
          if (!f) continue;
          const b = itemBox(d.it);
          drawGrid(ctx, f.grid.map((r) => (d.it.flipped ? r.slice().reverse() : r)), { x: Math.round(b.x), y: Math.round(b.y), scale: 1 });
          if (ed && sel === d.it.id) {
            ctx.strokeStyle = '#b4a2e3';
            ctx.lineWidth = 1;
            ctx.strokeRect(Math.round(b.x) - 1.5, Math.round(b.y) - 1.5, b.w + 3, b.h + 3);
          }
        } else {
          const pet = home.find((p) => p.id === d.c.id);
          if (!pet) continue;
          const grid = petGrid({ anim: animOf(d.c), frame: d.c.frame, breed: pet.breed, accessories: [pet.acc_head, pet.acc_neck].filter(Boolean) });
          const x = Math.round(d.c.x - (CANVAS_W * CAT_SCALE) / 2);
          const y = Math.round(d.c.y - CANVAS_H * CAT_SCALE);
          drawGrid(ctx, grid, { x, y, scale: CAT_SCALE, flip: !d.c.face });
          const heart = s.hearts.get(d.c.id);
          if (heart && heart > s.t) {
            ctx.fillStyle = '#f07aa0';
            ctx.font = '9px sans-serif';
            ctx.fillText('♥', x + CANVAS_W / 2, y + 4);
          }
        }
      }
    };
    const tick = () => {
      if (document.hidden) return;
      s.t += TICK;
      const { items: its, pets: ps, editing: ed } = live.current;
      const cats = [...s.cats.values()];
      if (!ed) {
        for (const c of cats) {
          const pet = ps.find((p) => p.id === c.id);
          if (pet) stepCat(c, TICK, s.t, { needs: pet.needs, personality: pet.personality, items: its, cats, random: s.random });
        }
      }
      draw();
    };
    draw();
    const timer = setInterval(tick, TICK * 1000);
    return () => clearInterval(timer);
  }, [scale]);

  const toRoom = (e) => {
    const r = canvasRef.current.getBoundingClientRect();
    return { x: ((e.clientX - r.left) / r.width) * ROOM_W, y: ((e.clientY - r.top) / r.height) * ROOM_H };
  };

  const onPointerDown = (e) => {
    const p = toRoom(e);
    const s = sim.current;
    if (editing) {
      const hit = [...items].reverse().find((it) => {
        const b = itemBox(it);
        return p.x >= b.x && p.x <= b.x + b.w && p.y >= b.y && p.y <= b.y + b.h;
      });
      setSelected(hit?.id ?? null);
      if (hit) {
        s.drag = { id: hit.id, dx: p.x - hit.x, dy: p.y - hit.y };
        e.currentTarget.setPointerCapture(e.pointerId);
      }
      return;
    }
    // Kediye tıklamak onu sever.
    const cat = [...s.cats.values()].find((c) => Math.abs(c.x - p.x) < 12 && p.y > c.y - 20 && p.y < c.y + 2);
    if (cat) {
      s.hearts.set(cat.id, s.t + 1.2);
      if (cat.state !== 'sleep') {
        cat.state = 'petted';
        cat.until = s.t + 2;
        cat.target = null;
      }
      call('stroke_pet', { id: cat.id }).catch((err) => setError(err.message));
    }
  };

  const onPointerMove = (e) => {
    const d = sim.current.drag;
    if (!d) return;
    const p = toRoom(e);
    const x = Math.round(Math.max(8, Math.min(ROOM_W - 8, p.x - d.dx)));
    const y = Math.round(Math.max(FLOOR_TOP, Math.min(FLOOR_BOTTOM, p.y - d.dy)));
    setItems((list) => list.map((it) => (it.id === d.id ? { ...it, x, y } : it)));
  };

  const onPointerUp = () => {
    const d = sim.current.drag;
    sim.current.drag = null;
    if (!d) return;
    const it = live.current.items.find((i) => i.id === d.id);
    if (it) call('move_home_item', { id: it.id, x: it.x, y: it.y, flipped: it.flipped }).catch((err) => setError(err.message));
  };

  const addItem = async (kind) => {
    try {
      const it = await call('add_home_item', { kind, x: 100 + Math.round(sim.current.random() * 120), y: 150 });
      setItems((l) => [...l, it]);
      setEditing(true);
      setSelected(it.id);
    } catch (e) {
      setError(e.message);
    }
  };

  const sel = items.find((i) => i.id === selected);
  const flip = () => {
    const it = { ...sel, flipped: !sel.flipped };
    setItems((l) => l.map((i) => (i.id === it.id ? it : i)));
    call('move_home_item', { id: it.id, x: it.x, y: it.y, flipped: it.flipped }).catch((e) => setError(e.message));
  };
  const remove = () => {
    call('delete_home_item', { id: sel.id }).then(() => {
      setItems((l) => l.filter((i) => i.id !== sel.id));
      setSelected(null);
    }, (e) => setError(e.message));
  };
  const changeStyle = (patch) => {
    const next = { ...style, ...patch };
    setStyle(next);
    call('set_home_style', { style: next }).catch((e) => setError(e.message));
  };

  return (
    <div className="cc-page cc-page--wide">
      <PageHeader
        title="Kedi Evi"
        subtitle="Kedilerinin dinlendiği, oynadığı sıcacık oda"
        actions={<Button variant={editing ? 'primary' : 'secondary'} onClick={() => { setEditing((v) => !v); setSelected(null); }}>{editing ? 'Düzenlemeyi bitir' : 'Odayı düzenle'}</Button>}
      />
      <ErrorNote error={error} />
      <div ref={wrapRef} className={`home-wrap ${editing ? 'is-editing' : ''}`}>
        <canvas
          ref={canvasRef}
          className="home-canvas"
          width={ROOM_W * scale}
          height={ROOM_H * scale}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          aria-label="Kedi Evi odası"
        />
      </div>
      {editing ? (
        <Card title="Düzenleme">
          <p className="cc-muted cc-gap">Eşyaları sürükleyerek taşıyın. Seçili eşyayı çevirebilir ya da kaldırabilirsiniz.</p>
          <div className="cc-actions">
            <Button size="sm" disabled={!sel} onClick={flip}>Çevir</Button>
            <Button size="sm" variant="danger" disabled={!sel} onClick={remove}>Kaldır</Button>
            {sel ? <Badge tone="lavender">{FURNITURE[sel.kind]?.name}</Badge> : null}
          </div>
        </Card>
      ) : null}
      <div className="cc-grid">
        <Card title="Eşya ekle">
          <div className="home-catalog">
            {Object.entries(FURNITURE).map(([kind, f]) => (
              <button key={kind} type="button" className="home-catalog__item" onClick={() => addItem(kind)}>
                <FurnitureIcon grid={f.grid} />
                <span>{f.name}</span>
              </button>
            ))}
          </div>
        </Card>
        <Card title="Oda görünümü">
          <p className="wk-field__label">Duvar</p>
          <div className="cc-chips cc-gap">
            {Object.entries(WALLS).map(([id, w]) => (
              <button key={id} type="button" className={`cc-chip ${style.wall === id ? 'is-on' : ''}`} onClick={() => changeStyle({ wall: id })}>{w.name}</button>
            ))}
          </div>
          <p className="wk-field__label">Zemin</p>
          <div className="cc-chips">
            {Object.entries(FLOORS).map(([id, f]) => (
              <button key={id} type="button" className={`cc-chip ${style.floor === id ? 'is-on' : ''}`} onClick={() => changeStyle({ floor: id })}>{f.name}</button>
            ))}
          </div>
        </Card>
        <Card title="Evdeki kediler">
          {homePets.length ? (
            <ul className="cc-list">
              {homePets.map((p) => (
                <li key={p.id} className="cc-list__row">
                  <span className="cc-list__title">{p.name}</span>
                  <Button size="sm" onClick={() => call('set_pet_location', { id: p.id, location: 'desktop' }).catch((e) => setError(e.message))}>Masaüstüne çıkar</Button>
                </li>
              ))}
            </ul>
          ) : (
            <EmptyState cat={<PixelCat pose="sleeping" size={48} />} title="Evde kedi yok">
              Kediler bölümünden ya da masaüstündeki kedinin sağ tık menüsünden bir kediyi eve gönderebilirsin.
            </EmptyState>
          )}
          <Button size="sm" variant="ghost" onClick={() => onNavigate('pets')}>Kediler bölümüne git</Button>
        </Card>
      </div>
    </div>
  );
}

function FurnitureIcon({ grid }) {
  const ref = useRef(null);
  useEffect(() => {
    const ctx = ref.current?.getContext('2d');
    if (!ctx) return;
    const s = Math.max(1, Math.floor(36 / Math.max(grid.length, grid[0].length)));
    ctx.canvas.width = grid[0].length * s;
    ctx.canvas.height = grid.length * s;
    drawGrid(ctx, grid, { scale: s });
  }, [grid]);
  return <canvas ref={ref} className="pet-sprite" aria-hidden="true" />;
}
