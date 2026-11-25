import { PixelCat } from '../ui/cats/PixelCat.jsx';
import { PixelIcon } from '../ui/PixelIcon.jsx';
import { NAV } from './nav.js';

export function Sidebar({ active, onSelect, onSearch }) {
  return (
    <nav className="cc-nav" aria-label="Ana gezinme">
      <div className="cc-nav__brand">
        <span className="cc-nav__logo" aria-hidden="true">
          <PixelCat pose="sitting" size={34} />
        </span>
        <p className="cc-nav__name">
          Ivily<span> Desktop</span>
        </p>
      </div>
      <button type="button" className="cc-nav__search" onClick={onSearch} aria-label="Ayarlarda ve özelliklerde ara (Ctrl+K)">
        <PixelIcon name="search" size={14} />
        <span>Ara…</span>
        <kbd>Ctrl K</kbd>
      </button>
      <ul className="cc-nav__list">
        {NAV.map((s, i) => (
          <li key={s.id}>
            <button
              type="button"
              className="cc-nav__item"
              aria-current={s.id === active ? 'page' : undefined}
              title={`${s.label} (Ctrl+${i + 1})`}
              onClick={() => onSelect(s.id)}
            >
              <span className="cc-nav__icon">
                <PixelIcon name={s.icon} />
              </span>
              {s.label}
            </button>
          </li>
        ))}
      </ul>
    </nav>
  );
}
