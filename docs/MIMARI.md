# Wakai Desktop — Mimari

Bu belge Wakai Desktop'un teknik mimarisini, modül sınırlarını ve önemli tasarım kararlarını açıklar. Belge, uygulamanın **gerçek** durumunu yansıtacak şekilde her geliştirme aşamasının sonunda güncellenir.

## 1. Genel Bakış

Wakai Desktop iki ana parçadan oluşur:

| Parça | Açıklama |
|---|---|
| **Masaüstü Widget Katmanı** | Duvar kağıdının hemen üstünde, masaüstü simgelerinin altında yaşayan bağımsız widget pencereleri. |
| **Kontrol Merkezi** | Widget'ları, içerikleri (takvim, görev, not) ve ayarları yöneten normal uygulama penceresi. |

```
┌──────────────────────────────── Rust (Tauri 2) ─────────────────────────────────┐
│ app (yaşam döngüsü, durum)   commands (IPC)   events (olay yayını)              │
│ db (SQLite + göçler)         widgets (kayıt, örnek, yönetici)                    │
│ desktop (Win32 masaüstü katmanı, girdi yönlendirme, kabuk izleyici)             │
│ monitor (CPU/RAM/GPU)  media (GSMTC)  weather (sağlayıcı)  scheduler (hatırlatıcı)│
└──────────────▲─────────────────────────────────────────────▲────────────────────┘
               │ invoke / emit                               │
   ┌───────────┴───────────┐                     ┌───────────┴───────────┐
   │ Kontrol Merkezi        │                     │ Widget pencereleri      │
   │ index.html (React)     │                     │ widget.html?id=... (React)│
   └────────────────────────┘                     └─────────────────────────┘
```

## 2. Temel İlkeler

1. **Tek doğruluk kaynağı Rust tarafıdır.** Kalıcı veriler yalnızca SQLite'ta tutulur; ön yüz durumu bir önbellektir.
2. **Olay güdümlü çalışma.** Veri değiştiğinde Rust ilgili olayı (`tasks:changed`, `widgets:changed` …) yayınlar; pencereler yalnızca etkilendikleri veriyi yeniden ister. Sürekli yoklama (polling) yapılmaz.
3. **Zamanlayıcılar ihtiyaca göre.** Hatırlatıcılar için tek bir zamanlayıcı iş parçacığı bir sonraki tetiklenme anına kadar uyur; değişiklikte yeniden hesaplar.
4. **Güvensiz Windows API çağrıları izole edilir.** Tüm `unsafe` Win32 kodu `desktop` modülündedir.
5. **Widget tanımı ≠ widget örneği.** Tanımlar kayıt defterinde (registry) statik olarak tutulur; örnekler veritabanında kalıcıdır.

## 3. Rust Modülleri (`src-tauri/src`)

| Modül | Sorumluluk |
|---|---|
| `app` | Uygulama durumu (`AppState`), başlatma ve kapatma sırası. |
| `db` | Bağlantı, göç (migration) çalıştırıcı, depo (repository) fonksiyonları. |
| `commands` | Ön yüze açılan Tauri komutları; tüm girdiler burada doğrulanır. |
| `events` | Olay adları ve yayın yardımcıları. |
| `widgets` | Widget kayıt defteri, örnek modeli, yaşam döngüsü yöneticisi, pencere oluşturma. |
| `desktop` | Windows masaüstü hiyerarşisi (Progman/WorkerW/DefView), pencere yerleştirme, z-sırası, fare girdisi yönlendirme, Explorer yeniden başlatma ve ekran değişikliği izleme. |
| `monitor` | Sistem kaynak ölçümü (CPU, bellek, GPU) — yalnızca abone varken ve ayarlanan aralıkla. |
| `media` | Windows Global System Media Transport Controls (GSMTC) entegrasyonu. |
| `weather` | Sağlayıcı arayüzü ve Open-Meteo uygulaması (API anahtarı gerektirmez). |
| `scheduler` | Etkinlik/görev hatırlatıcıları ve odak zamanlayıcısı bildirimleri. |
| `settings` | Uygulama ayarlarının okunması/yazılması ve varsayılanlar. |
| `tray`, `startup` | Sistem tepsisi, Windows ile başlatma. |

## 4. Ön Yüz (`src`)

- **React + JavaScript (JSX)**, TypeScript kullanılmaz.
- **İki giriş noktası** (Vite çoklu sayfa):
  - `index.html` → Kontrol Merkezi
  - `widget.html` → Widget barındırıcı; URL'deki `id` ile örneği bulur ve yalnızca ilgili widget modülünü **dinamik içe aktarma** ile yükler. Böylece her widget penceresi yalnızca kendi kodunu çalıştırır.
- `src/ui` → tasarım sistemi (CSS değişkenleri, temel bileşenler, piksel kedi sprite'ları).
- `src/lib` → IPC sarmalayıcıları, Türkçe yerelleştirme ve tarih yardımcıları.
- `src/widgets/<ad>` → her widget'ın bileşenleri.

## 5. Masaüstü Katmanı Stratejisi

Ayrıntılar: [WINDOWS_ENTEGRASYONU.md](WINDOWS_ENTEGRASYONU.md).

- Her widget örneği ayrı bir Tauri penceresidir; hepsi **aynı WebView2 ortamını** (tarayıcı ve GPU sürecini) paylaşır.
- Pencere oluşturulduktan sonra `desktop` modülü pencereyi `Progman` altındaki duvar kağıdı `WorkerW`'sinin alt penceresi yapar: simgelerin altında, duvar kağıdının üstünde.
- Simge katmanı fare olaylarını engellediği için, imleç bir widget'ın üzerine geldiğinde (Raw Input ile olay güdümlü algılanır) widget geçici olarak masaüstünün hemen üstüne, tüm uygulama pencerelerinin altına kaldırılır ve doğal fare/klavye girdisi alır.
- İmleç ayrılıp widget odağı kaybettiğinde widget katmana geri döner.

## 6. Veri Akışı Örneği (görev tamamlama)

1. Widget `toggle_task(id)` komutunu çağırır.
2. Rust girdiyi doğrular, SQLite'ta tek bir `UPDATE` çalıştırır.
3. Rust `tasks:changed` olayını tüm pencerelere yayınlar.
4. Görev listesini gösteren pencereler listeyi yeniden ister; diğer pencereler hiçbir şey yapmaz.

## 7. Bilinen Riskler

| Risk | Önlem |
|---|---|
| Masaüstü hiyerarşisi Windows sürümlerine göre değişir (24H2 ile değişti). | Hiyerarşi tespiti tek bir modülde, iki strateji (24H2+ ve eski düzen) ile. |
| Süreçler arası `SetParent` girdi kuyruklarını birleştirir; UI iş parçacığımız kilitlenirse masaüstü de yanıt vermez. | UI iş parçacığında engelleyici iş yapılmaz; ağır işler arka plan iş parçacıklarında. |
| Alt pencerede WebView2 saydamlığı garanti değildir. | Aşama 2'de gerçek davranış doğrulanır; gerekirse pencere bölgesi (`SetWindowRgn`) ile köşe kırpma. |
| Her WebView ayrı bir renderer süreci açar. | Ölçülür; widget sayısı ve kod boyutu sınırlandırılır, etkin olmayan widget'lar askıya alınır. |
