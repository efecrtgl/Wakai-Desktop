# Windows 11 Masaüstü Entegrasyonu

Bu belge Wakai Desktop'un widget'ları Windows 11 masaüstü katmanına nasıl yerleştirdiğini, kullanılan API'leri, **gerçekten doğrulanan** davranışları ve bilinen sınırlamaları açıklar.

Kod: `src-tauri/src/desktop/` (tüm `unsafe` Win32 kodu yalnızca bu klasördedir) ve `src-tauri/src/layer.rs` (Tauri pencereleriyle bağlantı).

## 1. Masaüstü Pencere Hiyerarşisi

| Pencere sınıfı | Görevi |
|---|---|
| `Progman` | Masaüstünün kök penceresi. |
| `SHELLDLL_DefView` | Masaüstü simgelerini barındıran kabuk görünümü. |
| `SysListView32` | Simgelerin kendisi. |
| `WorkerW` | Duvar kağıdını çizen yardımcı pencere. |

### 1.1 Windows 11 24H2 ve sonrası (geliştirme makinesinde doğrulandı)

Derleme 26200 üzerinde `Progman`'a `0x052C` gönderildikten sonraki gerçek hiyerarşi:

```
Progman                    (üst düzey, sanal ekranın tamamını kaplar)
 ├─ SHELLDLL_DefView       (simgeler, z-sırasında üstte)
 └─ WorkerW                (duvar kağıdı, altta)
      └─ Wakai widget'ları ← buraya bağlanır
```

**Önemli bulgu:** Widget'ı doğrudan `Progman`'ın alt penceresi yapıp z-sırasında `SHELLDLL_DefView`'in altına koymak yeterli değildir. WebView2 içeriği ayrı bir kompozisyon yüzeyinde çizildiği için, z-sırası doğru olsa bile widget simgelerin **üstünde** görünüyordu (ekran görüntüsüyle doğrulandı). Widget, `Progman` altındaki duvar kağıdı `WorkerW`'sinin alt penceresi yapıldığında simgeler doğru şekilde widget'ın üstünde çizilir.

### 1.2 Eski düzen (24H2 öncesi)

`0x052C`, `SHELLDLL_DefView`'i barındıran üst düzey `WorkerW`'nin arkasında ikinci bir üst düzey `WorkerW` oluşturur; widget'lar bu pencereye bağlanır. Kod bu düzeni de algılar ancak **bu düzen geliştirme ortamında çalışma zamanında test edilmemiştir** (makine 24H2+).

## 2. Etkileşim Modeli: "Üzerine Gelince Kaldır"

Simge katmanı (`SysListView32`) tüm ekranı kapladığından, simgelerin altındaki bir pencere fare olaylarını doğrudan alamaz. Wakai Desktop bunu şöyle çözer:

1. **Dinlenme durumu (Sunk):** Widget duvar kağıdı katmanındadır; simgeler üstündedir, masaüstü tıklamaları Explorer'a gider.
2. **İmleç widget'ın üzerine gelir:** Gizli bir izleyici pencere Raw Input (`RIDEV_INPUTSINK`) ile fare hareketlerini dinler. İmleç bir widget'ın sınırları içindeyse ve `WindowFromPoint` imlecin altında gerçekten masaüstü olduğunu söylüyorsa (üstte bir uygulama penceresi yoksa) widget **kaldırılır**.
3. **Kaldırılmış durum (Lifted):** Widget, `Progman`'ın z-sırasında hemen üstüne, **tüm uygulama pencerelerinin altına** üst düzey pencere olarak yerleştirilir (etkinleştirilmeden). Artık fare, tekerlek ve klavye girdisini doğal yoldan alır; sürükleme ve boyutlandırma normal pencere işlemleriyle yapılır.
4. **Geri dönüş:** İmleç widget'tan ayrıldığında ve widget odakta değilse (ör. not yazılmıyorsa) widget katmana geri döner. Odaktayken imleç ayrılırsa, odak kaybedilene kadar beklenir.

Bu modelde olaylar yutulmaz veya taklit edilmez; Explorer'a hiçbir girdi enjekte edilmez.

**Tıklama geçirgenliği:** Etkileşimsiz (click-through) işaretlenen widget hiç kaldırılmaz; masaüstü tıklamaları altından geçer. Bu ayar Kontrol Merkezi'nden her zaman geri alınabilir, bu yüzden widget kalıcı olarak erişilemez hale gelmez.

## 3. DPI ve Çoklu Monitör

Bir alt pencerenin DPI değeri, üst düzey atası olan `Progman`'ın DPI değeridir (birincil monitör). Geliştirme makinesinde sol monitör %125, birincil monitör %100 ölçekliydi ve iki sorun gözlendi:

