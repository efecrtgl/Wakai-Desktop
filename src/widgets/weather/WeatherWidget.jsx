// Hava Durumu Kedisi. Veri Rust'ta önbelleğe alınır; bu widget yalnızca ayarlardaki
// yenileme aralığında (varsayılan 30 dk) veri ister. Çevrim dışıyken son veriyi gösterir.
import { useEffect, useState } from 'react';
import { WidgetHeader, WidgetIconButton } from '../../ui/widget.jsx';
import { PixelIcon } from '../../ui/PixelIcon.jsx';
import { call } from '../../lib/ipc.js';
import { useSettings } from '../../lib/settings.js';
import { WEEKDAYS_SHORT, parseLocal, weekdayIndex, formatTime } from '../../lib/dates.js';
import { temperature } from '../../lib/format.js';
import { describe } from './codes.js';
import './weather.css';

function LocationPicker({ id, onPicked, onCancel }) {
  const [q, setQ] = useState('');
  const [results, setResults] = useState(null);
  const [error, setError] = useState(null);
  const [busy, setBusy] = useState(false);

  const search = async (e) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      setResults(await call('weather_search', { query: q }));
    } catch (err) {
      setError(err.message);
    } finally {
      setBusy(false);
    }
  };

  const pick = (p) =>
    call('update_widget_config', {
      id,
      patch: { location: { name: p.name, admin: p.admin, country: p.country, latitude: p.latitude, longitude: p.longitude } },
    }).then(onPicked, (e) => setError(e.message));

  return (
    <div className="wx__picker">
      <form className="wg-form__row" onSubmit={search}>
        <input className="wg-input" value={q} onChange={(e) => setQ(e.target.value)} placeholder="Şehir ara (ör. İzmir)" autoFocus maxLength={80} aria-label="Şehir ara" />
        <button type="submit" className="wg-btn wg-btn--primary" disabled={busy || !q.trim()} style={{ flex: 'none' }}>Ara</button>
      </form>
      {error ? <p className="wg-error">{error}</p> : null}
      {results?.length === 0 ? <p className="wx__muted">Sonuç bulunamadı.</p> : null}
      <ul className="wx__results">
        {results?.map((p) => (
          <li key={`${p.latitude},${p.longitude}`}>
            <button type="button" onClick={() => pick(p)}>
              <strong>{p.name}</strong>
              <span>{[p.admin, p.country].filter(Boolean).join(', ')}</span>
            </button>
          </li>
        ))}
      </ul>
      {onCancel ? <button type="button" className="wg-btn wg-btn--ghost" onClick={onCancel}>Vazgeç</button> : null}
    </div>
  );
}

export default function WeatherWidget({ id, config }) {
  const loc = config.location;
  const { settings } = useSettings();
  const refreshMinutes = settings?.weather_refresh_minutes ?? 30;
  const [result, setResult] = useState(null);
  const [error, setError] = useState(null);
  const [picking, setPicking] = useState(false);
  const [loading, setLoading] = useState(false);

  const lat = loc?.latitude;
  const lon = loc?.longitude;

  // İlk yükleme + yenileme aralığı. Önbellek tazeyse Rust ağa çıkmaz.
  useEffect(() => {
    if (lat == null || lon == null) return undefined;
    let alive = true;
    const run = () =>
      call('weather_get', { latitude: lat, longitude: lon, force: false }).then(
        (r) => {
          if (!alive) return;
          setResult(r);
          setError(null);
        },
        (e) => alive && setError(e.message),
      );
    run();
    const timer = setInterval(run, refreshMinutes * 60_000);
    return () => {
      alive = false;
      clearInterval(timer);
    };
  }, [lat, lon, refreshMinutes]);

  const refresh = async () => {
    setLoading(true);
    try {
      setResult(await call('weather_get', { latitude: lat, longitude: lon, force: true }));
      setError(null);
    } catch (e) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  if (!loc || picking) {
    return (
      <div className="wx">
        <WidgetHeader cat="curious" title="Hava Durumu" subtitle="Bir konum seçin" />
        <LocationPicker id={id} onPicked={() => setPicking(false)} onCancel={loc ? () => setPicking(false) : null} />
      </div>
    );
  }

  const r = result?.report;
  const now = r ? describe(r.current.code, r.current.is_day) : null;

  return (
    <div className="wx">
      <WidgetHeader
        cat={now?.cat ?? 'sitting'}
        title={loc.name}
        subtitle={loc.country}
        action={
          <>
            <WidgetIconButton label="Yenile" onClick={refresh} disabled={loading}>
              <PixelIcon name="refresh" size={12} />
            </WidgetIconButton>
            <WidgetIconButton label="Konumu değiştir" onClick={() => setPicking(true)}>⌖</WidgetIconButton>
          </>
        }
      />
      {error && !r ? <p className="wg-error">{error}</p> : null}
      {r ? (
        <>
          <div className="wx__now">
            <span className="wx__icon"><PixelIcon name={now.icon} size={40} /></span>
            <div>
              <p className="wx__temp">{temperature(r.current.temperature)}</p>
              <p className="wx__desc">{now.label}</p>
            </div>
            <dl className="wx__facts">
              <dt>Hissedilen</dt>
              <dd>{temperature(r.current.apparent)}</dd>
              <dt>Nem</dt>
              <dd>%{Math.round(r.current.humidity)}</dd>
              <dt>Rüzgâr</dt>
              <dd>{Math.round(r.current.wind_speed)} km/s</dd>
            </dl>
          </div>
          {config.show_forecast ? (
            <ul className="wx__days">
              {r.daily.slice(0, 5).map((d, i) => {
                const info = describe(d.code);
                const date = parseLocal(d.date);
                return (
                  <li key={d.date} title={info.label}>
                    <span className="wx__day">{i === 0 ? 'Bugün' : WEEKDAYS_SHORT[weekdayIndex(date)]}</span>
                    <PixelIcon name={info.icon} size={16} />
                    <span className="wx__range">
                      <b>{temperature(d.max)}</b> {temperature(d.min)}
                    </span>
                  </li>
                );
              })}
            </ul>
          ) : null}
          <p className={`wx__foot ${result.stale ? 'is-stale' : ''}`}>
            {result.stale ? 'Çevrim dışı · son veri ' : 'Güncellendi '}
            {formatTime(new Date(r.fetched_at))} · {r.provider}
          </p>
        </>
      ) : null}
    </div>
  );
}
