// Widget Mağazası: ilk sürümde yalnızca yerleşik widget'lar. Üçüncü taraf kod çalıştırılmaz.
import { useState } from 'react';
import { Badge, Button, ErrorNote } from '../../ui/components.jsx';
import { PixelCat } from '../../ui/cats/PixelCat.jsx';
import { call } from '../../lib/ipc.js';
import { useWidgetDefinitions, useWidgets } from '../../lib/widgets.js';
import { supportedKinds } from '../../widgets/registry.js';
import { PageHeader } from './PageHeader.jsx';

const CATEGORY = { productivity: 'Üretkenlik', time: 'Zaman', info: 'Bilgi', media: 'Medya', system: 'Sistem' };

export function StoreSection({ onNavigate }) {
  const defs = useWidgetDefinitions().filter((d) => supportedKinds.includes(d.kind));
  const { widgets } = useWidgets();
  const [error, setError] = useState(null);
  const [added, setAdded] = useState(null);

  const add = async (kind) => {
    try {
      await call('add_widget', { kind });
      setAdded(kind);
      setError(null);
    } catch (e) {
      setError(e.message);
    }
  };

  return (
    <div className="cc-page">
      <PageHeader title="Widget Mağazası" subtitle="Masaüstüne yeni kedi dostları ekle" />
      <ErrorNote error={error} />
      {added ? (
        <div className="cc-note cc-note--row">
          Widget masaüstüne eklendi! Birincil ekranın sağ üstünde bulabilirsiniz.
          <Button size="sm" onClick={() => onNavigate('widgets')}>Widget'larıma git</Button>
        </div>
      ) : null}
      <div className="cc-store-grid">
        {defs.map((d) => {
          const count = widgets?.filter((w) => w.kind === d.kind).length ?? 0;
          const blocked = !d.multi_instance && count > 0;
          return (
            <article key={d.kind} className="cc-store-card">
              <div className="cc-store-card__art">
                <PixelCat pose={d.cat} size={72} />
              </div>
              <div className="cc-store-card__body">
                <div className="cc-store-card__title">
                  <h3>{d.name}</h3>
                  <Badge tone="lavender">{CATEGORY[d.category] ?? d.category}</Badge>
                </div>
                <p className="cc-muted">{d.description}</p>
                <p className="cc-store-card__meta">
                  {d.multi_instance ? 'Birden fazla eklenebilir' : 'Tek örnek'}
                  {count ? ` · ${count} ekli` : ''}
                </p>
              </div>
              <Button variant="primary" size="sm" disabled={blocked} onClick={() => add(d.kind)} title={blocked ? 'Bu widget zaten ekli' : undefined}>
                {blocked ? 'Ekli' : 'Ekle'}
              </Button>
            </article>
          );
        })}
      </div>
    </div>
  );
}
