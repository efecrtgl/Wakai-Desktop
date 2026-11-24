// Gelişmiş etkinlik yönetimi: büyük ay görünümü, gün listesi ve tam etkinlik formu.
import { useMemo, useState } from 'react';
import { Button, Card } from '../../ui/components.jsx';
import { ColorDot } from '../../ui/widget.jsx';
import { useEventCategories, useEvents } from '../../lib/data.js';
import { WEEKDAYS_SHORT, addDays, addMonths, dateKey, eventTimeLabel, formatDayLong, isWeekend, monthMatrix, monthTitle, parseLocal } from '../../lib/dates.js';
import { EventForm } from '../../widgets/calendar/EventForm.jsx';
import { PageHeader } from './PageHeader.jsx';

export function CalendarSection() {
  const today = new Date();
  const todayKey = dateKey(today);
  const [month, setMonth] = useState(() => new Date(today.getFullYear(), today.getMonth(), 1));
  const [selected, setSelected] = useState(todayKey);
  const [editor, setEditor] = useState(null);
  const cells = useMemo(() => monthMatrix(month.getFullYear(), month.getMonth()), [month]);
  const { data: events } = useEvents(dateKey(cells[0]), dateKey(addDays(cells[41], 1)));
  const categories = useEventCategories();
  const catName = Object.fromEntries(categories.map((c) => [c.id, c.name]));

  const byDay = useMemo(() => {
    const m = new Map();
    for (const e of events ?? []) {
      const k = e.start_at.slice(0, 10);
      if (!m.has(k)) m.set(k, []);
      m.get(k).push(e);
    }
    return m;
  }, [events]);
  const dayEvents = byDay.get(selected) ?? [];
  const selectedDate = parseLocal(selected);

  return (
    <div className="cc-page cc-page--wide">
      <PageHeader
        title="Takvim"
        subtitle={monthTitle(month)}
        actions={
          <>
            <Button size="sm" onClick={() => setMonth((m) => addMonths(m, -1))} aria-label="Önceki ay">‹</Button>
            <Button size="sm" onClick={() => { setMonth(new Date(today.getFullYear(), today.getMonth(), 1)); setSelected(todayKey); }}>Bugün</Button>
            <Button size="sm" onClick={() => setMonth((m) => addMonths(m, 1))} aria-label="Sonraki ay">›</Button>
            <Button size="sm" variant="primary" onClick={() => setEditor({ event: null })}>Etkinlik ekle</Button>
          </>
        }
      />
      <div className="cc-split">
        <Card className="cc-month">
          <div className="cc-month__grid">
            {WEEKDAYS_SHORT.map((d, i) => (
              <span key={d} className={`cc-month__wd ${i >= 5 ? 'is-weekend' : ''}`}>{d}</span>
            ))}
            {cells.map((d) => {
              const k = dateKey(d);
              const evs = byDay.get(k) ?? [];
              const cls = ['cc-month__day', d.getMonth() !== month.getMonth() && 'is-outside', isWeekend(d) && 'is-weekend', k === todayKey && 'is-today', k === selected && 'is-selected']
                .filter(Boolean)
                .join(' ');
              return (
                <button key={k} type="button" className={cls} onClick={() => { setSelected(k); setEditor(null); }} aria-label={formatDayLong(d)}>
                  <span className="cc-month__num">{d.getDate()}</span>
                  {evs.slice(0, 3).map((e) => (
                    <span key={e.id} className="cc-month__chip" style={{ '--chip': `var(--label-${e.color})` }}>{e.title}</span>
                  ))}
                  {evs.length > 3 ? <span className="cc-month__more">+{evs.length - 3}</span> : null}
                </button>
              );
            })}
          </div>
        </Card>
        <Card title={selectedDate ? formatDayLong(selectedDate) : ''} className="cc-dayside">
          {editor ? (
            <EventForm key={editor.event?.id ?? 'new'} event={editor.event} defaultDate={selectedDate} categories={categories} onDone={() => setEditor(null)} />
          ) : dayEvents.length ? (
            <ul className="cc-list">
              {dayEvents.map((e) => (
                <li key={e.id}>
                  <button type="button" className="cc-list__item" onClick={() => setEditor({ event: e })}>
                    <ColorDot color={e.color} size={10} />
                    <span className="cc-list__time">{eventTimeLabel(e)}</span>
                    <span className="cc-list__title">
                      {e.title}
                      {e.description ? <span className="cc-muted">{e.description}</span> : null}
                    </span>
                    {e.category_id ? <span className="cc-muted">{catName[e.category_id]}</span> : null}
                    {e.reminder_minutes != null ? <span title="Hatırlatıcı açık">🔔</span> : null}
                  </button>
                </li>
              ))}
            </ul>
          ) : (
            <p className="cc-muted">Bu gün için etkinlik yok.</p>
          )}
        </Card>
      </div>
    </div>
  );
}
