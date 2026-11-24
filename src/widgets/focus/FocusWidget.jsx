// Odak Kedisi. Zamanlayıcının durumu Rust'tadır; burada yalnızca bitiş anından kalan süre
// hesaplanıp gösterilir. Saniyelik güncelleme YALNIZCA zamanlayıcı çalışırken yapılır.
import { useEffect, useState } from 'react';
import { PixelCat } from '../../ui/cats/PixelCat.jsx';
import { useFocusState } from '../../lib/data.js';
import { call } from '../../lib/ipc.js';
import '../../ui/widget.css';
import './focus.css';

const PHASE_LABELS = { idle: 'Hazır', work: 'Odaklan', short_break: 'Kısa mola', long_break: 'Uzun mola' };

function useTicker(active) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!active) return undefined;
    let timer;
    const tick = () => {
      setNow(Date.now());
      timer = setTimeout(tick, 1000 - (Date.now() % 1000) + 5);
    };
    tick();
    return () => clearTimeout(timer);
  }, [active]);
  return now;
}

function format(ms) {
  const total = Math.max(0, Math.ceil(ms / 1000));
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

export default function FocusWidget() {
  const [state, setState] = useFocusState();
  const [error, setError] = useState(null);
  const now = useTicker(Boolean(state?.running));

  if (!state) return null;
  const remaining = state.running && state.ends_at_ms ? state.ends_at_ms - now : state.remaining_ms;
  const progress = state.planned_ms ? 1 - remaining / state.planned_ms : 0;
  const isBreak = state.phase === 'short_break' || state.phase === 'long_break';
  const cat = !state.running ? (state.phase === 'idle' ? 'sleeping' : 'sitting') : isBreak ? 'relaxed' : 'working';
  const circumference = 2 * Math.PI * 44;

  const run = (cmd) => async () => {
    try {
      setState(await call(cmd));
      setError(null);
    } catch (e) {
      setError(e.message);
    }
  };

  return (
    <div className={`focus ${isBreak ? 'is-break' : ''}`}>
      <div className="focus__ring">
        <svg viewBox="0 0 100 100" aria-hidden="true">
          <circle cx="50" cy="50" r="44" className="focus__track" />
          <circle
            cx="50"
            cy="50"
            r="44"
            className="focus__bar"
            strokeDasharray={circumference}
            strokeDashoffset={circumference * (1 - Math.min(1, Math.max(0, progress)))}
          />
        </svg>
        <div className="focus__center">
          <PixelCat pose={cat} size={34} />
          <span className="focus__time" role="timer" aria-live="off">{format(remaining)}</span>
          <span className="focus__phase">{PHASE_LABELS[state.phase]}</span>
        </div>
      </div>
      <div className="focus__controls">
        {state.running ? (
          <button type="button" className="wg-btn wg-btn--primary" onClick={run('focus_pause')}>Duraklat</button>
        ) : (
          <button type="button" className="wg-btn wg-btn--primary" onClick={run('focus_start')}>{state.phase === 'idle' ? 'Başlat' : 'Devam et'}</button>
        )}
        <button type="button" className="wg-btn wg-btn--ghost" onClick={run('focus_skip')} disabled={state.phase === 'idle'} title="Sonraki aşamaya geç">Atla</button>
        <button type="button" className="wg-btn wg-btn--ghost" onClick={run('focus_reset')} title="Sıfırla">Sıfırla</button>
      </div>
      <p className="focus__stats">Bugün {state.completed_today} odak oturumu</p>
      {error ? <p className="wg-error">{error}</p> : null}
    </div>
  );
}
