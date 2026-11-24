// Görev düzenleme formu (widget ve Kontrol Merkezi ortak).
import { useState } from 'react';
import { call } from '../../lib/ipc.js';

export const PRIORITIES = [
  { value: 0, label: 'Düşük', tone: 'mint' },
  { value: 1, label: 'Normal', tone: 'lavender' },
  { value: 2, label: 'Yüksek', tone: 'pink' },
];

export function TaskForm({ task, onDone, compact = false }) {
  const [f, setF] = useState(() => ({
    title: task?.title ?? '',
    notes: task?.notes ?? '',
    priority: task?.priority ?? 1,
    due: task?.due_at ?? '',
    category: task?.category ?? '',
    reminder: task?.reminder_at ?? '',
  }));
  const [error, setError] = useState(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const set = (k) => (e) => setF((s) => ({ ...s, [k]: e.target.value }));

  const submit = async (e) => {
    e.preventDefault();
    const input = {
      title: f.title,
      notes: f.notes,
      priority: Number(f.priority),
      due_at: f.due || null,
      category: f.category,
      reminder_at: f.reminder || null,
    };
    try {
      if (task) await call('update_task', { id: task.id, input });
      else await call('create_task', { input });
      onDone();
    } catch (err) {
      setError(err.message);
    }
  };

  const remove = async () => {
    try {
      await call('delete_task', { id: task.id });
      onDone();
    } catch (err) {
      setError(err.message);
    }
  };

  return (
    <form className={`wg-form ${compact ? 'wg-form--compact' : ''}`} onSubmit={submit}>
      <input className="wg-input" value={f.title} onChange={set('title')} placeholder="Görev" maxLength={200} required autoFocus />
      <div className="wg-segment" role="radiogroup" aria-label="Öncelik">
        {PRIORITIES.map((p) => (
          <button
            key={p.value}
            type="button"
            role="radio"
            aria-checked={Number(f.priority) === p.value}
            className={`wg-segment__item tone-${p.tone}`}
            onClick={() => setF((s) => ({ ...s, priority: p.value }))}
          >
            {p.label}
          </button>
        ))}
      </div>
      <label className="wg-label">
        Son tarih
        <input className="wg-input" type="datetime-local" value={f.due} onChange={set('due')} />
      </label>
      <div className="wg-form__row">
        <input className="wg-input" value={f.category} onChange={set('category')} placeholder="Kategori" maxLength={40} aria-label="Kategori" />
        <input className="wg-input" type="datetime-local" value={f.reminder} onChange={set('reminder')} aria-label="Hatırlatıcı" title="Hatırlatıcı (isteğe bağlı)" />
      </div>
      <textarea className="wg-input wg-textarea" value={f.notes} onChange={set('notes')} placeholder="Not (isteğe bağlı)" maxLength={2000} />
      {error ? <p className="wg-error">{error}</p> : null}
      <div className="wg-form__actions">
        {task ? (
          confirmDelete ? (
            <button type="button" className="wg-btn wg-btn--danger" onClick={remove}>Silmeyi onayla</button>
          ) : (
            <button type="button" className="wg-btn wg-btn--ghost-danger" onClick={() => setConfirmDelete(true)}>Sil</button>
          )
        ) : null}
        <span className="wg-spacer" />
        <button type="button" className="wg-btn wg-btn--ghost" onClick={onDone}>Vazgeç</button>
        <button type="submit" className="wg-btn wg-btn--primary">{task ? 'Kaydet' : 'Ekle'}</button>
      </div>
    </form>
  );
}
