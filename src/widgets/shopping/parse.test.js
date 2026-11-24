import { describe, expect, it } from 'vitest';
import { parseItem } from './parse.js';

describe('alışveriş maddesi ayrıştırma', () => {
  it('miktarı ve birimi ayırır', () => {
    expect(parseItem('2 kg elma')).toEqual({ quantity: '2 kg', name: 'elma' });
    expect(parseItem('1,5 lt süt')).toEqual({ quantity: '1,5 lt', name: 'süt' });
    expect(parseItem('3 yumurta')).toEqual({ quantity: '3', name: 'yumurta' });
    expect(parseItem('2 Paket kedi maması')).toEqual({ quantity: '2 Paket', name: 'kedi maması' });
  });
  it('miktar yoksa tümü addır', () => {
    expect(parseItem('  ekmek  ')).toEqual({ quantity: '', name: 'ekmek' });
    expect(parseItem('7up')).toEqual({ quantity: '', name: '7up' });
    expect(parseItem('2')).toEqual({ quantity: '', name: '2' });
  });
});
