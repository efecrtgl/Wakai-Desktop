import { describe, expect, it } from 'vitest';
import { validateTheme, migrateTheme, compareVersions, safeValue, CONTRACT } from './schema.js';
import { BUILTIN_THEMES, compileTheme, cssFor, resolveTheme, contrast, attrsFor, builtinTheme, isHex } from './engine.js';

describe('tema şeması', () => {
  it('beş yerleşik tema uyarısız doğrulanır', () => {
    expect(BUILTIN_THEMES.map((t) => t.id)).toEqual(['pixel', 'modern', 'cozy', 'mono', 'y2k']);
    for (const t of BUILTIN_THEMES) {
      const r = validateTheme(t);
      expect(r.errors).toEqual([]);
      expect(r.warnings).toEqual([]);
    }
  });

  it('CSS enjeksiyonunu ve dış adresleri reddeder', () => {
    for (const v of ['red;} body{display:none', 'url(https://x)', 'URL (x)', 'expression(1)', '<b>', "image-set('a.png' 1x)", 'x\\26']) {
      expect(safeValue(v)).toBe(false);
    }
    expect(safeValue('linear-gradient(160deg, #fff 0%, rgba(0, 0, 0, 0.5) 100%)')).toBe(true);
    const r = validateTheme({ schema: 1, id: 'kotu', name: 'Kötü', tokens: { 'color-bg': 'url(x)', 'color-text': '#000', position: 'fixed' } });
    expect(r.ok).toBe(true);
    expect(r.theme.tokens).toEqual({ 'color-text': '#000' });
    expect(r.warnings.length).toBe(2);
  });

  it('ölümcül hatalar: kimlik, ad, şema, uyumluluk', () => {
    expect(validateTheme(null).ok).toBe(false);
    expect(validateTheme({ schema: 1, id: '../x', name: 'A' }).ok).toBe(false);
    expect(validateTheme({ schema: 1, id: 'aa' }).ok).toBe(false);
    expect(validateTheme({ schema: 99, id: 'aa', name: 'A' }).errors[0]).toMatch(/daha yeni/);
    expect(validateTheme({ schema: 1, id: 'aa', name: 'A', min_app: '9.0.0' }).errors[0]).toMatch(/9\.0\.0/);
  });

  it('şema 0 → 1 göçü: colors → tokens', () => {
    const m = migrateTheme({ id: 'eski', name: 'Eski', colors: { 'color-bg': '#fff' } });
    expect(m.schema).toBe(1);
    expect(m.tokens['color-bg']).toBe('#fff');
    expect(validateTheme({ id: 'eski', name: 'Eski', colors: { 'color-bg': '#fff' } }).ok).toBe(true);
  });

  it('geçersiz yapı seçenekleri varsayılana düşer, tema yine çalışır', () => {
    const r = validateTheme({ schema: 1, id: 'aa', name: 'A', style: { shape: 'blob' }, ambient: 'yanardağ' });
    expect(r.ok).toBe(true);
    expect(r.theme.style.shape).toBe('rounded');
    expect(r.theme.ambient).toBe('none');
  });

  it('sürüm karşılaştırma', () => {
    expect(compareVersions('1.1.5', '1.1.10')).toBe(-1);
    expect(compareVersions('2.0.0', '1.9.9')).toBe(1);
    expect(compareVersions(CONTRACT.app, CONTRACT.app)).toBe(0);
  });
});

describe('tema motoru', () => {
  it('çeşit belirteçleri temanın üzerine yazılır ve taban değişir', () => {
    const c = compileTheme(builtinTheme('modern'), { variant: 'dark' });
    expect(c.base).toBe('dark');
    expect(c.vars['color-bg']).toBe('#151315');
    expect(compileTheme(builtinTheme('modern'), { variant: 'yok' }).variant).toBe('');
  });

  it('vurgu rengi okunabilir metin rengiyle uygulanır; Tek Renk vurguyu kabul etmez', () => {
    const c = compileTheme(builtinTheme('cozy'), { accent: '#2266dd' });
    expect(c.vars['color-primary']).toBe('#2266dd');
    expect(contrast(c.vars['color-primary'], c.vars['color-text-on-primary'])).toBeGreaterThanOrEqual(4.5);
    expect(isHex(c.vars['color-primary-soft'])).toBe(true);
    const mono = compileTheme(builtinTheme('mono'), { accent: '#ff0000' });
    expect(mono.vars['color-primary']).toBe('#111111');
  });

  it('kullanıcı yoğunluğu temanınkini geçersiz kılar', () => {
    expect(compileTheme(builtinTheme('mono')).style.density).toBe('compact');
    expect(compileTheme(builtinTheme('mono'), { density: 'roomy' }).style.density).toBe('roomy');
    expect(compileTheme(builtinTheme('mono'), { density: 'auto' }).style.density).toBe('compact');
  });

  it('bilinmeyen ya da bozuk tema yedek temaya düşer', () => {
    const missing = resolveTheme('yok-boyle-tema');
    expect(missing.compiled.id).toBe('modern');
    expect(missing.fallbackReason).toMatch(/bulunamadı/);
    const broken = resolveTheme('ozel', { custom: { schema: 1, id: 'ozel' } });
    expect(broken.compiled.id).toBe('modern');
    const ok = resolveTheme('ozel', { custom: { schema: 1, id: 'ozel', name: 'Özel', tokens: { 'color-bg': '#000' } } });
    expect(ok.compiled.id).toBe('ozel');
    expect(ok.fallbackReason).toBeNull();
  });

  it('CSS metni yalnızca değişkenler içerir; önizleme kutusuna kapsamlanabilir', () => {
    const c = compileTheme(builtinTheme('pixel'));
    const css = cssFor(c, '.theme-sandbox');
    expect(css.startsWith('.theme-sandbox{color-scheme:light;--')).toBe(true);
    expect(css).toContain("--font-sans:'Pixelify Sans'");
    expect(css.split('{').length).toBe(2);
    expect(attrsFor(c)).toMatchObject({ shape: 'pixel', icons: 'pixel', rendering: 'pixelated' });
  });

  it('yerleşik temalar metin karşıtlığında WCAG AA (4.5:1) eşiğini geçer', () => {
    for (const t of BUILTIN_THEMES) {
      for (const variant of ['', ...Object.keys(t.variants)]) {
        const v = { ...compileTheme(builtinTheme('modern'), { variant: variant && builtinTheme('modern').variants[variant] ? variant : '' }).vars, ...compileTheme(t, { variant }).vars };
        const label = `${t.id}/${variant || 'varsayılan'}`;
        expect(contrast(v['color-text'], v['color-bg']), label).toBeGreaterThanOrEqual(4.5);
        expect(contrast(v['color-text'], v['color-surface']), label).toBeGreaterThanOrEqual(4.5);
        expect(contrast(v['color-text-muted'], v['color-surface']), label).toBeGreaterThanOrEqual(3);
      }
    }
  });
});
