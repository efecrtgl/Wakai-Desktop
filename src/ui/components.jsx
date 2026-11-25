// Paylaşılan temel arayüz bileşenleri. Stiller: components.css
import { useEffect, useId, useRef } from 'react';

export function Button({ variant = 'secondary', size = 'md', icon, children, className = '', ...rest }) {
  return (
    <button type="button" className={`wk-btn wk-btn--${variant} wk-btn--${size} ${className}`} {...rest}>
      {icon ? <span className="wk-btn__icon" aria-hidden="true">{icon}</span> : null}
      {children}
    </button>
  );
}

export function IconButton({ label, children, className = '', ...rest }) {
  return (
    <button type="button" className={`wk-icon-btn ${className}`} aria-label={label} title={label} {...rest}>
      {children}
    </button>
  );
}

export function Card({ title, action, children, className = '', ...rest }) {
  return (
    <section className={`wk-card ${className}`} {...rest}>
      {title || action ? (
        <header className="wk-card__header">
          {title ? <h3 className="wk-card__title">{title}</h3> : null}
          {action}
        </header>
      ) : null}
      {children}
    </section>
  );
}

export function Toggle({ checked, onChange, label, description, disabled, anchor }) {
  const id = useId();
  return (
    <div className="wk-row" id={anchor}>
      <div className="wk-row__text">
        <label htmlFor={id} className="wk-row__label">{label}</label>
        {description ? <p className="wk-row__desc">{description}</p> : null}
      </div>
      <button
        id={id}
        type="button"
        role="switch"
        aria-checked={checked}
        disabled={disabled}
        className="wk-switch"
        onClick={() => onChange(!checked)}
      >
        <span className="wk-switch__thumb" />
      </button>
    </div>
  );
}

export function Slider({ value, onChange, min, max, step = 1, label, format = (v) => v, description, anchor, disabled }) {
  const id = useId();
  return (
    <div className="wk-row" id={anchor}>
      <div className="wk-row__text">
        <label htmlFor={id} className="wk-row__label">{label}</label>
        {description ? <p className="wk-row__desc">{description}</p> : null}
      </div>
      <div className="wk-slider">
        <input
          id={id}
          type="range"
          min={min}
          max={max}
          step={step}
          value={value}
          disabled={disabled}
          onChange={(e) => onChange(Number(e.target.value))}
        />
        <output htmlFor={id}>{format(value)}</output>
      </div>
    </div>
  );
}

export function Select({ value, onChange, options, label, description, anchor }) {
  const id = useId();
  return (
    <div className="wk-row" id={anchor}>
      <div className="wk-row__text">
        <label htmlFor={id} className="wk-row__label">{label}</label>
        {description ? <p className="wk-row__desc">{description}</p> : null}
      </div>
      <select id={id} className="wk-select" value={value} onChange={(e) => onChange(e.target.value)}>
        {options.map((o) => (
          <option key={o.value} value={o.value}>{o.label}</option>
        ))}
      </select>
    </div>
  );
}

export function Field({ label, children, hint }) {
  return (
    <label className="wk-field">
      <span className="wk-field__label">{label}</span>
      {children}
      {hint ? <span className="wk-field__hint">{hint}</span> : null}
    </label>
  );
}

export function Badge({ tone = 'pink', children }) {
  return <span className={`wk-badge wk-badge--${tone}`}>{children}</span>;
}

export function EmptyState({ cat, title, children }) {
  return (
    <div className="wk-empty">
      {cat}
      <p className="wk-empty__title">{title}</p>
      {children ? <p className="wk-empty__text">{children}</p> : null}
    </div>
  );
}

export function ErrorNote({ error }) {
  if (!error) return null;
  return <p className="wk-error" role="alert">{String(error)}</p>;
}

/** Yükleniyor göstergesi (piksel noktalar; hareket azaltılınca sabit). */
export function Loading({ label = 'Yükleniyor…' }) {
  return (
    <div className="wk-loading" role="status">
      <span className="wk-loading__dots" aria-hidden="true">
        <i />
        <i />
        <i />
      </span>
      <span>{label}</span>
    </div>
  );
}

/** Kısa açıklama balonu. Fareyle ya da klavye odağıyla görünür. */
export function Tooltip({ text, children }) {
  return (
    <span className="wk-tooltip" tabIndex={0}>
      {children}
      <span className="wk-tooltip__bubble" role="tooltip">{text}</span>
    </span>
  );
}

/**
 * Kalıcı modal iletişim kutusu. Escape ve arka plan tıklaması `onClose` çağırır
 * (`dismissible` false değilse). Açılınca ilk odaklanabilir öğeye odaklanır.
 */
export function Dialog({ open, title, children, actions, onClose, dismissible = true, icon }) {
  const ref = useRef(null);
  useEffect(() => {
    if (!open) return undefined;
    const prev = document.activeElement;
    ref.current?.querySelector('button, [href], input, select, textarea')?.focus();
    const onKey = (e) => {
      if (e.key === 'Escape' && dismissible) onClose?.();
    };
    window.addEventListener('keydown', onKey);
    return () => {
      window.removeEventListener('keydown', onKey);
      prev?.focus?.();
    };
  }, [open, dismissible, onClose]);
  if (!open) return null;
  return (
    <div className="wk-dialog__backdrop" onMouseDown={(e) => e.target === e.currentTarget && dismissible && onClose?.()}>
      <div ref={ref} className="wk-dialog" role="dialog" aria-modal="true" aria-label={title}>
        {icon ? <div className="wk-dialog__icon">{icon}</div> : null}
        <h2 className="wk-dialog__title">{title}</h2>
        <div className="wk-dialog__body">{children}</div>
        {actions ? <div className="wk-dialog__actions">{actions}</div> : null}
      </div>
    </div>
  );
}

/** Birkaç seçenekten biri (radyo grubu). Ok tuşlarıyla gezilir. */
export function Segmented({ value, onChange, options, label, description, anchor }) {
  const onKey = (e) => {
    const i = options.findIndex((o) => o.value === value);
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
      e.preventDefault();
      onChange(options[(i + 1) % options.length].value);
    } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
      e.preventDefault();
      onChange(options[(i + options.length - 1) % options.length].value);
    }
  };
  return (
    <div className="wk-row" id={anchor}>
      <div className="wk-row__text">
        <span className="wk-row__label">{label}</span>
        {description ? <p className="wk-row__desc">{description}</p> : null}
      </div>
      <div className="wk-seg" role="radiogroup" aria-label={label} onKeyDown={onKey}>
        {options.map((o) => (
          <button
            key={o.value}
            type="button"
            role="radio"
            aria-checked={o.value === value}
            tabIndex={o.value === value ? 0 : -1}
            className="wk-seg__opt"
            onClick={() => onChange(o.value)}
          >
            {o.label}
          </button>
        ))}
      </div>
    </div>
  );
}
