// Uçtan uca test: ÇALIŞAN uygulamayı Chrome DevTools Protokolü üzerinden sürer.
//
// Kullanım (PowerShell):
//   $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222"
//   .\src-tauri\target\debug\wakai-desktop.exe
//   node scripts/e2e.mjs
//
// Test, oluşturduğu tüm kayıtları sonunda siler; mevcut kullanıcı verisine dokunmaz.

const PORT = process.env.CDP_PORT ?? 9222;
const targets = await (await fetch(`http://127.0.0.1:${PORT}/json`)).json();
const main = targets.find((t) => t.type === 'page' && t.url.endsWith('localhost/'));
if (!main) {
  console.error('Kontrol Merkezi sayfası bulunamadı. Uygulama CDP ile mi çalışıyor?');
  process.exit(2);
}

const ws = new WebSocket(main.webSocketDebuggerUrl);
await new Promise((r) => (ws.onopen = r));
let seq = 0;
const pending = new Map();
ws.onmessage = (m) => {
  const d = JSON.parse(m.data);
  pending.get(d.id)?.(d);
  pending.delete(d.id);
};

/** Komutu uygulama içinde çağırır; { ok, value | error } döner. */
function invoke(cmd, args = {}) {
  const expression = `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args)})
    .then(v => ({ ok: true, value: v }), e => ({ ok: false, error: String(e) }))`;
  return new Promise((resolve) => {
    pending.set(++seq, (d) => resolve(d.result?.result?.value ?? { ok: false, error: 'CDP hatası' }));
    ws.send(JSON.stringify({ id: seq, method: 'Runtime.evaluate', params: { expression, awaitPromise: true, returnByValue: true } }));
  });
}

let passed = 0;
const failures = [];
async function test(name, fn) {
  try {
    await fn();
    passed++;
    console.log(`  ✓ ${name}`);
  } catch (e) {
    failures.push(name);
    console.log(`  ✗ ${name}\n      ${e.message}`);
  }
}
function assert(cond, msg) {
  if (!cond) throw new Error(msg);
}
async function ok(cmd, args) {
  const r = await invoke(cmd, args);
  assert(r.ok, `${cmd} başarısız: ${r.error}`);
  return r.value;
}
async function fails(cmd, args, contains) {
  const r = await invoke(cmd, args);
  assert(!r.ok, `${cmd} hata vermeliydi`);
  if (contains) assert(r.error.includes(contains), `beklenen hata "${contains}", gelen "${r.error}"`);
  return r.error;
}

const now = new Date(Date.now() + 3 * 86400000);
const pad = (n) => String(n).padStart(2, '0');
const day = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
const cleanup = [];

console.log('Wakai Desktop uçtan uca testleri\n');

await test('uygulama bilgisi ve şema', async () => {
  const info = await ok('app_info');
  assert(info.schema_version >= 2, `şema ${info.schema_version}`);
});

await test('widget yaşam döngüsü: ekle → yapılandır → gizle → göster → kaldır', async () => {
  const w = await ok('add_widget', { kind: 'clock' });
  cleanup.push(['remove_widget', { id: w.id }]);
  const c = await ok('update_widget_config', { id: w.id, patch: { show_seconds: true, mode: 'analog' } });
  assert(c.config.show_seconds === true && c.config.mode === 'analog', 'yapılandırma uygulanmadı');
  assert((await ok('set_widget_visible', { id: w.id, visible: false })).visible === false, 'gizlenmedi');
  assert((await ok('set_widget_visible', { id: w.id, visible: true })).visible === true, 'gösterilmedi');
  await ok('remove_widget', { id: w.id });
  cleanup.pop();
  const list = await ok('list_widgets');
  assert(!list.some((x) => x.id === w.id), 'widget listede kaldı');
});

await test('geçersiz widget girdileri Türkçe hatayla reddedilir', async () => {
  await fails('add_widget', { kind: 'yok' }, 'Bilinmeyen widget türü');
  await fails('remove_widget', { id: '../x' }, 'Geçersiz widget kimliği');
  const w = await ok('add_widget', { kind: 'clock' });
  cleanup.push(['remove_widget', { id: w.id }]);
  await fails('update_widget_config', { id: w.id, patch: { show_seconds: 'evet' } }, 'geçersiz değer');
  await fails('update_widget_config', { id: w.id, patch: { hack: 1 } }, 'Bilinmeyen widget ayarı');
  await fails('set_widget_rect', { id: w.id, rect: { x: 0, y: 0, width: 0, height: 10 } }, 'Geçersiz widget boyutu');
});

