// Saat Kedisi. Güncellemeler bir sonraki saniye/dakika sınırına hizalanır ve yalnızca
// bu widget'ı yeniden çizer (her widget kendi penceresinde, kendi React ağacındadır).
import { memo } from 'react';
import { useNow } from '../../lib/time.js';
import { formatDayLong } from '../../lib/dates.js';
import { PixelCat } from '../../ui/cats/PixelCat.jsx';
import './clock.css';

const formatters = new Map();
function timeFormatter(cfg, timeZone) {
  const key = `${cfg.show_seconds}|${cfg.hour12}|${timeZone ?? ''}`;
  if (!formatters.has(key)) {
    try {
      formatters.set(
        key,
        new Intl.DateTimeFormat('tr-TR', {
          hour: '2-digit',
          minute: '2-digit',
          second: cfg.show_seconds && !timeZone ? '2-digit' : undefined,
          hour12: cfg.hour12,
          timeZone,
        }),
      );
    } catch {
      // Geçersiz saat dilimi adı: yerel saatle göster.
      return timeFormatter(cfg, undefined);
    }
  }
  return formatters.get(key);
}

function zoneLabel(tz) {
  return tz.split('/').pop().replaceAll('_', ' ');
}

const AnalogFace = memo(function AnalogFace({ h, m, s, showSeconds }) {
  const hourAngle = (h % 12) * 30 + m * 0.5;
  const minuteAngle = m * 6 + (showSeconds ? s * 0.1 : 0);
  return (
    <svg className="clock__analog" viewBox="0 0 100 100" aria-hidden="true">
      <circle cx="50" cy="50" r="46" className="clock__dial" />
      {Array.from({ length: 12 }, (_, i) => (
        <rect key={i} x="48.5" y="7" width="3" height={i % 3 === 0 ? 8 : 4} rx="1" className="clock__tick" transform={`rotate(${i * 30} 50 50)`} />
      ))}
      <rect x="47" y="24" width="6" height="30" rx="3" className="clock__hand clock__hand--h" transform={`rotate(${hourAngle} 50 50)`} />
      <rect x="48" y="13" width="4" height="41" rx="2" className="clock__hand clock__hand--m" transform={`rotate(${minuteAngle} 50 50)`} />
      {showSeconds ? <rect x="49.25" y="10" width="1.5" height="46" className="clock__hand--s" transform={`rotate(${s * 6} 50 50)`} /> : null}
      <circle cx="50" cy="50" r="3.5" className="clock__pin" />
    </svg>
  );
});

export default function ClockWidget({ config }) {
  const now = useNow(config.show_seconds ? 'second' : 'minute');
  const zones = Array.isArray(config.time_zones) ? config.time_zones.filter((z) => typeof z === 'string' && z) : [];
  const analog = config.mode === 'analog';

  return (
    <div className={`clock ${analog ? 'clock--analog' : ''}`}>
      <div className="clock__main">
        {analog ? (
          <AnalogFace h={now.getHours()} m={now.getMinutes()} s={now.getSeconds()} showSeconds={config.show_seconds} />
        ) : (
          <time className="clock__time" dateTime={now.toISOString()}>
            {timeFormatter(config).format(now)}
          </time>
        )}
        {config.show_date ? (
          <p className="clock__date">
            <PixelCat pose="sitting" size={18} /> {formatDayLong(now)}
          </p>
        ) : null}
      </div>
      {zones.length ? (
        <ul className="clock__zones">
          {zones.map((tz) => (
            <li key={tz}>
              <span>{zoneLabel(tz)}</span>
              <span className="clock__zone-time">{timeFormatter(config, tz).format(now)}</span>
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}
