# Performans

Performans Wakai Desktop'ta son rötuş değil, mimari bir gerekliliktir. Bu belge ölçüm yöntemini, gerçek ölçüm sonuçlarını ve bilinen darboğazları içerir.

> Kural: Burada yer alan her sayı gerçekten ölçülmüştür. Ölçülmemiş değer yazılmaz.

## 1. Ölçüm Yöntemi

### 1.1 Ölçülecek metrikler

| Metrik | Tanım |
|---|---|
| Başlatma süresi | Süreç başlangıcından tüm widget pencerelerinin yerleşmesine kadar geçen süre (uygulama içi zaman damgası ile). |
| Boştaki CPU | Kullanıcı etkileşimi yokken 60 saniyelik ortalama CPU kullanımı. |
| Boştaki bellek | Tüm süreçlerin (ana süreç + WebView2 süreçleri) özel çalışma kümesi (Private Working Set) toplamı. |
| Widget başına bellek | N ve N+1 widget arasındaki toplam bellek farkı. |
| Widget başına CPU | N ve N+1 widget arasındaki boştaki CPU farkı. |
| Sistem izleme yükü | Sistem Kedisi açık/kapalı iken CPU farkı. |
| Pencere oluşturma/yok etme | Widget ekleme ve kaldırma süreleri; kaldırma sonrası belleğin geri dönüp dönmediği. |

### 1.2 Araç

`scripts/measure.ps1` betiği (Aşama 9'da eklenecek):

- `wakai-desktop.exe` sürecini ve **onun alt süreçleri olan** `msedgewebview2.exe` süreçlerini bulur.
- Belirtilen süre boyunca her saniye `Get-Process` ile CPU zamanı ve özel bellek okur.
- Ortalama, en yüksek ve son değerleri raporlar.

### 1.3 Test koşulları

Her ölçümde şunlar kaydedilir: Windows derlemesi, CPU, RAM, WebView2 sürümü, derleme türü (release), etkin widget listesi ve ayarlar.

## 2. Bütçeler

Bütçeler ilk temel ölçüm alındıktan sonra (Aşama 9) gerçekçi şekilde belirlenecektir.

## 3. Sonuçlar

Henüz ölçüm yapılmadı.

## 4. Bilinen Darboğazlar

- Her WebView2 penceresi ayrı bir renderer süreci kullanır; widget sayısı arttıkça bellek doğrusal artar.
