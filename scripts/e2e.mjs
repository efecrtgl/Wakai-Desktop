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
const main = targets.find((t) => t.type === 'page' && (t.url.endsWith('localhost/') || t.url.endsWith(':1420/')));
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

/** Sayfada bir ifade değerlendirir (DOM denetimleri için). */
function evaluate(expression) {
  return new Promise((resolve) => {
    pending.set(++seq, (d) => resolve(d.result?.result?.value));
    ws.send(JSON.stringify({ id: seq, method: "Runtime.evaluate", params: { expression, awaitPromise: true, returnByValue: true } }));
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

console.log('Ivily Desktop uçtan uca testleri\n');

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
  // En küçük boyut widget ölçeğiyle (ve monitör ölçeğiyle) çarpılır.
  const k = (await ok('get_settings')).widget_scale;
  assert(r.rect.width >= Math.floor(180 * k) && r.rect.height >= Math.floor(100 * k), `boyut ${r.rect.width}x${r.rect.height} (ölçek ${k})`);
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

// --- 1.1.0 ---

await test('alışveriş listesi: oluştur → ekle → işaretle → sil', async () => {
  const list = await ok('create_shopping_list', { name: 'E2E market' });
  cleanup.push(['delete_shopping_list', { id: list.id }]);
  const item = await ok('add_shopping_item', { listId: list.id, name: 'Süt', quantity: '2' });
  await ok('set_shopping_item_checked', { id: item.id, checked: true });
  const items = await ok('list_shopping_items', { listId: list.id });
  assert(items.length === 1 && items[0].checked, 'öğe işaretlenmedi');
  await fails('add_shopping_item', { listId: list.id, name: '   ' });
  await ok('delete_shopping_list', { id: list.id });
  cleanup.pop();
});

await test('kedi: sahiplen → besle → koleksiyondan çıkar; geçersiz girdiler reddedilir', async () => {
  await fails('adopt_pet', { name: 'E2E', breed: 'ejderha', personality: 'playful' });
  await fails('adopt_pet', { name: '', breed: 'pink', personality: 'playful' });
  const pet = await ok('adopt_pet', { name: 'E2E Kedi', breed: 'pink', personality: 'calm' });
  cleanup.push(['release_pet', { id: pet.id }]);
  await ok('set_pet_location', { id: pet.id, location: 'home' });
  const fed = await invoke('feed_pet', { id: pet.id, food: 'kibble' });
  // Günlük yiyecek bitmişse anlaşılır bir hata beklenir; başka hata olmamalı.
  assert(fed.ok || /kalmadı|yok/i.test(fed.error), `besleme: ${fed.error}`);
  await fails('feed_pet', { id: pet.id, food: 'taş' });
  await ok('release_pet', { id: pet.id });
  cleanup.pop();
  assert(!(await ok('list_pets')).some((x) => x.id === pet.id), 'kedi silinmedi');
});

await test('Kedi Evi: eşya ekle → taşı → sil', async () => {
  await fails('add_home_item', { kind: 'roket', x: 10, y: 10 }, 'Bilinmeyen');
  const item = await ok('add_home_item', { kind: 'plant', x: 40, y: 150 });
  cleanup.push(['delete_home_item', { id: item.id }]);
  await ok('delete_home_item', { id: item.id });
  cleanup.pop();
});

await test('profiller: yerleşikler var, özel profil oluştur → sil, yerleşik silinemez', async () => {
  const list = await ok('list_profiles');
  assert(list.filter((p) => p.builtin).length === 4, 'yerleşik profil sayısı 4 değil');
  const custom = await ok('create_profile', { name: 'E2E profil', config: list[0].config });
  cleanup.push(['delete_profile', { id: custom.id }]);
  await fails('delete_profile', { id: list.find((p) => p.builtin).id });
  await ok('delete_profile', { id: custom.id });
  cleanup.pop();
});

await test('güncellemeler ve sürüm geçmişi', async () => {
  const st = await ok('update_status');
  const info = await ok('app_info');
  assert(st.current === info.version, 'sürüm uyuşmuyor');
  const h = await ok('version_history');
  assert(h.history.some((v) => v.version === info.version), 'şu anki sürüm geçmişte yok');
  await fails('rollback_to_version', { version: '../../x' });
  await fails('install_update', {}, 'Bekleyen');
});

await test('yedek inceleme ve seçmeli geri yükleme doğrulaması', async () => {
  const b = await ok('create_backup');
  const d = await ok('inspect_backup', { name: b.name });
  assert(d.valid && !d.newer, 'yedek geçersiz');
  assert(d.categories.some((c) => c.id === 'pets' && c.count !== null), 'kedi kategorisi yok');
  await fails('restore_backup_selective', { name: b.name, categories: [] }, 'kategori');
  await fails('restore_backup_selective', { name: b.name, categories: ['şifreler'] }, 'Bilinmeyen');
  await fails('inspect_backup', { name: '../wakai.db' }, 'Geçersiz');
  await ok('delete_backup', { name: b.name });
});

// --- 1.1.5 ---

await test('tema: geçersiz paket reddedilir, geçerli paket yüklenir → uygulanır → silinir', async () => {
  const before = await ok('get_settings');
  await fails('import_theme', { text: '{bozuk', replace: false }, 'JSON');
  await fails('import_theme', { text: JSON.stringify({ schema: 1, id: 'e2e-kotu', name: 'K', tokens: { 'color-bg': 'url(http://x)' } }), replace: false }, 'güvenli değil');
  await fails('import_theme', { text: JSON.stringify({ schema: 1, id: 'pixel', name: 'Sahte' }), replace: false }, 'yerleşik');
  const pack = { schema: 1, id: 'e2e-tema', name: 'E2E Tema', version: '1.0.0', base: 'dark', tokens: { 'color-bg': '#101010' }, pets: { palette: 'grayscale' } };
  cleanup.push(['delete_custom_theme', { id: 'e2e-tema' }]);
  const r = await ok('import_theme', { text: JSON.stringify(pack), replace: false });
  assert(r.id === 'e2e-tema', 'kimlik');
  assert((await ok('list_custom_themes')).some((t) => t.id === 'e2e-tema'), 'listede yok');
  await ok('update_settings', { patch: { theme: 'e2e-tema' } });
  await new Promise((r) => setTimeout(r, 600));
  const applied = await evaluate('document.documentElement.dataset.themeId');
  assert(applied === 'e2e-tema', `tema uygulanmadı: ${applied}`);
  await ok('delete_custom_theme', { id: 'e2e-tema' });
  cleanup.pop();
  const after = await ok('get_settings');
  assert(after.theme === 'pixel', 'silinen etkin tema varsayılana dönmeli');
  await ok('update_settings', { patch: { theme: before.theme, theme_variant: before.theme_variant } });
});

await test('Kedi Evi: dünya hazır, yerleşim doğrulanır, olay işlenir, envanter korunur', async () => {
  const w = await ok('get_world');
  assert(w.rooms.length === 3 && w.items.length > 0, 'dünya boş');
  await fails('place_world_item', { kind: 'roket', room: 'living', tx: 1, ty: 5, rot: 0 }, 'Bilinmeyen');
  await fails('place_world_item', { kind: 'plant', room: 'mutfak', tx: 1, ty: 5, rot: 0 }, 'oda');
  const kind = Object.keys(w.inventory).find((k) => !['window', 'painting', 'shelf', 'rug', 'tree', 'sofa', 'cathouse', 'bed'].includes(k));
  if (kind) {
    let placed = null;
    for (let tx = 0; tx < 20 && !placed; tx++)
      for (let ty = 4; ty < 12 && !placed; ty++) {
        const res = await invoke('place_world_item', { kind, room: 'playroom', tx, ty, rot: 0 });
        if (res.ok) placed = res.value;
      }
    assert(placed, 'eşya yerleştirilemedi');
    await fails('move_world_item', { id: placed.id, room: 'playroom', tx: placed.tx, ty: 0, rot: 0 }, 'sığmıyor');
    await ok('remove_world_item', { id: placed.id });
    const again = await ok('get_world');
    assert((again.inventory[kind] ?? 0) === (w.inventory[kind] ?? 0), 'envanter sayısı değişti');
  }
  const pet = w.pets[0];
  if (pet) {
    const r = await ok('world_event', { input: { pet_id: pet.id, kind: 'explore' } });
    assert(typeof r.pet.needs.curiosity === 'number', 'merak alanı yok');
    await fails('world_event', { input: { pet_id: pet.id } }, 'Olay');
    await fails('world_event', { input: { pet_id: 999999, kind: 'pet' } });
  }
});

await test('arama ve yeni gezinme', async () => {
  const count = await evaluate("document.querySelectorAll('.cc-nav__item').length");
  assert(count === 7, `bölüm sayısı ${count}`);
});

for (const [cmd, args] of cleanup.reverse()) await invoke(cmd, args);
ws.close();

const widgetPage = targets.find((t) => t.type === 'page' && t.url.includes('widget'));
if (widgetPage) {
  await test('widget penceresi yıkıcı komutları çağıramaz', async () => {
    const w = new WebSocket(widgetPage.webSocketDebuggerUrl);
    await new Promise((r) => (w.onopen = r));
    const res = await new Promise((resolve) => {
      w.onmessage = (m) => resolve(JSON.parse(m.data).result?.result?.value);
      w.send(JSON.stringify({ id: 1, method: 'Runtime.evaluate', params: { expression: "window.__TAURI_INTERNALS__.invoke('delete_backup', { name: 'wakai-20000101-000000.db' }).then(() => 'ok', (e) => String(e))", awaitPromise: true, returnByValue: true } }));
    });
    w.close();
    assert(String(res).includes('Kontrol Merkezi'), `beklenmeyen sonuç: ${res}`);
    // (aynı koruma: remove_world_item, import_theme, delete_custom_theme)
  });
}
console.log(`\n${passed} başarılı, ${failures.length} başarısız`);
process.exitCode = failures.length ? 1 : 0;
