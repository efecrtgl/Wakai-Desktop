# Windows 11 Masaüstü Entegrasyonu

Bu belge Wakai Desktop'un widget'ları Windows 11 masaüstü katmanına nasıl yerleştirdiğini, kullanılan API'leri ve **bilinen sınırlamaları** açıklar.

> Durum: Aşama 0 — araştırma ve plan. Uygulama ve doğrulama sonuçları Aşama 2'de bu belgeye eklenecektir.

## 1. Masaüstü Pencere Hiyerarşisi

Windows masaüstü, Explorer sürecine ait birkaç pencereden oluşur:

| Pencere sınıfı | Görevi |
|---|---|
| `Progman` | “Program Manager”; masaüstünün kök penceresi. |
| `SHELLDLL_DefView` | Masaüstü simgelerini barındıran kabuk görünümü. |
| `SysListView32` | Simgelerin kendisi (DefView'in alt penceresi). |
| `WorkerW` | Duvar kağıdını çizen yardımcı pencere. |

### 1.1 Windows 11 24H2 ve sonrası (bu makinede doğrulandı)

Geliştirme makinesinde (Windows 11, derleme 26200) `Progman`'a `0x052C` mesajı gönderildikten sonra yapılan incelemenin sonucu:

```
Progman
 ├─ SHELLDLL_DefView   (z-sırasında üstte — simgeler)
 └─ WorkerW            (z-sırasında altta — duvar kağıdı)
```

Bu düzende widget pencereleri `Progman`'ın alt penceresi yapılıp `SHELLDLL_DefView`'in hemen altına yerleştirilir. Sonuç olarak widget duvar kağıdının üstünde, simgelerin altında görünür.

### 1.2 Eski düzen (24H2 öncesi)

`0x052C` mesajı, `SHELLDLL_DefView`'i barındıran üst düzey `WorkerW`'nin hemen arkasında ikinci bir üst düzey `WorkerW` oluşturur. Widget'lar bu ikinci `WorkerW`'ye bağlanır. Kod her iki düzeni de algılar.

## 2. Kullanılan API'ler

- `FindWindowW`, `FindWindowExW`, `EnumWindows`, `GetWindow` — hiyerarşi tespiti.
- `SendMessageTimeoutW(Progman, 0x052C)` — duvar kağıdı `WorkerW`'sinin oluşturulmasını ister (belgelenmemiş ama yaygın kullanılan davranış; yalnızca `desktop::hierarchy` içinde kullanılır).
- `SetParent`, `SetWindowLongPtrW` (`WS_CHILD`, `WS_EX_NOACTIVATE`, `WS_EX_TOOLWINDOW`) — pencereyi katmana bağlama.
- `SetWindowPos` — konum ve z-sırası.
- `RegisterWindowMessageW("TaskbarCreated")` — Explorer yeniden başladığında haber alma.
- `WM_DISPLAYCHANGE`, `WM_SETTINGCHANGE` — çözünürlük, monitör ve çalışma alanı değişiklikleri.
- `RegisterRawInputDevices` (`RIDEV_INPUTSINK`) — simge katmanının altında kalan widget'lara fare girdisi yönlendirme.

## 3. Etkileşim Stratejisi

Simge katmanı (`SysListView32`) tüm ekranı kapladığından widget'lar fare olaylarını doğrudan alamaz. Planlanan çözüm:

1. Gizli bir pencere Raw Input ile fare olaylarını dinler.
2. İmlecin altındaki pencere masaüstü simge katmanıysa ve imleç bir widget'ın sınırları içindeyse,
3. ve imlecin altında bir masaüstü simgesi yoksa,
4. olay widget'ın WebView girdi penceresine iletilir.

Klavye girdisi bu yolla iletilemez. Metin girmek gereken durumlarda widget geçici olarak etkileşim moduna alınır ve odak verilir.

## 4. Yapılmayacaklar

- `HWND_TOPMOST` kullanılmaz.
- Explorer sonlandırılmaz veya yeniden başlatılmaz.
- Explorer pencerelerinin stilleri değiştirilmez; yalnızca kendi pencerelerimiz taşınır.

## 5. Sınırlamalar

Aşama 2 doğrulamasından sonra bu bölüm gerçek bulgularla doldurulacaktır.
