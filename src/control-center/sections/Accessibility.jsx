// Erişilebilirlik: temadan bağımsız hareket, okunabilirlik, ses ve klavye ayarları.
// Bu ayarlar kedilerin ya da Kedi Evi dünyasının verilerini değiştirmez.
import { Button, Card, Select, Slider, Toggle } from '../../ui/components.jsx';
import { playThemeSound } from '../../theme/engine.js';
import { useActiveTheme } from '../../theme/context.js';

export const ANIMATION_OPTIONS = [
  { value: 'full', label: 'Tam (önerilen)' },
  { value: 'reduced', label: 'Azaltılmış' },
  { value: 'off', label: 'Kapalı' },
];

const SHORTCUTS = [
  ['Ctrl + K', 'Ayarlarda ve özelliklerde ara'],
  ['Ctrl + 1 … 7', 'Ana bölümler arasında geç'],
  ['Tab / Shift + Tab', 'Denetimler arasında gezin'],
  ['← → / ↑ ↓', 'Sekmeler ve ayar kategorileri arasında geç'],
  ['Esc', 'Pencereyi, aramayı ya da önizlemeyi kapat'],
  ['Kedi Evi: R', 'Seçili eşyayı döndür'],
  ['Kedi Evi: Delete', 'Seçili eşyayı envantere kaldır'],
  ['Kedi Evi: 1 … 3', 'Odalar arasında geç'],
];

export function AccessibilitySettings({ settingsApi }) {
  const { settings: s, update } = settingsApi;
  const theme = useActiveTheme();
  if (!s) return null;
  return (
    <>
      <Card title="Hareket">
        <Select
          anchor="set-motion"
          label="Animasyon yoğunluğu"
          value={s.animation_quality}
          options={ANIMATION_OPTIONS}
          onChange={(v) => update({ animation_quality: v })}
          description="Geçişler, kedi animasyonları ve Kedi Evi efektleri. Kapalıyken etkileşim aynen çalışır."
        />
        <Toggle
          label="Hareketi azalt"
          description="Tüm animasyonları durdurur. Windows'taki “hareketi azalt” tercihi de dikkate alınır."
          checked={s.reduced_motion}
          onChange={(v) => update({ reduced_motion: v })}
        />
      </Card>
      <Card title="Okunabilirlik">
        <Slider
          anchor="set-text-scale"
          label="Yazı boyutu"
          value={Math.round(s.text_scale * 100)}
          min={85}
          max={140}
          step={5}
          format={(v) => `%${v}`}
          onChange={(v) => update({ text_scale: v / 100 })}
          description="Tüm temalarda Kontrol Merkezi ve widget yazılarını büyütür."
        />
        <Toggle
          anchor="set-contrast"
          label="Yüksek karşıtlık"
          description="Soluk metinleri ve ince kenarlıkları belirginleştirir, odak çerçevesini kalınlaştırır."
          checked={s.high_contrast}
          onChange={(v) => update({ high_contrast: v })}
        />
      </Card>
      <Card title="Ses">
        <Toggle
          anchor="set-sounds"
          label="Arayüz sesleri"
          description="Temanın kısa tıklama ve bildirim sesleri. Ses dosyası indirilmez; sesler anında üretilir."
          checked={s.ui_sounds}
          onChange={(v) => update({ ui_sounds: v })}
        />
        <div className="cc-actions">
          <Button size="sm" disabled={!s.ui_sounds} onClick={() => playThemeSound(theme, 'success', s.ui_sounds)}>
            Sesi dene
          </Button>
        </div>
      </Card>
      <Card title="Klavye kısayolları" id="set-shortcuts">
        <dl className="cc-dl cc-dl--keys">
          {SHORTCUTS.map(([k, d]) => (
            <div key={k} className="cc-dl__row">
              <dt>
                <kbd>{k}</kbd>
              </dt>
              <dd>{d}</dd>
            </div>
          ))}
        </dl>
      </Card>
    </>
  );
}
