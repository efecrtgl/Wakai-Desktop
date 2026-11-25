// Kişiselleştirme › Görünüm: temanın sınırları içinde vurgu rengi, yoğunluk, yazı tipi ve
// masaüstü widget görünümü. Temanın izin vermediği seçenekler açıklamayla devre dışıdır.
import { Button, Card, ErrorNote, Segmented, Select, Slider, Toggle } from '../../ui/components.jsx';
import { useThemeApi } from '../../theme/context.js';
import { builtinTheme } from '../../theme/engine.js';

const ACCENTS = ['#ef7d9a', '#e0789a', '#c8734f', '#e2a03f', '#4f9d69', '#3a8fb7', '#4f7cff', '#8a5cf6', '#b44dff', '#1a1a1a'];

const FONTS = [
  { value: 'system', label: 'Temanın yazı tipi' },
  { value: 'modern', label: 'Bahnschrift' },
  { value: 'mono', label: 'Cascadia (daktilo)' },
];

const DENSITY = [
  { value: 'auto', label: 'Tema' },
  { value: 'compact', label: 'Sıkı' },
  { value: 'normal', label: 'Normal' },
  { value: 'roomy', label: 'Ferah' },
];

export function LookSection({ settingsApi, onNavigate }) {
  const { settings: s, update, error } = settingsApi;
  const api = useThemeApi();
  if (!s) return null;
  const theme = api?.theme;
  const compiled = api?.compiled;
  const variants = theme ? [{ value: '', label: theme.base === 'dark' ? 'Koyu' : 'Açık' }, ...Object.entries(theme.variants ?? {}).map(([id, v]) => ({ value: id, label: v.name }))] : [];
  const accentAllowed = theme?.allow_accent !== false;
  const shapeLocked = compiled && ['pixel', 'sharp'].includes(compiled.style.shape);
  const themeName = theme?.name ?? builtinTheme(s.theme)?.name ?? s.theme;

  return (
    <>
      <ErrorNote error={error} />
      <Card title="Tema" action={<Button size="sm" variant="ghost" onClick={() => onNavigate('themes')}>Tema kitaplığı</Button>}>
        <p className="cc-muted">
          Etkin tema: <strong>{themeName}</strong>
          {api?.fallbackReason ? ` — yüklenemedi, Modern kullanılıyor (${api.fallbackReason})` : ''}
        </p>
        {variants.length > 1 ? (
          <Segmented label="Çeşit" value={compiled?.variant ?? ''} options={variants} onChange={(v) => update({ theme_variant: v })} />
        ) : null}
      </Card>

      <Card title="Vurgu rengi" id="set-accent">
        {accentAllowed ? (
          <>
            <div className="cc-accents" role="radiogroup" aria-label="Vurgu rengi">
              <button type="button" role="radio" aria-checked={!s.theme_accent} className="cc-accent cc-accent--theme" onClick={() => update({ theme_accent: null })} title="Temanın kendi rengi">
                <span>Tema</span>
              </button>
              {ACCENTS.map((c) => (
                <button key={c} type="button" role="radio" aria-checked={s.theme_accent?.toLowerCase() === c} aria-label={`Renk ${c}`} className="cc-accent" style={{ '--swatch': c }} onClick={() => update({ theme_accent: c })} />
              ))}
              <label className="cc-accent cc-accent--custom" title="Özel renk">
                <input type="color" value={s.theme_accent ?? '#ef7d9a'} onChange={(e) => update({ theme_accent: e.target.value })} aria-label="Özel vurgu rengi" />
              </label>
            </div>
            <p className="cc-muted">Düğmeler, seçili öğeler ve vurgular bu rengi kullanır. Metin rengi okunabilirliğe göre otomatik seçilir.</p>
          </>
        ) : (
          <p className="cc-muted">{themeName} teması kendi renk düzenini korur; vurgu rengi bu temada kullanılmaz.</p>
        )}
      </Card>

      <Card title="Düzen ve yazı">
        <Segmented anchor="set-density" label="Arayüz yoğunluğu" value={s.ui_density} options={DENSITY} onChange={(v) => update({ ui_density: v })} description="Boşlukları sıkılaştırır ya da ferahlatır." />
        <Select anchor="set-font" label="Yazı tipi" value={s.font_family} options={FONTS} onChange={(v) => update({ font_family: v })} description="“Temanın yazı tipi” her temanın kendi seçimidir (Piksel'de bitmap yazı tipi)." />
      </Card>

      <Card title="Masaüstü widget'ları" id="set-widget-look">
        <Slider label="Opaklık" value={Math.round(s.widget_opacity * 100)} min={40} max={100} step={5} format={(v) => `%${v}`} onChange={(v) => update({ widget_opacity: v / 100 })} description="Widget arka planının saydamlığı." />
        <Slider label="Ölçek" value={Math.round(s.widget_scale * 100)} min={75} max={150} step={5} format={(v) => `%${v}`} onChange={(v) => update({ widget_scale: v / 100 })} description="Widget'ları ve içeriklerini orantılı olarak büyütür ya da küçültür." />
        <Slider
          label="Köşe yuvarlaklığı"
          value={s.corner_radius}
          min={0}
          max={28}
          step={2}
          format={(v) => `${v} px`}
          onChange={(v) => update({ corner_radius: v })}
          disabled={shapeLocked}
          description={shapeLocked ? `${themeName} temasında köşeler temanın biçimini izler.` : undefined}
        />
        <Toggle label="Gölgeler" checked={s.shadows} onChange={(v) => update({ shadows: v })} />
      </Card>
    </>
  );
}
