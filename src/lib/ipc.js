// Rust komutları için ince sarmalayıcılar ve olay kancaları.
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useRef } from 'react';

/** Komutu çağırır; Rust'tan gelen Türkçe hata mesajını Error olarak fırlatır. */
export async function call(cmd, args) {
  try {
    return await invoke(cmd, args);
  } catch (err) {
    throw new Error(typeof err === 'string' ? err : (err?.message ?? 'Beklenmeyen bir hata oluştu.'), { cause: err });
  }
}

/** Bir Rust olayını bileşen yaşadığı sürece dinler. İşleyici değişse de yeniden abone olmaz. */
export function useTauriEvent(name, handler) {
  const ref = useRef(handler);
  useEffect(() => {
    ref.current = handler;
  });
  useEffect(() => {
    let disposed = false;
    let unlisten;
    listen(name, (e) => ref.current(e.payload)).then((fn) => {
      if (disposed) fn();
      else unlisten = fn;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [name]);
}
