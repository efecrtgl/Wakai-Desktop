// Yalıtılmış tema önizlemesi: temanın değişkenleri ve yapı öznitelikleri yalnızca bu kutuya
// uygulanır (cssFor(…, '[data-sandbox="…"]') + data-*). Uygulamanın etkin teması, ayarlar,
// widget'lar ve kedi verileri değişmez.
import { useId, useMemo } from 'react';
import { attrsFor, cssFor } from '../../theme/engine.js';
import { RawPixelIcon } from '../../ui/PixelIcon.jsx';
import { LineIcon, LINE_ICONS } from '../../ui/LineIcon.jsx';
import { PetSprite } from '../../pets/PetSprite.jsx';

function SbIcon({ set, name }) {
  return set === 'line' && LINE_ICONS[name] ? <LineIcon name={name} size={14} /> : <RawPixelIcon name={name} size={14} />;
}

/**
 * @param {{ compiled: object, size?: 'card'|'large', label?: string }} p
 */
export function ThemeSandbox({ compiled, size = 'card', label }) {
  const key = useId().replace(/[^a-zA-Z0-9]/g, '');
  const css = useMemo(() => cssFor(compiled, `[data-sandbox="${key}"]`), [compiled, key]);
  const attrs = attrsFor(compiled);
  const data = Object.fromEntries(Object.entries(attrs).map(([k, v]) => [`data-${k.replace(/[A-Z]/g, (c) => '-' + c.toLowerCase())}`, v]));
  const icons = compiled.style.icons;
  return (
    <div className={`sb sb--${size}`} data-sandbox={key} {...data} role="img" aria-label={label ?? `${compiled.name} teması önizlemesi`}>
      <style>{css}</style>
      <div className="sb__nav">
        <span className="sb__brand">Ivily</span>
        <span className="sb__item sb__item--active">
          <SbIcon set={icons} name="home" /> {size === 'large' ? 'Genel Bakış' : ''}
        </span>
        <span className="sb__item">
          <SbIcon set={icons} name="widgets" /> {size === 'large' ? "Widget'lar" : ''}
        </span>
        <span className="sb__item">
          <SbIcon set={icons} name="paw" /> {size === 'large' ? 'Kedi Evi' : ''}
        </span>
      </div>
      <div className="sb__main">
        <p className="sb__title">Günaydın</p>
        <div className="wk-card sb__card">
          <div className="sb__row">
            <PetSprite breed="pink" anim="sit" scale={size === 'large' ? 3 : 2} palette={compiled.pets?.palette} />
            <div className="sb__lines">
              <span className="sb__line" />
              <span className="sb__line sb__line--short" />
            </div>
          </div>
          <div className="sb__row">
            <span className="wk-btn wk-btn--primary wk-btn--sm">Uygula</span>
            <span className="wk-switch" aria-checked="true" role="presentation">
              <span className="wk-switch__thumb" />
            </span>
          </div>
        </div>
        {size === 'large' ? (
          <div className="widget-frame sb__widget">
            <span className="sb__wtitle">Görevler</span>
            <span className="sb__check">☐ Kedi maması al</span>
            <span className="sb__check">☑ Sunumu bitir</span>
          </div>
        ) : null}
      </div>
    </div>
  );
}
