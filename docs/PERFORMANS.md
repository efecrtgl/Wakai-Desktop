# Performans

Performans Wakai Desktop'ta son rötuş değil, mimari bir gerekliliktir. Bu belge ölçüm yöntemini, **gerçek** ölçüm sonuçlarını, yapılan iyileştirmeleri ve bilinen darboğazları içerir.

> Kural: Burada yer alan her sayı gerçekten ölçülmüştür. Ölçülmemiş değer yazılmaz.

## 1. Test Koşulları

| Öğe | Değer |
|---|---|
| İşletim sistemi | Windows 11 Home, derleme 26200 |
| İşlemci | 32 mantıksal çekirdek |
| Bellek | 31,2 GB |
| Monitörler | 2 (1920×1200 %125 + 1920×1080 %100) |
| WebView2 Runtime | 158.0.4370.33 |
| Derleme | `cargo build --release` (LTO, `opt-level = "s"`, `codegen-units = 1`) |
| Widget'lar | Takvim, Görev, Not, Saat ×2, Odak, Hava Durumu, Müzik, Sistem (9 pencere) |
| Ölçüm sırasında | Spotify çalıyordu (Müzik Kedisi olaylar alıyordu); masaüstünün büyük kısmı başka pencerelerle örtülüydü |
| Tarih | 24 Kasım 2026 |

## 2. Yöntem

### 2.1 Araç: `scripts/measure.ps1`

```powershell
powershell -File scripts\measure.ps1 -Duration 60 -Label "9 widget, boşta"
```

- `wakai-desktop.exe` sürecini ve ondan türeyen **tüm** süreçleri (WebView2 tarayıcı, GPU, yardımcı ve renderer süreçleri) `Win32_Process.ParentProcessId` ile bulur.
- Saniyede bir örnekler: özel bellek (Private Bytes) toplamı, çalışma kümesi (Working Set) toplamı, toplam CPU süresi.
- **CPU %** = CPU süresi farkı / (geçen süre × mantıksal çekirdek sayısı). Yani %1, *tüm işlemcinin* %1'idir (bu makinede ≈ bir çekirdeğin %32'si).

### 2.2 Senaryolar

