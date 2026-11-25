import { BUILTIN_THEMES } from '../../theme/engine.js';
// Masaüstü profilleri: hızlı geçiş, profil başına widget görünürlüğü, kedi davranışı,
// animasyon ve bildirim ayarları; özel profiller.
import { useCallback, useEffect, useState } from 'react';
import { Badge, Button, Card, ErrorNote, Select, Toggle } from '../../ui/components.jsx';
import { call, useTauriEvent } from '../../lib/ipc.js';
import { useWidgetDefinitions, useWidgets } from '../../lib/widgets.js';
import { ACTIVITY_LEVELS } from '../../pets/labels.js';
import { ANIMATION_OPTIONS } from './Accessibility.jsx';
import { PageHeader } from './PageHeader.jsx';

const WIDGET_MODES = [
  { value: 'keep', label: 'Widget görünürlüğüne dokunma' },
  { value: 'all', label: 'Tüm widget\'ları göster' },
  { value: 'kinds', label: 'Yalnızca seçili türleri göster' },
  { value: 'selected', label: 'Yalnızca seçili widget\'ları göster' },
];

const DESCRIPTIONS = {
  working: 'Üretkenlik widget\'ları açık, kediler sakin, animasyonlar azaltılmış.',
  gaming: 'Widget\'lar ve kediler gizli; animasyon ve bildirim yok. Arka plan işi en aza iner.',
  relaxing: 'Tüm widget\'lar açık, kediler daha hareketli.',
  focus: 'Yalnızca takvim, görev, odak, saat ve not; kediler yerinde, animasyon yok.',
};

function ProfileEditor({ profile, widgets, defs, onSaved, onError }) {
  const [name, setName] = useState(profile.name);
  const [cfg, setCfg] = useState(profile.config);
  const set = (patch) => setCfg((c) => ({ ...c, ...patch }));
  const toggleIn = (key, value) => set({ [key]: cfg[key].includes(value) ? cfg[key].filter((v) => v !== value) : [...cfg[key], value] });
  const byKind = Object.fromEntries(defs.map((d) => [d.kind, d]));

  const save = async () => {
    try {
      await call('update_profile', { id: profile.id, name: profile.builtin ? null : name, config: cfg });
      onSaved('Profil kaydedildi.');
    } catch (e) {
      onError(e.message);
    }
  };

  return (
    <div className="prof-editor">
      {!profile.builtin ? (
        <label className="wk-field">
          <span className="wk-field__label">Profil adı</span>
          <input className="wk-input" value={name} onChange={(e) => setName(e.target.value)} maxLength={40} />
        </label>
      ) : null}
      <Toggle label="Masaüstü widget'ları" description="Kapalıyken tüm widget pencereleri kapanır." checked={cfg.widgets_visible} onChange={(v) => set({ widgets_visible: v })} />
      <Select label="Widget görünürlüğü" value={cfg.widget_mode} options={WIDGET_MODES} onChange={(v) => set({ widget_mode: v })} description="Profil uygulanınca hangi widget'ların görüneceği. Konumlar değişmez." />
      {cfg.widget_mode === 'kinds' ? (
        <div className="cc-chips cc-gap">
          {defs.map((d) => (
            <button key={d.kind} type="button" className={`cc-chip ${cfg.widget_kinds.includes(d.kind) ? 'is-on' : ''}`} onClick={() => toggleIn('widget_kinds', d.kind)}>{d.name}</button>
          ))}
        </div>
      ) : null}
      {cfg.widget_mode === 'selected' ? (
        <div className="cc-chips cc-gap">
          {widgets.map((w) => (
            <button key={w.id} type="button" className={`cc-chip ${cfg.widget_ids.includes(w.id) ? 'is-on' : ''}`} onClick={() => toggleIn('widget_ids', w.id)}>
              {byKind[w.kind]?.name ?? w.kind} · {w.rect.x},{w.rect.y}
            </button>
          ))}
        </div>
      ) : null}
      <Toggle label="Masaüstü kedileri" checked={cfg.pets_enabled} onChange={(v) => set({ pets_enabled: v })} />
      <Select label="Kedi etkinliği" value={cfg.pet_activity} options={ACTIVITY_LEVELS} onChange={(v) => set({ pet_activity: v })} />
      <Toggle label="Tam ekranda kedileri gizle" checked={cfg.pets_hide_fullscreen} onChange={(v) => set({ pets_hide_fullscreen: v })} />
      <Select label="Animasyonlar" value={cfg.animation_quality} options={ANIMATION_OPTIONS} onChange={(v) => set({ animation_quality: v })} />
      <Toggle label="Bildirimler" checked={cfg.notifications_enabled} onChange={(v) => set({ notifications_enabled: v })} />
      <Select
        label="Tema"
        value={cfg.theme ? `${cfg.theme}|${cfg.theme_variant ?? ''}` : ''}
        options={[{ value: '', label: 'Değiştirme' }, ...BUILTIN_THEMES.flatMap((t) => [{ value: `${t.id}|`, label: t.name }, ...Object.entries(t.variants).map(([v, d]) => ({ value: `${t.id}|${v}`, label: `${t.name} · ${d.name}` }))])]}
        onChange={(v) => {
          const [theme, variant] = v ? v.split('|') : [null, null];
          set({ theme: theme || null, theme_variant: variant || null });
        }}
        description="Profil uygulanınca geçilecek tema. Kedi Evi ve widget yerleşimi değişmez."
      />
      <div className="cc-actions">
        <Button variant="primary" size="sm" onClick={save}>Kaydet</Button>
      </div>
    </div>
  );
}

