// Ayarları yükleyen ve değişiklikleri tüm pencerelerde senkron tutan kanca.
import { useCallback, useEffect, useState } from 'react';
import { call, useTauriEvent } from './ipc.js';

export function useSettings() {
  const [settings, setSettings] = useState(null);
  const [error, setError] = useState(null);

  useEffect(() => {
    call('get_settings').then(setSettings, setError);
  }, []);

  useTauriEvent('settings:changed', setSettings);

  const update = useCallback(async (patch) => {
    setError(null);
    try {
      setSettings(await call('update_settings', { patch }));
    } catch (e) {
      setError(e.message);
    }
  }, []);

  return { settings, update, error };
}
