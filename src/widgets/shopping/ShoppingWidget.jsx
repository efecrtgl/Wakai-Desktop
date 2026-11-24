// Alışveriş Kedisi: her widget örneği bir alışveriş listesine bağlıdır (config.list_id).
// Widget silinse de liste ve maddeleri kalır.
import { useCallback, useEffect, useRef, useState } from 'react';
import { ProgressBar, WidgetHeader, WidgetIconButton } from '../../ui/widget.jsx';
import { PixelCat } from '../../ui/cats/PixelCat.jsx';
import { call, useTauriEvent } from '../../lib/ipc.js';
import { useCelebration } from '../../lib/motion.js';
import { parseItem } from './parse.js';
import './shopping.css';

function ItemRow({ item, onToggle, onSave, onDelete }) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState('');
  const inputRef = useRef(null);

  const start = () => {
    setDraft(item.quantity ? `${item.quantity} ${item.name}` : item.name);
    setEditing(true);
    requestAnimationFrame(() => inputRef.current?.select());
  };
  const commit = () => {
    setEditing(false);
    const p = parseItem(draft);
    if (p.name && (p.name !== item.name || p.quantity !== item.quantity)) onSave(item, p);
  };

  return (
    <li className={`shop__item ${item.checked ? 'is-checked' : ''}`}>
      <button
        type="button"
        className="shop__check"
        role="checkbox"
        aria-checked={item.checked}
        aria-label={item.checked ? 'Sepetten çıkar' : 'Sepete at'}
        onClick={() => onToggle(item)}
      >
        {item.checked ? '✓' : ''}
      </button>
      {editing ? (
        <input
          ref={inputRef}
          className="wg-input shop__edit"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => {
            if (e.key === 'Enter') commit();
            if (e.key === 'Escape') setEditing(false);
          }}
          maxLength={150}
          aria-label="Maddeyi düzenle"
        />
      ) : (
        <button type="button" className="shop__name" onDoubleClick={start} title="Düzenlemek için çift tıklayın">
          <span>{item.name}</span>
          {item.quantity ? <span className="shop__qty">{item.quantity}</span> : null}
        </button>
      )}
      <WidgetIconButton label="Sil" className="shop__delete" onClick={() => onDelete(item)}>×</WidgetIconButton>
    </li>
  );
}

