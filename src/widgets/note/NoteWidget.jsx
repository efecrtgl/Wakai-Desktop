// Not Kedisi: her widget örneği bir nota bağlıdır (config.note_id). Widget silinse de not kalır.
// Yazarken otomatik kayıt yapılır: son tuştan 600 ms sonra, odak kaybında ve pencere kapanırken.
import { Fragment, useCallback, useEffect, useRef, useState } from 'react';
import { WidgetHeader, WidgetIconButton, LABEL_COLORS } from '../../ui/widget.jsx';
import { call, useTauriEvent } from '../../lib/ipc.js';
import { parseNote, toggleCheck } from './format.js';
import './note.css';

const SAVE_DELAY = 600;

function Inline({ parts }) {
  return parts.map((p, i) =>
    p.type === 'bold' ? <strong key={i}>{p.text}</strong> : p.type === 'italic' ? <em key={i}>{p.text}</em> : <Fragment key={i}>{p.text}</Fragment>,
  );
}

function NoteView({ body, onToggle }) {
  const lines = parseNote(body);
  return (
    <div className="note__view">
      {lines.map((l) => {
        switch (l.type) {
          case 'heading':
            return <p key={l.index} className="note__h"><Inline parts={l.parts} /></p>;
          case 'bullet':
            return <p key={l.index} className="note__li"><Inline parts={l.parts} /></p>;
          case 'check':
            return (
              <label key={l.index} className={`note__check ${l.checked ? 'is-done' : ''}`} onClick={(e) => e.stopPropagation()}>
                <input type="checkbox" checked={l.checked} onChange={() => onToggle(l.index)} />
                <span><Inline parts={l.parts} /></span>
              </label>
            );
          case 'blank':
            return <p key={l.index} className="note__blank" />;
          default:
            return <p key={l.index}><Inline parts={l.parts} /></p>;
        }
      })}
    </div>
  );
}

export default function NoteWidget({ id, config }) {
  const [note, setNote] = useState(null);
  const [status, setStatus] = useState('loading'); // loading | ready | missing | error
  const [editing, setEditing] = useState(false);
  const [showColors, setShowColors] = useState(false);
  const [error, setError] = useState(null);
  const pending = useRef(null); // { timer, note }
  const textRef = useRef(null);
  const noteId = config.note_id;

  // Bağlı notu yükle; yoksa yeni not oluşturup widget'a bağla.
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        if (typeof noteId === 'number') {
          const n = await call('get_note', { id: noteId });
          if (!cancelled) {
            setNote(n);
            setStatus('ready');
          }
        } else {
          const n = await call('create_note', { input: {} });
          await call('update_widget_config', { id, patch: { note_id: n.id } });
          if (!cancelled) {
            setNote(n);
            setStatus('ready');
            setEditing(true);
          }
        }
      } catch (e) {
        if (cancelled) return;
        if (e.message === 'Kayıt bulunamadı.') setStatus('missing');
        else {
          setError(e.message);
          setStatus('error');
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [id, noteId]);

  const flush = useCallback(async () => {
    const p = pending.current;
    if (!p) return;
    clearTimeout(p.timer);
    pending.current = null;
    const n = p.note;
    try {
      await call('update_note', { id: n.id, input: { title: n.title, body: n.body, color: n.color, pinned: n.pinned } });
      setError(null);
    } catch (e) {
      setError(e.message);
    }
  }, []);

  const noteRef = useRef(note);
  useEffect(() => {
    noteRef.current = note;
  }, [note]);

  const change = (patch, immediate = false) => {
    const next = { ...(pending.current?.note ?? noteRef.current), ...patch };
    if (pending.current) clearTimeout(pending.current.timer);
    pending.current = { note: next, timer: setTimeout(flush, immediate ? 0 : SAVE_DELAY) };
    setNote(next);
  };

  // Pencere kapanırken ya da bileşen kalkarken bekleyen değişikliği yaz.
  useEffect(() => {
    window.addEventListener('beforeunload', flush);
    return () => {
      window.removeEventListener('beforeunload', flush);
      flush();
    };
  }, [flush]);

  // Not başka bir yerde (Kontrol Merkezi) değiştiyse ve burada bekleyen yazma yoksa yenile.
  useTauriEvent('notes:changed', (changedId) => {
    if (!note || changedId !== note.id || pending.current) return;
    call('get_note', { id: note.id }).then(setNote, () => setStatus('missing'));
  });

  const startEditing = () => {
    setEditing(true);
    requestAnimationFrame(() => textRef.current?.focus());
  };

  const createReplacement = async () => {
    try {
      const n = await call('create_note', { input: {} });
      await call('update_widget_config', { id, patch: { note_id: n.id } });
    } catch (e) {
      setError(e.message);
    }
  };

  if (status === 'loading') return null;
  if (status === 'missing') {
    return (
      <div className="note note--missing">
        <WidgetHeader cat="curious" title="Not bulunamadı" subtitle="Bu not silinmiş olabilir." />
        <button type="button" className="wg-btn wg-btn--primary" onClick={createReplacement}>Yeni not oluştur</button>
      </div>
    );
  }
  if (status === 'error') return <p className="wg-error">{error}</p>;

  return (
    <div className="note" data-color={note.color} style={{ '--note-font': `${config.font_size ?? 14}px` }}>
      <WidgetHeader
        cat="relaxed"
        title={
          <input
            className="note__title"
            value={note.title}
            placeholder="Başlıksız not"
            maxLength={120}
            onChange={(e) => change({ title: e.target.value })}
            onBlur={flush}
            aria-label="Not başlığı"
          />
        }
        action={
          <>
            <WidgetIconButton label={note.pinned ? 'Sabitlemeyi kaldır' : 'Sabitle'} aria-pressed={note.pinned} onClick={() => change({ pinned: !note.pinned }, true)}>
              {note.pinned ? '📌' : '📍'}
            </WidgetIconButton>
            <WidgetIconButton label="Renk" onClick={() => setShowColors((v) => !v)}>●</WidgetIconButton>
          </>
        }
      />
      {showColors ? (
        <div className="wg-colors note__colors" role="radiogroup" aria-label="Not rengi">
          {LABEL_COLORS.map((c) => (
            <button
              key={c}
              type="button"
              role="radio"
              aria-checked={note.color === c}
              className="wg-color"
              style={{ background: `var(--label-${c})` }}
              onClick={() => {
                change({ color: c }, true);
                setShowColors(false);
              }}
            />
          ))}
        </div>
      ) : null}
      {editing ? (
        <textarea
          ref={textRef}
          className="note__text"
          value={note.body}
          onChange={(e) => change({ body: e.target.value })}
          onBlur={() => {
            flush();
            setEditing(false);
          }}
          placeholder={'Bir şeyler yaz…\n# başlık   - madde   [ ] yapılacak   **kalın**'}
          aria-label="Not içeriği"
        />
      ) : (
        <div className="note__scroll" onClick={startEditing} role="button" tabIndex={0} onKeyDown={(e) => e.key === 'Enter' && startEditing()}>
          {note.body.trim() ? (
            <NoteView body={note.body} onToggle={(line) => change({ body: toggleCheck(note.body, line) }, true)} />
          ) : (
            <p className="note__placeholder">Yazmak için tıkla…</p>
          )}
        </div>
      )}
      {error ? <p className="wg-error">{error}</p> : null}
    </div>
  );
}
