import { useEffect, useRef, useState } from 'react';
import { Badge, Button, Card, EmptyState, ErrorNote, Toggle } from '../../ui/components.jsx';
import { PixelCat } from '../../ui/cats/PixelCat.jsx';
import { call } from '../../lib/ipc.js';
import { tr } from '../../lib/i18n.js';
import { useWidgetDefinitions, useWidgets } from '../../lib/widgets.js';
import { WidgetConfigForm } from '../widgetConfig.jsx';
import { PageHeader } from './PageHeader.jsx';

function SizeEditor({ instance, onError }) {
  const [w, setW] = useState(instance.rect.width);
  const [h, setH] = useState(instance.rect.height);
  const apply = (e) => {
    e.preventDefault();
    call('set_widget_rect', { id: instance.id, rect: { ...instance.rect, width: Number(w), height: Number(h) } }).catch((err) => onError(err.message));
  };
  return (
    <form className="wk-row" onSubmit={apply}>
      <span className="wk-row__label">{tr.widgets.size}</span>
      <span className="cc-inline">
        <input className="wk-input cc-num" type="number" min={80} max={4000} value={w} onChange={(e) => setW(e.target.value)} aria-label="Genişlik" />
        ×
        <input className="wk-input cc-num" type="number" min={80} max={4000} value={h} onChange={(e) => setH(e.target.value)} aria-label="Yükseklik" />
        <Button size="sm" type="submit">Uygula</Button>
      </span>
    </form>
  );
}

function WidgetCard({ instance, definition, highlighted, onError }) {
  const [open, setOpen] = useState(highlighted);
  const [confirming, setConfirming] = useState(false);
  const ref = useRef(null);
  const act = (cmd, args) => call(cmd, { id: instance.id, ...args }).catch((e) => onError(e.message));

  useEffect(() => {
    if (highlighted) ref.current?.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }, [highlighted]);

  return (
    <div ref={ref} className={`cc-widget ${highlighted ? 'is-highlighted' : ''}`}>
      <button type="button" className="cc-widget__head" onClick={() => setOpen((v) => !v)} aria-expanded={open}>
        <PixelCat pose={definition?.cat ?? 'face'} size={36} />
        <span className="cc-widget__name">
          {definition?.name ?? instance.kind}
          <span className="cc-muted">{instance.rect.width}×{instance.rect.height} · {instance.rect.x}, {instance.rect.y}</span>
        </span>
        {instance.click_through ? <Badge tone="lavender">Tıklama geçirgen</Badge> : null}
        {instance.visible ? <Badge tone="mint">{tr.widgets.visible}</Badge> : <Badge tone="gray">Gizli</Badge>}
        <span className="cc-widget__chevron" aria-hidden="true">{open ? '▴' : '▾'}</span>
      </button>
      {open ? (
        <div className="cc-widget__body">
          <Toggle label={tr.widgets.visible} checked={instance.visible} onChange={(v) => act('set_widget_visible', { visible: v })} />
          <Toggle label={tr.widgets.clickThrough} description={tr.widgets.clickThroughDesc} checked={instance.click_through} onChange={(v) => act('set_widget_click_through', { value: v })} />
          <SizeEditor key={`${instance.rect.width}x${instance.rect.height}`} instance={instance} onError={onError} />
          <WidgetConfigForm kind={instance.kind} config={instance.config} onPatch={(patch) => act('update_widget_config', { patch })} />
          <div className="cc-actions">
            <Button size="sm" onClick={() => act('reset_widget_position')}>{tr.widgets.resetPosition}</Button>
            <Button size="sm" onClick={() => act('reset_widget')}>{tr.widgets.reset}</Button>
            <span className="wg-spacer" />
            {confirming ? (
              <>
                <span className="cc-muted">{tr.widgets.confirmRemove}</span>
                <Button size="sm" variant="danger" onClick={() => act('remove_widget')}>{tr.common.yes}</Button>
                <Button size="sm" variant="ghost" onClick={() => setConfirming(false)}>{tr.common.cancel}</Button>
              </>
            ) : (
              <Button size="sm" variant="danger" onClick={() => setConfirming(true)}>{tr.widgets.remove}</Button>
            )}
          </div>
        </div>
      ) : null}
    </div>
  );
}

export function WidgetsSection({ settingsApi, focusWidget, onNavigate }) {
  const { widgets, error, setError } = useWidgets();
  const defs = useWidgetDefinitions();
  const byKind = Object.fromEntries(defs.map((d) => [d.kind, d]));
  const settings = settingsApi.settings;

  return (
    <div className="cc-page">
      <PageHeader
        title={tr.widgets.title}
        subtitle={tr.widgets.subtitle}
        actions={<Button variant="primary" onClick={() => onNavigate('store')}>{tr.widgets.add}</Button>}
      />
      <ErrorNote error={error} />
      {settings && !settings.widgets_visible ? (
        <div className="cc-note cc-note--row">
          {tr.widgets.hiddenGlobally}
          <Button size="sm" onClick={() => settingsApi.update({ widgets_visible: true })}>Göster</Button>
        </div>
      ) : null}
      {widgets && widgets.length === 0 ? (
        <EmptyState cat={<PixelCat pose="curious" size={64} />} title={tr.widgets.empty}>
          Widget Mağazası'ndan ilk kedi dostunu ekle.
        </EmptyState>
      ) : null}
      <div className="cc-stack">
        {widgets?.map((w) => (
          <WidgetCard key={w.id} instance={w} definition={byKind[w.kind]} highlighted={w.id === focusWidget} onError={setError} />
        ))}
      </div>
      <Card>
        <p className="cc-muted">
          İpucu: Masaüstündeki bir widget'ın üzerine gelince öne çıkar. Üst kenarından sürükleyerek taşıyabilir, sağ alt köşesinden boyutlandırabilir,
          sağ tıklayarak menüsünü açabilirsiniz.
        </p>
      </Card>
    </div>
  );
}
