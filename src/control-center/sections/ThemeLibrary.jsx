// Tema kitaplığı: yüklü temalara göz atma, yalıtılmış önizleme, uygulama, favoriler,
// içe/dışa aktarma ve özel tema yönetimi. Tamamen yerel çalışır; hesap ya da çevrim içi
// mağaza gerekmez.
import { useMemo, useRef, useState } from 'react';
import { Badge, Button, Card, Dialog, ErrorNote, Segmented } from '../../ui/components.jsx';
import { PixelIcon } from '../../ui/PixelIcon.jsx';
import { call } from '../../lib/ipc.js';
import { useThemeApi } from '../../theme/context.js';
import { BUILTIN_THEMES, compileTheme } from '../../theme/engine.js';
import { CONTRACT, validateTheme } from '../../theme/schema.js';
import { PageHeader } from './PageHeader.jsx';
import { ThemeSandbox } from './ThemeSandbox.jsx';

const STYLE_LABELS = {
  shape: { rounded: 'Yuvarlak köşeler', soft: 'Yumuşak köşeler', sharp: 'Keskin köşeler', pixel: 'Piksel biçim' },
  surface: { flat: 'Düz yüzeyler', soft: 'Yumuşak gölgeler', paper: 'Kâğıt dokusu', glossy: 'Parlak krom' },
  icons: { pixel: 'Piksel simgeler', line: 'Çizgi simgeler' },
  texture: { none: null, paper: 'Kâğıt zemin', grid: 'Izgara zemin', scanline: 'Tarama çizgileri', dots: 'Noktalı zemin' },
};
const AMBIENT_LABELS = { none: null, dust: 'Toz zerreleri', leaves: 'Uçuşan yapraklar', stars: 'Yıldızlar', sparkles: 'Parıltılar' };

function features(theme) {
  const out = [STYLE_LABELS.shape[theme.style.shape], STYLE_LABELS.surface[theme.style.surface], STYLE_LABELS.icons[theme.style.icons], STYLE_LABELS.texture[theme.style.texture], AMBIENT_LABELS[theme.ambient]];
  if (theme.pets?.palette && theme.pets.palette !== 'none') out.push('Temaya özel kedi renkleri');
  const v = Object.keys(theme.variants ?? {}).length;
  if (v) out.push(`${v + 1} çeşit`);
  return out.filter(Boolean);
}

function readFile(file) {
  return new Promise((resolve, reject) => {
    if (file.size > CONTRACT.limits.pack_bytes) {
      reject(new Error('Tema dosyası çok büyük (en fazla 2 MB).'));
      return;
    }
    const r = new FileReader();
    r.onload = () => resolve(String(r.result));
    r.onerror = () => reject(new Error('Dosya okunamadı.'));
    r.readAsText(file);
  });
}

