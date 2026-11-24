// İçerik verisi için kancalar. Veri Rust'tan alınır; ilgili olay geldiğinde yeniden istenir.
import { useCallback, useEffect, useState } from 'react';
import { call, useTauriEvent } from './ipc.js';

/** Bir komutu çağırıp sonucu tutan, verilen olay geldiğinde yenileyen genel kanca. */
export function useQuery(cmd, args, event) {
  const [data, setData] = useState(null);
  const [error, setError] = useState(null);
  const key = JSON.stringify(args ?? {});
  const reload = useCallback(() => {
    call(cmd, JSON.parse(key)).then(
      (d) => {
        setData(d);
        setError(null);
      },
      (e) => setError(e.message),
    );
  }, [cmd, key]);
  useEffect(reload, [reload]);
  useTauriEvent(event, reload);
  return { data, error, reload, setError };
}

export const useEvents = (from, to) => useQuery('list_events', { from, to }, 'events:changed');
export const useUpcomingEvents = (limit) => useQuery('upcoming_events', { limit }, 'events:changed');
export const useTasks = () => useQuery('list_tasks', { includeCompleted: true }, 'tasks:changed');
export const useNotes = () => useQuery('list_notes', {}, 'notes:changed');

export function useEventCategories() {
  const [cats, setCats] = useState([]);
  useEffect(() => {
    call('list_event_categories').then(setCats, console.error);
  }, []);
  return cats;
}

/** Odak zamanlayıcısının durumu. */
export function useFocusState() {
  const [state, setState] = useState(null);
  useEffect(() => {
    call('focus_state').then(setState, console.error);
  }, []);
  useTauriEvent('focus:changed', setState);
  return [state, setState];
}

/** Pencere görünürken bir arka plan hizmetine abone olur (system | media). */
export function useSubscription(topic) {
  useEffect(() => {
    call('subscribe', { name: topic }).catch(console.error);
    return () => {
      call('unsubscribe', { name: topic }).catch(() => {});
    };
  }, [topic]);
}

/** Canlı sistem istatistikleri; yalnızca bu kanca kullanılırken örnekleme yapılır. */
export function useSystemStats() {
  useSubscription('system');
  const [stats, setStats] = useState(null);
  useEffect(() => {
    call('system_stats').then((s) => s && setStats(s), console.error);
  }, []);
  useTauriEvent('system:stats', setStats);
  return stats;
}

/** Windows medya oturumu durumu. */
export function useMediaState() {
  useSubscription('media');
  const [state, setState] = useState(null);
  useEffect(() => {
    call('media_state').then(setState, console.error);
  }, []);
  useTauriEvent('media:changed', setState);
  return state;
}
