// Paylaşılan temel arayüz bileşenleri. Stiller: components.css
import { useId } from 'react';

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

export function Toggle({ checked, onChange, label, description, disabled }) {
  const id = useId();
  return (
    <div className="wk-row">
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

export function Slider({ value, onChange, min, max, step = 1, label, format = (v) => v, description }) {
  const id = useId();
  return (
    <div className="wk-row">
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
          onChange={(e) => onChange(Number(e.target.value))}
        />
        <output htmlFor={id}>{format(value)}</output>
      </div>
    </div>
  );
}

export function Select({ value, onChange, options, label, description }) {
  const id = useId();
  return (
    <div className="wk-row">
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
