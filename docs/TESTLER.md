# Testler

## Özet

| Katman | Komut | Adet | Ortam |
|---|---|---|---|
| Rust birim testleri | `cargo test` (src-tauri) | 55 | Her ortam |
| Rust bütünleşme testleri | `cargo test --test integration` | 6 | Her ortam (gerçek dosya üzerinde SQLite) |
| Windows çalışma zamanı testleri | `cargo test -- --ignored` | 2 | Gerçek Windows masaüstü oturumu + internet |
| Ön yüz birim testleri | `npm test` | 12 | Node (Vitest) |
| Uçtan uca testler | `node scripts/e2e.mjs` | 12 | Çalışan uygulama (CDP ile) |

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
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222"
.\src-tauri\target\debug\wakai-desktop.exe
node scripts/e2e.mjs
```

> Not: Bu ortam değişkeni uygulamanın kendi tarayıcı bağımsız değişkenlerinin (renderer paylaşımı) yerine geçer; bu yüzden performans ölçümleri CDP olmadan yapılır.

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
