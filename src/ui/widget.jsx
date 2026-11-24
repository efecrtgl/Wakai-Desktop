// Widget'lar için ortak görsel yapı taşları. Stiller: widget.css
import { PixelCat } from './cats/PixelCat.jsx';
import './widget.css';

/** Widget başlığı: küçük kedi, başlık ve isteğe bağlı sağ taraf eylemi. */
export function WidgetHeader({ cat = 'sitting', title, subtitle, action, celebrate = false }) {
  return (
    <header className="wg-header">
      <PixelCat pose={cat} size={30} celebrate={celebrate} />
      <div className="wg-header__text">
        <p className="wg-header__title">{title}</p>
        {subtitle ? <p className="wg-header__subtitle">{subtitle}</p> : null}
      </div>
      {action ? <div className="wg-header__action">{action}</div> : null}
    </header>
  );
}

/** Piksel köşeli ilerleme çubuğu (0–1). */
export function ProgressBar({ value, label }) {
  const pct = Math.round(Math.max(0, Math.min(1, value || 0)) * 100);
  return (
    <div className="wg-progress" role="progressbar" aria-valuenow={pct} aria-valuemin={0} aria-valuemax={100} aria-label={label}>
      <div className="wg-progress__fill" style={{ transform: `scaleX(${pct / 100})` }} />
    </div>
  );
}

/** Küçük renk etiketi noktası. */
export function ColorDot({ color, size = 8 }) {
  return <span className="wg-dot" style={{ background: `var(--label-${color}, var(--label-pink))`, width: size, height: size }} />;
}

/** Widget içi küçük simge düğmesi. */
export function WidgetIconButton({ label, children, className = '', ...rest }) {
  return (
    <button type="button" className={`wg-icon-btn ${className}`} aria-label={label} title={label} {...rest}>
      {children}
    </button>
  );
}

export const LABEL_COLORS = ['pink', 'lavender', 'peach', 'mint', 'sky', 'butter'];
