// Sayı biçimlendirme yardımcıları (Türkçe).
const nf1 = new Intl.NumberFormat('tr-TR', { maximumFractionDigits: 1 });
const nf0 = new Intl.NumberFormat('tr-TR', { maximumFractionDigits: 0 });

export function bytes(n) {
  if (!n) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.min(units.length - 1, Math.floor(Math.log(n) / Math.log(1024)));
  return `${(i >= 3 ? nf1 : nf0).format(n / 1024 ** i)} ${units[i]}`;
}

export function percent(v) {
  return v == null ? '—' : `%${nf0.format(v)}`;
}

export function duration(secs) {
  const d = Math.floor(secs / 86400);
  const h = Math.floor((secs % 86400) / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (d) return `${d} gün ${h} sa`;
  if (h) return `${h} sa ${m} dk`;
  return `${m} dk`;
}

export function temperature(v) {
  // `|| 0` eksi sıfırı (-0) önler: -0,4° → "0°".
  return `${nf0.format(Math.round(v) || 0)}°`;
}
