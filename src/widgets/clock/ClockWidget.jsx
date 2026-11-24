import { useNow } from '../../lib/time.js';
import './clock.css';

const timeFormat = (cfg) =>
  new Intl.DateTimeFormat('tr-TR', {
    hour: '2-digit',
    minute: '2-digit',
    second: cfg.show_seconds ? '2-digit' : undefined,
    hour12: cfg.hour12,
  });

const dateFormat = new Intl.DateTimeFormat('tr-TR', { weekday: 'long', day: 'numeric', month: 'long' });

export default function ClockWidget({ config }) {
  const now = useNow(config.show_seconds ? 'second' : 'minute');
  return (
    <div className="clock">
      <time className="clock__time" dateTime={now.toISOString()}>
        {timeFormat(config).format(now)}
      </time>
      {config.show_date ? <p className="clock__date">{dateFormat.format(now)}</p> : null}
    </div>
  );
}
