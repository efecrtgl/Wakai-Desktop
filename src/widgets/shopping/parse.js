// "2 kg elma" → { quantity: '2 kg', name: 'elma' }. Miktar yoksa tüm metin addır.
const UNITS = 'kg|g|gr|gram|lt|l|litre|ml|adet|paket|kutu|şişe|demet|dilim|kavanoz|poşet';
const PATTERN = new RegExp(`^(\\d+(?:[.,]\\d+)?\\s*(?:${UNITS})?\\.?)\\s+(.+)$`, 'i');

export function parseItem(text) {
  const t = text.trim().replace(/\s+/g, ' ');
  const m = PATTERN.exec(t);
  if (!m) return { quantity: '', name: t };
  return { quantity: m[1].trim(), name: m[2].trim() };
}
