// Kontrol Merkezi bilgi mimarisi: bölümler, sekmeler, eski bölüm kimliklerinin yönlendirmesi
// ve ayar/özellik arama dizini. Bileşen içermez; test edilebilir.

/** Ana gezinme. `tabs` olan bölümlerde ilk sekme varsayılandır. */
export const NAV = [
  { id: 'overview', label: 'Genel Bakış', icon: 'home' },
  {
    id: 'widgets',
    label: "Widget'lar",
    icon: 'widgets',
    tabs: [
      { id: 'mine', label: 'Masaüstümde' },
      { id: 'store', label: 'Widget ekle' },
    ],
  },
  {
    id: 'planner',
    label: 'Planlayıcı',
    icon: 'calendar',
    tabs: [
      { id: 'calendar', label: 'Takvim' },
      { id: 'tasks', label: 'Görevler' },
      { id: 'notes', label: 'Notlar' },
    ],
  },
  {
    id: 'pethome',
    label: 'Kedi Evi',
    icon: 'paw',
    tabs: [
      { id: 'world', label: 'Dünya' },
      { id: 'cats', label: 'Kediler' },
    ],
  },
  { id: 'themes', label: 'Temalar', icon: 'layers' },
  {
    id: 'personalize',
    label: 'Kişiselleştirme',
    icon: 'sliders',
    tabs: [
      { id: 'look', label: 'Görünüm' },
      { id: 'profiles', label: 'Profiller' },
    ],
  },
  {
    id: 'settings',
    label: 'Ayarlar',
    icon: 'gear',
    tabs: [
      { id: 'general', label: 'Genel' },
      { id: 'accessibility', label: 'Erişilebilirlik' },
      { id: 'performance', label: 'Performans' },
      { id: 'notifications', label: 'Bildirimler' },
      { id: 'updates', label: 'Güncellemeler' },
      { id: 'backup', label: 'Yedekleme' },
      { id: 'about', label: 'Hakkında' },
    ],
  },
];

/** 1.1.0 bölüm kimlikleri (widget menüsü, tepsi, kedi menüsü bunları gönderir). */
const LEGACY = {
  store: ['widgets', 'store'],
  calendar: ['planner', 'calendar'],
  tasks: ['planner', 'tasks'],
  notes: ['planner', 'notes'],
  pets: ['pethome', 'cats'],
  profiles: ['personalize', 'profiles'],
  appearance: ['themes', null],
  performance: ['settings', 'performance'],
  notifications: ['settings', 'notifications'],
};

/** Bölüm/sekme isteğini geçerli bir rotaya çevirir. Bilinmeyen bölüm → Genel Bakış. */
export function resolveRoute(section, sub = null) {
  if (LEGACY[section]) {
    const [s, t] = LEGACY[section];
    return { section: s, sub: sub ?? t };
  }
  const nav = NAV.find((n) => n.id === section);
  if (!nav) return { section: 'overview', sub: null };
  if (!nav.tabs) return { section: nav.id, sub: null };
  const tab = nav.tabs.find((t) => t.id === sub) ?? nav.tabs[0];
  return { section: nav.id, sub: tab.id };
}

/** Türkçe duyarlı arama normalleştirmesi: küçük harf, aksanlar ve ı/İ eşlenir. */
export function normalize(text) {
  return String(text)
    .toLocaleLowerCase('tr')
    .replace(/ı/g, 'i')
    .replace(/ş/g, 's')
    .replace(/ğ/g, 'g')
    .replace(/ü/g, 'u')
    .replace(/ö/g, 'o')
    .replace(/ç/g, 'c')
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '');
}

/**
 * Arama dizini: her kayıt bir ayar ya da özelliktir. `anchor` sayfadaki öğenin kimliğidir
 * (bulunursa kaydırılıp vurgulanır).
 */