export default function ShoppingWidget({ id, config }) {
  const listId = config.list_id;
  const [list, setList] = useState(null);
  const [items, setItems] = useState([]);
  const [status, setStatus] = useState('loading'); // loading | ready | missing
  const [error, setError] = useState(null);
  const [draft, setDraft] = useState('');
  const [basketOpen, setBasketOpen] = useState(true);
  const [celebrating, celebrate] = useCelebration(1100);

  /** Listeyi ve maddelerini getirir; liste silinmişse `null` döner. */
  const fetchList = useCallback(async () => {
    const l = (await call('list_shopping_lists')).find((x) => x.id === listId);
    return l ? { list: l, items: await call('list_shopping_items', { listId }) } : null;
  }, [listId]);

  const apply = useCallback((data) => {
    if (!data) {
      setStatus('missing');
      return;
    }
    setList(data.list);
    setItems(data.items);
    setStatus('ready');
  }, []);

  const load = useCallback(() => fetchList().then(apply, (e) => setError(e.message)), [fetchList, apply]);

  // Liste yoksa oluştur ve widget'a bağla; varsa yükle.
  useEffect(() => {
    let alive = true;
    if (typeof listId === 'number') {
      fetchList().then(
        (d) => alive && apply(d),
        (e) => alive && setError(e.message),
      );
    } else {
      call('create_shopping_list', { name: 'Alışveriş listesi' })
        .then((l) => call('update_widget_config', { id, patch: { list_id: l.id } }))
        .catch((e) => alive && setError(e.message));
    }
    return () => {
      alive = false;
    };
  }, [id, listId, fetchList, apply]);

  useTauriEvent('shopping:changed', (changed) => {
    if (changed === listId) load();
  });

  const run = (cmd, args) => call(cmd, args).catch((e) => setError(e.message));

  const add = async (e) => {
    e.preventDefault();
    const p = parseItem(draft);
    if (!p.name) return;
    setDraft('');
    await run('add_shopping_item', { listId, name: p.name, quantity: p.quantity });
  };

  const toggle = async (item) => {
    await run('set_shopping_item_checked', { id: item.id, checked: !item.checked });
    // Son madde de sepete girince küçük bir kutlama.
    if (!item.checked && items.filter((i) => !i.checked).length === 1) celebrate();
  };

  const recreate = () =>
    call('create_shopping_list', { name: 'Alışveriş listesi' })
      .then((l) => call('update_widget_config', { id, patch: { list_id: l.id } }))
      .catch((e) => setError(e.message));

  if (status === 'missing') {
    return (
      <div className="shop shop--empty">
        <WidgetHeader cat="curious" title="Liste bulunamadı" subtitle="Bu liste silinmiş olabilir." />
        <button type="button" className="wg-btn wg-btn--primary" onClick={recreate}>Yeni liste oluştur</button>
      </div>
    );
  }
  if (status === 'loading') return error ? <p className="wg-error">{error}</p> : null;

  const open = items.filter((i) => !i.checked);
  const basket = items.filter((i) => i.checked);
  const showBasket = config.show_checked && basket.length > 0;

  return (
    <div className="shop">
      <WidgetHeader
        cat={celebrating ? 'happy' : 'sitting'}
        celebrate={celebrating}
        title={list?.name ?? 'Alışveriş'}
        subtitle={items.length ? (open.length ? `${open.length} madde kaldı` : 'Hepsi sepette!') : 'Liste boş'}
      />
      {items.length ? <ProgressBar value={basket.length / items.length} label="Sepetteki maddeler" /> : null}

      <form className="shop__add" onSubmit={add}>
        <input
          className="wg-input"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          placeholder="Ekle: 2 kg elma"
          maxLength={150}
          aria-label="Yeni madde"
        />
        <button type="submit" className="shop__add-btn" aria-label="Ekle" disabled={!draft.trim()}>+</button>
      </form>
      {error ? <p className="wg-error">{error}</p> : null}

      <div className="shop__scroll">
        {items.length === 0 ? (
          <div className="shop__empty">
            <PixelCat pose="curious" size={44} />
            <p>Listen boş. Yukarıdan ilk maddeyi ekle!</p>
          </div>
        ) : null}
        <ul className="shop__list">
          {open.map((i) => (
            <ItemRow
              key={i.id}
              item={i}
              onToggle={toggle}
              onSave={(it, p) => run('update_shopping_item', { id: it.id, name: p.name, quantity: p.quantity })}
              onDelete={(it) => run('delete_shopping_item', { id: it.id })}
            />
          ))}
        </ul>
        {showBasket ? (
          <section className="shop__basket">
            <div className="shop__basket-head">
              <button type="button" className="shop__basket-toggle" aria-expanded={basketOpen} onClick={() => setBasketOpen((v) => !v)}>
                {basketOpen ? '▾' : '▸'} Sepette ({basket.length})
              </button>
              <button type="button" className="wg-btn wg-btn--ghost" onClick={() => run('clear_checked_shopping_items', { listId })}>
                Temizle
              </button>
            </div>
            {basketOpen ? (
              <ul className="shop__list">
                {basket.map((i) => (
                  <ItemRow
                    key={i.id}
                    item={i}
                    onToggle={toggle}
                    onSave={(it, p) => run('update_shopping_item', { id: it.id, name: p.name, quantity: p.quantity })}
                    onDelete={(it) => run('delete_shopping_item', { id: it.id })}
                  />
                ))}
              </ul>
            ) : null}
          </section>
        ) : null}
      </div>
    </div>
  );
}