| Sorun | Çözüm |
|---|---|
| Katmandaki widget'ın içeriği %125 monitörde %100 ölçekle (küçük) çiziliyordu; kaldırılınca büyüyordu. | WebView2'nin otomatik ölçek algılaması kapatılır (`ShouldDetectMonitorScaleChanges = false`) ve `RasterizationScale`, widget'ın bulunduğu monitörün etkin DPI'ına göre elle ayarlanır. Doğrulandı. |
| Üst düzey ↔ alt pencere geçişlerinde `WM_DPICHANGED`, pencere çerçevesinin (tao) boyutu önerilen dikdörtgene göre değiştirmesine yol açabilir. | Widget pencereleri bir alt sınıfla (`SetWindowSubclass`) `WM_DPICHANGED` mesajını yutar; geometri yalnızca `desktop` modülü tarafından yönetilir. |

Konumlar ve boyutlar her zaman **fiziksel piksel** ve **ekran koordinatı** olarak saklanır; alt pencere yerleşiminde `ScreenToClient` ile ebeveyn koordinatına çevrilir. Bu sayede sanal ekranın negatif koordinatlı bölgelerindeki monitörler (birincil monitörün solundaki) doğru çalışır — geliştirme makinesinde sol monitörde doğrulandı.

## 4. Kurtarma

| Olay | Davranış | Doğrulama |
|---|---|---|
| Explorer yeniden başlar (`TaskbarCreated`) | Hiyerarşi yeniden bulunur; yaşayan pencereler yeniden bağlanır; ebeveyniyle birlikte yok edilen pencereler widget yöneticisine bildirilir ve yeniden oluşturulur. | Kod yolu mevcut. Kullanıcının çalışan oturumunda Explorer yeniden başlatılmadığı için **çalışma zamanında henüz test edilmedi**. |
| Çözünürlük / monitör değişikliği (`WM_DISPLAYCHANGE`, `SPI_SETWORKAREA`) | Her widget en yakın monitörün çalışma alanına sığdırılır; bağlantısı kesilen monitördeki widget görünür bir monitöre taşınır; ölçek yeniden hesaplanır. | Sığdırma mantığı birim testleriyle doğrulandı; gerçek monitör değişikliği **henüz test edilmedi**. |
| Oturum kapanışı (`WM_ENDSESSION`) | SQLite WAL kontrol noktası yazılır. | Test edilmedi. |
| Masaüstü penceresi kaybolursa | Her işlemden önce `IsWindow` ile kontrol edilir, gerekirse hiyerarşi yeniden aranır. Masaüstü bulunamazsa widget normal pencere olarak gösterilir (`Detached`). | Kod yolu mevcut. |

## 5. Uygulama Yaşam Döngüsü

| Davranış | Uygulama | Doğrulama |
|---|---|---|
| Sistem tepsisi | Sol tık: Kontrol Merkezi. Menü: Kontrol Merkezi'ni aç, Widget'ları göster (onay kutusu, ayarla eşitlenir), Tüm widget'ları etkileşimli yap (tıklama geçirgenliğini toplu kapatan güvenlik çıkışı), Çıkış. | Simge oluşturuluyor; menü öğeleri elle tıklanarak test edilmedi. |
| Tepsiye kapatma | "Kapatınca tepside çalış" açıkken Kontrol Merkezi penceresi gizlenmez, **yok edilir** (WebView belleği serbest kalır); uygulama widget'larla çalışmaya devam eder. Kapalıyken pencereyi kapatmak uygulamadan çıkar. | Her iki durum da doğrulandı. |
| Son pencere kapanınca | Widget'lar gizli ve Kontrol Merkezi kapalı olsa bile uygulama tepside çalışmayı sürdürür (`ExitRequested` engellenir). Yalnızca açık çıkış uygulamayı kapatır. | Doğrulandı. |
| Tek örnek | İkinci başlatma yeni süreç açmaz; çalışan örnekte Kontrol Merkezi'ni açar (gerekirse yeniden oluşturur). | Doğrulandı. |
| Windows ile başlatma | `HKCU\...\Run` kaydı (`--autostart` bağımsız değişkeniyle). Otomatik başlatmada yalnızca widget'lar gelir. | Kayıt ayarla eşitleniyor; gerçek oturum açılışı Aşama 11'de kurulu sürümle doğrulanacak. |
| Düzenli kapanış | Bekleyen widget konumları yazılır, WAL veritabanına aktarılır. | Kod yolu mevcut. |
| Beklenmeyen hata | Panikler `wakai.log` dosyasına yazılır. | — |
| Monitör çıkarılıp geri takılması | Ekran değişince widget'lar **kayıtlı** konumlarından görünür alana sığdırılır ama sığdırılmış konum kaydedilmez; monitör geri gelince widget eski yerine döner. | Mantık birim testleriyle; gerçek monitör değişikliği test edilmedi. |

