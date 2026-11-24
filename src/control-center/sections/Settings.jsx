import { useCallback, useEffect, useState } from 'react';
import { Button, Card, Dialog, ErrorNote, Toggle } from '../../ui/components.jsx';
import { call } from '../../lib/ipc.js';
import { bytes } from '../../lib/format.js';
import { tr } from '../../lib/i18n.js';
import { PageHeader } from './PageHeader.jsx';
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
      setNote(found ? `Wakai Desktop ${found.version} yüklenebilir.` : 'Wakai Desktop güncel.');
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
    <Card title="Sürüm geçmişi">
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
        <p>Wakai Desktop yeniden başlatılacak ve {confirm} sürümüyle açılacak. Veritabanınız önce yedeklenir; widget'larınız, kedileriniz ve ayarlarınız korunur.</p>
        <p className="cc-muted">Otomatik güncelleme denetimi açıksa yeni sürüm yeniden önerilir; istemiyorsanız "Daha Sonra" deyin ya da denetimi kapatın.</p>
      </Dialog>
    </Card>
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
  const remove = (name) => call('delete_backup', { name }).then(reload, (e) => setError(e.message));
  const restore = (name) => call('restore_backup', { name }).catch((e) => setError(e.message));

  return (
    <Card title="Yedekler" action={<Button size="sm" onClick={create} disabled={busy}>Şimdi yedekle</Button>}>
      <p className="cc-muted">
        Takvim, görev, not ve widget ayarlarınızın tam kopyası. En fazla 20 yedek tutulur. Ayrıca her veritabanı güncellemesinden önce otomatik yedek alınır.
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
            {confirm === b.name ? (
              <>
                <span className="cc-muted">Uygulama yeniden başlatılacak. Mevcut veriler de önce yedeklenir.</span>
                <Button size="sm" variant="primary" onClick={() => restore(b.name)}>Geri yükle</Button>
                <Button size="sm" variant="ghost" onClick={() => setConfirm(null)}>Vazgeç</Button>
              </>
            ) : (
              <>
                <Button size="sm" onClick={() => setConfirm(b.name)}>Geri yükle…</Button>
                <Button size="sm" variant="ghost" onClick={() => remove(b.name)}>Sil</Button>
              </>
            )}
          </li>
        ))}
      </ul>
    </Card>
  );
}

export function SettingsSection({ settingsApi }) {
  const { settings, update, error } = settingsApi;
  const [info, setInfo] = useState(null);
  useEffect(() => {
    call('app_info').then(setInfo, console.error);
  }, []);
  if (!settings) return <div className="cc-page">{tr.common.loading}</div>;

  return (
    <div className="cc-page">
      <PageHeader title={tr.settings.title} />
      <ErrorNote error={error} />
      <Card title={tr.settings.general}>
        <Toggle label={tr.settings.closeToTray} description={tr.settings.closeToTrayDesc} checked={settings.close_to_tray} onChange={(v) => update({ close_to_tray: v })} />
        <Toggle label={tr.settings.launchOnStartup} description={tr.settings.launchOnStartupDesc} checked={settings.launch_on_startup} onChange={(v) => update({ launch_on_startup: v })} />
      </Card>
      <Updates settings={settings} update={update} />
      <VersionHistory />
      <Backups />
      <Card title="Hakkında">
        <dl className="cc-dl">
          <dt>Sürüm</dt>
          <dd>{info?.version ?? '—'}</dd>
          <dt>Veritabanı şeması</dt>
          <dd>{info ? `v${info.schema_version}` : '—'}</dd>
          <dt>Veri konumu</dt>
          <dd className="cc-dl__path">{info?.database_path ?? '—'}</dd>
          <dt>Hava durumu verisi</dt>
          <dd>Open-Meteo (open-meteo.com, CC BY 4.0)</dd>
        </dl>
        <p className="cc-muted">Wakai Desktop hesap gerektirmez; tüm verileriniz bu bilgisayarda kalır. İnternete yalnızca Hava Durumu Kedisi ve güncelleme denetimi bağlanır.</p>
      </Card>
    </div>
  );
}
