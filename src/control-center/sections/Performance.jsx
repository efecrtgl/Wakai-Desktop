import { useEffect, useState } from 'react';
import { Card, ErrorNote, Select, Slider, Toggle } from '../../ui/components.jsx';
import { ProgressBar } from '../../ui/widget.jsx';
import { call } from '../../lib/ipc.js';
import { useSystemStats } from '../../lib/data.js';
import { bytes, percent } from '../../lib/format.js';
import { ANIMATION_OPTIONS } from './Appearance.jsx';
import { PageHeader } from './PageHeader.jsx';

/** Wakai Desktop'un kendi bellek kullanımı (ana süreç + WebView2 süreçleri). Sayfa açıkken 5 sn'de bir. */
function AppUsage() {
  const [usage, setUsage] = useState(null);
  useEffect(() => {
    let alive = true;
    const load = () => call('app_resource_usage').then((u) => alive && setUsage(u), console.error);
    load();
    const t = setInterval(load, 5000);
    return () => {
      alive = false;
      clearInterval(t);
    };
  }, []);
  if (!usage) return <p className="cc-muted">Ölçülüyor…</p>;
  return (
    <dl className="cc-dl">
      <dt>Toplam bellek (özel)</dt>
      <dd>{bytes(usage.private_bytes)}</dd>
      <dt>Süreç sayısı</dt>
      <dd>{usage.processes} (ana süreç + WebView2)</dd>
      <dt>Açık widget penceresi</dt>
      <dd>{usage.widget_windows}</dd>
    </dl>
  );
}

export function PerformanceSection({ settingsApi }) {
  const { settings: s, update, error } = settingsApi;
  const stats = useSystemStats();
  if (!s) return null;
  return (
    <div className="cc-page">
      <PageHeader title="Performans" subtitle="Wakai Desktop arka planda hafif kalacak şekilde tasarlandı" />
      <ErrorNote error={error} />
      <Card title="Kaynak kullanımı">
        <Toggle
          label="Düşük kaynak modu"
          description="Animasyonları kapatır ve sistem izleme aralığını iki katına çıkarır."
          checked={s.reduced_resource_mode}
          onChange={(v) => update({ reduced_resource_mode: v })}
        />
        <Select label="Animasyon kalitesi" value={s.animation_quality} options={ANIMATION_OPTIONS} onChange={(v) => update({ animation_quality: v })} />
        <Slider
          label="Sistem izleme aralığı"
          value={s.monitor_interval_secs}
          min={1}
          max={30}
          format={(v) => `${v} sn`}
          onChange={(v) => update({ monitor_interval_secs: v })}
          description="Sistem Kedisi ya da bu sayfa açıkken ölçüm sıklığı. İkisi de kapalıyken hiç ölçüm yapılmaz."
        />
        <Slider
          label="Hava durumu yenileme"
          value={s.weather_refresh_minutes}
          min={15}
          max={180}
          step={15}
          format={(v) => `${v} dk`}
          onChange={(v) => update({ weather_refresh_minutes: v })}
        />
      </Card>
      <Card title="Başlangıç">
        <Toggle
          label="Windows ile başlat"
          description="Oturum açınca widget'lar geri gelir; Kontrol Merkezi açılmaz."
          checked={s.launch_on_startup}
          onChange={(v) => update({ launch_on_startup: v })}
        />
      </Card>
      <div className="cc-grid">
        <Card title="Wakai Desktop">
          <AppUsage />
        </Card>
        <Card title="Sistem">
          {stats ? (
            <div className="cc-meters">
              <span>İşlemci {percent(stats.cpu)}</span>
              <ProgressBar value={(stats.cpu ?? 0) / 100} label="İşlemci" />
              <span>
                Bellek {bytes(stats.memory_used)} / {bytes(stats.memory_total)}
              </span>
              <ProgressBar value={stats.memory_total ? stats.memory_used / stats.memory_total : 0} label="Bellek" />
              <span>GPU {stats.gpu == null ? 'ölçülemiyor' : percent(stats.gpu)}</span>
              <ProgressBar value={(stats.gpu ?? 0) / 100} label="GPU" />
            </div>
          ) : (
            <p className="cc-muted">Ölçülüyor…</p>
          )}
        </Card>
      </div>
    </div>
  );
}
