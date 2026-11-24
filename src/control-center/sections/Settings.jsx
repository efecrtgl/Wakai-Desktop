import { useCallback, useEffect, useState } from 'react';
import { Button, Card, ErrorNote, Toggle } from '../../ui/components.jsx';
import { call } from '../../lib/ipc.js';
import { bytes } from '../../lib/format.js';
import { tr } from '../../lib/i18n.js';
import { PageHeader } from './PageHeader.jsx';

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
        <p className="cc-muted">Wakai Desktop hesap gerektirmez; tüm verileriniz bu bilgisayarda kalır. Yalnızca Hava Durumu Kedisi internete bağlanır.</p>
      </Card>
    </div>
  );
}
