import { useState } from 'react';
import { ProgressBar, WidgetHeader, WidgetIconButton } from '../../ui/widget.jsx';
import { useTasks } from '../../lib/data.js';
import { call } from '../../lib/ipc.js';
import { useCelebration } from '../../lib/motion.js';
import { useNow } from '../../lib/time.js';
import { formatTime, parseLocal, relativeDay } from '../../lib/dates.js';
import { PRIORITIES, TaskForm } from './TaskForm.jsx';
import './tasks.css';

function dueLabel(task, now) {
  const d = parseLocal(task.due_at);
  if (!d) return null;
  const time = d.getHours() || d.getMinutes() ? ` ${formatTime(d)}` : '';
  return { text: `${relativeDay(d, now)}${time}`, overdue: !task.completed && d < now };
}

function TaskRow({ task, now, onEdit, onToggle }) {
  const due = dueLabel(task, now);
  const prio = PRIORITIES[task.priority] ?? PRIORITIES[1];
  return (
    <li className={`task ${task.completed ? 'is-done' : ''}`}>
      <button type="button" className="task__check" role="checkbox" aria-checked={task.completed} aria-label={task.completed ? 'Tamamlanmadı olarak işaretle' : 'Tamamlandı olarak işaretle'} onClick={() => onToggle(task)}>
        {task.completed ? '✓' : ''}
      </button>
      <button type="button" className="task__body" onClick={() => onEdit(task)}>
        <span className={`task__prio tone-${prio.tone}`} title={`Öncelik: ${prio.label}`} />
        <span className="task__title">{task.title}</span>
        {task.category ? <span className="task__cat">{task.category}</span> : null}
        {due ? <span className={`task__due ${due.overdue ? 'is-overdue' : ''}`}>{due.text}</span> : null}
      </button>
    </li>
  );
}

export default function TaskWidget({ config }) {
  const now = useNow('minute');
  const { data: tasks, error, setError } = useTasks();
  const [editing, setEditing] = useState(null); // null | 'new' | task
  const [quick, setQuick] = useState('');
  const [showDone, setShowDone] = useState(config.show_completed);
  const [celebrating, celebrate] = useCelebration(1100);

  const filter = config.category_filter?.trim();
  const all = (tasks ?? []).filter((t) => !filter || t.category === filter);
  const open = all.filter((t) => !t.completed);
  const done = all.filter((t) => t.completed);
  const progress = all.length ? done.length / all.length : 0;

  const toggle = async (task) => {
    try {
      await call('set_task_completed', { id: task.id, completed: !task.completed });
      if (!task.completed) celebrate();
    } catch (e) {
      setError(e.message);
    }
  };

  const addQuick = async (e) => {
    e.preventDefault();
    if (!quick.trim()) return;
    try {
      await call('create_task', { input: { title: quick, category: filter ?? '' } });
      setQuick('');
    } catch (err) {
      setError(err.message);
    }
  };

  return (
    <div className="tasks">
      <WidgetHeader
        cat={celebrating ? 'happy' : 'working'}
        celebrate={celebrating}
        title={filter ? `Görevler · ${filter}` : 'Görevler'}
        subtitle={all.length ? `${done.length}/${all.length} tamamlandı` : 'Liste boş'}
        action={
          <>
            <WidgetIconButton label={showDone ? 'Tamamlananları gizle' : 'Tamamlananları göster'} aria-pressed={showDone} onClick={() => setShowDone((v) => !v)}>
              {showDone ? '◉' : '◎'}
            </WidgetIconButton>
            <WidgetIconButton label="Ayrıntılı görev ekle" onClick={() => setEditing('new')}>＋</WidgetIconButton>
          </>
        }
      />
      <ProgressBar value={progress} label="Tamamlanma oranı" />

      {editing ? (
        <div className="tasks__editor">
          <TaskForm key={editing === 'new' ? 'new' : editing.id} task={editing === 'new' ? null : editing} onDone={() => setEditing(null)} compact />
        </div>
      ) : (
        <>
          <form className="tasks__quick" onSubmit={addQuick}>
            <input className="wg-input" value={quick} onChange={(e) => setQuick(e.target.value)} placeholder="Yeni görev yaz, Enter'a bas" maxLength={200} aria-label="Yeni görev" />
          </form>
          {error ? <p className="wg-error">{error}</p> : null}
          <ul className="tasks__list">
            {open.map((t) => (
              <TaskRow key={t.id} task={t} now={now} onEdit={setEditing} onToggle={toggle} />
            ))}
            {showDone ? done.map((t) => <TaskRow key={t.id} task={t} now={now} onEdit={setEditing} onToggle={toggle} />) : null}
          </ul>
          {tasks && open.length === 0 ? <p className="tasks__empty">{all.length ? 'Hepsi bitti! Kedi gurur duyuyor. 🐾' : 'Henüz görev yok.'}</p> : null}
        </>
      )}
    </div>
  );
}
