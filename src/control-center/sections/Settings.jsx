import { useCallback, useEffect, useState } from 'react';
import { Badge, Button, Card, Dialog, ErrorNote, Loading, Segmented, Toggle } from '../../ui/components.jsx';
import { call } from '../../lib/ipc.js';
import { bytes } from '../../lib/format.js';
import { tr } from '../../lib/i18n.js';
import { SectionTabs } from './SectionTabs.jsx';
import { NAV } from '../nav.js';
import { AccessibilitySettings } from './Accessibility.jsx';
import { PerformanceSection } from './Performance.jsx';
import { NotificationsSection } from './Notifications.jsx';
import { RELEASE_NOTES } from '../../lib/releaseNotes.js';

function Updates({ settings, update }) {
  const [status, setStatus] = useState(null);
  const [busy, setBusy] = useState(false);
  const [note, setNote] = useState(null);
  const [error, setError] = useState(null);

  const reload = useCallback(() => call('update_status').then(setStatus, (e) => setError(e.message)), []);
  useEffect(() => {
    reload();
  }, [reload]);

  const check = async () => {
    setBusy(true);
    setError(null);
    setNote(null);
    try {
      const found = await call('check_for_updates');
      setNote(found ? `Ivily Desktop ${found.version} yüklenebilir.` : 'Ivily Desktop güncel.');
      await reload();
    } catch (e) {
      setError(e.message);
    } finally {
      setBusy(false);
    }
  };

  const last = status?.last_check ? new Date(status.last_check).toLocaleString('tr-TR', { dateStyle: 'medium', timeStyle: 'short' }) : 'Henüz denetlenmedi';

  return (
    <Card title="Güncellemeler" action={<Button size="sm" onClick={check} disabled={busy}>{busy ? 'Denetleniyor…' : 'Güncellemeleri denetle'}</Button>}>
      <Toggle
        label="Güncellemeleri otomatik denetle"
        description="Açılıştan kısa süre sonra, en fazla günde bir kez arka planda denetlenir. Güncelleme sizin onayınız olmadan asla kurulmaz."
        checked={settings.auto_update_check}
        onChange={(v) => update({ auto_update_check: v })}
      />
      <Segmented
        label="Güncelleme kanalı"
        value={settings.update_channel}
        options={[
          { value: 'stable', label: 'Kararlı' },
          { value: 'prerelease', label: 'Ön sürüm' },
        ]}
        onChange={(v) => update({ update_channel: v })}
        description={settings.update_channel === 'prerelease' ? 'Ön sürümler yeni özellikleri erken getirir ama hata içerebilir. Kararlı kanala dönebilirsiniz; kurulu sürüm geri alınmaz (Sürüm geçmişi’nden dönülebilir).' : 'Yalnızca kararlı sürümler önerilir. Ön sürümler size asla kendiliğinden önerilmez.'}
      />
      <dl className="cc-dl">
        <dt>Şu anki sürüm</dt>
        <dd>{status?.current ?? '—'}</dd>
        <dt>Son denetim</dt>
        <dd>{last}</dd>
      </dl>
      {note ? <p className="cc-note">{note}</p> : null}
      <ErrorNote error={error} />
      <p className="cc-muted">Güncellemeler GitHub Releases üzerinden indirilir ve kurulmadan önce dijital imzaları doğrulanır.</p>
    </Card>
  );
}

const fmtDate = (ms) => new Date(ms).toLocaleDateString('tr-TR', { day: 'numeric', month: 'long', year: 'numeric' });

