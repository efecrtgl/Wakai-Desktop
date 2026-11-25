# Ivily Desktop 1.1.5 — Ön sürüm notları

**Wakai Desktop artık Ivily Desktop.** Bu ön sürüm yeni bir ad, baştan tasarlanmış bir Kontrol Merkezi, değiştirilebilir temalar ve oynanabilir bir Kedi Evi getiriyor. Widget'larınız, kedileriniz, notlarınız, görevleriniz, takviminiz, profilleriniz ve yedekleriniz olduğu gibi korunur.

> Bu bir **ön sürümdür**: yeni özellikler içerir ve hata barındırabilir. Kararlı güncelleme kanalındaki kurulumlara kendiliğinden önerilmez.

## Yeni

- **Beş tema:** Piksel (yeni varsayılan; bitmap yazı tipi, keskin pikseller), Modern, Sıcak Yuva, Tek Renk ve Y2K; her biri açık ve koyu çeşitleriyle. Tema; arayüzü, widget'ları, simgeleri, kedilerin renklerini ve Kedi Evi'ni birlikte değiştirir. Yeniden başlatma gerekmez.
- **Tema kitaplığı:** temaları kendi küçük önizlemelerinde inceleyin, uygulamanın tamamında geçici olarak deneyin (Uygula / Vazgeç), favorilere ekleyin, `.ivilytheme` dosyası olarak içe ve dışa aktarın.
- **Kedi Evi dünyası:** üç oda, kendi kararlarını veren kediler, doğrudan fareyle oyun (sevme, kucağa alma, oyuncak fırlatma, mama ve tarak), eşya yerleştirme ve döndürme, gün ve gece, pencereden yağmur ve kar, anılar, arkadaşlıklar, alışkanlıklar, seviyeler ve açılan eşyalar. Ayrıntılar: [Kedi Evi](KEDI_EVI.md).
- **Yeni ihtiyaçlar:** sosyallik, güven, merak, temizlik. **İkincil özellikler:** Yaramaz, Utangaç, Tembel.
- **Yeni Kontrol Merkezi:** Genel Bakış, Widget'lar, Planlayıcı, Kedi Evi, Temalar, Kişiselleştirme, Ayarlar.
- **Arama (Ctrl+K):** ayarları ve özellikleri bulur, doğrudan ilgili ayara götürür.
- **Kişiselleştirme:** vurgu rengi, arayüz yoğunluğu, yazı tipi.
- **Erişilebilirlik:** yazı boyutu, yüksek karşıtlık, animasyon yoğunluğu, arayüz sesleri (varsayılan kapalı), klavye kısayolları.
- **Güncelleme kanalı:** Kararlı ya da Ön sürüm.

## İyileştirmeler

- Profiller artık bir tema da seçebilir.
- Masaüstü kedileri temanın kedi paletini kullanır.
- Kedilere tımarlanma, gerinme, esneme, tırmalama ve sinme animasyonları eklendi.

## Değişenler

- Uygulama adı, pencere başlıkları, tepsi, bildirimler, kısayollar ve yükleyici "Ivily Desktop" oldu. Program klasörü `%LOCALAPPDATA%\Ivily Desktop` oldu; eski "Wakai Desktop" kısayolları ve kaldırma kaydı yükleyici tarafından temizlenir. **Veri klasörü değişmedi** (`%APPDATA%\com.wakai.desktop`).
- 1.1.0'daki görünüm seçiminiz (Wakai, Siyah, Beyaz) Modern temanın karşılık gelen çeşidine taşınır. Yeni kurulumlar Piksel temayla başlar.
- Eski "Görünüm" bölümü Temalar ve Kişiselleştirme'ye ayrıldı; "Performans" ve "Bildirimler" Ayarlar'ın altına taşındı.

## Bilinen sorunlar ve sınırlamalar

- Otomatik güncelleme için GitHub deposunun (ya da ayrı bir sürüm deposunun) herkese açık olması gerekir; şu an depo herkese açık değil.
- 1.1.0'da güncelleme kanalı seçimi olmadığı için 1.1.0 kurulumları ön sürümü otomatik görmez; denemek için 1.1.5 yükleyicisini bir kez elle kurun.
- Kedi Evi'nde kediler odalar arasında kendiliğinden geçmez; kapıya sürükleyerek taşınırlar.
- Ortam sesleri (yağmur sesi vb.) yoktur; yalnızca isteğe bağlı kısa arayüz sesleri vardır.
- Kedi Evi'ndeki fare etkileşimleri otomatik testlerde sentetik fare olaylarıyla denendi; dokunmatik ekran denenmedi.
- Önceki sürüme (1.1.0) dönülürse 1.1.0 kendi eski Kedi Evi düzenini gösterir; 1.1.5'te eklenen tema, dünya ve gelişim verileri silinmez, yeniden 1.1.5'e geçildiğinde kullanılır.

## Deneysel

- Kedi yapay zekâsının denge ayarları (ne sıklıkla oynadıkları, uyudukları) geri bildirimle değişebilir.
- Tema paketi biçimi (şema 1) ileride genişleyebilir; şema sürümü ve uyumluluk denetimi bunun için vardır.
