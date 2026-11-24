// Ayarlardaki hareket tercihini <html data-motion> özniteliğine yansıtır.
import { useEffect } from 'react';

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
