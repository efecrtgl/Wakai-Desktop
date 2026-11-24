import { useMemo, useState } from 'react';
import { WidgetHeader, WidgetIconButton, ColorDot } from '../../ui/widget.jsx';
import { useEventCategories, useEvents, useUpcomingEvents } from '../../lib/data.js';
import { useNow } from '../../lib/time.js';
import {
  WEEKDAYS_SHORT,
  addDays,
  addMonths,
  dateKey,
  eventTimeLabel,
  formatDayLong,
  isWeekend,
  monthMatrix,
  monthTitle,
  parseLocal,
  relativeDay,
} from '../../lib/dates.js';
import { EventForm } from './EventForm.jsx';
import './calendar.css';

export default function CalendarWidget({ config }) {
  const now = useNow('minute');
  const todayKey = dateKey(now);
  const [month, setMonth] = useState(() => new Date(now.getFullYear(), now.getMonth(), 1));
  const [selected, setSelected] = useState(todayKey);
  // null | { mode: 'new' } | { mode: 'edit', event }
  const [editor, setEditor] = useState(null);

  const cells = useMemo(() => monthMatrix(month.getFullYear(), month.getMonth()), [month]);
  const from = dateKey(cells[0]);
  const to = dateKey(addDays(cells[41], 1));
  const { data: events } = useEvents(from, to);
  const { data: upcoming } = useUpcomingEvents(config.upcoming_count ?? 3);
  const categories = useEventCategories();

  const byDay = useMemo(() => {
    const map = new Map();
    for (const ev of events ?? []) {
      const k = ev.start_at.slice(0, 10);
      if (!map.has(k)) map.set(k, []);
      map.get(k).push(ev);
    }
    return map;
  }, [events]);

  const selectedDate = parseLocal(selected);
  const dayEvents = byDay.get(selected) ?? [];

  const goToday = () => {
    setMonth(new Date(now.getFullYear(), now.getMonth(), 1));
    setSelected(todayKey);
  };

  return (
    <div className="cal">
      <WidgetHeader
        cat="curious"
        title={monthTitle(month)}
        action={
          <>
            <WidgetIconButton label="Önceki ay" onClick={() => setMonth((m) => addMonths(m, -1))}>‹</WidgetIconButton>
            <WidgetIconButton label="Bugün" onClick={goToday}>•</WidgetIconButton>
            <WidgetIconButton label="Sonraki ay" onClick={() => setMonth((m) => addMonths(m, 1))}>›</WidgetIconButton>
          </>
        }
      />

      <div className="cal__grid" role="grid" aria-label={monthTitle(month)}>
        {WEEKDAYS_SHORT.map((d, i) => (
          <span key={d} className={`cal__wd ${i >= 5 ? 'is-weekend' : ''}`}>{d}</span>
        ))}
        {cells.map((d) => {
          const k = dateKey(d);
          const evs = byDay.get(k);
          const cls = [
            'cal__day',
            d.getMonth() !== month.getMonth() && 'is-outside',
            isWeekend(d) && 'is-weekend',
            k === todayKey && 'is-today',
            k === selected && 'is-selected',
          ]
            .filter(Boolean)
            .join(' ');
          return (
            <button
              key={k}
              type="button"
              className={cls}
              onClick={() => {
                setSelected(k);
                setEditor(null);
              }}
              aria-label={`${formatDayLong(d)}${evs ? `, ${evs.length} etkinlik` : ''}`}
            >
              <span>{d.getDate()}</span>
              {evs ? (
                <span className="cal__dots">
                  {evs.slice(0, 3).map((e) => (
                    <ColorDot key={e.id} color={e.color} size={4} />
                  ))}
                </span>
              ) : null}
            </button>
          );
        })}
      </div>

      <div className="cal__panel">
        {editor ? (
          <EventForm
            key={editor.event?.id ?? 'new'}
            event={editor.event}
            defaultDate={selectedDate}
            categories={categories}
            onDone={() => setEditor(null)}
            compact
          />
        ) : (
          <>
            <div className="cal__panel-head">
              <p className="cal__panel-title">{selected === todayKey ? 'Bugün' : selectedDate ? formatDayLong(selectedDate) : ''}</p>
              <WidgetIconButton label="Etkinlik ekle" onClick={() => setEditor({ mode: 'new' })}>＋</WidgetIconButton>
            </div>
            {dayEvents.length ? (
              <ul className="cal__list">
                {dayEvents.map((ev) => (
                  <li key={ev.id}>
                    <button type="button" className="cal__event" onClick={() => setEditor({ mode: 'edit', event: ev })}>
                      <ColorDot color={ev.color} />
                      <span className="cal__event-time">{eventTimeLabel(ev)}</span>
                      <span className="cal__event-title">{ev.title}</span>
                      {ev.reminder_minutes != null ? <span className="cal__bell" title="Hatırlatıcı açık">🔔</span> : null}
                    </button>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="cal__empty">Bu gün için plan yok.</p>
            )}

            {config.show_upcoming && upcoming?.length ? (
              <div className="cal__upcoming">
                <p className="cal__section">Yaklaşanlar</p>
                <ul className="cal__list">
                  {upcoming.map((ev) => {
                    const d = parseLocal(ev.start_at);
                    return (
                      <li key={ev.id}>
                        <button
                          type="button"
                          className="cal__event"
                          onClick={() => {
                            setMonth(new Date(d.getFullYear(), d.getMonth(), 1));
                            setSelected(dateKey(d));
                          }}
                        >
                          <ColorDot color={ev.color} />
                          <span className="cal__event-time">{relativeDay(d, now)}</span>
                          <span className="cal__event-title">{ev.title}</span>
                        </button>
                      </li>
                    );
                  })}
                </ul>
              </div>
            ) : null}
          </>
        )}
      </div>
    </div>
  );
}
