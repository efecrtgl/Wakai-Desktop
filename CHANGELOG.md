# Değişiklik Günlüğü

Bu projedeki önemli değişiklikler bu dosyada belgelenir. Biçim [Keep a Changelog](https://keepachangelog.com/tr/1.1.0/) esas alınarak hazırlanmıştır.

## [Yayınlanmadı] — Ivily Desktop 1.1.5 (ön sürüm)

### Eklendi
- Tema motoru: temalar bildirimsel paketlerdir (renkler, yazı tipleri, biçim, yüzey, kenarlık, simge seti, doku, hareket karakteri, kedi paleti, Kedi Evi zemini/duvarı, ortam efekti, sesler). Yeniden başlatma gerekmeden tek adımda değişir; bozuk tema uygulamayı etkilemez, güvenli Modern temaya dönülür.
- Beş tema: **Piksel** (varsayılan; bitmap yazı tipi, sert gölgeler, piksel simgeler), **Modern**, **Sıcak Yuva**, **Tek Renk**, **Y2K**; her biri açık/koyu çeşitleriyle.
- Vektör çizgi simge seti (piksel olmayan temalarda).
- Yeni Kontrol Merkezi: Genel Bakış, Widget'lar, Planlayıcı, Kedi Evi, Temalar, Kişiselleştirme ve Ayarlar. Ayarlar kategorilere ayrıldı (Genel, Erişilebilirlik, Performans, Bildirimler, Güncellemeler, Yedekleme, Hakkında).
- Uygulama geneli arama (Ctrl+K): ayarları ve özellikleri Türkçe karakter duyarsız bulur, ilgili ayara gidip vurgular. Ctrl+1…7 ile bölümler arasında geçiş.
- Tema kitaplığı: yalıtılmış önizlemeler, uygulamanın tamamında geçici önizleme (Uygula / Vazgeç), favoriler, sürüm ve uyumluluk bilgisi, `.ivilytheme` içe/dışa aktarma, özel tema kaldırma.
- Kişiselleştirme: vurgu rengi (okunabilir metin rengi otomatik), arayüz yoğunluğu, yazı tipi, widget görünümü.
- **Kedi Evi dünyası:** üç oda (Oturma odası, Yatak odası, Oyun odası), kapılardan geçiş, karo tabanlı yerleşim, çarpışma ve A* yol bulma. Doğrudan fareyle oynanır: kediyi üzerinde gezinerek sev, sürükleyip taşı, kapıya bırakıp diğer odaya geçir; oyuncağı fırlat, kediler kovalar; mamayı kaba ya da kediye sürükle; tarakla tara. Düzenle kipinde mobilyayı taşı, döndür (R), envantere kaldır (Delete); yeşil/kırmızı yerleşim önizlemesi.
- Kendi kararını veren kediler: ihtiyaçlar, kişilik, ikincil özellik (Yaramaz, Utangaç, Tembel), alışkanlıklar, günün saati, eşyalar ve diğer kedilerle ilişkiler; uyuma, yeme, su içme, oynama, tımarlanma, tırmalama, tırmanma, pencereden bakma, keşif, arkadaşına sokulma, ilgi isteme, saklanma. Kedinin ne yaptığı ve nedeni yan panelde görünür.
- Yeni ihtiyaçlar: sosyallik, güven, merak, temizlik. Kedi ve ev seviyeleri, seviyeyle açılan eşyalar, anılar, arkadaşlıklar, alışkanlıklar (en sevdiği oyuncak, eşya, uyku yeri, saat dilimi).
- Yeni animasyonlar: tımarlanma, gerinme, esneme, tırmalama, sinme; uyanma → gerinme geçişi.
- Gün döngüsü (gerçek saat ya da hızlı gün), akşam/gece ışığı, gece yanan lambalar, pencereden yağmur, kar ve yıldızlar, temaya göre ortam parçacıkları.
- Çevrim dışı ilerleme (en fazla 24 saat): kediler kaptan yer, oyuncakları dağıtır; dönüşte kısa bir özet.
- Masaüstü kedileri de temanın kedi paletini kullanır.
- Profiller artık bir tema (ve çeşidi) da seçebilir; profil değişince Kedi Evi ve widget yerleşimi korunur.
- Kediler bölümü: sahiplenirken ikincil özellik seçimi, kedi kartında anılar.
- Erişilebilirlik: yazı boyutu (%85–140), yüksek karşıtlık, animasyon yoğunluğu, arayüz sesleri (varsayılan kapalı), klavye kısayolları listesi.

### Değişti
- Wakai Desktop artık **Ivily Desktop**. Pencere başlıkları, tepsi, bildirimler, kısayollar ve yükleyici yeni adı kullanır. Veri klasörü ve tüm kullanıcı verileri olduğu gibi kalır.
- Yükleyici eski "Wakai Desktop" kurulumunu taşır: eski kısayollar Ivily kısayollarıyla değiştirilir, eski program klasörü ve kaldırma kaydı temizlenir, eski otomatik başlatma kaydı kaldırılır.
- 1.1.0'daki görünüm seçimi (Wakai/Siyah/Beyaz) korunur: Modern temanın karşılık gelen çeşidine taşınır.
- Wakai'den gelen kullanıcılara bir kez, sessiz bir "Wakai Desktop artık Ivily Desktop." notu gösterilir.

## [1.1.0] — 2026-11-25

### Eklendi
- Görünüm modları: Wakai (özgün), Siyah ve Beyaz.
- Yazı tipi seçimi (Windows 11 ile gelen yazı tipleri) ve tutarlı yazı ölçeği.
- Alışveriş Kedisi: miktar ayrıştırmalı alışveriş listesi.
- Masaüstü kedileri: yerel, hafif kedi motoru; 8 cins, 6 kişilik, ihtiyaçlar, besleme, sevme, oyuncaklar, aksesuarlar, başarımlar.
- Kedi Evi: düzenlenebilir piksel sanatı oda.
- Masaüstü profilleri (Çalışma, Oyun, Dinlenme, Odaklanma, özel profiller), tepsiden hızlı geçiş ve isteğe bağlı tam ekranda otomatik Oyun profili.
- İmzalı otomatik güncellemeler (Tauri Updater + GitHub Releases): açılışta arka planda denetim, "Yeni bir sürüm hazır!" penceresi, indirme ilerlemesi, elle denetim, kapatılabilir. Onaysız kurulum yok.
- Sürüm geçmişi, kurulumdan önce önceki sürümün saklanması, Ayarlar'dan önceki sürüme dönüş ve art arda başarısız açılışta otomatik geri dönüş.
- Yedek inceleme (bütünlük, şema uyumluluğu, içerik) ve seçmeli geri yükleme (yalnızca seçilen kategoriler).
- Tepsi menüsüne "Masaüstü kedileri" ve profil alt menüsü.

### Değişti
- Takvim Kedisi yeniden tasarlandı (okunaklı başlık, Bugün kısayolu, klavye gezinmesi, renk çubuklu etkinlikler, kompakt düzen).
- Widget'lar kendi boyutlarına göre sadeleşir.
- Yedekler atomik yazılır ve doğrulanır; geri yükleme geçici dosya üzerinden doğrulanarak yapılır.
- Geri yükleme, geri dönüş, güncelleme kurulumu ve yedek silme yalnızca Kontrol Merkezi'nden yapılabilir.
- Widget'lar açıkken özel bellek 375 MB'tan 362 MB'a indi; kedi motoru ölçülebilir ek yük getirmez (docs/PERFORMANCE.md).

### Geçiş notu
- 1.0.0 güncelleyici içermediği için 1.1.0 bir kez elle kurulmalıdır; veriler korunur ve göçler otomatik uygulanır. Sonraki sürümler otomatik güncellenir.

## [1.0.0] — 2026-11-24

İlk sürüm.

### Düzeltildi
- Bozuk ayar kaydı ayarların yüklenmesini engelliyordu.
- Eksi sıfır sıcaklık "-0°" görünüyordu.

### Kaldırıldı
- İşlevi olmayan "Örtülünce askıya al" ayarı (gerekçe: docs/PERFORMANS.md §7).

### Eklendi
- Türkçe NSIS yükleyici (kullanıcı başına kurulum, yönetici izni gerekmez, yaklaşık 2,5 MB), uygulama bilgileri, MIT lisansı.
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
- Sistem tepsisi: Kontrol Merkezi, widget görünürlüğü, toplu etkileşim kilidi açma, çıkış.
- Tepsiye kapatmada Kontrol Merkezi penceresi tamamen kapatılır (bellek serbest kalır) ve gerektiğinde yeniden oluşturulur.
- Monitör geri takıldığında widget'lar kayıtlı konumlarına döner.
- Düzenli kapanış ve panik günlüğü.
- Performans: boşta CPU ≈65 kat azaldı (piksel kedi animasyonları seyrek zamanlayıcılara taşındı), WebView renderer süreçleri paylaşıldı, widget'lara düşük bellek hedefi; ölçüm betiği `scripts/measure.ps1` ve ölçümlerle Türkçe performans raporu.
- Bozuk veritabanı dosyası karantinaya alınır; uygulama yeni veritabanıyla açılır ve kullanıcıyı bilgilendirir.
- Bütünleşme testleri, Windows çalışma zamanı testleri ve uçtan uca test betiği (`scripts/e2e.mjs`).
- Hafif günlük kaydı (`wakai.log`, yalnızca uyarı ve üstü).
