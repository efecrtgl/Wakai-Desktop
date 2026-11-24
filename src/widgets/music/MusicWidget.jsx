// Müzik Kedisi: Windows medya oturumunu (GSMTC) gösterir ve kontrol eder.
// Uygulama kendi oynatıcısını içermez; hangi uygulama çalıyorsa onu yönetir.
import { useState } from 'react';
import { PixelCat } from '../../ui/cats/PixelCat.jsx';
import { PixelIcon } from '../../ui/PixelIcon.jsx';
import { useMediaState } from '../../lib/data.js';
import { call } from '../../lib/ipc.js';
import './music.css';

const STATUS = { playing: 'Çalıyor', paused: 'Duraklatıldı', stopped: 'Durduruldu', none: '' };

export default function MusicWidget({ config }) {
  const m = useMediaState();
  const [error, setError] = useState(null);

  const control = async (action) => {
    try {
      const ok = await call('media_control', { action });
      setError(ok ? null : 'Uygulama bu komutu kabul etmedi.');
    } catch (e) {
      setError(e.message);
    }
  };

  if (!m) return null;
  if (!m.available) {
    return (
      <div className="music music--empty">
        <PixelCat pose="sleeping" size={52} />
        <div>
          <p className="music__title">Şu an bir şey çalmıyor</p>
          <p className="music__artist">Spotify, tarayıcı ya da başka bir oynatıcıda müzik başlatın.</p>
        </div>
      </div>
    );
  }

  const playing = m.status === 'playing';
  return (
    <div className="music">
      <div className="music__art">
        {config.show_artwork && m.thumbnail ? <img src={m.thumbnail} alt="" /> : <PixelCat pose={playing ? 'happy' : 'sitting'} size={56} wag={playing} />}
      </div>
      <div className="music__info">
        <p className="music__title" title={m.title}>{m.title || 'Bilinmeyen parça'}</p>
        <p className="music__artist" title={m.artist}>{m.artist || '—'}</p>
        <p className="music__meta">
          {m.app}
          {STATUS[m.status] ? ` · ${STATUS[m.status]}` : ''}
        </p>
        <div className="music__controls">
          <button type="button" className="music__btn" onClick={() => control('previous')} disabled={!m.can_previous} aria-label="Önceki parça" title="Önceki">
            <PixelIcon name="previous" size={14} />
          </button>
          <button type="button" className="music__btn music__btn--main" onClick={() => control('play_pause')} disabled={!m.can_play_pause} aria-label={playing ? 'Duraklat' : 'Oynat'} title={playing ? 'Duraklat' : 'Oynat'}>
            <PixelIcon name={playing ? 'pause' : 'play'} size={16} />
          </button>
          <button type="button" className="music__btn" onClick={() => control('next')} disabled={!m.can_next} aria-label="Sonraki parça" title="Sonraki">
            <PixelIcon name="next" size={14} />
          </button>
        </div>
        {error ? <p className="wg-error">{error}</p> : null}
      </div>
    </div>
  );
}