- Kontrol Merkezi kapalı ölçümler `--autostart` ile başlatılarak yapıldı (yalnızca widget'lar açılır).
- Widget sayısı, uygulama kapalıyken veritabanında görünürlük değiştirilerek ayarlandı.
- Her ölçümden önce 10–12 sn ısınma beklendi; ölçüm süresi 30–60 sn.
- **Başlatma süresi:** süreç başlatıldıktan sonra uygulamaya ait WebView2 süreç sayısının 2 sn boyunca sabit kaldığı ilk an (tüm pencerelerin WebView'ı oluşmuş). 3 deneme.

### 2.3 Uygulama içi gösterge

Kontrol Merkezi → Performans → "Wakai Desktop" kartı aynı süreç ağacı toplamını (`app_resource_usage` komutu) gösterir.

## 3. Sonuçlar

### 3.1 İyileştirme adımları (9 widget, Kontrol Merkezi kapalı, boşta)

| Adım | Süreç | Özel bellek (ort.) | Çalışma kümesi (ort.) | CPU (ort., tüm işlemci) |
|---|---|---|---|---|
| Başlangıç (temel ölçüm) | 15 | 542,1 MB | 1138 MB | **%2,133** |
| 1. Piksel kedi animasyonları seyrek zamanlayıcılara taşındı | 15 | 465,3 MB | 1037 MB | **%0,033** |
| 2. Renderer süreçleri paylaşıldı (`--renderer-process-limit=2`) | 11 | 370,0 MB | 768 MB | %0,100 |
| 3. Widget WebView'larına düşük bellek hedefi | 11 | 375,9 MB | **369 MB** | %0,067 |

Toplam: CPU **≈ 65 kat**, özel bellek **%31**, çalışma kümesi **%68** azaldı.

### 3.2 Widget sayısına göre (son sürüm)

| Senaryo | Süreç | Özel bellek | Çalışma kümesi | CPU |
|---|---|---|---|---|
| Widget'lar gizli, Kontrol Merkezi kapalı | 1 | **2,6 MB** | 15 MB | %0 |
| 1 widget (Saat) | 7 | 169,4 MB | 212,9 MB | %0 |
| 9 widget | 11 | 374,6 MB | 398,7 MB | %0 (30 sn ortalaması yuvarlanmış) |
| 9 widget + Kontrol Merkezi açık (Genel Bakış) | 12 | 454,2 MB | 780,5 MB | %0,267 |

Çıkarımlar:

- **WebView2 taban maliyeti** (tarayıcı + GPU + yardımcı süreçler + ilk renderer): ≈ 167 MB özel bellek. Bu, Tauri/WebView2 mimarisinin sabit bedelidir ve ilk widget açıldığında bir kez ödenir.
- **Widget başına ek maliyet:** (374,6 − 169,4) / 8 ≈ **26 MB** özel bellek.
- **Kontrol Merkezi:** açıkken ≈ +80 MB. Tepsiye kapatıldığında pencere tamamen yok edildiği için bu bellek geri verilir.
- Widget'lar gizliyken uygulama yalnızca Rust sürecinden ibarettir (2,6 MB).

### 3.3 Başlatma süresi (9 widget + Kontrol Merkezi)

| Deneme | Süre |
|---|---|
| 1 | 1478 ms |
| 2 | 1392 ms |
| 3 | 1736 ms |

### 3.4 Widget oluşturma/yok etme

Geliştirme (debug) derlemesinde Kontrol Merkezi üzerinden ölçüldü: ekleme ≈ 116 ms, gizleme ≈ 28 ms, gösterme ≈ 92 ms, kaldırma ≈ 27 ms. Gizlenen/kaldırılan widget'ın penceresi ve WebView'ı tamamen kapatılır.

## 4. Bulunan Darboğazlar ve Çözümleri

### 4.1 SVG'de sonsuz CSS animasyonları (çözüldü)

Temel ölçümde, hiçbir şey olmazken GPU süreci 10 saniyede ≈ 4,5 sn (bir çekirdeğin %45'i), bazı renderer'lar ≈ 1–1,6 sn CPU kullanıyordu. Neden: piksel kedilerin göz kırpma, Zz ve soru işareti animasyonları SVG `<g>` öğelerinde **sonsuz** CSS animasyonlarıydı. Bu animasyonlar bileşikleştiricide çalışamadığı için Chromium onları `steps()` olsa bile her karede ana iş parçacığında işliyor ve sürekli yeniden birleştiriyordu.

Çözüm (`src/ui/cats/useCatMotion.js`): değişiklikler seyrek zamanlayıcılarla yapılır — göz kırpma 4–9 sn'de bir 150 ms, Zz/soru işareti 1,5 sn'de bir kare değişimi. Hareket azaltıldığında ya da sayfa görünmezken zamanlayıcı yoktur. CSS animasyonu yalnızca kullanıcı etkileşimi sırasında (üzerine gelince kuyruk sallama, tek seferlik kutlama) kalır.

### 4.2 Pencere başına bir renderer süreci (iyileştirildi)

Varsayılan olarak her WebView ayrı bir renderer süreci açar. Tüm sayfalar aynı kökenden (`tauri.localhost`) geldiği için `--renderer-process-limit=2` ile renderer'lar paylaşılır. Sınır 1 değil 2 seçildi: tek bir renderer takılırsa tüm widget'ların donmasını önlemek için.

### 4.3 Çalışma kümesi (iyileştirildi)

Widget WebView'larına `MemoryUsageTargetLevel = Low` verildi (`ICoreWebView2_19`). Özel bellek değişmedi ama çalışma kümesi yarıya indi: işletim sistemi bu sayfaları gerektiğinde geri alabilir.

## 5. Tasarım Gereği Düşük Kaynak Kullanımı

| Alan | Davranış |
|---|---|
| Saat | Güncelleme bir sonraki saniye/dakika sınırına hizalı; saniyeler kapalıysa dakikada bir. |
| Odak zamanlayıcısı | Kalan süre bitiş anından hesaplanır; saniyelik çizim yalnızca çalışırken. Aşama bitişini Rust zamanlayıcısı yakalar. |
| Hatırlatıcılar | Tek iş parçacığı bir sonraki tetiklenme anına kadar uyur (en fazla 5 dk). Yoklama yok. |
| Sistem izleme | Yalnızca Sistem Kedisi ya da ilgili Kontrol Merkezi sayfası açıkken; ayarlanan aralıkla (varsayılan 3 sn). |
| Medya | GSMTC olaylarıyla; yalnızca Müzik Kedisi açıkken. |
| Hava durumu | Önbellek tazeyse ağa çıkılmaz; yenileme aralığı 15–180 dk. |
| Fare izleme | Raw Input yalnızca fare hareket ettiğinde mesaj üretir; boşta iş parçacığı uyur. |
| Not otomatik kaydı | 600 ms gecikmeli; içerik değişmediyse diske yazılmaz. |
| Widget konumları | Sürükleme bitince 600 ms sonra tek işlemle yazılır. |
| Günlük | Üretimde yalnızca uyarı ve hatalar. |

## 6. Performans Bütçeleri

Yukarıdaki ölçümlere dayanarak belirlenen bütçeler. Bir değişiklik bunları aşarsa birleştirmeden önce incelenmelidir.

| Metrik | Bütçe | Mevcut |
|---|---|---|
| Boşta CPU (9 widget, KM kapalı) | ≤ %0,2 (tüm işlemci) | %0,03–0,1 |
| Özel bellek (9 widget, KM kapalı) | ≤ 420 MB | ≈ 375 MB |
| Widget başına ek özel bellek | ≤ 35 MB | ≈ 26 MB |
| Widget'lar gizliyken bellek | ≤ 10 MB | 2,6 MB |
| Başlatma (9 widget) | ≤ 2,5 sn | 1,4–1,7 sn |

## 7. Değerlendirilip Ertelenenler

- **Masaüstü örtülünce widget'ları askıya alma.** Değerlendirildi, uygulanmadı: boşta widget'lar zaten ≈ %0 CPU kullanıyor. Büyütme/küçültme değişikliklerini güvenilir yakalamak sistem genelinde sık tetiklenen konum olaylarına abone olmayı ya da yoklamayı gerektiriyor. Bu da kazanacağından fazla maliyet getiriyor. Önceki sürümdeki işlevsiz ayar kaldırıldı.
- **Tüm widget'ları tek bir pencerede çizmek.** Bellek açısından daha ucuz olurdu, ancak bağımsız widget pencereleri gereksinimiyle çelişir ve her widget'ın ayrı yaşam döngüsünü ortadan kaldırır.

## 8. Bilinen Darboğazlar

- WebView2'nin ≈ 167 MB'lık taban maliyeti ilk widget açıldığında kaçınılmazdır.
- Ölçümler tek bir makinede alındı; daha az çekirdekli sistemlerde CPU yüzdeleri orantılı olarak daha yüksek görünür (aynı mutlak CPU süresi).
- GPU kullanımı ayrıca ölçülmedi; GPU sürecinin CPU süresi (≈ 0) ve görünür animasyon olmaması dolaylı göstergedir.
