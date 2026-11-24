// Piksel kedi animasyonlarının seyrek zamanlayıcıları.
//
// Neden CSS değil: SVG <g> öğelerindeki sonsuz CSS animasyonları (steps() olsa bile)
// bileşikleştiricide (compositor) çalışamaz; Chromium onları her karede ana iş parçacığında
// işler ve sürekli yeniden birleştirir. Ölçümde bu, widget başına %10–16 çekirdek ve GPU
// sürecinde ~%45 çekirdek CPU'ya mal oluyordu. Burada değişiklikler seyrek zamanlayıcılarla
// yapılır: göz kırpma 4–9 sn'de bir 150 ms; Zz/soru işareti 1,5 sn'de bir kare değişimi.
// Hareket azaltıldığında ya da sayfa görünmezken hiçbir zamanlayıcı çalışmaz.
import { useEffect, useState, useSyncExternalStore } from 'react';

function readMotion() {
  if (window.matchMedia?.('(prefers-reduced-motion: reduce)').matches) return 'off';
  return document.documentElement.dataset.motion ?? 'full';
}

function subscribeMotion(cb) {
  const mo = new MutationObserver(cb);
  mo.observe(document.documentElement, { attributes: true, attributeFilter: ['data-motion'] });
  const mq = window.matchMedia?.('(prefers-reduced-motion: reduce)');
  mq?.addEventListener('change', cb);
  document.addEventListener('visibilitychange', cb);
  return () => {
    mo.disconnect();
    mq?.removeEventListener('change', cb);
    document.removeEventListener('visibilitychange', cb);
  };
}

/** Animasyon çalışabilir mi? (tam hareket modu ve sayfa görünür) */
export function useMotionAllowed() {
  return useSyncExternalStore(subscribeMotion, () => readMotion() === 'full' && !document.hidden);
}

/** Rastgele aralıklarla kısa süreli `true` döner (göz kırpma). */
export function useBlink(enabled) {
  const [blink, setBlink] = useState(false);
  useEffect(() => {
    if (!enabled) return undefined;
    let timer;
    const schedule = () => {
      timer = setTimeout(() => {
        setBlink(true);
        timer = setTimeout(() => {
          setBlink(false);
          schedule();
        }, 150);
      }, 4000 + Math.random() * 5000);
    };
    schedule();
    return () => {
      clearTimeout(timer);
      setBlink(false);
    };
  }, [enabled]);
  return blink;
}

/** Belirli aralıkla 0/1 arasında gidip gelen kare sayacı. */
export function useFrame(enabled, interval = 1500) {
  const [frame, setFrame] = useState(0);
  useEffect(() => {
    if (!enabled) return undefined;
    const t = setInterval(() => setFrame((f) => 1 - f), interval);
    return () => {
      clearInterval(t);
      setFrame(0);
    };
  }, [enabled, interval]);
  return frame;
}
