// Bölüm içi sekmeler. Gömülü bölümler kendi sayfa başlıklarını göstermez (EmbedContext).
import { createContext, useContext } from 'react';
import { PageHeader } from './PageHeader.jsx';

export const EmbedContext = createContext(false);
export const useEmbedded = () => useContext(EmbedContext);

/**
 * @param {{ title: string, subtitle?: string, tabs: {id,label}[], active: string,
 *   onChange: (id) => void, children: any, actions?: any }} p
 */
export function SectionTabs({ title, subtitle, tabs, active, onChange, children, actions, vertical = false }) {
  const onKey = (e) => {
    const i = tabs.findIndex((t) => t.id === active);
    const fwd = vertical ? 'ArrowDown' : 'ArrowRight';
    const back = vertical ? 'ArrowUp' : 'ArrowLeft';
    if (e.key === fwd || e.key === back) {
      e.preventDefault();
      const next = tabs[(i + (e.key === fwd ? 1 : tabs.length - 1)) % tabs.length];
      onChange(next.id);
      requestAnimationFrame(() => document.getElementById(`tab-${next.id}`)?.focus());
    }
  };
  return (
    <div className={vertical ? 'cc-page cc-page--split' : 'cc-page'}>
      <PageHeader title={title} subtitle={subtitle} actions={actions} />
      <div className={vertical ? 'cc-tabs cc-tabs--vertical' : 'cc-tabs'} role="tablist" aria-label={title} aria-orientation={vertical ? 'vertical' : 'horizontal'} onKeyDown={onKey}>
        {tabs.map((t) => (
          <button
            key={t.id}
            id={`tab-${t.id}`}
            type="button"
            role="tab"
            className="cc-tabs__tab"
            aria-selected={t.id === active}
            tabIndex={t.id === active ? 0 : -1}
            onClick={() => onChange(t.id)}
          >
            {t.label}
          </button>
        ))}
      </div>
      <div className="cc-tabpanel" role="tabpanel" aria-labelledby={`tab-${active}`}>
        <EmbedContext.Provider value={true}>{children}</EmbedContext.Provider>
      </div>
    </div>
  );
}