await test('en küçük boyut korunur', async () => {
  const w = await ok('add_widget', { kind: 'clock' });
  cleanup.push(['remove_widget', { id: w.id }]);
  const r = await ok('set_widget_rect', { id: w.id, rect: { ...w.rect, width: 20, height: 20 } });
  assert(r.rect.width >= 180 && r.rect.height >= 100, `boyut ${r.rect.width}x${r.rect.height}`);
});

await test('takvim: oluştur → listele → güncelle → sil', async () => {
  await fails('create_event', { input: { title: '', start_at: `${day}T10:00` } }, 'boş olamaz');
  await fails('create_event', { input: { title: 'x', start_at: `${day}T10:00`, end_at: `${day}T09:00` } }, 'Bitiş zamanı');
  const e = await ok('create_event', { input: { title: 'E2E etkinlik', start_at: `${day}T10:00`, color: 'mint', reminder_minutes: 15 } });
  cleanup.push(['delete_event', { id: e.id }]);
  const list = await ok('list_events', { from: day, to: '9999-01-01' });
  assert(list.some((x) => x.id === e.id), 'etkinlik listede yok');
  const u = await ok('update_event', { id: e.id, input: { title: 'E2E güncel', start_at: `${day}T11:00`, color: 'sky' } });
  assert(u.title === 'E2E güncel' && u.start_at.endsWith('11:00'), 'güncellenmedi');
  await ok('delete_event', { id: e.id });
  cleanup.pop();
});

await test('görevler: oluştur → tamamla → geri al → sil', async () => {
  await fails('create_task', { input: { title: 'x', priority: 7 } }, 'öncelik');
  const t = await ok('create_task', { input: { title: 'E2E görev', priority: 2, due_at: `${day}T18:00` } });
  cleanup.push(['delete_task', { id: t.id }]);
  assert((await ok('set_task_completed', { id: t.id, completed: true })).completed, 'tamamlanmadı');
  assert(!(await ok('set_task_completed', { id: t.id, completed: false })).completed, 'geri alınmadı');
  await ok('delete_task', { id: t.id });
  cleanup.pop();
});

await test('notlar: oluştur → güncelle → sil', async () => {
  const n = await ok('create_note', { input: { title: 'E2E not', body: '- madde' } });
  cleanup.push(['delete_note', { id: n.id }]);
  const u = await ok('update_note', { id: n.id, input: { title: 'E2E not', body: '[x] bitti', color: 'sky', pinned: true } });
  assert(u.pinned && u.body === '[x] bitti', 'not güncellenmedi');
  await fails('update_note', { id: n.id, input: { color: '#fff' } }, 'renk');
  await ok('delete_note', { id: n.id });
  cleanup.pop();
});

await test('odak zamanlayıcısı: başlat → duraklat → sıfırla', async () => {
  const s = await ok('focus_start');
  assert(s.running && s.phase === 'work', 'başlamadı');
  const p = await ok('focus_pause');
  assert(!p.running && p.remaining_ms > 0, 'duraklamadı');
  const r = await ok('focus_reset');
  assert(r.phase === 'idle', 'sıfırlanmadı');
});

await test('tek örnekli widget ikinci kez eklenemez', async () => {
  const list = await ok('list_widgets');
  let created = null;
  if (!list.some((w) => w.kind === 'system')) {
    created = await ok('add_widget', { kind: 'system' });
    cleanup.push(['remove_widget', { id: created.id }]);
  }
  await fails('add_widget', { kind: 'system' }, 'zaten ekli');
});

await test('ayarlar doğrulanır ve sınırlanır', async () => {
  const before = await ok('get_settings');
  await fails('update_settings', { patch: { shadows: 'evet' } }, 'Geçersiz ayar');
  const s = await ok('update_settings', { patch: { monitor_interval_secs: 999 } });
  assert(s.monitor_interval_secs === 60, `sınır uygulanmadı: ${s.monitor_interval_secs}`);
  await ok('update_settings', { patch: { monitor_interval_secs: before.monitor_interval_secs } });
});

await test('yedek oluştur ve sil', async () => {
  const b = await ok('create_backup');
  assert(b.size > 0, 'yedek boş');
  assert((await ok('list_backups')).some((x) => x.name === b.name), 'yedek listede yok');
  await ok('delete_backup', { name: b.name });
  await fails('delete_backup', { name: '../wakai.db' }, 'Geçersiz yedek adı');
});

await test('kaynak kullanımı ölçülebilir', async () => {
  const u = await ok('app_resource_usage');
  assert(u.processes >= 1 && u.private_bytes > 0, 'ölçüm yok');
});

for (const [cmd, args] of cleanup.reverse()) await invoke(cmd, args);
ws.close();
console.log(`\n${passed} başarılı, ${failures.length} başarısız`);
process.exit(failures.length ? 1 : 0);
