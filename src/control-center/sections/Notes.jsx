// Not yönetimi. Bir not birden fazla yerde gösterilebilir: masaüstündeki Not Kedisi
// ile burada yapılan düzenlemeler `notes:changed` olayıyla eşitlenir.
import { useEffect, useRef, useState } from 'react';
import { Badge, Button, Card, EmptyState, ErrorNote } from '../../ui/components.jsx';
import { PixelCat } from '../../ui/cats/PixelCat.jsx';
import { LABEL_COLORS } from '../../ui/widget.jsx';
import { useNotes } from '../../lib/data.js';
import { useWidgets } from '../../lib/widgets.js';
import { call } from '../../lib/ipc.js';
import { parseLocal, relativeDay } from '../../lib/dates.js';
import { PageHeader } from './PageHeader.jsx';

function NoteEditor({ note, boundWidgets, onDeleted, onError }) {
  const [draft, setDraft] = useState(note);
  const [confirm, setConfirm] = useState(false);
  const timer = useRef(null);
  const latest = useRef(draft);

  useEffect(() => () => clearTimeout(timer.current), []);

  const save = (next) => call('update_note', { id: note.id, input: { title: next.title, body: next.body, color: next.color, pinned: next.pinned } }).catch((e) => onError(e.message));

  const change = (patch, immediate = false) => {
    const next = { ...latest.current, ...patch };
    latest.current = next;
    setDraft(next);
    clearTimeout(timer.current);
    timer.current = setTimeout(() => save(next), immediate ? 0 : 600);
  };

  const remove = async () => {
    try {
      await call('delete_note', { id: note.id });
      onDeleted();
    } catch (e) {
      onError(e.message);
    }
  };

  return (
    <Card className="cc-note-editor">
      <input className="cc-note-editor__title" value={draft.title} placeholder="Başlıksız not" maxLength={120} onChange={(e) => change({ title: e.target.value })} aria-label="Not başlığı" />
      <textarea
        className="wk-input wk-textarea cc-note-editor__body"
        value={draft.body}
        onChange={(e) => change({ body: e.target.value })}
        placeholder={'# başlık   - madde   [ ] yapılacak   **kalın**   *italik*'}
        aria-label="Not içeriği"
      />
      <div className="cc-actions">
        <div className="wg-colors" role="radiogroup" aria-label="Not rengi">
          {LABEL_COLORS.map((c) => (
            <button key={c} type="button" role="radio" aria-checked={draft.color === c} className="wg-color" style={{ background: `var(--label-${c})` }} onClick={() => change({ color: c }, true)} />
          ))}
        </div>
        <Button size="sm" variant={draft.pinned ? 'primary' : 'secondary'} onClick={() => change({ pinned: !draft.pinned }, true)}>
          {draft.pinned ? 'Sabitlendi' : 'Sabitle'}
        </Button>
        <span className="wg-spacer" />
        {confirm ? (
          <>
            <span className="cc-muted">
              {boundWidgets ? `Bu not ${boundWidgets} widget'ta gösteriliyor. ` : ''}Silinsin mi?
            </span>
            <Button size="sm" variant="danger" onClick={remove}>Sil</Button>
            <Button size="sm" variant="ghost" onClick={() => setConfirm(false)}>Vazgeç</Button>
          </>
        ) : (
          <Button size="sm" variant="danger" onClick={() => setConfirm(true)}>Notu sil</Button>
        )}
      </div>
    </Card>
  );
}

export function NotesSection() {
  const { data: notes, error, setError } = useNotes();
  const { widgets } = useWidgets();
  const [selectedId, setSelectedId] = useState(null);
  const selected = notes?.find((n) => n.id === selectedId) ?? null;
  const bound = (id) => widgets?.filter((w) => w.kind === 'note' && w.config?.note_id === id).length ?? 0;

  const create = async () => {
    try {
      const n = await call('create_note', { input: {} });
      setSelectedId(n.id);
    } catch (e) {
      setError(e.message);
    }
  };

  return (
    <div className="cc-page cc-page--wide">
      <PageHeader title="Notlar" subtitle={notes ? `${notes.length} not` : ''} actions={<Button variant="primary" onClick={create}>Yeni not</Button>} />
      <ErrorNote error={error} />
      {notes && notes.length === 0 ? (
        <EmptyState cat={<PixelCat pose="relaxed" size={64} />} title="Henüz not yok">Yeni bir not oluşturun ya da masaüstüne Not Kedisi ekleyin.</EmptyState>
      ) : (
        <div className="cc-split">
          <Card className="cc-notes-list">
            <ul className="cc-list">
              {notes?.map((n) => (
                <li key={n.id}>
                  <button type="button" className={`cc-list__item ${n.id === selectedId ? 'is-active' : ''}`} onClick={() => setSelectedId(n.id)}>
                    <span className="wg-dot" style={{ background: `var(--label-${n.color})`, width: 10, height: 10 }} />
                    <span className="cc-list__title">
                      {n.title || 'Başlıksız not'}
                      <span className="cc-muted">{n.body.split('\n')[0].slice(0, 60) || 'Boş'}</span>
                    </span>
                    {n.pinned ? <Badge tone="pink">Sabit</Badge> : null}
                    {bound(n.id) ? <Badge tone="lavender">Masaüstünde</Badge> : null}
                    <span className="cc-muted">{relativeDay(parseLocal(n.updated_at))}</span>
                  </button>
                </li>
              ))}
            </ul>
          </Card>
          {selected ? (
            <NoteEditor key={selected.id} note={selected} boundWidgets={bound(selected.id)} onDeleted={() => setSelectedId(null)} onError={setError} />
          ) : (
            <Card>
              <p className="cc-muted">Düzenlemek için soldan bir not seçin.</p>
            </Card>
          )}
        </div>
      )}
    </div>
  );
}