function VersionHistory() {
  const [data, setData] = useState(null);
  const [error, setError] = useState(null);
  const [confirm, setConfirm] = useState(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    call('version_history').then(setData, (e) => setError(e.message));
  }, []);

  const rollback = async () => {
    setBusy(true);
    try {
      await call('rollback_to_version', { version: confirm });
    } catch (e) {
      setError(e.message);
      setBusy(false);
      setConfirm(null);
    }
  };

  const notes = data ? RELEASE_NOTES[data.current] : null;

  return (
    <Card title="Sürüm geçmişi" id="set-versions">
      <ErrorNote error={error} />
      {notes ? (
        <div className="ver-notes">
          <strong>{data.current} yenilikleri — {notes.title}</strong>
          <ul>
            {notes.items.map((t) => <li key={t}>{t}</li>)}
          </ul>
        </div>
      ) : null}
      {data?.history?.length ? (
        <dl className="cc-dl">
          {data.history.map((h) => (
            <div key={h.version} className="ver-row">
              <dt>{h.version}{h.version === data.current ? ' (şu anki)' : ''}</dt>
              <dd>{fmtDate(h.first_seen)} tarihinden beri · {h.launches} açılış</dd>
            </div>
          ))}
        </dl>
      ) : null}
      {data?.previous?.length ? (
        <>
          <p className="cc-muted">Güncellemeden önce saklanan sürümler. Yeni sürümde sorun yaşarsanız önceki sürüme dönebilirsiniz; verileriniz korunur (önce yedek alınır).</p>
          <div className="cc-actions">
            {data.previous.map((p) => (
              <Button key={p.version} size="sm" onClick={() => setConfirm(p.version)} disabled={busy}>{p.version} sürümüne dön</Button>
            ))}
          </div>
        </>
      ) : (
        <p className="cc-muted">Henüz saklanmış önceki sürüm yok. Bir güncelleme kurulduğunda, kurulumdan önceki sürüm geri dönüş için saklanır.</p>
      )}
      <Dialog
        open={Boolean(confirm)}
        title={`${confirm} sürümüne dönülsün mü?`}
        onClose={() => !busy && setConfirm(null)}
        dismissible={!busy}
        actions={
          <>
            <Button variant="ghost" onClick={() => setConfirm(null)} disabled={busy}>Vazgeç</Button>
            <Button variant="primary" onClick={rollback} disabled={busy}>{busy ? 'Dönülüyor…' : 'Önceki sürüme dön'}</Button>
          </>
        }
      >
        <p>Ivily Desktop yeniden başlatılacak ve {confirm} sürümüyle açılacak. Veritabanınız önce yedeklenir; widget'larınız, kedileriniz ve ayarlarınız korunur.</p>
        <p className="cc-muted">Otomatik güncelleme denetimi açıksa yeni sürüm yeniden önerilir; istemiyorsanız "Daha Sonra" deyin ya da denetimi kapatın.</p>
      </Dialog>
    </Card>
  );
}

const BACKUP_CATEGORIES = {
  settings: 'Ayarlar',
  widgets: "Widget'lar ve konumları",
  calendar: 'Takvim etkinlikleri',
  tasks: 'Görevler',
  notes: 'Notlar',
  shopping: 'Alışveriş listeleri',
  focus: 'Odak geçmişi',
  pets: 'Kediler ve Kedi Evi',
  profiles: 'Profiller',
};

