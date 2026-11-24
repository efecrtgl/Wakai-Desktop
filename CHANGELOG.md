# Değişiklik Günlüğü

Bu projedeki önemli değişiklikler bu dosyada belgelenir. Biçim [Keep a Changelog](https://keepachangelog.com/tr/1.1.0/) esas alınarak hazırlanmıştır.

## [Yayınlanmadı]

### Eklendi
- Tauri 2 + React (JavaScript) proje temeli.
- SQLite veritabanı, sürümlü göçler ve göç öncesi otomatik yedek.
- Türlü uygulama ayarları (doğrulama ve sınırlandırma ile).
- Pastel tasarım belirteçleri ve temel arayüz bileşenleri.
- Kontrol Merkezi kabuğu: Genel Bakış ve Ayarlar.
- Windows ile başlatma ayarı (gerçek başlangıç kaydı ile).
- Tek örnek (single-instance) davranışı.
- Windows 11 masaüstü katmanı: widget'lar duvar kağıdının üstünde, simgelerin altında (24H2 hiyerarşisi doğrulandı).
- "Üzerine gelince kaldır" etkileşim modeli (Raw Input ile olay güdümlü).
- Karma DPI'lı çoklu monitörlerde doğru içerik ölçeği.
- Explorer yeniden başlatma, ekran değişikliği ve oturum kapanışı izleme.
- Widget çalışma zamanı: kayıt defteri, örnekler, yaşam döngüsü, doğrulanmış yapılandırma, gecikmeli konum kaydı, kaybolan pencerelerin yeniden oluşturulması.
- Widget çerçevesi: sürükleme, boyutlandırma ve sağ tık menüsü.
- Kontrol Merkezi'nin tamamı: Genel Bakış (istatistikler, yaklaşanlar, görevler, performans, hızlı kontroller), Widget'larım (widget'a özel ayar formları, boyut, görünürlük, tıklama geçirgenliği, sıfırlama), Widget Mağazası, Takvim, Görevler, Notlar, Görünüm, Performans, Bildirimler ve Ayarlar.
- Veritabanı yedekleme ve güvenli geri yükleme (bir sonraki açılışta, öncesinde otomatik yedekle).
- Widget ölçeği: tüm widget'ları ve içeriklerini orantılı büyütme/küçültme.
- Deneme bildirimi ve uygulamanın kendi kaynak kullanımı göstergesi.
- Takvim Kedisi: Türkçe aylık takvim, etkinlik ekleme/düzenleme/silme, kategoriler, renk etiketleri, hatırlatıcılar, yaklaşan etkinlikler.
- Görev Kedisi: öncelik, son tarih, kategori, hatırlatıcı, ilerleme çubuğu ve tamamlama kutlaması.
- Not Kedisi: otomatik kayıt, sabitleme, renkler, basit biçimlendirme (başlık, madde, yapılacak kutusu, kalın, italik).
- Saat Kedisi: dijital/analog, saniye, 12/24 saat, tarih ve ek saat dilimleri.
- Odak Kedisi: duvar saatine dayalı Pomodoro zamanlayıcısı, kısa/uzun molalar, oturum geçmişi, bildirimler.
- Olay güdümlü hatırlatıcı zamanlayıcısı ve Windows bildirimleri.
- Veritabanı göçü 002 (içerik tabloları).
- Hava Durumu Kedisi: Open-Meteo (API anahtarsız), konum arama, 5 günlük tahmin, önbellek ve çevrim dışı gösterim.
- Müzik Kedisi: Windows medya oturumu (GSMTC) ile parça bilgisi, kapak ve oynatma kontrolleri.
- Sistem Kedisi: CPU, bellek, GPU (PDH), disk ve açık kalma süresi; yalnızca açıkken ve ayarlanan aralıkla ölçüm.
- Abonelik modeli: arka plan hizmetleri yalnızca gösteren bir pencere varken çalışır.
- Özgün piksel kedi ailesi: oturan, mutlu, meraklı, rahat, çalışan, uyuyan kedi ve yüz; göz kırpma, kuyruk sallama, Zz ve kutlama animasyonları.
- Piksel simge seti ve widget görsel yapı taşları.
- Hafif günlük kaydı (`wakai.log`, yalnızca uyarı ve üstü).
