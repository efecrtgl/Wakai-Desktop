import { describe, expect, it } from 'vitest';
import { NAV, normalize, resolveRoute, search, SEARCH_INDEX } from './nav.js';

describe('gezinme', () => {
  it('1.1.0 bölüm kimlikleri yeni rotalara yönlenir', () => {
    expect(resolveRoute('calendar')).toEqual({ section: 'planner', sub: 'calendar' });
    expect(resolveRoute('store')).toEqual({ section: 'widgets', sub: 'store' });
    expect(resolveRoute('pets')).toEqual({ section: 'pethome', sub: 'cats' });
    expect(resolveRoute('performance')).toEqual({ section: 'settings', sub: 'performance' });
    expect(resolveRoute('appearance')).toEqual({ section: 'themes', sub: null });
  });

  it('bilinmeyen bölüm ve sekme güvenli varsayılana düşer', () => {
    expect(resolveRoute('yok')).toEqual({ section: 'overview', sub: null });
    expect(resolveRoute('settings', 'yok')).toEqual({ section: 'settings', sub: 'general' });
    expect(resolveRoute('widgets')).toEqual({ section: 'widgets', sub: 'mine' });
  });

  it('ana gezinme en fazla 7 bölüm (Ctrl+1…7)', () => {
    expect(NAV.length).toBeLessThanOrEqual(7);
  });

  it('arama dizinindeki her kayıt geçerli bir rotaya gider', () => {
    for (const it of SEARCH_INDEX) {
      const r = resolveRoute(it.section, it.sub ?? null);
      expect(r.section, it.label).toBe(it.section);
      if (it.sub) expect(r.sub, it.label).toBe(it.sub);
    }
  });
});

describe('arama', () => {
  it('Türkçe karakterler ve büyük/küçük harf duyarsız', () => {
    expect(normalize('IŞIK Ğüz Çöl İnce')).toBe('isik guz col ince');
    expect(search('yazi boyutu')[0].label).toBe('Yazı boyutu');
    expect(search('YEDEK')[0].label).toMatch(/Yedek/);
    expect(search('karsitlik')[0].label).toBe('Yüksek karşıtlık');
  });

  it('anahtar sözcüklerle bulur; tüm sözcükler eşleşmeli', () => {
    expect(search('y2k').some((r) => r.section === 'themes')).toBe(true);
    expect(search('başlangıç').some((r) => r.anchor === 'set-startup')).toBe(true);
    expect(search('tema zzzz')).toEqual([]);
    expect(search('   ')).toEqual([]);
  });
});
