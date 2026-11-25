// Genel Bakış: masaüstünün, kedilerin ve günün özeti; en sık kullanılan işlemler tek tıkla.
import { useEffect, useState } from 'react';
import { Badge, Button, Card, Toggle } from '../../ui/components.jsx';
import { ColorDot, ProgressBar } from '../../ui/widget.jsx';
import { PixelIcon } from '../../ui/PixelIcon.jsx';
import { PetSprite } from '../../pets/PetSprite.jsx';
import { mood } from '../../pets/labels.js';
import { useTasks, useUpcomingEvents } from '../../lib/data.js';
import { useWidgets } from '../../lib/widgets.js';
import { call, useTauriEvent } from '../../lib/ipc.js';
import { useNow } from '../../lib/time.js';
import { eventTimeLabel, formatDayLong, parseLocal, relativeDay } from '../../lib/dates.js';
import { bytes } from '../../lib/format.js';
import { useThemeApi } from '../../theme/context.js';
import { PageHeader } from './PageHeader.jsx';
import '../../widgets/tasks/tasks.css';

function greeting(h) {
  if (h < 6) return 'İyi geceler';
  if (h < 12) return 'Günaydın';
  if (h < 18) return 'İyi günler';
  return 'İyi akşamlar';
}

function usePets() {
  const [pets, setPets] = useState(null);
  const load = () => {
    call('list_pets').then(setPets, () => setPets([]));
  };
  useEffect(load, []);
  useTauriEvent('pets:changed', load);
  return pets;
}

function useProfiles() {
  const [list, setList] = useState([]);
  useEffect(() => {
    call('list_profiles').then(setList, () => setList([]));
  }, []);
  return list;
}

function useAppUsage() {
  const [usage, setUsage] = useState(null);
  useEffect(() => {
    call('app_resource_usage').then(setUsage, () => {});
  }, []);
  return usage;
}

/** Tıklanabilir durum kutucuğu. */
function Tile({ icon, label, value, hint, onClick }) {
  return (
    <button type="button" className="ov-tile" onClick={onClick}>
      <span className="ov-tile__icon">
        <PixelIcon name={icon} size={18} />
      </span>
      <span className="ov-tile__text">
        <span className="ov-tile__value">{value}</span>
        <span className="ov-tile__label">{label}</span>
        {hint ? <span className="ov-tile__hint">{hint}</span> : null}
      </span>
    </button>
  );
}