export function ProfilesSection({ settingsApi }) {
  const [profiles, setProfiles] = useState(null);
  const [selected, setSelected] = useState(null);
  const [error, setError] = useState(null);
  const [note, setNote] = useState(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const { widgets } = useWidgets();
  const defs = useWidgetDefinitions();
  const s = settingsApi.settings;

  const load = useCallback(() => call('list_profiles').then(setProfiles, (e) => setError(e.message)), []);
  useEffect(() => {
    load();
  }, [load]);
  useTauriEvent('profiles:changed', load);

  const current = profiles?.find((p) => p.id === (selected ?? s?.active_profile)) ?? profiles?.[0];

  const apply = async (id) => {
    try {
      const p = await call('apply_profile', { id });
      setNote(`"${p.name}" profili uygulandı.`);
      setError(null);
    } catch (e) {
      setError(e.message);
    }
  };

  const create = async () => {
    try {
      const p = await call('create_profile', { name: 'Yeni profil', config: current?.config ?? {} });
      await load();
      setSelected(p.id);
    } catch (e) {
      setError(e.message);
    }
  };

  const remove = async () => {
    try {
      await call('delete_profile', { id: current.id });
      setSelected(null);
      setConfirmDelete(false);
    } catch (e) {
      setError(e.message);
    }
  };

  const reset = async () => {
    try {
      await call('reset_profile', { id: current.id });
      setNote('Profil varsayılan ayarlarına döndü.');
    } catch (e) {
      setError(e.message);
    }
  };

  return (
    <div className="cc-page">
      <PageHeader title="Profiller" subtitle="Tek dokunuşla çalışma, oyun, dinlenme ya da odak düzenine geç" actions={<Button onClick={create}>Yeni profil</Button>} />
      <ErrorNote error={error} />
      {note ? <p className="cc-note">{note}</p> : null}
      <div className="prof-list">
        {profiles?.map((p) => (
          <div key={p.id} className={`prof-card ${current?.id === p.id ? 'is-selected' : ''}`}>
            <button type="button" className="prof-card__body" onClick={() => setSelected(p.id)}>
              <span className="prof-card__name">{p.name}</span>
              <span className="cc-muted">{p.builtin ? DESCRIPTIONS[p.builtin] : 'Özel profil'}</span>
            </button>
            <div className="prof-card__foot">
              {s?.active_profile === p.id ? <Badge tone="mint">Etkin</Badge> : <Button size="sm" variant="primary" onClick={() => apply(p.id)}>Uygula</Button>}
            </div>
          </div>
        ))}
      </div>

      {current ? (
        <Card
          title={`${current.name} profilini düzenle`}
          action={
            current.builtin ? (
              <Button size="sm" variant="ghost" onClick={reset}>Varsayılana döndür</Button>
            ) : confirmDelete ? (
              <span className="cc-inline">
                <Button size="sm" variant="danger" onClick={remove}>Silmeyi onayla</Button>
                <Button size="sm" variant="ghost" onClick={() => setConfirmDelete(false)}>Vazgeç</Button>
              </span>
            ) : (
              <Button size="sm" variant="danger" onClick={() => setConfirmDelete(true)}>Sil</Button>
            )
          }
        >
          <ProfileEditor key={`${current.id}:${JSON.stringify(current.config)}:${current.name}`} profile={current} widgets={widgets ?? []} defs={defs} onSaved={setNote} onError={setError} />
          {s?.active_profile === current.id ? <p className="cc-muted">Değişikliklerin hemen etkili olması için kaydettikten sonra profili yeniden uygulayın.</p> : null}
        </Card>
      ) : null}

      {s ? (
        <Card title="Otomatik geçiş">
          <Toggle
            label="Tam ekranda Oyun profiline geç"
            description="Tam ekran bir uygulama öndeyken Oyun profili uygulanır, çıkınca önceki profile dönülür. Açıkken 3 saniyede bir hafif bir kontrol yapılır. Kenarlıksız pencere kipindeki bazı oyunlar algılanamayabilir."
            checked={s.auto_gaming_profile}
            onChange={(v) => settingsApi.update({ auto_gaming_profile: v })}
          />
        </Card>
      ) : null}
    </div>
  );
}
