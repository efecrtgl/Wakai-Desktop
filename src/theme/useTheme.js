// Tema motorunu ayarlara bağlayan kanca. Kontrol Merkezi ve her widget penceresi kullanır.
// Ayar değişince (settings:changed) tema yeniden başlatma gerektirmeden, tek adımda değişir.
import { useCallback, useEffect, useLayoutEffect, useMemo, useState, useSyncExternalStore } from 'react';
import { call, useTauriEvent } from '../lib/ipc.js';
import { applyTheme, builtinTheme, resolveTheme, FALLBACK_THEME_ID, compileTheme, getThemeError, setThemeError, subscribeThemeError } from './engine.js';

function parseManifest(text) {
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

/** Yüklü özel temalar: [{ id, name, version, installed_at, manifest(obj|null) }] */
export function useCustomThemes() {
  const [list, setList] = useState([]);
  const [loaded, setLoaded] = useState(false);
  const reload = useCallback(() => {
    call('list_custom_themes').then(
      (rows) => {
        setLoaded(true);
        setList(
          rows.map((r) => ({ ...r, manifest: parseManifest(r.manifest) })),
        );
      },
      () => {
        setLoaded(true);
        setList([]);
      },
    );
  }, []);
  useEffect(reload, [reload]);
  useTauriEvent('themes:changed', reload);
  return { customThemes: list, customThemesLoaded: loaded, reloadCustomThemes: reload };
}

/**
 * @param {object|null} settings uygulama ayarları
 * @returns {{ compiled, theme, fallbackReason, error, preview, setPreview, customThemes, reloadCustomThemes }}
 */
export function useThemeEngine(settings) {
  const { customThemes, customThemesLoaded, reloadCustomThemes } = useCustomThemes();
  // Tam uygulama önizlemesi: kalıcı değildir, ayarlara yazılmaz.
  const [preview, setPreview] = useState(null);
  const error = useSyncExternalStore(subscribeThemeError, getThemeError);

  const target = preview ?? (settings ? { id: settings.theme, variant: settings.theme_variant } : null);
  const accent = preview ? (preview.accent ?? settings?.theme_accent) : settings?.theme_accent;
  const density = settings?.ui_density ?? 'auto';

  const resolved = useMemo(() => {
    if (!target) return null;
    const custom = builtinTheme(target.id) ? null : (customThemes.find((t) => t.id === target.id)?.manifest ?? null);
    // Özel tema listesi henüz gelmediyse yedeğe düşmek yerine bekle (gereksiz yanıp sönme olmasın).
    if (!builtinTheme(target.id) && !custom && !customThemesLoaded) return null;
    return resolveTheme(target.id, { custom, variant: target.variant, accent, density });
  }, [target?.id, target?.variant, accent, density, customThemes, customThemesLoaded]); // eslint-disable-line react-hooks/exhaustive-deps

  useLayoutEffect(() => {
    if (!resolved) return;
    try {
      applyTheme(resolved.compiled);
      setThemeError(null);
    } catch (e) {
      // Uygulama sırasında hata: yedek temaya dön, uygulama kullanılabilir kalsın.
      setThemeError(e.message);
      try {
        applyTheme(compileTheme(builtinTheme(FALLBACK_THEME_ID)));
      } catch {
        // tokens.css varsayılanları devrede kalır
      }
    }
  }, [resolved]);

  // Temadan bağımsız erişilebilirlik ayarları.
  const textScale = settings?.text_scale ?? 1;
  const contrastHigh = !!settings?.high_contrast;
  useLayoutEffect(() => {
    const root = document.documentElement;
    root.style.setProperty('--text-scale', String(textScale));
    if (contrastHigh) root.dataset.contrast = 'high';
    else delete root.dataset.contrast;
  }, [textScale, contrastHigh]);

  return {
    compiled: resolved?.compiled ?? null,
    theme: resolved?.theme ?? null,
    fallbackReason: resolved?.fallbackReason ?? null,
    error,
    preview,
    setPreview,
    customThemes,
    reloadCustomThemes,
  };
}
