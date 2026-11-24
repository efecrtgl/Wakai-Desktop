// Not Kedisi için basit biçimlendirme ayrıştırıcısı. Zengin metin yerine düz metin
// üzerinde birkaç kural desteklenir. Çıktı veri yapısıdır; HTML üretilmez (XSS riski yok).
//   # Başlık      - madde      [ ] yapılacak   [x] yapıldı   **kalın**   *italik*

const INLINE = /(\*\*[^*\n]+\*\*|\*[^*\n]+\*)/g;

export function parseInline(text) {
  const parts = [];
  let last = 0;
  for (const m of text.matchAll(INLINE)) {
    if (m.index > last) parts.push({ type: 'text', text: text.slice(last, m.index) });
    const tok = m[0];
    if (tok.startsWith('**')) parts.push({ type: 'bold', text: tok.slice(2, -2) });
    else parts.push({ type: 'italic', text: tok.slice(1, -1) });
    last = m.index + tok.length;
  }
  if (last < text.length) parts.push({ type: 'text', text: text.slice(last) });
  return parts;
}

export function parseNote(body) {
  return body.split('\n').map((line, index) => {
    let m;
    if ((m = /^#{1,3}\s+(.*)$/.exec(line))) return { type: 'heading', index, parts: parseInline(m[1]) };
    if ((m = /^\s*\[( |x|X)\]\s?(.*)$/.exec(line))) return { type: 'check', index, checked: m[1] !== ' ', parts: parseInline(m[2]) };
    if ((m = /^\s*[-*]\s+(.*)$/.exec(line))) return { type: 'bullet', index, parts: parseInline(m[1]) };
    if (!line.trim()) return { type: 'blank', index };
    return { type: 'para', index, parts: parseInline(line) };
  });
}

/** Görünüm modunda bir onay kutusuna tıklanınca ilgili satırı değiştirir. */
export function toggleCheck(body, lineIndex) {
  const lines = body.split('\n');
  const line = lines[lineIndex];
  if (line === undefined) return body;
  lines[lineIndex] = line.replace(/\[( |x|X)\]/, (_, c) => (c === ' ' ? '[x]' : '[ ]'));
  return lines.join('\n');
}