function RestoreDialog({ name, onClose, onError }) {
  const [info, setInfo] = useState(null);
  const [mode, setMode] = useState('all');
  const [picked, setPicked] = useState([]);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    call('inspect_backup', { name }).then(setInfo, (e) => onError(e.message));
  }, [name, onError]);

  const toggle = (id) => setPicked((p) => (p.includes(id) ? p.filter((x) => x !== id) : [...p, id]));

  const restore = async () => {
    setBusy(true);
    try {
      if (mode === 'all') await call('restore_backup', { name });
      else await call('restore_backup_selective', { name, categories: picked });
    } catch (e) {
      onError(e.message);
      setBusy(false);
      onClose();
    }
  };

  const ready = info?.valid && (mode === 'all' || picked.length > 0);

  return (
    <Dialog
      open
      title="Yedeği geri yükle"
      onClose={() => !busy && onClose()}
      dismissible={!busy}
      actions={
        <>
          <Button variant="ghost" onClick={onClose} disabled={busy}>Vazgeç</Button>
          <Button variant="primary" onClick={restore} disabled={!ready || busy}>{busy ? 'Yeniden başlatılıyor…' : 'Geri yükle'}</Button>
        </>
      }
    >
      {!info ? <Loading label="Yedek denetleniyor…" /> : null}
      {info && !info.valid ? <p className="wk-error">Bu yedek bozuk ya da geçerli bir Ivily Desktop yedeği değil; geri yüklenemez.</p> : null}
      {info?.valid ? (
        <>
          <p>
            <strong>{info.created}</strong> · {bytes(info.size)} · <Badge tone="mint">Bütünlük denetimi geçti</Badge>
          </p>
          {info.newer ? (
            <p className="cc-note">Bu yedek daha yeni bir Ivily Desktop sürümüyle alınmış. Bu sürümün tanımadığı veriler korunur ama kullanılmaz.</p>
          ) : info.schema_version < info.supported_version ? (
            <p className="cc-muted">Daha eski bir sürümden alınmış; geri yüklendikten sonra otomatik olarak güncellenir.</p>
          ) : null}
          <div className="cc-chips cc-gap">
            <button type="button" className={`cc-chip ${mode === 'all' ? 'is-on' : ''}`} onClick={() => setMode('all')}>Tümünü geri yükle</button>
            <button type="button" className={`cc-chip ${mode === 'pick' ? 'is-on' : ''}`} onClick={() => setMode('pick')}>Yalnızca seçtiklerim</button>
          </div>
          {mode === 'pick' ? (
            <ul className="bk-cats">
              {info.categories.map((c) => (
                <li key={c.id}>
                  <label className={c.count === null ? 'is-disabled' : ''}>
                    <input type="checkbox" checked={picked.includes(c.id)} disabled={c.count === null} onChange={() => toggle(c.id)} />
                    <span>{BACKUP_CATEGORIES[c.id] ?? c.id}</span>
                    <span className="cc-muted">{c.count === null ? 'bu yedekte yok' : `${c.count} kayıt`}</span>
                  </label>
                </li>
              ))}
            </ul>
          ) : null}
          <p className="cc-muted">
            {mode === 'all'
              ? 'Tüm veriler yedekteki hâline döner.'
              : 'Yalnızca seçilen kategoriler yedekteki hâline döner; diğer verileriniz olduğu gibi kalır.'}{' '}
            Uygulama yeniden başlatılır ve mevcut veriler önce ayrıca yedeklenir.
          </p>
        </>
      ) : null}
    </Dialog>
  );
}

