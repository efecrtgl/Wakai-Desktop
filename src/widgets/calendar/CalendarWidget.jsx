// Takvim Kedisi (1.1.0 yeniden tasarım). Verisi ve işlevleri 1.0.0 ile aynıdır.
// Yenilikler: daha okunaklı başlık, "Bugün" kısayolu, ok tuşlarıyla gün seçimi,
// renk çubuklu etkinlik listesi ve küçük boyutlar için kompakt düzen (kapsayıcı sorguları).
import { useMemo, useRef, useState } from 'react';
import { WidgetIconButton } from '../../ui/widget.jsx';
import { PixelCat } from '../../ui/cats/PixelCat.jsx';
import { useEventCategories, useEvents, useUpcomingEvents } from '../../lib/data.js';
import { useNow } from '../../lib/time.js';
import {
  MONTHS,
  WEEKDAYS_SHORT,
  addDays,
  addMonths,
  dateKey,
  eventTimeLabel,
  formatDayLong,
  isWeekend,
  monthMatrix,
  parseLocal,
  relativeDay,
} from '../../lib/dates.js';
import { EventForm } from './EventForm.jsx';
import './calendar.css';

const KEY_STEPS = { ArrowLeft: -1, ArrowRight: 1, ArrowUp: -7, ArrowDown: 7 };

export default function CalendarWidget({ config }) {
  const now = useNow('minute');
  const todayKey = dateKey(now);
  const [month, setMonth] = useState(() => new Date(now.getFullYear(), now.getMonth(), 1));
  const [selected, setSelected] = useState(todayKey);
  const [editor, setEditor] = useState(null); // null | { event: Event | null }
  const gridRef = useRef(null);

  const cells = useMemo(() => monthMatrix(month.getFullYear(), month.getMonth()), [month]);
  const { data: events } = useEvents(dateKey(cells[0]), dateKey(addDays(cells[41], 1)));
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
  const onCurrentMonth = month.getFullYear() === now.getFullYear() && month.getMonth() === now.getMonth();

  const select = (d) => {
    const k = dateKey(d);
    setSelected(k);
    setEditor(null);
    if (d.getMonth() !== month.getMonth() || d.getFullYear() !== month.getFullYear()) {
      setMonth(new Date(d.getFullYear(), d.getMonth(), 1));
    }
  };

  const goToday = () => select(now);

  const onGridKey = (e) => {
    const step = KEY_STEPS[e.key];
    if (!step || !selectedDate) return;
    e.preventDefault();
    const next = addDays(selectedDate, step);
    select(next);
    requestAnimationFrame(() => gridRef.current?.querySelector(`[data-day="${dateKey(next)}"]`)?.focus());
  };

  const panelTitle = selected === todayKey ? `Bugün · ${selectedDate.getDate()} ${MONTHS[selectedDate.getMonth()]}` : formatDayLong(selectedDate);

  return (
    <div className="cal">
      <header className="cal__head">
        <PixelCat pose="curious" size={28} />
        <h2 className="cal__title">
          {MONTHS[month.getMonth()]} <span>{month.getFullYear()}</span>
        </h2>
        <div className="cal__nav">
          {!onCurrentMonth || selected !== todayKey ? (
            <button type="button" className="cal__today" onClick={goToday}>Bugün</button>
          ) : null}
          <WidgetIconButton label="Önceki ay" onClick={() => setMonth((m) => addMonths(m, -1))}>‹</WidgetIconButton>
          <WidgetIconButton label="Sonraki ay" onClick={() => setMonth((m) => addMonths(m, 1))}>›</WidgetIconButton>
        </div>
      </header>

      <div className="cal__grid" role="grid" aria-label={`${MONTHS[month.getMonth()]} ${month.getFullYear()}`} ref={gridRef} onKeyDown={onGridKey}>
        {WEEKDAYS_SHORT.map((d, i) => (
          <span key={d} className={`cal__wd ${i >= 5 ? 'is-weekend' : ''}`} role="columnheader">{d}</span>
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
            evs && 'has-events',
          ]
            .filter(Boolean)
            .join(' ');
          return (
            <button
              key={k}
              type="button"
              role="gridcell"
              data-day={k}
              tabIndex={k === selected ? 0 : -1}
              aria-selected={k === selected}
              className={cls}
              onClick={() => select(d)}
              aria-label={`${formatDayLong(d)}${evs ? `, ${evs.length} etkinlik` : ''}`}
            >
              <span className="cal__num">{d.getDate()}</span>
              {evs ? (
                <span className="cal__dots" aria-hidden="true">
                  {evs.slice(0, 3).map((e) => (
                    <i key={e.id} style={{ background: `var(--label-${e.color})` }} />
                  ))}
                </span>
              ) : null}
            </button>
          );
        })}
      </div>

      <section className="cal__panel" aria-live="polite">
        {editor ? (
          <EventForm key={editor.event?.id ?? 'new'} event={editor.event} defaultDate={selectedDate} categories={categories} onDone={() => setEditor(null)} compact />
        ) : (
          <>
            <div className="cal__panel-head">
              <p className="cal__panel-title">{panelTitle}</p>
              {dayEvents.length ? <span className="cal__count">{dayEvents.length}</span> : null}
              <WidgetIconButton label="Etkinlik ekle" className="cal__add" onClick={() => setEditor({ event: null })}>＋</WidgetIconButton>
            </div>
            {dayEvents.length ? (
              <ul className="cal__list">
                {dayEvents.map((ev) => (
                  <li key={ev.id}>
                    <button type="button" className="cal__event" style={{ '--ev': `var(--label-${ev.color})` }} onClick={() => setEditor({ event: ev })}>
                      <span className="cal__event-time">{eventTimeLabel(ev)}</span>
                      <span className="cal__event-title">{ev.title}</span>
                      {ev.reminder_minutes != null ? <span className="cal__bell" title="Hatırlatıcı açık" aria-label="Hatırlatıcı açık">🔔</span> : null}
                    </button>
                  </li>
                ))}
              </ul>
            ) : (
              <button type="button" className="cal__empty" onClick={() => setEditor({ event: null })}>
                Plan yok — eklemek için dokun
              </button>
            )}

            {config.show_upcoming && upcoming?.length ? (
              <div className="cal__upcoming">
                <p className="cal__section">Yaklaşanlar</p>
                <ul className="cal__list">
                  {upcoming.map((ev) => {
                    const d = parseLocal(ev.start_at);
                    return (
                      <li key={ev.id}>
                        <button type="button" className="cal__event cal__event--soft" style={{ '--ev': `var(--label-${ev.color})` }} onClick={() => select(d)}>
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
      </section>
    </div>
  );
}