## 6. Widget'ların Kullandığı Windows Hizmetleri

| Widget | API | Davranış |
|---|---|---|
| Müzik Kedisi | WinRT `Windows.Media.Control` (GSMTC): `GlobalSystemMediaTransportControlsSessionManager` | Olay güdümlü: `CurrentSessionChanged`, `MediaPropertiesChanged`, `PlaybackInfoChanged`. Yalnızca bir Müzik Kedisi açıkken etkin. Oynat/duraklat, önceki/sonraki gerçek oturuma gönderilir; uygulama içinde oynatıcı yoktur. Geliştirme makinesinde Spotify oturumu (başlık, sanatçı, kapak) doğrulandı. Komutlar kullanıcının müziğini kesmemek için otomatik test edilmedi. |
| Sistem Kedisi | `GetSystemTimes`, `GlobalMemoryStatusEx`, `GetDiskFreeSpaceExW`, PDH `\GPU Engine(*engtype_3D)\Utilization Percentage` | Yalnızca abone varken, ayarlanan aralıkla (varsayılan 3 sn; düşük kaynak modunda 2 katı). GPU sayacı okunamazsa "Ölçülemiyor" gösterilir. Değerler geliştirme makinesinde gerçek okumalarla doğrulandı. |
| Hava Durumu Kedisi | WinHTTP (`WinHttpOpen` … `WinHttpReadData`) | Windows sertifika deposu ve SChannel ile HTTPS. WinRT `HttpClient` geliştirme makinesinde Let's Encrypt sertifikalı Open-Meteo sunucularında `0x80072F8F` hatası verdiği (sertifika hatası listelemeden) için WinHTTP tercih edildi. |

## 7. Kullanılan API'ler

- Hiyerarşi: `FindWindowW`, `FindWindowExW`, `EnumWindows`, `SendMessageTimeoutW(0x052C)` (zaman aşımlı).
- Yerleşim: `SetParent`, `SetWindowLongPtrW` (yalnızca kendi pencerelerimizde), `SetWindowPos`, `ScreenToClient`, `GetWindow(GW_HWNDPREV)`.
- İzleme: `RegisterRawInputDevices`, `WindowFromPoint`, `GetCursorPos`, `RegisterWindowMessageW("TaskbarCreated")`.
- Monitörler: `EnumDisplayMonitors`, `MonitorFromRect`, `GetMonitorInfoW`, `GetDpiForMonitor`.
- WebView2: `ICoreWebView2Controller3::SetRasterizationScale`.

## 8. Yapılmayanlar

- `HWND_TOPMOST` kullanılmaz.
- Explorer sonlandırılmaz veya yeniden başlatılmaz.
- Explorer pencerelerinin stilleri değiştirilmez, onlara girdi enjekte edilmez.
- Fare kancası (`WH_MOUSE_LL`) kullanılmaz; sistemin girdi gecikmesine etki edilmez.

## 9. Bilinen Sınırlamalar

- **Simgelerle çakışma:** İmleç simgelerle çakışan bir widget'ın üzerine geldiğinde widget kaldırılır ve o sırada çakışan simgelerin üstünde görünür. Widget'ları simgelerin bulunmadığı alanlara yerleştirmek önerilir.
- **Süreçler arası ebeveynlik:** `SetParent` ile Explorer penceresine bağlanmak iki iş parçacığının girdi kuyruklarını ilişkilendirir. Bu yüzden widget pencerelerinin UI iş parçacığında engelleyici iş yapılmaz.
- `0x052C` mesajı belgelenmemiş bir Explorer davranışıdır; gelecekteki Windows sürümlerinde değişebilir. Bu durumda kod `Progman`'a bağlanmaya, o da olmazsa normal pencereye geri düşer.
- Ölçüm aracı notu: Sistem DPI'ına duyarlı (`SetProcessDPIAware`) araçlar, %125 monitördeki widget koordinatlarını sanallaştırılmış (0.8 katı) gösterir. Doğrulama araçları Per-Monitor-V2 bağlamında çalıştırılmalıdır.
- GPU kullanımı yalnızca 3D motorlarının toplamıdır (Görev Yöneticisi'nin "3D" grafiğine yakın); video kodlama/kopyalama motorları dahil değildir.
- Müzik Kedisi yalnızca Windows medya oturumu yayınlayan uygulamaları görür.
