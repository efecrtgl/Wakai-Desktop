// Uygulama geneli ayar ve özellik araması (Ctrl+K). Klavye ile tamamen kullanılabilir.
import { useEffect, useMemo, useRef, useState } from 'react';
import { NAV, search } from './nav.js';
import { PixelIcon } from '../ui/PixelIcon.jsx';

const SUGGESTED = ['Tema kitaplığı', 'Kedi Evi dünyası', 'Yeni widget ekle', 'Masaüstü profilleri', 'Güncellemeleri denetle'];

export function CommandPalette({ open, onClose, onPick }) {
  const [query, setQuery] = useState('');
  const [index, setIndex] = useState(0);
  const inputRef = useRef(null);

  useEffect(() => {
    if (!open) return;
    inputRef.current?.focus();
  }, [open]);

  const results = useMemo(() => (query.trim() ? search(query) : null), [query]);
  const list = results ?? SUGGESTED.map((label) => search(label, undefined, 1)[0]).filter(Boolean);

  if (!open) return null;
  const close = () => {
    setQuery('');
    setIndex(0);
    onClose();
  };
  const pick = (item) => {
    close();
    onPick(item);
  };
  const onKey = (e) => {
    if (e.key === 'Escape') close();
    else if (e.key === 'ArrowDown') {
      e.preventDefault();
      setIndex((i) => Math.min(list.length - 1, i + 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setIndex((i) => Math.max(0, i - 1));
    } else if (e.key === 'Enter' && list[index]) pick(list[index]);
  };
  const where = (item) => {
    const n = NAV.find((x) => x.id === item.section);
    const t = n?.tabs?.find((x) => x.id === item.sub);
    return t ? `${n.label} › ${t.label}` : n?.label;
  };

  return (
    <div className="cc-palette" role="presentation" onMouseDown={(e) => e.target === e.currentTarget && close()}>
      <div className="cc-palette__panel" role="dialog" aria-modal="true" aria-label="Ara">
        <div className="cc-palette__field">
          <PixelIcon name="search" size={16} />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setIndex(0);
            }}
            onKeyDown={onKey}
            placeholder="Ayar ya da özellik ara… (ör. tema, yedek, başlangıç)"
            aria-label="Arama"
            aria-controls="cc-palette-list"
            aria-activedescendant={list[index] ? `cc-pal-${index}` : undefined}
          />
          <kbd>Esc</kbd>
        </div>
        {list.length ? (
          <ul className="cc-palette__list" id="cc-palette-list" role="listbox">
            {!results ? <li className="cc-palette__hint">Önerilenler</li> : null}
            {list.map((item, i) => (
              <li key={item.label} id={`cc-pal-${i}`} role="option" aria-selected={i === index}>
                <button type="button" onMouseEnter={() => setIndex(i)} onClick={() => pick(item)}>
                  <span className="cc-palette__label">{item.label}</span>
                  <span className="cc-palette__where">{where(item)}</span>
                </button>
              </li>
            ))}
          </ul>
        ) : (
          <p className="cc-palette__empty">“{query}” için sonuç yok. Farklı bir sözcük deneyin: örneğin “kedi”, “widget” ya da “güncelleme”.</p>
        )}
      </div>
    </div>
  );
}
