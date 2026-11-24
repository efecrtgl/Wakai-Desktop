export function PageHeader({ title, subtitle, actions }) {
  return (
    <header className="cc-page__header">
      <div>
        <h1 className="cc-page__title">{title}</h1>
        {subtitle ? <p className="cc-page__subtitle">{subtitle}</p> : null}
      </div>
      {actions ? <div className="cc-page__actions">{actions}</div> : null}
    </header>
  );
}
