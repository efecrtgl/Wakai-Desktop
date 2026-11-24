// Etkinlik oluşturma/düzenleme formu. Hem Takvim Kedisi widget'ında (kompakt) hem
// Kontrol Merkezi'ndeki Takvim bölümünde kullanılır.
import { useState } from 'react';
import { call } from '../../lib/ipc.js';
import { dateKey, parseLocal } from '../../lib/dates.js';
import { LABEL_COLORS } from '../../ui/widget.jsx';

export const REMINDER_OPTIONS = [
  { value: '', label: 'Hatırlatma yok' },
  { value: '0', label: 'Başladığında' },
  { value: '5', label: '5 dakika önce' },
  { value: '15', label: '15 dakika önce' },
  { value: '30', label: '30 dakika önce' },
  { value: '60', label: '1 saat önce' },
  { value: '1440', label: '1 gün önce' },
];

const COLOR_NAMES = { pink: 'Pembe', lavender: 'Lavanta', peach: 'Şeftali', mint: 'Nane', sky: 'Gök', butter: 'Tereyağı' };

function initialState(event, defaultDate) {
  if (event) {
    const start = parseLocal(event.start_at);
    const end = parseLocal(event.end_at);
    return {
      title: event.title,
      description: event.description ?? '',
      date: event.start_at.slice(0, 10),
      time: event.all_day ? '09:00' : event.start_at.slice(11, 16),
      endTime: end && start && dateKey(end) === dateKey(start) ? event.end_at.slice(11, 16) : '',
      allDay: event.all_day,
      categoryId: event.category_id ?? '',
      color: event.color,
      reminder: event.reminder_minutes == null ? '' : String(event.reminder_minutes),
    };
  }
  return {
    title: '',
    description: '',
    date: dateKey(defaultDate ?? new Date()),
    time: '09:00',
    endTime: '',
    allDay: false,
    categoryId: '',
    color: 'pink',
    reminder: '15',
  };
}

export function EventForm({ event, defaultDate, categories, onDone, compact = false }) {
  const [f, setF] = useState(() => initialState(event, defaultDate));
  const [error, setError] = useState(null);
  const [busy, setBusy] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const set = (k) => (e) => setF((s) => ({ ...s, [k]: e.target.type === 'checkbox' ? e.target.checked : e.target.value }));

  const onCategory = (e) => {
    const id = e.target.value;
    const cat = categories.find((c) => String(c.id) === id);
    setF((s) => ({ ...s, categoryId: id, color: cat ? cat.color : s.color }));
  };

  const submit = async (e) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    const input = {
      title: f.title,
      description: f.description,
      start_at: `${f.date}T${f.allDay ? '00:00' : f.time}`,
      end_at: !f.allDay && f.endTime ? `${f.date}T${f.endTime}` : null,
      all_day: f.allDay,
      category_id: f.categoryId === '' ? null : Number(f.categoryId),
      color: f.color,
      reminder_minutes: f.reminder === '' ? null : Number(f.reminder),
    };
    try {
      if (event) await call('update_event', { id: event.id, input });
      else await call('create_event', { input });
      onDone();
    } catch (err) {
      setError(err.message);
    } finally {
      setBusy(false);
    }
  };

  const remove = async () => {
    try {
      await call('delete_event', { id: event.id });
      onDone();
    } catch (err) {
      setError(err.message);
    }
  };

  return (
    <form className={`wg-form ${compact ? 'wg-form--compact' : ''}`} onSubmit={submit}>
      <input className="wg-input" placeholder="Etkinlik adı" value={f.title} onChange={set('title')} autoFocus maxLength={200} required />
      <div className="wg-form__row">
        <input className="wg-input" type="date" value={f.date} onChange={set('date')} required aria-label="Tarih" />
        {!f.allDay ? <input className="wg-input" type="time" value={f.time} onChange={set('time')} required aria-label="Başlangıç saati" /> : null}
        {!f.allDay && !compact ? <input className="wg-input" type="time" value={f.endTime} onChange={set('endTime')} aria-label="Bitiş saati" title="Bitiş saati (isteğe bağlı)" /> : null}
      </div>
      <label className="wg-check">
        <input type="checkbox" checked={f.allDay} onChange={set('allDay')} /> Tüm gün
      </label>
      <div className="wg-form__row">
        <select className="wg-input" value={f.categoryId} onChange={onCategory} aria-label="Kategori">
          <option value="">Kategori yok</option>
          {categories.map((c) => (
            <option key={c.id} value={c.id}>{c.name}</option>
          ))}
        </select>
        <select className="wg-input" value={f.reminder} onChange={set('reminder')} aria-label="Hatırlatıcı">
          {REMINDER_OPTIONS.map((o) => (
            <option key={o.value} value={o.value}>{o.label}</option>
          ))}
        </select>
      </div>
      <div className="wg-colors" role="radiogroup" aria-label="Renk etiketi">
        {LABEL_COLORS.map((c) => (
          <button
            key={c}
            type="button"
            role="radio"
            aria-checked={f.color === c}
            aria-label={COLOR_NAMES[c]}
            title={COLOR_NAMES[c]}
            className="wg-color"
            style={{ background: `var(--label-${c})` }}
            onClick={() => setF((s) => ({ ...s, color: c }))}
          />
        ))}
      </div>
      {!compact ? <textarea className="wg-input wg-textarea" placeholder="Açıklama (isteğe bağlı)" value={f.description} onChange={set('description')} maxLength={5000} /> : null}
      {error ? <p className="wg-error">{error}</p> : null}
      <div className="wg-form__actions">
        {event ? (
          confirmDelete ? (
            <button type="button" className="wg-btn wg-btn--danger" onClick={remove}>Silmeyi onayla</button>
          ) : (
            <button type="button" className="wg-btn wg-btn--ghost-danger" onClick={() => setConfirmDelete(true)}>Sil</button>
          )
        ) : null}
        <span className="wg-spacer" />
        <button type="button" className="wg-btn wg-btn--ghost" onClick={onDone}>Vazgeç</button>
        <button type="submit" className="wg-btn wg-btn--primary" disabled={busy}>{event ? 'Kaydet' : 'Ekle'}</button>
      </div>
    </form>
  );
}
