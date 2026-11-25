// "Yeni bir sürüm hazır!" penceresi. Güncelleme yalnızca kullanıcı onaylarsa indirilip kurulur.
import { useEffect, useState } from 'react';
import { Button, Dialog } from '../ui/components.jsx';
import { call, useTauriEvent } from '../lib/ipc.js';
import { bytes } from '../lib/format.js';

// "Daha Sonra" denen sürüm bu oturumda yeniden sorulmaz (elle denetim yine gösterir).
let dismissedVersion = null;

export function UpdateDialog() {
  const [update, setUpdate] = useState(null);
  const [progress, setProgress] = useState(null);
  const [error, setError] = useState(null);

  useEffect(() => {
    call('update_status').then((s) => {
      if (s.pending && s.pending.version !== dismissedVersion) setUpdate(s.pending);
    }, console.error);
  }, []);

  useTauriEvent('update:available', (info) => {
    if (info?.manual || info?.version !== dismissedVersion) {
      setError(null);
      setUpdate(info);
    }
  });
  useTauriEvent('update:progress', (p) => setProgress(p));

  const busy = progress !== null && !error;

  const later = () => {
    if (busy) return;
    dismissedVersion = update?.version ?? null;
    setUpdate(null);
  };

  const install = async () => {
    setError(null);
    setProgress({ downloaded: 0, total: null });
    try {
      await call('install_update');
    } catch (e) {
      setError(e.message);
      setProgress(null);
    }
  };

  const pct = progress?.total ? Math.min(100, Math.round((progress.downloaded / progress.total) * 100)) : null;

  return (
    <Dialog
      open={Boolean(update)}
      title="Yeni bir sürüm hazır!"
      onClose={later}
      dismissible={!busy}
      actions={
        <>
          <Button variant="ghost" onClick={later} disabled={busy}>Daha Sonra</Button>
          <Button variant="primary" onClick={install} disabled={busy}>Güncellemeyi İndir</Button>
        </>
      }
    >
      <p>
        Ivily Desktop <strong>{update?.version}</strong> yüklenebilir (şu anki sürüm: {update?.current}).
      </p>
      {update?.notes ? <div className="upd-notes">{update.notes}</div> : null}
      {busy ? (
        <div className="upd-progress" role="status">
          {progress.installing ? (
            <span>Kuruluyor… Ivily Desktop kısa süre kapanıp yeniden açılacak.</span>
          ) : (
            <>
              <div className="upd-bar">
                <div className="upd-bar__fill" style={{ width: `${pct ?? 5}%` }} />
              </div>
              <span className="cc-muted">
                İndiriliyor… {bytes(progress.downloaded)}
                {progress.total ? ` / ${bytes(progress.total)} (%${pct})` : ''}
              </span>
            </>
          )}
        </div>
      ) : null}
      {error ? <p className="wk-error">{error}</p> : null}
      <p className="cc-muted">Kurulumdan önce verileriniz otomatik olarak yedeklenir ve mevcut sürüm geri dönüş için saklanır.</p>
    </Dialog>
  );
}
