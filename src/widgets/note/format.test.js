import { describe, expect, it } from 'vitest';
import { parseInline, parseNote, toggleCheck } from './format.js';

describe('not biçimlendirme', () => {
  it('satır türlerini tanır', () => {
    const lines = parseNote('# Alışveriş\n- süt\n[ ] mama\n[x] kum\n\nnormal');
    expect(lines.map((l) => l.type)).toEqual(['heading', 'bullet', 'check', 'check', 'blank', 'para']);
    expect(lines[2].checked).toBe(false);
    expect(lines[3].checked).toBe(true);
  });

  it('kalın ve italik', () => {
    expect(parseInline('a **b** *c* d')).toEqual([
      { type: 'text', text: 'a ' },
      { type: 'bold', text: 'b' },
      { type: 'text', text: ' ' },
      { type: 'italic', text: 'c' },
      { type: 'text', text: ' d' },
    ]);
  });

  it('HTML olduğu gibi metin kalır', () => {
    expect(parseInline('<img onerror=x>')).toEqual([{ type: 'text', text: '<img onerror=x>' }]);
  });

  it('onay kutusunu değiştirir', () => {
    const body = 'başlık\n[ ] mama\n[x] kum';
    expect(toggleCheck(body, 1)).toBe('başlık\n[x] mama\n[x] kum');
    expect(toggleCheck(body, 2)).toBe('başlık\n[ ] mama\n[ ] kum');
    expect(toggleCheck(body, 9)).toBe(body);
  });
});
