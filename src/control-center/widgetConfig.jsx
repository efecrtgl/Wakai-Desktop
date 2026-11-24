// Widget türüne göre yapılandırma formu. Alanlar Rust'taki varsayılan yapılandırmayla
// eşleşir; Rust yamayı yeniden doğrular (bilinmeyen anahtar ve yanlış tür reddedilir).
import { useState } from 'react';
import { Select, Slider, Toggle } from '../ui/components.jsx';

const FIELDS = {
  calendar: [
    { key: 'show_upcoming', type: 'toggle', label: 'Yaklaşan etkinlikleri göster' },
    { key: 'upcoming_count', type: 'slider', label: 'Yaklaşan etkinlik sayısı', min: 1, max: 8 },
  ],
  tasks: [
    { key: 'show_completed', type: 'toggle', label: 'Tamamlanan görevleri göster' },
    { key: 'category_filter', type: 'text', label: 'Yalnızca bu kategori', placeholder: 'Boş = tüm görevler', max: 40 },
  ],
  note: [{ key: 'font_size', type: 'slider', label: 'Yazı boyutu', min: 11, max: 24, format: (v) => `${v} px` }],
  clock: [
    { key: 'mode', type: 'select', label: 'Görünüm', options: [{ value: 'digital', label: 'Dijital' }, { value: 'analog', label: 'Analog' }] },
    { key: 'show_seconds', type: 'toggle', label: 'Saniyeleri göster', description: 'Kapalıyken saat dakikada bir güncellenir (daha az kaynak).' },
    { key: 'hour12', type: 'toggle', label: '12 saat biçimi (ÖÖ/ÖS)' },
    { key: 'show_date', type: 'toggle', label: 'Tarihi göster' },
    { key: 'time_zones', type: 'zones', label: 'Ek saat dilimleri' },
  ],
  focus: [
    { key: 'work_minutes', type: 'slider', label: 'Çalışma süresi', min: 5, max: 90, step: 5, format: (v) => `${v} dk` },
    { key: 'short_break_minutes', type: 'slider', label: 'Kısa mola', min: 1, max: 30, format: (v) => `${v} dk` },
    { key: 'long_break_minutes', type: 'slider', label: 'Uzun mola', min: 5, max: 60, step: 5, format: (v) => `${v} dk` },
    { key: 'long_break_every', type: 'slider', label: 'Uzun mola sıklığı', min: 2, max: 8, format: (v) => `${v} oturumda bir` },
    { key: 'auto_start', type: 'toggle', label: 'Sonraki aşamayı otomatik başlat' },
    { key: 'notify', type: 'toggle', label: 'Aşama bitince bildirim göster' },
  ],
  weather: [{ key: 'show_forecast', type: 'toggle', label: '5 günlük tahmini göster' }],
  music: [{ key: 'show_artwork', type: 'toggle', label: 'Albüm kapağını göster' }],
  system: [
    { key: 'show_gpu', type: 'toggle', label: 'GPU kullanımını göster' },
    { key: 'show_disk', type: 'toggle', label: 'Disk doluluğunu göster' },
  ],
};

function isValidZone(tz) {
  try {
    new Intl.DateTimeFormat('tr-TR', { timeZone: tz });
    return true;
  } catch {
    return false;
  }
}

function ZonesField({ value, onChange }) {
  const [draft, setDraft] = useState('');
  const [error, setError] = useState(null);
  const zones = Array.isArray(value) ? value : [];
  const add = (e) => {
    e.preventDefault();
    const tz = draft.trim();
    if (!tz) return;
    if (!isValidZone(tz)) {
      setError('Geçersiz saat dilimi. Örnek: Europe/London, Asia/Tokyo');
      return;
    }
    if (!zones.includes(tz)) onChange([...zones, tz].slice(0, 6));
    setDraft('');
    setError(null);
  };
  return (
    <div className="wk-row wk-row--stack">
      <span className="wk-row__label">Ek saat dilimleri</span>
      <div className="cc-chips">
        {zones.map((z) => (
          <button key={z} type="button" className="cc-chip" onClick={() => onChange(zones.filter((x) => x !== z))} title="Kaldır">
            {z} ×
          </button>
        ))}
      </div>
      <form className="cc-inline" onSubmit={add}>
        <input className="wk-input" value={draft} onChange={(e) => setDraft(e.target.value)} placeholder="Europe/London" aria-label="Saat dilimi ekle" />
        <button type="submit" className="wk-btn wk-btn--secondary wk-btn--sm">Ekle</button>
      </form>
      {error ? <p className="wk-error">{error}</p> : null}
    </div>
  );
}

export function WidgetConfigForm({ kind, config, onPatch }) {
  const fields = FIELDS[kind] ?? [];
  if (!fields.length) return null;
  return (
    <div className="cc-config">
      {fields.map((f) => {
        const value = config?.[f.key];
        switch (f.type) {
          case 'toggle':
            return <Toggle key={f.key} label={f.label} description={f.description} checked={Boolean(value)} onChange={(v) => onPatch({ [f.key]: v })} />;
          case 'slider':
            return (
              <Slider key={f.key} label={f.label} value={Number(value) || f.min} min={f.min} max={f.max} step={f.step ?? 1} format={f.format} onChange={(v) => onPatch({ [f.key]: v })} />
            );
          case 'select':
            return <Select key={f.key} label={f.label} value={value} options={f.options} onChange={(v) => onPatch({ [f.key]: v })} />;
          case 'text':
            return (
              <div key={f.key} className="wk-row">
                <span className="wk-row__label">{f.label}</span>
                <input
                  className="wk-input"
                  defaultValue={value ?? ''}
                  placeholder={f.placeholder}
                  maxLength={f.max}
                  onBlur={(e) => e.target.value !== (value ?? '') && onPatch({ [f.key]: e.target.value.trim() })}
                />
              </div>
            );
          case 'zones':
            return <ZonesField key={f.key} value={value} onChange={(v) => onPatch({ [f.key]: v })} />;
          default:
            return null;
        }
      })}
    </div>
  );
}
