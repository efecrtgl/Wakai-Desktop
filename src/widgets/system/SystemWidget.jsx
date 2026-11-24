// Sistem Kedisi. Değerler Rust'taki izleme iş parçacığından gelir; bu widget açık değilken
// örnekleme yapılmaz. GPU sayacı okunamazsa uydurma değer yerine "ölçülemiyor" gösterilir.
import { ProgressBar, WidgetHeader } from '../../ui/widget.jsx';
import { useSystemStats } from '../../lib/data.js';
import { bytes, duration, percent } from '../../lib/format.js';
import './system.css';

function Meter({ label, value, detail }) {
  return (
    <div className="sys__meter">
      <div className="sys__row">
        <span className="sys__label">{label}</span>
        <span className="sys__value">{percent(value)}</span>
      </div>
      <ProgressBar value={value == null ? 0 : value / 100} label={label} />
      {detail ? <span className="sys__detail">{detail}</span> : null}
    </div>
  );
}

export default function SystemWidget({ config }) {
  const s = useSystemStats();
  const busy = s?.cpu != null && s.cpu > 70;

  if (!s) {
    return (
      <div className="sys">
        <WidgetHeader cat="sleeping" title="Sistem" subtitle="Ölçülüyor…" />
      </div>
    );
  }

  const memPct = s.memory_total ? (s.memory_used / s.memory_total) * 100 : null;
  const diskPct = s.disk_total ? (s.disk_used / s.disk_total) * 100 : null;

  return (
    <div className="sys">
      <WidgetHeader cat={busy ? 'working' : 'relaxed'} title="Sistem" subtitle={`Açık kalma: ${duration(s.uptime_secs)}`} />
      <Meter label="İşlemci" value={s.cpu} />
      <Meter label="Bellek" value={memPct} detail={`${bytes(s.memory_used)} / ${bytes(s.memory_total)}`} />
      {config.show_gpu ? (
        s.gpu == null ? (
          <div className="sys__meter">
            <div className="sys__row">
              <span className="sys__label">GPU</span>
              <span className="sys__value sys__na" title="GPU performans sayaçları bu sistemde okunamıyor">Ölçülemiyor</span>
            </div>
          </div>
        ) : (
          <Meter label="GPU (3D)" value={s.gpu} />
        )
      ) : null}
      {config.show_disk ? <Meter label="Disk" value={diskPct} detail={`${bytes(s.disk_used)} / ${bytes(s.disk_total)}`} /> : null}
      <p className="sys__foot">{s.interval_secs} saniyede bir güncellenir</p>
    </div>
  );
}
