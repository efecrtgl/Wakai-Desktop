// Kediler: koleksiyon, ihtiyaçlar, etkileşimler, aksesuarlar, başarımlar ve kedi ayarları.
import { useCallback, useEffect, useState } from 'react';
import { Badge, Button, Card, Dialog, EmptyState, ErrorNote, Select, Slider, Toggle, Segmented } from '../../ui/components.jsx';
import { ProgressBar } from '../../ui/widget.jsx';
import { PixelCat } from '../../ui/cats/PixelCat.jsx';
import { PetSprite } from '../../pets/PetSprite.jsx';
import { ACTIVITY_LEVELS, LOCK_HINTS, NEEDS, PERSONALITIES, mood } from '../../pets/labels.js';
import { call, useTauriEvent } from '../../lib/ipc.js';
import { PageHeader } from './PageHeader.jsx';

function usePetsData() {
  const [pets, setPets] = useState(null);
  const [catalog, setCatalog] = useState(null);
  const [progress, setProgress] = useState(null);
  const [error, setError] = useState(null);
  const reload = useCallback(() => {
    Promise.all([call('list_pets'), call('pet_catalog'), call('pet_progress')]).then(
      ([p, c, g]) => {
        setPets(p);
        setCatalog(c);
        setProgress(g);
      },
      (e) => setError(e.message),
    );
  }, []);
  useEffect(() => {
    reload();
    // İhtiyaçlar zamanla değişir; sayfa açıkken yarım dakikada bir tazele.
    const t = setInterval(reload, 30000);
    return () => clearInterval(t);
  }, [reload]);
  useTauriEvent('pets:changed', reload);
  return { pets, catalog, progress, error, setError, reload };
}

