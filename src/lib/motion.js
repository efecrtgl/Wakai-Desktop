// Ayarlardaki hareket tercihini <html data-motion> özniteliğine yansıtır.
import { useCallback, useEffect, useRef, useState } from 'react';

export function motionMode(settings) {
  if (!settings) return 'full';
  if (settings.reduced_resource_mode || settings.animation_quality === 'off') return 'off';
  if (settings.reduced_motion || settings.animation_quality === 'reduced') return 'reduced';
  return 'full';
}

export function useMotionAttribute(settings) {
  const mode = motionMode(settings);
  useEffect(() => {
    document.documentElement.dataset.motion = mode;
  }, [mode]);
  return mode;
}

/** Küçük bir kutlama animasyonunu tetikler: [aktif mi, tetikle]. */
export function useCelebration(durationMs = 1000) {
  const [active, setActive] = useState(false);
  const timer = useRef(null);
  useEffect(() => () => clearTimeout(timer.current), []);
  const trigger = useCallback(() => {
    clearTimeout(timer.current);
    setActive(true);
    timer.current = setTimeout(() => setActive(false), durationMs);
  }, [durationMs]);
  return [active, trigger];
}
