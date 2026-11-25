// Tam uygulama tema önizlemesi sırasında üstte görünen şerit. Önizleme ayarlara yazılmaz;
// "Uygula" seçilene kadar hiçbir kalıcı değişiklik yapılmaz.
import { useThemeApi } from '../theme/context.js';

export function PreviewBar({ settingsApi }) {
  const api = useThemeApi();
  const p = api?.preview;
  if (!p) return null;
  const apply = async () => {
    const patch = { theme: p.id, theme_variant: p.variant ?? '' };
    if (p.accent !== undefined) patch.theme_accent = p.accent;
    await settingsApi.update(patch);
    api.setPreview(null);
  };
  return (
    <div className="cc-preview-bar" role="status">
      <span>
        <strong>Önizleme:</strong> {api.compiled?.name ?? p.id}
        {api.fallbackReason ? ` — ${api.fallbackReason}` : ''}
      </span>
      <button type="button" className="wk-btn wk-btn--primary wk-btn--sm" onClick={apply}>
        Uygula
      </button>
      <button type="button" className="wk-btn wk-btn--ghost wk-btn--sm" onClick={() => api.setPreview(null)}>
        Vazgeç
      </button>
    </div>
  );
}
