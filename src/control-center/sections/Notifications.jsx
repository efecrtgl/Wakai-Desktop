import { useState } from 'react';
import { Button, Card, ErrorNote, Toggle } from '../../ui/components.jsx';
import { call } from '../../lib/ipc.js';
import { PageHeader } from './PageHeader.jsx';

export function NotificationsSection({ settingsApi }) {
  const { settings: s, update, error } = settingsApi;
  const [result, setResult] = useState(null);
  if (!s) return null;

  const test = async () => {
    try {
      await call('send_test_notification');
      setResult('Bildirim gönderildi. Görmediyseniz Windows bildirim ayarlarını kontrol edin.');
    } catch (e) {
      setResult(e.message);
    }
  };

  return (
    <div className="cc-page">
      <PageHeader title="Bildirimler" subtitle="Hatırlatıcılar ve odak oturumları" />
      <ErrorNote error={error} />
      <Card title="Genel">
        <Toggle
          label="Bildirimleri etkinleştir"
          description="Kapalıyken hatırlatıcılar sessizce işaretlenir, bildirim gösterilmez."
          checked={s.notifications_enabled}
          onChange={(v) => update({ notifications_enabled: v })}
        />
        <div className="cc-actions">
          <Button size="sm" onClick={test} disabled={!s.notifications_enabled}>
            Deneme bildirimi gönder
          </Button>
          {result ? <span className="cc-muted">{result}</span> : null}
        </div>
      </Card>
      <Card title="Neler bildirilir?">
        <ul className="cc-bullets">
          <li>
            <strong>Takvim:</strong> her etkinlik için seçtiğiniz hatırlatma süresinde (etkinlik formundan ayarlanır).
          </li>
          <li>
            <strong>Görevler:</strong> hatırlatıcı zamanı verilmiş açık görevler.
          </li>
          <li>
            <strong>Odak Kedisi:</strong> çalışma ve mola bitişleri (widget ayarlarından kapatılabilir).
          </li>
        </ul>
        <p className="cc-muted">
          Uygulama kapalıyken kaçırılan etkinlik hatırlatıcıları, etkinlik henüz başlamadıysa ya da en fazla 1 saat önce başladıysa açılışta gösterilir.
        </p>
      </Card>
    </div>
  );
}
