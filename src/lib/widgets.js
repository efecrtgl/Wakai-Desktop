// Widget listesi ve tanımları için kancalar.
import { useCallback, useEffect, useState } from 'react';
import { call, useTauriEvent } from './ipc.js';

export function useWidgetDefinitions() {
  const [defs, setDefs] = useState([]);
  useEffect(() => {
    call('list_widget_definitions').then(setDefs, console.error);
  }, []);
  return defs;
}

export function useWidgets() {
  const [widgets, setWidgets] = useState(null);
  const [error, setError] = useState(null);
  const reload = useCallback(() => {
    call('list_widgets').then(setWidgets, (e) => setError(e.message));
  }, []);
  useEffect(reload, [reload]);
  useTauriEvent('widgets:changed', reload);
  return { widgets, error, reload, setError };
}
