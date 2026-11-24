// Türkçe tarih yardımcıları. Veritabanında tarihler yerel zaman olarak
// "YYYY-MM-DDTHH:MM" biçiminde tutulur; burada Date nesnelerine dönüştürülür.

const pad = (n) => String(n).padStart(2, '0');

export const MONTHS = ['Ocak', 'Şubat', 'Mart', 'Nisan', 'Mayıs', 'Haziran', 'Temmuz', 'Ağustos', 'Eylül', 'Ekim', 'Kasım', 'Aralık'];
export const WEEKDAYS_SHORT = ['Pzt', 'Sal', 'Çar', 'Per', 'Cum', 'Cmt', 'Paz'];
export const WEEKDAYS = ['Pazartesi', 'Salı', 'Çarşamba', 'Perşembe', 'Cuma', 'Cumartesi', 'Pazar'];

/** Date → "YYYY-MM-DD" (yerel). */
export function dateKey(d) {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

/** Date → "YYYY-MM-DDTHH:MM" (yerel). */
export function toLocal(d) {
  return `${dateKey(d)}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

/** "YYYY-MM-DDTHH:MM" ya da "YYYY-MM-DD" → Date (yerel). Geçersizse null. */
export function parseLocal(s) {
  if (!s) return null;
  const m = /^(\d{4})-(\d{2})-(\d{2})(?:T(\d{2}):(\d{2}))?/.exec(s);
  if (!m) return null;
  const d = new Date(+m[1], +m[2] - 1, +m[3], +(m[4] ?? 0), +(m[5] ?? 0));
  return Number.isNaN(d.getTime()) ? null : d;
}

export function addDays(d, n) {
  const r = new Date(d);
  r.setDate(r.getDate() + n);
  return r;
}

export function addMonths(d, n) {
  return new Date(d.getFullYear(), d.getMonth() + n, 1);
}

/** Pazartesi=0 … Pazar=6 */
export function weekdayIndex(d) {
  return (d.getDay() + 6) % 7;
}

export function isWeekend(d) {
  return weekdayIndex(d) >= 5;
}

export function isSameDay(a, b) {
  return a && b && dateKey(a) === dateKey(b);
}

/** Pazartesiyle başlayan 6×7'lik ay ızgarası. */
export function monthMatrix(year, month) {
  const first = new Date(year, month, 1);
  const start = addDays(first, -weekdayIndex(first));
  return Array.from({ length: 42 }, (_, i) => addDays(start, i));
}

export function monthTitle(d) {
  return `${MONTHS[d.getMonth()]} ${d.getFullYear()}`;
}

/** "24 Kasım Salı" */
export function formatDayLong(d) {
  return `${d.getDate()} ${MONTHS[d.getMonth()]} ${WEEKDAYS[weekdayIndex(d)]}`;
}

/** "24 Kas" */
export function formatDayShort(d) {
  return `${d.getDate()} ${MONTHS[d.getMonth()].slice(0, 3)}`;
}

export function formatTime(d, hour12 = false) {
  if (!hour12) return `${pad(d.getHours())}:${pad(d.getMinutes())}`;
  const h = d.getHours() % 12 || 12;
  return `${h}:${pad(d.getMinutes())} ${d.getHours() < 12 ? 'ÖÖ' : 'ÖS'}`;
}

/** "Bugün", "Yarın", "Dün" ya da "24 Kas". */
export function relativeDay(d, now = new Date()) {
  const diff = Math.round((new Date(d.getFullYear(), d.getMonth(), d.getDate()) - new Date(now.getFullYear(), now.getMonth(), now.getDate())) / 86400000);
  if (diff === 0) return 'Bugün';
  if (diff === 1) return 'Yarın';
  if (diff === -1) return 'Dün';
  return formatDayShort(d);
}

/** Etkinliğin kısa zaman etiketi: "Tüm gün" ya da "09:30". */
export function eventTimeLabel(ev) {
  if (ev.all_day) return 'Tüm gün';
  const d = parseLocal(ev.start_at);
  return d ? formatTime(d) : '';
}
