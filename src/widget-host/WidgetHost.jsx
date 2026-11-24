// Widget penceresinin kökü: örneği yükler, değişiklikleri dinler, görünüm ayarlarını uygular.
import { Suspense, useCallback, useEffect, useState } from 'react';
import { call, useTauriEvent } from '../lib/ipc.js';
import { useSettings } from '../lib/settings.js';
import { useMotionAttribute } from '../lib/motion.js';
import { WIDGET_COMPONENTS } from '../widgets/registry.js';
import { WidgetShell } from './WidgetShell.jsx';
import { tr } from '../lib/i18n.js';

function useAppearance(settings) {
  useMotionAttribute(settings);
  useEffect(() => {
    if (!settings) return;
    const root = document.documentElement.style;
    root.setProperty('--widget-opacity', String(settings.widget_opacity));
    root.setProperty('--widget-radius', `${settings.corner_radius}px`);
    document.documentElement.dataset.shadows = settings.shadows ? 'on' : 'off';
  }, [settings]);
}

export function WidgetHost({ id }) {
  const [detail, setDetail] = useState(null);
  const [error, setError] = useState(null);
  const { settings } = useSettings();
  useAppearance(settings);

  useEffect(() => {
    if (!id) return;
    call('get_widget', { id }).then(setDetail, (e) => setError(e.message));
  }, [id]);

  useTauriEvent(
    'widget:updated',
    useCallback(
      (instance) => {
        if (instance.id === id) setDetail((d) => (d ? { ...d, instance } : d));
      },
      [id],
    ),
  );

  if (error) return <div className="widget-frame widget-frame--error">{error}</div>;
  if (!detail) return null;

  const Component = WIDGET_COMPONENTS[detail.instance.kind];
  return (
    <WidgetShell instance={detail.instance} definition={detail.definition}>
      {Component ? (
        <Suspense fallback={null}>
          <Component id={id} config={detail.instance.config} settings={settings} />
        </Suspense>
      ) : (
        <p className="widget-unsupported">{tr.widget.unsupported}</p>
      )}
    </WidgetShell>
  );
}
