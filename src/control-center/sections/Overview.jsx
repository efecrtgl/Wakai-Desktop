import { Badge, Button, Card, Toggle } from '../../ui/components.jsx';
import { PixelCat } from '../../ui/cats/PixelCat.jsx';
import { ColorDot, ProgressBar } from '../../ui/widget.jsx';
import { useFocusState, useSystemStats, useTasks, useUpcomingEvents } from '../../lib/data.js';
import { useWidgetDefinitions, useWidgets } from '../../lib/widgets.js';
import { call } from '../../lib/ipc.js';
import { useNow } from '../../lib/time.js';
import { dateKey, eventTimeLabel, formatDayLong, parseLocal, relativeDay } from '../../lib/dates.js';
import { bytes, percent } from '../../lib/format.js';
import { PageHeader } from './PageHeader.jsx';
import '../../widgets/tasks/tasks.css';

function greeting(h) {
  if (h < 6) return 'İyi geceler';
  if (h < 12) return 'Günaydın';
  if (h < 18) return 'İyi günler';
  return 'İyi akşamlar';
}

function Stat({ label, value, hint, cat }) {
  return (
    <div className="cc-stat">
      <PixelCat pose={cat} size={40} />
      <div>
        <p className="cc-stat__value">{value}</p>
        <p className="cc-stat__label">{label}</p>
        {hint ? <p className="cc-stat__hint">{hint}</p> : null}
      </div>
    </div>
  );
}

export function Overview({ settingsApi, onNavigate }) {
  const now = useNow('minute');
  const { widgets } = useWidgets();
  const defs = useWidgetDefinitions();
  const { data: tasks } = useTasks();
  const { data: upcoming } = useUpcomingEvents(5);
  const [focus] = useFocusState();
  const stats = useSystemStats();
  const { settings: s, update } = settingsApi;
  const byKind = Object.fromEntries(defs.map((d) => [d.kind, d]));

  const visible = widgets?.filter((w) => w.visible).length ?? 0;
  const open = tasks?.filter((t) => !t.completed) ?? [];
  const overdue = open.filter((t) => t.due_at && parseLocal(t.due_at) < now).length;
  const done = (tasks?.length ?? 0) - open.length;
  const todayEvents = upcoming?.filter((e) => e.start_at.startsWith(dateKey(now))).length ?? 0;

  return (
    <div className="cc-page">
      <PageHeader title={`${greeting(now.getHours())}! 🐾`} subtitle={formatDayLong(now)} actions={<PixelCat pose="happy" size={56} wag />} />

      <div className="cc-stats">
        <Stat cat="sitting" label="Etkin widget" value={`${visible}/${widgets?.length ?? 0}`} hint={s && !s.widgets_visible ? 'Masaüstünde gizli' : null} />
        <Stat cat="working" label="Açık görev" value={open.length} hint={overdue ? `${overdue} gecikmiş` : null} />
        <Stat cat="curious" label="Bugün kalan etkinlik" value={todayEvents} />
        <Stat cat="relaxed" label="Bugünkü odak" value={focus?.completed_today ?? 0} hint={focus?.running ? 'Şu an çalışıyor' : null} />
      </div>

      <div className="cc-grid">
        <Card title="Yaklaşan etkinlikler" action={<Button size="sm" variant="ghost" onClick={() => onNavigate('calendar')}>Takvim</Button>}>
          {upcoming?.length ? (
            <ul className="cc-list">
              {upcoming.map((e) => (
                <li key={e.id} className="cc-list__row">
                  <ColorDot color={e.color} size={10} />
                  <span className="cc-list__time">{relativeDay(parseLocal(e.start_at), now)}</span>
                  <span className="cc-list__time">{eventTimeLabel(e)}</span>
                  <span className="cc-list__title">{e.title}</span>
                </li>
              ))}
            </ul>
          ) : (
            <p className="cc-muted">Yaklaşan etkinlik yok.</p>
          )}
        </Card>

        <Card title="Görevler" action={<Button size="sm" variant="ghost" onClick={() => onNavigate('tasks')}>Tümü</Button>}>
          <ProgressBar value={tasks?.length ? done / tasks.length : 0} label="Tamamlanma" />
          <p className="cc-muted cc-gap">{done}/{tasks?.length ?? 0} tamamlandı</p>
          <ul className="cc-list">
            {open.slice(0, 5).map((t) => (
              <li key={t.id} className="cc-list__row">
                <button type="button" className="task__check" role="checkbox" aria-checked={false} aria-label="Tamamla" onClick={() => call('set_task_completed', { id: t.id, completed: true })} />
                <span className="cc-list__title">{t.title}</span>
                {t.due_at ? <span className="cc-muted">{relativeDay(parseLocal(t.due_at), now)}</span> : null}
              </li>
            ))}
          </ul>
        </Card>

        <Card title="Performans" action={<Button size="sm" variant="ghost" onClick={() => onNavigate('performance')}>Ayrıntılar</Button>}>
          {stats ? (
            <div className="cc-meters">
              <span>İşlemci {percent(stats.cpu)}</span>
              <ProgressBar value={(stats.cpu ?? 0) / 100} label="İşlemci" />
              <span>Bellek {bytes(stats.memory_used)} / {bytes(stats.memory_total)}</span>
              <ProgressBar value={stats.memory_total ? stats.memory_used / stats.memory_total : 0} label="Bellek" />
            </div>
          ) : (
            <p className="cc-muted">Ölçülüyor…</p>
          )}
        </Card>

        <Card title="Hızlı widget kontrolleri" action={<Button size="sm" variant="ghost" onClick={() => onNavigate('widgets')}>Yönet</Button>}>
          {widgets?.length ? (
            widgets.map((w) => (
              <Toggle key={w.id} label={byKind[w.kind]?.name ?? w.kind} checked={w.visible} onChange={(v) => call('set_widget_visible', { id: w.id, visible: v })} />
            ))
          ) : (
            <Button size="sm" variant="primary" onClick={() => onNavigate('store')}>İlk widget'ı ekle</Button>
          )}
        </Card>

        {s ? (
          <Card title="Hızlı ayarlar" action={<Badge tone="mint">Çalışıyor</Badge>}>
            <Toggle label="Widget'ları göster" checked={s.widgets_visible} onChange={(v) => update({ widgets_visible: v })} />
            <Toggle label="Hareketi azalt" checked={s.reduced_motion} onChange={(v) => update({ reduced_motion: v })} />
            <Toggle label="Bildirimler" checked={s.notifications_enabled} onChange={(v) => update({ notifications_enabled: v })} />
            <Toggle label="Düşük kaynak modu" checked={s.reduced_resource_mode} onChange={(v) => update({ reduced_resource_mode: v })} />
          </Card>
        ) : null}
      </div>
    </div>
  );
}
