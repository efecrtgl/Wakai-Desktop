import { useState } from 'react';
import { Button, Card, EmptyState, ErrorNote } from '../../ui/components.jsx';
import { PixelCat } from '../../ui/cats/PixelCat.jsx';
import { ProgressBar } from '../../ui/widget.jsx';
import { useTasks } from '../../lib/data.js';
import { call } from '../../lib/ipc.js';
import { formatTime, parseLocal, relativeDay } from '../../lib/dates.js';
import { PRIORITIES, TaskForm } from '../../widgets/tasks/TaskForm.jsx';
import { PageHeader } from './PageHeader.jsx';
import '../../widgets/tasks/tasks.css';

const FILTERS = [
  { id: 'open', label: 'Açık' },
  { id: 'done', label: 'Tamamlanan' },
  { id: 'all', label: 'Tümü' },
];

export function TasksSection() {
  const { data: tasks, error, setError } = useTasks();
  const [filter, setFilter] = useState('open');
  const [category, setCategory] = useState('');
  const [editing, setEditing] = useState(null);
  const now = new Date();

  const all = tasks ?? [];
  const categories = [...new Set(all.map((t) => t.category).filter(Boolean))].sort((a, b) => a.localeCompare(b, 'tr'));
  const shown = all
    .filter((t) => (filter === 'open' ? !t.completed : filter === 'done' ? t.completed : true))
    .filter((t) => !category || t.category === category);
  const done = all.filter((t) => t.completed).length;

  const toggle = (t) => call('set_task_completed', { id: t.id, completed: !t.completed }).catch((e) => setError(e.message));
  const clear = () => call('clear_completed_tasks').catch((e) => setError(e.message));

  return (
    <div className="cc-page">
      <PageHeader
        title="Görevler"
        subtitle={`${all.length - done} açık · ${done} tamamlandı`}
        actions={
          <>
            {done ? <Button size="sm" variant="ghost" onClick={clear}>Tamamlananları temizle</Button> : null}
            <Button variant="primary" onClick={() => setEditing('new')}>Görev ekle</Button>
          </>
        }
      />
      <ErrorNote error={error} />
      <ProgressBar value={all.length ? done / all.length : 0} label="Tamamlanma oranı" />
      {editing ? (
        <Card title={editing === 'new' ? 'Yeni görev' : 'Görevi düzenle'}>
          <TaskForm key={editing === 'new' ? 'new' : editing.id} task={editing === 'new' ? null : editing} onDone={() => setEditing(null)} />
        </Card>
      ) : null}
      <div className="cc-toolbar">
        <div className="wg-segment" role="tablist">
          {FILTERS.map((f) => (
            <button key={f.id} type="button" role="tab" aria-checked={filter === f.id} className="wg-segment__item" onClick={() => setFilter(f.id)}>
              {f.label}
            </button>
          ))}
        </div>
        {categories.length ? (
          <select className="wk-select" value={category} onChange={(e) => setCategory(e.target.value)} aria-label="Kategori filtresi">
            <option value="">Tüm kategoriler</option>
            {categories.map((c) => (
              <option key={c} value={c}>{c}</option>
            ))}
          </select>
        ) : null}
      </div>
      {tasks && shown.length === 0 ? (
        <EmptyState cat={<PixelCat pose={filter === 'open' && all.length ? 'happy' : 'sleeping'} size={64} />} title={filter === 'open' && all.length ? 'Açık görev kalmadı!' : 'Burada görev yok'} />
      ) : (
        <Card>
          <ul className="cc-list">
            {shown.map((t) => {
              const due = parseLocal(t.due_at);
              const prio = PRIORITIES[t.priority] ?? PRIORITIES[1];
              return (
                <li key={t.id} className={`cc-task ${t.completed ? 'is-done' : ''}`}>
                  <button type="button" className="task__check" role="checkbox" aria-checked={t.completed} onClick={() => toggle(t)} aria-label="Tamamlandı">
                    {t.completed ? '✓' : ''}
                  </button>
                  <button type="button" className="cc-list__item" onClick={() => setEditing(t)}>
                    <span className={`task__prio tone-${prio.tone}`} title={`Öncelik: ${prio.label}`} />
                    <span className="cc-list__title">
                      {t.title}
                      {t.notes ? <span className="cc-muted">{t.notes}</span> : null}
                    </span>
                    {t.category ? <span className="task__cat">{t.category}</span> : null}
                    {due ? (
                      <span className={`task__due ${!t.completed && due < now ? 'is-overdue' : ''}`}>
                        {relativeDay(due, now)} {formatTime(due)}
                      </span>
                    ) : null}
                    {t.reminder_at ? <span title="Hatırlatıcı açık">🔔</span> : null}
                  </button>
                </li>
              );
            })}
          </ul>
        </Card>
      )}
    </div>
  );
}