export function Overview({ settingsApi, onNavigate }) {
  const now = useNow('minute');
  const { widgets } = useWidgets();
  const { data: tasks } = useTasks();
  const { data: upcoming } = useUpcomingEvents(4);
  const pets = usePets();
  const profiles = useProfiles();
  const usage = useAppUsage();
  const api = useThemeApi();
  const { settings: s, update } = settingsApi;
  if (!s) return null;

  const visible = widgets?.filter((w) => w.visible).length ?? 0;
  const open = tasks?.filter((t) => !t.completed) ?? [];
  const overdue = open.filter((t) => t.due_at && parseLocal(t.due_at) < now).length;
  const activeProfile = profiles.find((p) => p.id === s.active_profile);
  const homePets = pets?.filter((p) => p.location === 'home') ?? [];
  const needy = pets?.filter((p) => mood(p.needs) !== 'Keyifli' && mood(p.needs) !== 'Çok mutlu') ?? [];

  const applyProfile = async (id) => {
    try {
      await call('apply_profile', { id });
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="cc-page ov">
      <PageHeader title={`${greeting(now.getHours())}!`} subtitle={formatDayLong(now)} />

      <div className="ov-tiles">
        <Tile icon="widgets" label="Widget masaüstünde" value={`${s.widgets_visible ? visible : 0}/${widgets?.length ?? 0}`} hint={!s.widgets_visible ? 'Tümü gizli' : null} onClick={() => onNavigate('widgets')} />
        <Tile icon="layers" label="Tema" value={api?.compiled?.name ?? '—'} onClick={() => onNavigate('themes')} />
        <Tile icon="sliders" label="Profil" value={activeProfile?.name ?? 'Yok'} onClick={() => onNavigate('personalize', 'profiles')} />
        <Tile icon="paw" label="Kedi" value={pets?.length ?? 0} hint={needy.length ? `${needy.length} kedinin ihtiyacı var` : pets?.length ? 'Hepsi keyifli' : null} onClick={() => onNavigate('pethome', 'cats')} />
      </div>

      <div className="ov-grid">
        <Card className="ov-pets" title="Kedi Evi" action={<Button size="sm" variant="primary" onClick={() => onNavigate('pethome', 'world')}>Kedi Evi'ne git</Button>}>
          {pets?.length ? (
            <ul className="ov-pets__list">
              {pets.slice(0, 4).map((p) => (
                <li key={p.id} className="ov-pet">
                  <PetSprite breed={p.breed} anim={p.needs.rest < 25 ? 'sleep' : 'sit'} scale={2} accessories={[p.acc_head, p.acc_neck].filter(Boolean)} />
                  <div className="ov-pet__text">
                    <span className="ov-pet__name">{p.name}</span>
                    <span className="ov-pet__mood">
                      {mood(p.needs)} · {p.location === 'home' ? 'Kedi Evi’nde' : 'masaüstünde'}
                    </span>
                    <ProgressBar value={p.needs.satiety / 100} label="Tokluk" />
                  </div>
                </li>
              ))}
            </ul>
          ) : (
            <div className="ov-empty">
              <p className="cc-muted">Henüz kediniz yok. İlk kedinizi sahiplenin; masaüstünde ve Kedi Evi'nde yaşasın.</p>
              <Button size="sm" variant="primary" onClick={() => onNavigate('pethome', 'cats')}>
                Kedi sahiplen
              </Button>
            </div>
          )}
          {homePets.length ? <p className="cc-muted">{homePets.length} kedi şu an Kedi Evi'nde.</p> : null}
        </Card>

        <Card title="Bugün" action={<Button size="sm" variant="ghost" onClick={() => onNavigate('planner')}>Planlayıcı</Button>}>
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
          <p className="ov-sub">
            {open.length} açık görev{overdue ? ` · ${overdue} gecikmiş` : ''}
          </p>
          <ul className="cc-list">
            {open.slice(0, 4).map((t) => (
              <li key={t.id} className="cc-list__row">
                <button type="button" className="task__check" role="checkbox" aria-checked={false} aria-label={`${t.title} görevini tamamla`} onClick={() => call('set_task_completed', { id: t.id, completed: true })} />
                <span className="cc-list__title">{t.title}</span>
                {t.due_at ? <span className="cc-muted">{relativeDay(parseLocal(t.due_at), now)}</span> : null}
              </li>
            ))}
          </ul>
        </Card>

        <Card title="Hızlı işlemler">
          {profiles.length ? (
            <div className="ov-chips" role="group" aria-label="Profil seç">
              {profiles.map((p) => (
                <button key={p.id} type="button" className="ov-chip" aria-pressed={p.id === s.active_profile} onClick={() => applyProfile(p.id)}>
                  {p.name}
                </button>
              ))}
            </div>
          ) : null}
          <Toggle label="Widget'ları göster" checked={s.widgets_visible} onChange={(v) => update({ widgets_visible: v })} />
          <Toggle label="Masaüstü kedileri" checked={s.pets_enabled} onChange={(v) => update({ pets_enabled: v })} />
          <Toggle label="Düşük kaynak modu" checked={s.reduced_resource_mode} onChange={(v) => update({ reduced_resource_mode: v })} />
          <div className="cc-actions">
            <Button size="sm" onClick={() => onNavigate('widgets', 'store')} icon={<PixelIcon name="plus" size={12} />}>
              Widget ekle
            </Button>
          </div>
        </Card>

        <Card title="Uygulama" action={<Badge tone="mint">Çalışıyor</Badge>}>
          <dl className="cc-dl">
            <dt>Bellek</dt>
            <dd>{usage ? bytes(usage.private_bytes) : 'Ölçülüyor…'}</dd>
            <dt>Açık widget penceresi</dt>
            <dd>{usage?.widget_windows ?? '—'}</dd>
          </dl>
          <div className="cc-actions">
            <Button size="sm" variant="ghost" onClick={() => onNavigate('settings', 'performance')}>
              Performans
            </Button>
            <Button size="sm" variant="ghost" onClick={() => onNavigate('settings', 'updates')}>
              Güncellemeler
            </Button>
          </div>
        </Card>
      </div>
    </div>
  );
}