function AdoptDialog({ open, catalog, onClose, onError }) {
  const [name, setName] = useState('');
  const [breed, setBreed] = useState('pink');
  const [personality, setPersonality] = useState('playful');
  const [quirk, setQuirk] = useState('none');
  const [busy, setBusy] = useState(false);
  const breedName = Object.fromEntries(catalog?.breeds ?? []);

  const adopt = async () => {
    setBusy(true);
    try {
      await call('adopt_pet', { name: name.trim() || breedName[breed] || 'Kedi', breed, personality, quirk });
      setName('');
      onClose();
    } catch (e) {
      onError(e.message);
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title="Yeni bir kedi sahiplen"
      actions={
        <>
          <Button variant="ghost" onClick={onClose}>Vazgeç</Button>
          <Button variant="primary" onClick={adopt} disabled={busy}>Sahiplen</Button>
        </>
      }
    >
      <div className="pets-adopt">
        <div className="pets-adopt__preview">
          <PetSprite breed={breed} anim="idle" scale={5} animate interval={700} />
        </div>
        <label className="wk-field">
          <span className="wk-field__label">Adı</span>
          <input className="wk-input" value={name} onChange={(e) => setName(e.target.value)} placeholder={breedName[breed]} maxLength={24} />
        </label>
        <div className="wk-field">
          <span className="wk-field__label">Cins</span>
          <div className="pets-breeds" role="radiogroup" aria-label="Cins">
            {(catalog?.breeds ?? []).map(([id, label]) => (
              <button key={id} type="button" role="radio" aria-checked={breed === id} className="pets-breed" onClick={() => setBreed(id)} title={label}>
                <PetSprite breed={id} anim="sit" scale={2} />
                <span>{label}</span>
              </button>
            ))}
          </div>
        </div>
        <div className="wk-field">
          <span className="wk-field__label">Kişilik</span>
          <select className="wk-select" value={personality} onChange={(e) => setPersonality(e.target.value)}>
            {Object.entries(PERSONALITIES).map(([id, p]) => (
              <option key={id} value={id}>{p.name}</option>
            ))}
          </select>
          <span className="wk-field__hint">{PERSONALITIES[personality].desc}</span>
        </div>
        <div className="wk-field">
          <span className="wk-field__label">İkincil özellik</span>
          <select className="wk-select" value={quirk} onChange={(e) => setQuirk(e.target.value)}>
            {Object.entries(QUIRKS).map(([id, q]) => (
              <option key={id} value={id}>{q.name}</option>
            ))}
          </select>
          <span className="wk-field__hint">{QUIRKS[quirk].desc}</span>
        </div>
      </div>
    </Dialog>
  );
}

const QUIRKS = {
  none: { name: 'Yok', desc: 'Yalnızca ana kişiliği belirler.' },
  mischievous: { name: 'Yaramaz', desc: 'Oyuncakları dağıtır, tırmalamayı sever.' },
  shy: { name: 'Utangaç', desc: 'Kucağa alınmaktan çekinir, ürkünce saklanır; güveni yavaş kazanılır.' },
  lazy: { name: 'Tembel', desc: 'Uzun uzun dinlenir, az tırmanır.' },
};

/** Kedinin anıları (Kedi Evi'nde olanlar). */
function Memories({ id }) {
  const [list, setList] = useState(null);
  return (
    <details className="pet-card__memories" onToggle={(e) => e.currentTarget.open && !list && call('pet_memories', { id }).then(setList, () => setList([]))}>
      <summary>Anılar</summary>
      {list?.length ? (
        <ul>
          {list.map((m) => (
            <li key={m.at + m.text}>
              <span className="cc-muted">{m.at.slice(0, 10)}</span> {m.text}
            </li>
          ))}
        </ul>
      ) : (
        <p className="cc-muted">{list ? 'Henüz anı yok. Kedi Evi’nde vakit geçirdikçe oluşur.' : 'Yükleniyor…'}</p>
      )}
    </details>
  );
}

function PetCard({ pet, catalog, onError }) {
  const [anim, setAnim] = useState('sit');
  const [confirm, setConfirm] = useState(false);
  const [renaming, setRenaming] = useState(false);
  const [newName, setNewName] = useState(pet.name);
  const breedName = Object.fromEntries(catalog.breeds)[pet.breed] ?? pet.breed;
  const onDesktop = pet.location === 'desktop';
  const accessories = [pet.acc_head, pet.acc_neck].filter(Boolean);

  const act = async (cmd, args, react) => {
    try {
      await call(cmd, { id: pet.id, ...args });
      if (react) {
        setAnim(react);
        setTimeout(() => setAnim('sit'), 2200);
      }
    } catch (e) {
      onError(e.message);
    }
  };

  const rename = (e) => {
    e.preventDefault();
    setRenaming(false);
    if (newName.trim() && newName !== pet.name) act('rename_pet', { name: newName });
  };

  return (
    <article className="pet-card">
      <div className="pet-card__art">
        <PetSprite breed={pet.breed} anim={anim} accessories={accessories} scale={4} animate={anim !== 'sit'} interval={300} />
      </div>
      <div className="pet-card__head">
        {renaming ? (
          <form onSubmit={rename}>
            <input className="wk-input" value={newName} onChange={(e) => setNewName(e.target.value)} onBlur={rename} maxLength={24} autoFocus aria-label="Yeni ad" />
          </form>
        ) : (
          <h3 className="pet-card__name" onDoubleClick={() => setRenaming(true)} title="Yeniden adlandırmak için çift tıklayın">{pet.name}</h3>
        )}
        <p className="cc-muted">
          {breedName} · {PERSONALITIES[pet.personality]?.name}
          {pet.quirk && pet.quirk !== 'none' ? ` · ${QUIRKS[pet.quirk]?.name}` : ''} · {mood(pet.needs)}
        </p>
        <div className="pet-card__badges">
          <Badge tone={onDesktop ? 'mint' : 'lavender'}>{onDesktop ? 'Masaüstünde' : "Kedi Evi'nde"}</Badge>
          {pet.paused ? <Badge tone="gray">Yerinde duruyor</Badge> : null}
        </div>
      </div>

      <div className="pet-card__needs">
        {NEEDS.map((n) => (
          <div key={n.key} className="pet-need">
            <span>{n.label}</span>
            <ProgressBar value={pet.needs[n.key] / 100} label={n.label} />
            <span className="pet-need__value">{Math.round(pet.needs[n.key])}</span>
          </div>
        ))}
      </div>

      <div className="pet-card__actions">
        <Button size="sm" variant="primary" onClick={() => act('stroke_pet', {}, 'petted')}>Sev</Button>
        <select className="wk-select pet-select" value="" onChange={(e) => e.target.value && act('feed_pet', { food: e.target.value }, 'eat')} aria-label="Besle">
          <option value="">Besle…</option>
          {catalog.foods.map((f) => (
            <option key={f.id} value={f.id} disabled={!f.count}>{f.name} ({f.count})</option>
          ))}
        </select>
        <select className="wk-select pet-select" value="" onChange={(e) => e.target.value && act('play_with_pet', { toy: e.target.value }, 'play')} aria-label="Oyna">
          <option value="">Oyna…</option>
          {catalog.toys.filter((t) => t.unlocked).map((t) => (
            <option key={t.id} value={t.id}>{t.name}</option>
          ))}
        </select>
        {onDesktop ? <Button size="sm" onClick={() => call('summon_pet', { id: pet.id })} title="Kediyi imlecin yanına çağırır">Çağır</Button> : null}
      </div>

      <div className="pet-card__row">
        <Toggle label="Masaüstünde dolaşsın" checked={onDesktop} onChange={(v) => act('set_pet_location', { location: v ? 'desktop' : 'home' })} />
        <Toggle label="Yerinde dursun" description="Dolaşmaz; oturur, bakınır ya da uyur." checked={pet.paused} onChange={(v) => act('set_pet_paused', { paused: v })} />
      </div>

      <div className="pet-card__acc">
        {[
          ['head', 'Baş', pet.acc_head],
          ['neck', 'Boyun', pet.acc_neck],
        ].map(([slot, label, current]) => (
          <label key={slot} className="wk-field">
            <span className="wk-field__label">{label}</span>
            <select className="wk-select" value={current ?? ''} onChange={(e) => act('set_pet_accessory', { slot, accessory: e.target.value || null })}>
              <option value="">Yok</option>
              {catalog.accessories.filter((a) => a.slot === slot).map((a) => (
                <option key={a.id} value={a.id} disabled={!a.unlocked}>
                  {a.name}{a.unlocked ? '' : ` — kilitli: ${LOCK_HINTS[a.unlocked_by] ?? ''}`}
                </option>
              ))}
            </select>
          </label>
        ))}
      </div>

      <div className="pet-card__footer">
        {confirm ? (
          <>
            <span className="cc-muted">{pet.name} koleksiyondan çıkarılsın mı? Bu geri alınamaz.</span>
            <Button size="sm" variant="danger" onClick={() => act('release_pet')}>Evet</Button>
            <Button size="sm" variant="ghost" onClick={() => setConfirm(false)}>Vazgeç</Button>
          </>
        ) : (
          <Button size="sm" variant="ghost" onClick={() => setConfirm(true)}>Koleksiyondan çıkar</Button>
        )}
      </div>
      <Memories id={pet.id} />
    </article>
  );
}

function Achievements({ progress }) {
  if (!progress) return null;
  const done = progress.achievements.filter((a) => a.unlocked_at).length;
  return (
    <Card title="Başarımlar" action={<Badge tone="pink">{done}/{progress.achievements.length}</Badge>}>
      <p className="cc-muted cc-gap">
        {progress.feeds} besleme · {progress.strokes} sevgi · {progress.plays} oyun · {progress.feed_streak} gün üst üste besleme
      </p>
      <ul className="pets-ach">
        {progress.achievements.map((a) => (
          <li key={a.id} className={a.unlocked_at ? 'is-done' : ''}>
            <span className="pets-ach__icon" aria-hidden="true">{a.unlocked_at ? '★' : '☆'}</span>
            <span className="cc-list__title">
              {a.name}
              <span className="cc-muted">{a.description} Ödül: {a.reward}.</span>
            </span>
          </li>
        ))}
      </ul>
    </Card>
  );
}

function PetSettings({ settingsApi }) {
  const { settings: s, update } = settingsApi;
  if (!s) return null;
  return (
    <Card title="Kedi ayarları">
      <Toggle label="Masaüstü kedileri" description="Kapalıyken hiçbir kedi masaüstünde görünmez; Kedi Evi açık kalır." checked={s.pets_enabled} onChange={(v) => update({ pets_enabled: v })} />
      <Select label="Etkinlik düzeyi" value={s.pet_activity} options={ACTIVITY_LEVELS} onChange={(v) => update({ pet_activity: v })} description="Profiller bu değeri değiştirebilir." />
      <Slider label="Boyut" value={s.pet_size} min={2} max={5} format={(v) => ['', '', 'Küçük', 'Orta', 'Büyük', 'Çok büyük'][v]} onChange={(v) => update({ pet_size: v })} />
      <Slider label="Aynı anda en fazla" value={s.pets_max_active} min={1} max={12} format={(v) => `${v} kedi`} onChange={(v) => update({ pets_max_active: v })} description="Her kedi ayrı bir yerel pencere ve hareket hesabıdır." />
      <Toggle label="Tam ekranda gizle" description="Oyun, video ya da sunum gibi tam ekran bir uygulama öndeyken kediler gizlenir." checked={s.pets_hide_fullscreen} onChange={(v) => update({ pets_hide_fullscreen: v })} />
      <Segmented
        label="Kedi Evi saati"
        value={s.world_time}
        options={[
          { value: 'real', label: 'Gerçek saat' },
          { value: 'fast', label: 'Hızlı gün' },
        ]}
        onChange={(v) => update({ world_time: v })}
        description="Hızlı günde bir oyun saati bir dakika sürer; gece ve gündüz daha sık değişir."
      />
      <Toggle label="Kedi Evi'nde hava durumu" description="Pencereden yağmur ve kar görünür; kediler buna tepki verir." checked={s.world_weather} onChange={(v) => update({ world_weather: v })} />
      <Toggle anchor="set-ambient" label="Ortam efektleri" description="Temaya göre uçuşan toz, yaprak, yıldız ya da parıltılar. Hareket azaltıldığında zaten kapalıdır." checked={s.ambient_effects} onChange={(v) => update({ ambient_effects: v })} />
    </Card>
  );
}

export function PetsSection({ settingsApi }) {
  const { pets, catalog, progress, error, setError } = usePetsData();
  const [adopting, setAdopting] = useState(false);

  return (
    <div className="cc-page cc-page--wide">
      <PageHeader title="Kediler" subtitle="Masaüstünde dolaşan kedi dostların" actions={<Button variant="primary" onClick={() => setAdopting(true)}>Kedi sahiplen</Button>} />
      <ErrorNote error={error} />
      {catalog ? (
        <p className="cc-muted">
          Bugünkü yiyecekler: {catalog.foods.map((f) => `${f.name} ${f.count}`).join(' · ')} — her gün ücretsiz yenilenir.
        </p>
      ) : null}
      {pets && pets.length === 0 ? (
        <EmptyState cat={<PixelCat pose="curious" size={64} />} title="Henüz kedin yok">
          İlk kedini sahiplen; masaüstünde dolaşmaya başlasın.
        </EmptyState>
      ) : null}
      {catalog ? (
        <div className="pets-grid">
          {pets?.map((p) => (
            <PetCard key={p.id} pet={p} catalog={catalog} onError={setError} />
          ))}
        </div>
      ) : null}
      <div className="cc-grid">
        <Achievements progress={progress} />
        <PetSettings settingsApi={settingsApi} />
      </div>
      <Card>
        <p className="cc-muted">
          Masaüstünde: kediyi sevmek için fareyi üstünde ileri geri gezdir, taşımak için sürükle, menü için sağ tıkla. Uyuyan kedi fare geçince uyanmaz.
        </p>
      </Card>
      <AdoptDialog open={adopting} catalog={catalog} onClose={() => setAdopting(false)} onError={setError} />
    </div>
  );
}
