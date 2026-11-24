import { PixelCat } from '../ui/cats/PixelCat.jsx';
import { tr } from '../lib/i18n.js';
import { PixelIcon } from '../ui/PixelIcon.jsx';

export function Sidebar({ sections, active, onSelect }) {
  return (
    <nav className="cc-nav" aria-label="Ana gezinme">
      <div className="cc-nav__brand">
        <PixelCat pose="sitting" size={40} />
        <div>
          <p className="cc-nav__name">{tr.app.name}</p>
          <p className="cc-nav__tagline">{tr.app.tagline}</p>
        </div>
      </div>
      <ul className="cc-nav__list">
        {sections.map((s) => (
          <li key={s.id}>
            <button
              type="button"
              className="cc-nav__item"
              aria-current={s.id === active ? 'page' : undefined}
              onClick={() => onSelect(s.id)}
            >
              <span className="cc-nav__icon"><PixelIcon name={s.icon} /></span>
              {s.label}
            </button>
          </li>
        ))}
      </ul>
    </nav>
  );
}