function Backups() {
  const [list, setList] = useState(null);
  const [error, setError] = useState(null);
  const [confirm, setConfirm] = useState(null);
  const [busy, setBusy] = useState(false);

  const reload = useCallback(() => call('list_backups').then(setList, (e) => setError(e.message)), []);
  useEffect(() => {
    reload();
  }, [reload]);

  const create = async () => {
    setBusy(true);
    try {
      await call('create_backup');
      await reload();
      setError(null);
    } catch (e) {
      setError(e.message);
    } finally {
      setBusy(false);
    }
  };
  const closeDialog = useCallback(() => setConfirm(null), []);
  const remove = (name) => call('delete_backup', { name }).then(reload, (e) => setError(e.message));

  return (
    <Card title="Yedekler" action={<Button size="sm" onClick={create} disabled={busy}>Şimdi yedekle</Button>}>
      <p className="cc-muted">
        Tüm verilerinizin (widget'lar, takvim, görevler, notlar, alışveriş, kediler, profiller, ayarlar) doğrulanmış tam kopyası. En fazla 20 yedek tutulur. Veritabanı ve uygulama güncellemelerinden önce otomatik yedek alınır. Geri yüklerken tümünü ya da yalnızca seçtiğiniz kategorileri geri alabilirsiniz.
      </p>
      <ErrorNote error={error} />
      {list?.length === 0 ? <p className="cc-muted">Henüz yedek yok.</p> : null}
      <ul className="cc-list">
        {list?.map((b) => (
          <li key={b.name} className="cc-backup">
            <span className="cc-list__title">
              {b.created}
              <span className="cc-muted">
                {b.name} · {bytes(b.size)}
              </span>
            </span>
            <Button size="sm" onClick={() => setConfirm(b.name)}>Geri yükle…</Button>
            <Button size="sm" variant="ghost" onClick={() => remove(b.name)}>Sil</Button>
          </li>
        ))}
      </ul>
      {confirm ? <RestoreDialog name={confirm} onClose={closeDialog} onError={setError} /> : null}
    </Card>
  );
}

function About() {
  const [info, setInfo] = useState(null);
  useEffect(() => {
    call('app_info').then(setInfo, console.error);
  }, []);
  return (
    <Card title="Ivily Desktop">
      <dl className="cc-dl">
        <dt>Sürüm</dt>
        <dd>{info?.version ?? '—'}</dd>
        <dt>Veritabanı şeması</dt>
        <dd>{info ? `v${info.schema_version}` : '—'}</dd>
        <dt>Veri konumu</dt>
        <dd className="cc-dl__path">{info?.database_path ?? '—'}</dd>
        <dt>Eski adı</dt>
        <dd>Wakai Desktop (1.1.0 ve öncesi)</dd>
        <dt>Hava durumu verisi</dt>
        <dd>Open-Meteo (open-meteo.com, CC BY 4.0)</dd>
        <dt>Piksel yazı tipi</dt>
        <dd>Pixelify Sans (SIL Open Font License 1.1)</dd>
      </dl>
      <p className="cc-muted">Ivily Desktop hesap gerektirmez; tüm verileriniz bu bilgisayarda kalır. İnternete yalnızca Hava Durumu Kedisi ve güncelleme denetimi bağlanır.</p>
    </Card>
  );
}

function General({ settings, update }) {
  return (
    <>
      <Card title="Pencere ve başlangıç">
        <Toggle anchor="set-tray" label={tr.settings.closeToTray} description={tr.settings.closeToTrayDesc} checked={settings.close_to_tray} onChange={(v) => update({ close_to_tray: v })} />
        <Toggle anchor="set-startup" label={tr.settings.launchOnStartup} description={tr.settings.launchOnStartupDesc} checked={settings.launch_on_startup} onChange={(v) => update({ launch_on_startup: v })} />
      </Card>
      <Card title="Masaüstü">
        <Toggle label="Widget'ları masaüstünde göster" description="Kapatınca tüm widget pencereleri kapanır ve kaynak kullanmaz." checked={settings.widgets_visible} onChange={(v) => update({ widgets_visible: v })} />
        <Toggle label="Masaüstü kedileri" description="Kediler görev çubuğunda ve pencerelerin üstünde dolaşır." checked={settings.pets_enabled} onChange={(v) => update({ pets_enabled: v })} />
      </Card>
    </>
  );
}

export function SettingsSection({ settingsApi, route, onNavigate }) {
  const { settings, update, error } = settingsApi;
  const tabs = NAV.find((n) => n.id === 'settings').tabs;
  const sub = route?.sub ?? 'general';
  let body;
  if (!settings) body = <Loading />;
  else if (sub === 'general') body = <General settings={settings} update={update} />;
  else if (sub === 'accessibility') body = <AccessibilitySettings settingsApi={settingsApi} />;
  else if (sub === 'performance') body = <PerformanceSection settingsApi={settingsApi} />;
  else if (sub === 'notifications') body = <NotificationsSection settingsApi={settingsApi} />;
  else if (sub === 'updates')
    body = (
      <>
        <Updates settings={settings} update={update} />
        <VersionHistory />
      </>
    );
  else if (sub === 'backup') body = <Backups />;
  else body = <About />;
  return (
    <SectionTabs title={tr.settings.title} tabs={tabs} active={sub} onChange={(t) => onNavigate('settings', t)} vertical>
      <ErrorNote error={error} />
      {body}
    </SectionTabs>
  );
}
