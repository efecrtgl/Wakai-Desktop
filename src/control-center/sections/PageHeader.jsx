import { useEmbedded } from './SectionTabs.jsx';

export function PageHeader({ title, subtitle, actions }) {
  const embedded = useEmbedded();
  // Sekme içinde: başlık bölümün kendisinde; yalnızca eylemler (ör. "Ekle") gösterilir.
  if (embedded) return actions ? <div className="cc-page__toolbar">{actions}</div> : null;
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
