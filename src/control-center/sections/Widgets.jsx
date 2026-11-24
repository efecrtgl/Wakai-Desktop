import { useState } from 'react';
import { Badge, Button, Card, EmptyState, ErrorNote, Toggle } from '../../ui/components.jsx';
import { PixelCat } from '../../ui/cats/PixelCat.jsx';
import { call } from '../../lib/ipc.js';
import { tr } from '../../lib/i18n.js';
import { useWidgetDefinitions, useWidgets } from '../../lib/widgets.js';
import { supportedKinds } from '../../widgets/registry.js';
import { PageHeader } from './PageHeader.jsx';

function WidgetRow({ instance, definition, onError }) {
  const [confirming, setConfirming] = useState(false);
  const act = (cmd, args) => call(cmd, { id: instance.id, ...args }).catch((e) => onError(e.message));
  const { x, y, width, height } = instance.rect;

  return (
    <Card
      title={definition?.name ?? instance.kind}
      action={instance.visible ? <Badge tone="mint">{tr.widgets.visible}</Badge> : <Badge tone="gray">{tr.widget.hide}</Badge>}
    >
      <p className="cc-muted">
        {tr.widgets.position}: {x}, {y} · {tr.widgets.size}: {width}×{height}
      </p>
      <Toggle label={tr.widgets.visible} checked={instance.visible} onChange={(v) => act('set_widget_visible', { visible: v })} />
      <Toggle
        label={tr.widgets.clickThrough}
        description={tr.widgets.clickThroughDesc}
        checked={instance.click_through}
        onChange={(v) => act('set_widget_click_through', { value: v })}
      />
      <div className="cc-actions">
        <Button size="sm" onClick={() => act('reset_widget_position')}>{tr.widgets.resetPosition}</Button>
        <Button size="sm" onClick={() => act('reset_widget')}>{tr.widgets.reset}</Button>
        {confirming ? (
          <>
            <Button size="sm" variant="danger" onClick={() => act('remove_widget')}>{tr.common.yes}</Button>
            <Button size="sm" variant="ghost" onClick={() => setConfirming(false)}>{tr.common.cancel}</Button>
          </>
        ) : (
          <Button size="sm" variant="danger" onClick={() => setConfirming(true)} title={tr.widgets.confirmRemove}>
            {tr.widgets.remove}
          </Button>
        )}
      </div>
      {confirming ? <p className="cc-muted">{tr.widgets.confirmRemove}</p> : null}
    </Card>
  );
}

export function WidgetsSection({ settingsApi }) {
  const { widgets, error, setError } = useWidgets();
  const defs = useWidgetDefinitions();
  const byKind = Object.fromEntries(defs.map((d) => [d.kind, d]));
  const addable = defs.filter((d) => supportedKinds.includes(d.kind));

  const add = (kind) => call('add_widget', { kind }).catch((e) => setError(e.message));

  return (
    <div className="cc-page">
      <PageHeader title={tr.widgets.title} subtitle={tr.widgets.subtitle} />
      <ErrorNote error={error} />
      {settingsApi.settings && !settingsApi.settings.widgets_visible ? <p className="cc-note">{tr.widgets.hiddenGlobally}</p> : null}

      {widgets && widgets.length === 0 ? (
        <EmptyState cat={<PixelCat pose="face" size={56} />} title={tr.widgets.empty}>
          {tr.widgets.emptyHint}
        </EmptyState>
      ) : null}

      <div className="cc-grid">
        {widgets?.map((w) => (
          <WidgetRow key={w.id} instance={w} definition={byKind[w.kind]} onError={setError} />
        ))}
      </div>

      <Card title={tr.widgets.available}>
        <div className="cc-store">
          {addable.map((d) => (
            <div key={d.kind} className="cc-store__item">
              <div>
                <p className="cc-store__name">{d.name}</p>
                <p className="cc-muted">{d.description}</p>
              </div>
              <Button size="sm" variant="primary" onClick={() => add(d.kind)}>{tr.widgets.add}</Button>
            </div>
          ))}
        </div>
      </Card>
    </div>
  );
}
