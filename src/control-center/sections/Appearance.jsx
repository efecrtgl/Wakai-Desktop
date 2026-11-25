import { BUILTIN_THEMES } from '../../theme/engine.js';
import { Card, ErrorNote, Select, Slider, Toggle } from '../../ui/components.jsx';
import { PixelCat } from '../../ui/cats/PixelCat.jsx';
import { POSES } from '../../ui/cats/sprites.js';
import { PageHeader } from './PageHeader.jsx';

export const ANIMATION_OPTIONS = [
  { value: 'full', label: 'Tam (önerilen)' },
  { value: 'reduced', label: 'Azaltılmış' },
  { value: 'off', label: 'Kapalı' },
];


const FONTS = [
  { value: 'system', label: 'Varsayılan (Segoe UI)' },
  { value: 'modern', label: 'Modern (Bahnschrift)' },
  { value: 'mono', label: 'Daktilo (Cascadia)' },
];

export function AppearanceSection({ settingsApi }) {
  const { settings: s, update, error } = settingsApi;
  if (!s) return null;
  return (
    <div className="cc-page">
      <PageHeader title="Görünüm" subtitle="Pastel pembe piksel kedi teması" />
      <ErrorNote error={error} />
      <Card title="Görünüm modu">
        <div className="cc-themes" role="radiogroup" aria-label="Tema">
          {BUILTIN_THEMES.map((t) => (
            <button
              key={t.id}
              type="button"
              role="radio"
              aria-checked={s.theme === t.id}
              className="cc-theme"
              onClick={() => update({ theme: t.id, theme_variant: '' })}
            >
              <span className="cc-theme__swatch" aria-hidden="true">
                <span style={{ background: t.tokens['color-bg'] }} />
                <span style={{ background: t.tokens['color-primary'] }} />
                <span style={{ background: t.tokens['color-text'] }} />
              </span>
              <span className="cc-theme__name">{t.name}</span>
              <span className="cc-muted">{t.description}</span>
            </button>
          ))}
        </div>
        <Select
          label="Yazı tipi"
          value={s.font_family}
          options={FONTS}
          onChange={(v) => update({ font_family: v })}
          description="Tümü Windows 11 ile birlikte gelir; ek dosya indirilmez."
        />
      </Card>
      <Card title="Masaüstü widget'ları">
        <Toggle
          label="Widget'ları masaüstünde göster"
          description="Kapatınca tüm widget pencereleri kapanır ve kaynak kullanmaz."
          checked={s.widgets_visible}
          onChange={(v) => update({ widgets_visible: v })}
        />
        <Slider
          label="Opaklık"
          value={Math.round(s.widget_opacity * 100)}
          min={40}
          max={100}
          step={5}
          format={(v) => `%${v}`}
          onChange={(v) => update({ widget_opacity: v / 100 })}
          description="Widget arka planının saydamlığı."
        />
        <Slider
          label="Ölçek"
          value={Math.round(s.widget_scale * 100)}
          min={75}
          max={150}
          step={5}
          format={(v) => `%${v}`}
          onChange={(v) => update({ widget_scale: v / 100 })}
          description="Widget'ları ve içeriklerini orantılı olarak büyütür ya da küçültür."
        />
        <Slider label="Köşe yuvarlaklığı" value={s.corner_radius} min={0} max={28} step={2} format={(v) => `${v} px`} onChange={(v) => update({ corner_radius: v })} />
        <Toggle label="Gölgeler" checked={s.shadows} onChange={(v) => update({ shadows: v })} />
      </Card>
      <Card title="Hareket">
        <Select
          label="Animasyonlar"
          value={s.animation_quality}
          options={ANIMATION_OPTIONS}
          onChange={(v) => update({ animation_quality: v })}
          description="Göz kırpma, kuyruk sallama ve geçişler."
        />
        <Toggle
          label="Hareketi azalt"
          description="Tüm animasyonları durdurur. Windows'taki “hareketi azalt” tercihi de dikkate alınır."
          checked={s.reduced_motion}
          onChange={(v) => update({ reduced_motion: v })}
        />
      </Card>
      <Card title="Kedi ailesi">
        <div className="cc-cats">
          {POSES.map((p) => (
            <PixelCat key={p} pose={p} size={56} wag />
          ))}
        </div>
        <p className="cc-muted">Yeni temalar ileriki sürümlerde gelecek; tasarım sistemi buna hazır.</p>
      </Card>
    </div>
  );
}