export function ThemeLibrary({ settingsApi }) {
  const { settings: s, update, error } = settingsApi;
  const api = useThemeApi();
  const [filter, setFilter] = useState('all');
  const [selectedId, setSelectedId] = useState(null);
  const [variant, setVariant] = useState(null);
  const [note, setNote] = useState(null);
  const [opError, setOpError] = useState(null);
  const [pending, setPending] = useState(null); // içe aktarma onayı bekleyen paket
  const [confirmDelete, setConfirmDelete] = useState(null);
  const fileRef = useRef(null);

  const customs = api?.customThemes;
  const entries = useMemo(() => {
    const list = BUILTIN_THEMES.map((t) => ({ id: t.id, theme: t, builtin: true, errors: [] }));
    for (const c of customs ?? []) {
      const r = c.manifest ? validateTheme(c.manifest) : { ok: false, errors: ['Tema dosyası okunamadı.'] };
      list.push({ id: c.id, theme: r.ok ? r.theme : null, builtin: false, errors: r.ok ? [] : r.errors, warnings: r.warnings ?? [], name: c.name, version: c.version, installed: c.installed_at });
    }
    return list;
  }, [customs]);

  if (!s) return null;
  const favorites = new Set(s.theme_favorites ?? []);
  const shown = entries.filter((e) => (filter === 'fav' ? favorites.has(e.id) : filter === 'custom' ? !e.builtin : true));
  const selected = entries.find((e) => e.id === (selectedId ?? s.theme)) ?? entries[0];
  const selVariant = variant ?? (selected.id === s.theme ? s.theme_variant : '');
  const selCompiled = selected.theme ? compileTheme(selected.theme, { variant: selVariant, accent: s.theme_accent, density: s.ui_density }) : null;

  const select = (id) => {
    setSelectedId(id);
    setVariant(null);
  };
  const apply = (id, v = '') => {
    api?.setPreview(null);
    update({ theme: id, theme_variant: v });
    setNote(null);
  };
  const toggleFavorite = (id) => {
    const next = favorites.has(id) ? [...favorites].filter((f) => f !== id) : [...favorites, id];
    update({ theme_favorites: next });
  };
  const exportTheme = async (id) => {
    setOpError(null);
    try {
      const path = await call('export_theme', { id });
      setNote(`Dışa aktarıldı: ${path}`);
    } catch (e) {
      setOpError(e.message);
    }
  };
  const onFile = async (e) => {
    const file = e.target.files?.[0];
    e.target.value = '';
    if (!file) return;
    setOpError(null);
    try {
      const text = await readFile(file);
      let parsed;
      try {
        parsed = JSON.parse(text);
      } catch {
        throw new Error('Tema dosyası geçerli bir JSON değil.');
      }
      const r = validateTheme(parsed);
      if (!r.ok) throw new Error(r.errors.join(' '));
      setPending({ text, theme: r.theme, warnings: r.warnings, replace: (customs ?? []).some((c) => c.id === r.theme.id) });
    } catch (err) {
      setOpError(err.message);
    }
  };
  const confirmImport = async () => {
    const p = pending;
    setPending(null);
    try {
      const summary = await call('import_theme', { text: p.text, replace: p.replace });
      setNote(`“${summary.name}” yüklendi.`);
      setFilter('all');
      select(summary.id);
    } catch (err) {
      setOpError(err.message);
    }
  };
  const doDelete = async () => {
    const id = confirmDelete;
    setConfirmDelete(null);
    try {
      await call('delete_custom_theme', { id });
      setNote('Tema kaldırıldı. Ayarlarınız ve verileriniz etkilenmedi.');
      select(s.theme === id ? 'pixel' : null);
    } catch (err) {
      setOpError(err.message);
    }
  };

  const variantOptions = selected.theme ? [{ value: '', label: selected.theme.base === 'dark' ? 'Koyu' : 'Açık' }, ...Object.entries(selected.theme.variants ?? {}).map(([id, v]) => ({ value: id, label: v.name }))] : [];
  const isActive = (e) => e.id === s.theme;

  return (
    <div className="cc-page">
      <PageHeader
        title="Temalar"
        subtitle="Her tema arayüzün, widget'ların, kedilerin ve Kedi Evi'nin görünümünü birlikte değiştirir"
        actions={
          <>
            <input ref={fileRef} type="file" accept=".ivilytheme,.json,application/json" hidden onChange={onFile} />
            <Button icon={<PixelIcon name="upload" size={14} />} onClick={() => fileRef.current?.click()}>
              Tema içe aktar
            </Button>
          </>
        }
      />
      <ErrorNote error={error ?? opError} />
      {note ? <p className="cc-note" role="status">{note}</p> : null}

      <div className="tl">
        <div className="tl__list">
          <Segmented
            label="Göster"
            value={filter}
            onChange={setFilter}
            options={[
              { value: 'all', label: 'Tümü' },
              { value: 'fav', label: 'Favoriler' },
              { value: 'custom', label: 'Yüklenenler' },
            ]}
          />
          {shown.length ? (
            <ul className="tl__grid">
              {shown.map((e) => (
                <li key={e.id}>
                  <article className="tl-card" aria-current={selected.id === e.id ? 'true' : undefined}>
                    <button type="button" className="tl-card__body" onClick={() => select(e.id)} aria-label={`${e.theme?.name ?? e.name} temasını incele`}>
                      {e.theme ? <ThemeSandbox compiled={compileTheme(e.theme)} /> : <div className="tl-card__broken">Bu tema yüklenemiyor</div>}
                    </button>
                    <div className="tl-card__meta">
                      <span className="tl-card__name">{e.theme?.name ?? e.name}</span>
                      {isActive(e) ? <Badge tone="mint">Etkin</Badge> : null}
                      {!e.builtin ? <Badge tone="lavender">Yüklendi</Badge> : null}
                      {e.errors.length ? <Badge tone="peach">Uyumsuz</Badge> : null}
                      <button type="button" className="tl-card__fav" aria-pressed={favorites.has(e.id)} aria-label={favorites.has(e.id) ? 'Favorilerden çıkar' : 'Favorilere ekle'} onClick={() => toggleFavorite(e.id)}>
                        <PixelIcon name="star" size={14} />
                      </button>
                    </div>
                  </article>
                </li>
              ))}
            </ul>
          ) : (
            <p className="cc-muted tl__empty">{filter === 'fav' ? 'Henüz favori tema yok. Bir temanın yıldızına tıklayarak ekleyebilirsiniz.' : 'Henüz yüklenmiş tema yok. “Tema içe aktar” ile bir .ivilytheme dosyası ekleyebilirsiniz.'}</p>
          )}
        </div>

        <Card className="tl__detail" title={selected.theme?.name ?? selected.name}>
          {selCompiled ? (
            <>
              <ThemeSandbox compiled={selCompiled} size="large" />
              {variantOptions.length > 1 ? <Segmented label="Çeşit" value={selVariant} options={variantOptions} onChange={setVariant} /> : null}
              <p className="tl__desc">{selected.theme.description}</p>
              <ul className="tl__features">
                {features(selected.theme).map((f) => (
                  <li key={f}>{f}</li>
                ))}
              </ul>
              <dl className="cc-dl tl__facts">
                <dt>Sürüm</dt>
                <dd>{selected.theme.version}</dd>
                <dt>Hazırlayan</dt>
                <dd>{selected.theme.author || '—'}</dd>
                <dt>Uyumluluk</dt>
                <dd>Ivily {selected.theme.min_app}+ · şema v{selected.theme.schema} · uyumlu</dd>
                {selected.warnings?.length ? (
                  <>
                    <dt>Uyarılar</dt>
                    <dd>{selected.warnings.length} geçersiz değer yok sayıldı</dd>
                  </>
                ) : null}
              </dl>
              <div className="cc-actions">
                <Button variant="primary" disabled={isActive(selected) && selVariant === s.theme_variant} onClick={() => apply(selected.id, selVariant)}>
                  {isActive(selected) && selVariant === s.theme_variant ? 'Uygulandı' : 'Uygula'}
                </Button>
                <Button onClick={() => api?.setPreview({ id: selected.id, variant: selVariant })} disabled={isActive(selected) && selVariant === s.theme_variant}>
                  Uygulamada önizle
                </Button>
                <Button variant="ghost" icon={<PixelIcon name="download" size={14} />} onClick={() => exportTheme(selected.id)}>
                  Dışa aktar
                </Button>
                {!selected.builtin ? (
                  <Button variant="danger" onClick={() => setConfirmDelete(selected.id)}>
                    Kaldır
                  </Button>
                ) : null}
              </div>
            </>
          ) : (
            <>
              <p className="wk-error">Bu tema bu sürümle uyumlu değil ya da bozuk; uygulanamaz.</p>
              <ul className="cc-bullets">
                {selected.errors.map((x) => (
                  <li key={x}>{x}</li>
                ))}
              </ul>
              <div className="cc-actions">
                <Button variant="danger" onClick={() => setConfirmDelete(selected.id)}>
                  Kaldır
                </Button>
              </div>
            </>
          )}
        </Card>
      </div>

      <Dialog
        open={!!pending}
        title={pending ? `“${pending.theme.name}” temasını yükle` : ''}
        onClose={() => setPending(null)}
        actions={
          <>
            <Button variant="ghost" onClick={() => setPending(null)}>
              Vazgeç
            </Button>
            <Button variant="primary" onClick={confirmImport}>
              {pending?.replace ? 'Güncelle' : 'Yükle'}
            </Button>
          </>
        }
      >
        {pending ? (
          <>
            <ThemeSandbox compiled={compileTheme(pending.theme)} size="large" />
            <p className="cc-muted">
              Sürüm {pending.theme.version}
              {pending.theme.author ? ` · ${pending.theme.author}` : ''}. Temalar yalnızca görünüm bilgisi içerir; kod çalıştıramaz ve dosyalarınıza erişemez.
            </p>
            {pending.replace ? <p className="cc-note">Aynı kimlikli yüklü tema bu sürümle değiştirilecek.</p> : null}
            {pending.warnings.length ? <p className="cc-note">{pending.warnings.length} geçersiz değer yok sayılacak; tema yine de çalışır.</p> : null}
          </>
        ) : null}
      </Dialog>

      <Dialog
        open={!!confirmDelete}
        title="Tema kaldırılsın mı?"
        onClose={() => setConfirmDelete(null)}
        actions={
          <>
            <Button variant="ghost" onClick={() => setConfirmDelete(null)}>
              Vazgeç
            </Button>
            <Button variant="danger" onClick={doDelete}>
              Kaldır
            </Button>
          </>
        }
      >
        <p>Tema bu bilgisayardan kaldırılır. Etkin temaysa Piksel temaya geçilir; widget'larınız, kedileriniz ve Kedi Evi etkilenmez.</p>
      </Dialog>
    </div>
  );
}