export const SEARCH_INDEX = [
  { label: 'Genel Bakış', section: 'overview', keywords: 'ana sayfa özet durum' },
  { label: "Widget'ları göster / gizle", section: 'widgets', sub: 'mine', keywords: 'masaüstü görünür gizle' },
  { label: 'Yeni widget ekle', section: 'widgets', sub: 'store', keywords: 'mağaza saat hava takvim not görev müzik sistem odak alışveriş' },
  { label: 'Takvim ve etkinlikler', section: 'planner', sub: 'calendar', keywords: 'etkinlik randevu hatırlatıcı' },
  { label: 'Görevler', section: 'planner', sub: 'tasks', keywords: 'yapılacaklar liste' },
  { label: 'Notlar', section: 'planner', sub: 'notes', keywords: 'not defter' },
  { label: 'Kedi Evi dünyası', section: 'pethome', sub: 'world', keywords: 'oyun oda mobilya envanter oyuncak' },
  { label: 'Kediler: sahiplen, besle, aksesuar', section: 'pethome', sub: 'cats', keywords: 'kedi evcil hayvan mama aksesuar başarım anı' },
  { label: 'Kedi Evi saati, hava ve ortam efektleri', section: 'pethome', sub: 'cats', anchor: 'set-ambient', keywords: 'gece gündüz yağmur kar parçacık efekt hızlı gün' },
  { label: 'Tema kitaplığı', section: 'themes', keywords: 'tema piksel modern sıcak yuva tek renk y2k görünüm koyu açık içe aktar dışa aktar' },
  { label: 'Vurgu rengi', section: 'personalize', sub: 'look', anchor: 'set-accent', keywords: 'renk ana renk özelleştir' },
  { label: 'Arayüz yoğunluğu', section: 'personalize', sub: 'look', anchor: 'set-density', keywords: 'sıkı ferah boşluk' },
  { label: 'Yazı tipi', section: 'personalize', sub: 'look', anchor: 'set-font', keywords: 'font bahnschrift cascadia' },
  { label: 'Widget opaklığı, ölçeği ve köşeleri', section: 'personalize', sub: 'look', anchor: 'set-widget-look', keywords: 'saydamlık boyut gölge köşe yuvarlaklığı' },
  { label: 'Masaüstü profilleri', section: 'personalize', sub: 'profiles', keywords: 'çalışma oyun dinlenme odaklanma profil' },
  { label: 'Tepside çalışmaya devam et', section: 'settings', sub: 'general', anchor: 'set-tray', keywords: 'kapat tepsi sistem tepsisi' },
  { label: 'Windows ile başlat', section: 'settings', sub: 'general', anchor: 'set-startup', keywords: 'başlangıç otomatik açılış' },
  { label: 'Hareketi azalt / animasyonlar', section: 'settings', sub: 'accessibility', anchor: 'set-motion', keywords: 'animasyon hareket erişilebilirlik' },
  { label: 'Yazı boyutu', section: 'settings', sub: 'accessibility', anchor: 'set-text-scale', keywords: 'büyük yazı okunabilirlik ölçek' },
  { label: 'Yüksek karşıtlık', section: 'settings', sub: 'accessibility', anchor: 'set-contrast', keywords: 'kontrast okunabilirlik' },
  { label: 'Arayüz sesleri', section: 'settings', sub: 'accessibility', anchor: 'set-sounds', keywords: 'ses tıklama' },
  { label: 'Klavye kısayolları', section: 'settings', sub: 'accessibility', anchor: 'set-shortcuts', keywords: 'kısayol klavye ctrl' },
  { label: 'Düşük kaynak modu', section: 'settings', sub: 'performance', anchor: 'set-low-resource', keywords: 'pil performans bellek işlemci' },
  { label: 'Kaynak kullanımı', section: 'settings', sub: 'performance', keywords: 'bellek işlemci cpu ram' },
  { label: 'Bildirimler', section: 'settings', sub: 'notifications', keywords: 'hatırlatıcı uyarı' },
  { label: 'Güncellemeleri denetle', section: 'settings', sub: 'updates', keywords: 'güncelleme sürüm yeni ön sürüm kanal' },
  { label: 'Sürüm geçmişi ve geri dönüş', section: 'settings', sub: 'updates', anchor: 'set-versions', keywords: 'önceki sürüm geri al' },
  { label: 'Yedekler ve geri yükleme', section: 'settings', sub: 'backup', keywords: 'yedek geri yükle veri' },
  { label: 'Hakkında', section: 'settings', sub: 'about', keywords: 'sürüm lisans veri konumu' },
];

/** Basit, hızlı arama: tüm sorgu sözcükleri etiket ya da anahtar sözcüklerde geçmeli. */
export function search(query, index = SEARCH_INDEX, limit = 8) {
  const words = normalize(query).split(/\s+/).filter(Boolean);
  if (!words.length) return [];
  const scored = [];
  for (const item of index) {
    const label = normalize(item.label);
    const hay = `${label} ${normalize(item.keywords ?? '')}`;
    if (!words.every((w) => hay.includes(w))) continue;
    const score = words.reduce((s, w) => s + (label.startsWith(w) ? 3 : label.includes(w) ? 2 : 1), 0);
    scored.push({ item, score });
  }
  return scored
    .sort((a, b) => b.score - a.score)
    .slice(0, limit)
    .map((s) => s.item);
}
