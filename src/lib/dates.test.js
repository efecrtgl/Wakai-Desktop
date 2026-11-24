import { describe, expect, it } from 'vitest';
import { dateKey, eventTimeLabel, formatDayLong, formatTime, isWeekend, monthMatrix, parseLocal, relativeDay, toLocal } from './dates.js';

describe('tarih yardımcıları', () => {
  it('yerel biçime çevirir ve geri okur', () => {
    const d = new Date(2026, 10, 24, 9, 5);
    expect(toLocal(d)).toBe('2026-11-24T09:05');
    expect(parseLocal('2026-11-24T09:05').getTime()).toBe(d.getTime());
    expect(parseLocal('2026-11-24').getHours()).toBe(0);
    expect(parseLocal('bozuk')).toBeNull();
  });

  it('ay ızgarası pazartesiyle başlar ve 42 gündür', () => {
    const m = monthMatrix(2026, 10); // Kasım 2026, 1 Kasım Pazar
    expect(m).toHaveLength(42);
    expect(dateKey(m[0])).toBe('2026-10-26'); // Pazartesi
    expect(dateKey(m[6])).toBe('2026-11-01');
    expect(m[0].getDay()).toBe(1);
  });

  it('hafta sonunu tanır', () => {
    expect(isWeekend(new Date(2026, 10, 28))).toBe(true); // Cumartesi
    expect(isWeekend(new Date(2026, 10, 27))).toBe(false); // Cuma
  });

  it('Türkçe biçimler', () => {
    expect(formatDayLong(new Date(2026, 10, 24))).toBe('24 Kasım Salı');
    expect(formatTime(new Date(2026, 0, 1, 14, 3))).toBe('14:03');
    expect(formatTime(new Date(2026, 0, 1, 14, 3), true)).toBe('2:03 ÖS');
    const now = new Date(2026, 10, 24, 12);
    expect(relativeDay(new Date(2026, 10, 24, 23), now)).toBe('Bugün');
    expect(relativeDay(new Date(2026, 10, 25), now)).toBe('Yarın');
    expect(relativeDay(new Date(2026, 11, 3), now)).toBe('3 Ara');
    expect(eventTimeLabel({ all_day: true, start_at: '2026-11-24T00:00' })).toBe('Tüm gün');
  });
});
