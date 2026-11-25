// Wakai Desktop'tan gelen kullanıcılara bir kez gösterilen, sessiz ad değişikliği notu.
import { useEffect, useState } from 'react';
import { call } from '../lib/ipc.js';

export function BrandNotice() {
  const [show, setShow] = useState(false);
  useEffect(() => {
    call('brand_notice').then(setShow, () => setShow(false));
  }, []);
  if (!show) return null;
  const dismiss = () => {
    setShow(false);
    call('dismiss_brand_notice').catch(() => {});
  };
  return (
    <div className="cc-brand-note" role="status">
      <p>
        <strong>Wakai Desktop artık Ivily Desktop.</strong> Widget'larınız, kedileriniz ve tüm verileriniz olduğu gibi korunuyor.
      </p>
      <button type="button" className="cc-brand-note__close" onClick={dismiss} aria-label="Bildirimi kapat">
        Tamam
      </button>
    </div>
  );
}
