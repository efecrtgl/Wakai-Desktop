import { describe, expect, it } from 'vitest';
import { bytes, duration, percent, temperature } from './format.js';
import { describe as weather } from '../widgets/weather/codes.js';

describe('biçimlendirme', () => {
  it('bayt', () => {
    expect(bytes(0)).toBe('0 B');
    expect(bytes(1536)).toBe('2 KB');
    expect(bytes(5 * 1024 ** 3)).toBe('5 GB');
  });
  it('yüzde ve süre', () => {
    expect(percent(null)).toBe('—');
    expect(percent(12.6)).toBe('%13');
    expect(duration(59 * 60)).toBe('59 dk');
    expect(duration(3 * 86400 + 7200)).toBe('3 gün 2 sa');
  });
  it('sıcaklık', () => {
    expect(temperature(5.6)).toBe('6°');
    expect(temperature(-0.4)).toBe('0°');
    expect(temperature(-3.6)).toBe('-4°');
  });
});

describe('hava durumu kodları', () => {
  it('gündüz/gece ve bilinmeyen kod', () => {
    expect(weather(0, true)).toMatchObject({ label: 'Açık', icon: 'sun' });
    expect(weather(0, false)).toMatchObject({ icon: 'moon', cat: 'sleeping' });
    expect(weather(95).icon).toBe('storm');
    expect(weather(1234).label).toBe('Bilinmiyor');
  });
});
