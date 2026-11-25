# Testler

## Özet

| Katman | Komut | Adet | Ortam |
|---|---|---|---|
| Rust birim testleri | `cargo test` (src-tauri) | 94 | Her ortam |
| Rust bütünleşme testleri | `cargo test --test integration` | 6 | Her ortam (gerçek dosya üzerinde SQLite) |
| Windows çalışma zamanı testleri | `cargo test -- --ignored` | 2 | Gerçek Windows masaüstü oturumu + internet |
| Ön yüz birim testleri | `npm test` | 21 | Node (Vitest) |
| Uçtan uca testler | `node scripts/e2e.mjs` | 19 | Çalışan uygulama (CDP ile), yalıtılmış veri klasörü |

## Kapsam

**Rust birim testleri:** göçler ve geri alma, ayarların doğrulanması/sınırlanması, widget kayıt defteri ve yapılandırma birleştirme, etkinlik/görev/not depoları (doğrulama, sıralama, hatırlatıcı sıfırlama), odak zamanlayıcısı (duvar saati, duraklatma, döngü, otomatik başlatma), zamanlayıcı yardımcıları, dikdörtgen sığdırma (negatif koordinatlı monitörler), abonelik kenar geçişleri, base64, hava durumu ayrıştırma/önbellek/çevrim dışı, HTTPS URL ayrıştırma, yedekleme/geri yükleme ve yol geçişi koruması, bozuk veritabanının karantinaya alınması, süreç ölçümü, gerçek bellek/disk okumaları.

**Bütünleşme testleri** (`src-tauri/tests/integration.rs`):
- Şema v1'den yükseltme: veriler korunur, göç öncesi yedek oluşur.
- Daha yeni sürümün şeması geri alınmaz.
- Widget ve not verileri birbirinden bağımsızdır.
- Bozuk ayar kaydı varsayılanlara düşer.
- Odak oturumları yeniden başlatmadan sonra korunur.
- Yedekle → değiştir → geri yükle akışı.

**Windows çalışma zamanı testleri** (`--ignored`):
- `locates_desktop`: masaüstü hiyerarşisi bulunur (geliştirme makinesinde: `Modern` düzen).
- `real_https`: WinHTTP ile gerçek Open-Meteo isteği.

**Uçtan uca testler** (`scripts/e2e.mjs`): widget yaşam döngüsü, Türkçe doğrulama hataları, en küçük boyut, takvim/görev/not CRUD, odak zamanlayıcısı, tek örnekli widget kuralı, ayar sınırlama, yedekleme, kaynak ölçümü. Test oluşturduğu tüm kayıtları siler.

```powershell
# Birincil verilere dokunmamak için ayrı bir veri klasörü kullanın (isteğe bağlı olarak gerçek veritabanının kopyasıyla):
$env:WAKAI_DATA_DIR="$env:TEMP\wakai-e2e"
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222"
.\src-tauri\target\debug\wakai-desktop.exe
node scripts/e2e.mjs
```

> Not: Bu ortam değişkeni uygulamanın kendi tarayıcı bağımsız değişkenlerinin (renderer paylaşımı) yerine geçer; bu yüzden performans ölçümleri CDP olmadan yapılır.

## 1.1.0 ek kapsamı

**Rust birim testleri (yeni):** kedi modeli ve davranışı (ihtiyaçların geçen süreyle değişimi, çevrim dışı taban/üst sınır, kişilik ağırlıkları, etkinlik düzeyi), sprite verisi ve çizim, kedi hizmeti (sahiplenme, besleme, günlük yenileme, seri, başarımlar), Kedi Evi tek seferlik kurulum ve sıkıştırma, profiller (yerleşikler, görünürlük kuralları, ayar katmanı), alışveriş deposu, sürüm karşılaştırma, önceki sürümün saklanması, exe değişimi ve başarısız geri dönüşte geri alma, sürüm başına açılış sağlığı, otomatik geri dönüş koşulları, sürüm geçmişi, atomik yedek, bozuk/yabancı yedek reddi, yedek inceleme, seçmeli geri yükleme.

**Ön yüz birim testleri (yeni):** alışveriş girdisi ayrıştırma, kedi sprite çizimi (tüm animasyon × cins, aksesuarlar), Kedi Evi simülasyonu.

**Uçtan uca (yeni):** alışveriş listesi, kedi sahiplenme/besleme/çıkarma ve geçersiz girdiler, Kedi Evi eşyaları, profiller (yerleşik silinemez), güncelleme durumu ve sürüm geçmişi, yedek inceleme ve seçmeli geri yükleme doğrulaması, widget penceresinin yıkıcı komutları çağıramaması.

**Elle / yalıtılmış ortamda yapılan testler:** imzalı güncelleme (geçerli/değiştirilmiş/eski/erişilemez; `docs/GUNCELLEMELER.md`), otomatik ve elle geri dönüş (`docs/GUNCELLEMELER.md`), arayüzden seçmeli geri yükleme ve bozuk yedek (`docs/YEDEKLER.md`), performans (`docs/PERFORMANCE.md`).

**Gerçek fareyle denenmemiş olanlar:** kediyi okşama, sürükleyip fırlatma, sağ tık menüsü, pencere üstüne zıplama, tepsi profil menüsü, gerçek bir tam ekran oyunla otomatik Oyun profili, gerçek bir NSIS güncellemesinin uçtan uca kurulumu (imzalı ilk GitHub sürümünden sonra).

## Testlerin Bulduğu Hatalar

| Hata | Nasıl bulundu | Düzeltme |
|---|---|---|
| Bozuk ayar JSON'u ayarların yüklenmesini tamamen engelliyordu | Bütünleşme testi | Yalnızca JSON hatası varsayılanlara düşer; veritabanı hataları iletilir. |
| -0,4 °C "-0°" olarak gösteriliyordu | Ön yüz birim testi | Eksi sıfır önlendi. |
| Bozuk veritabanı dosyası uygulamanın açılmasını engelliyordu | Gözden geçirme | Dosya karantinaya alınır, yeni veritabanıyla başlanır, kullanıcı bildirim alır. |
| Senkron komutta pencere oluşturmak kilitleniyordu | Uçtan uca deneme (Aşama 3) | Pencere komutları `async` + ana iş parçacığı. |
| Sonsuz SVG CSS animasyonları boşta CPU harcıyordu | Performans ölçümü (Aşama 9) | Seyrek zamanlayıcılar. |

## Otomatik Test Edilemeyenler

Kullanıcının çalışan oturumunu bozmamak için şunlar otomatik test edilmedi: Explorer'ın yeniden başlatılması, monitör çıkarıp takma, medya oynatma komutları (kullanıcının müziğini keserdi), tepsi menüsüne tıklama, gerçek fareyle sürükleme/boyutlandırma.

### 1.1.0 sırasında bulunanlar

- `WM_MOUSELEAVE` kapsam dışıyken her mesajı yakalayan desen oluyordu (derleyici uyarısı).
- `PetStore` kaydedilmeden kullanılıyordu (ilk gerçek çalıştırma).
- Kedi kartında aksesuar seçimi taşıyordu (ekran görüntüsü).
- Güncelleme kurulumu, indirme başarısız olsa bile yedek ve exe kopyası bırakıyordu → hazırlık imza doğrulamasından sonraya alındı.
- Geçersiz sürümle geri dönüş isteği önce yedek alıyordu → doğrulama öne alındı.
- E2E en küçük boyut testi widget ölçeğini hesaba katmıyordu (uygulama doğru, test düzeltildi); betik çıkışta Windows'ta libuv iddiasıyla düşüyordu.
