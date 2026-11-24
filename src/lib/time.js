// Zaman yardımcıları. Saat güncellemeleri bir sonraki saniye/dakika sınırına hizalanır;
// böylece gereksiz yeniden çizim olmaz ve saat kaymaz.
import { useEffect, useState } from 'react';

/** Şimdiki zamanı verir; `unit` 'second' ya da 'minute' sınırlarında güncellenir. */
export function useNow(unit = 'minute') {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    let timer;
    const step = unit === 'second' ? 1000 : 60000;
    const schedule = () => {
      const t = Date.now();
      timer = setTimeout(() => {
        setNow(new Date());
        schedule();
      }, step - (t % step) + 5);
    };
    schedule();
    // Uyku/hazırda bekleme sonrası hemen düzelt.
    const onVisible = () => {
      if (!document.hidden) setNow(new Date());
    };
    document.addEventListener('visibilitychange', onVisible);
    return () => {
      clearTimeout(timer);
      document.removeEventListener('visibilitychange', onVisible);
    };
  }, [unit]);
  return now;
}
