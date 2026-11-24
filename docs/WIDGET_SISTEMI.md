# Widget Sistemi

Wakai Desktop'ta widget **tanımları** ile widget **örnekleri** birbirinden ayrıdır.

| Kavram | Nerede | Açıklama |
|---|---|---|
| Tanım (`WidgetDefinition`) | `src-tauri/src/widgets/registry.rs` | Tür kimliği, Türkçe ad, açıklama, kategori, varsayılan/en küçük boyut, varsayılan yapılandırma, çoklu örnek desteği, etkileşimler. Koda gömülüdür. |
| Örnek (`WidgetInstance`) | SQLite `widget_instances` | Kimlik, tür, konum, boyut, görünürlük, tıklama geçirgenliği, örneğe özel yapılandırma. |
| Ön yüz bileşeni | `src/widgets/registry.js` | Tür kimliği → tembel yüklenen React bileşeni. |

## Yaşam Döngüsü

```
Kayıt (registry) ─► Oluştur (add) ─► Başlat (pencere + WebView) ─► Göster (masaüstü katmanına bağla)
                                       ▲                               │
              Geri yükle (restore_all) ┘          Gizle (pencere kapatılır, bellek serbest kalır)
                                                                       │
                         Yapılandır (update_config) ── Sıfırla (reset) ─┴─► Yok et (remove)
```

| Adım | Rust fonksiyonu | Not |
|---|---|---|
| Kayıt | `registry::DEFINITIONS` | Statik. |
| Oluştur | `manager::add` | Varsayılan yapılandırma ve konumla veritabanına yazar. Varsayılan konum birincil monitörün sağ üstünden kademelidir, monitör ölçeğine göre büyütülür. |
| Başlat + Göster | `manager::spawn` (iç) | Gizli pencere oluşturulur, `layer::attach_window` ile masaüstü katmanına bağlanır. |
| Gizle | `manager::set_visible(false)` | Pencere **yok edilir**; WebView renderer süreci kapanır. Gizli widget sıfır kaynak kullanır. |
| Yapılandır | `manager::update_config` | Yama, varsayılan yapılandırmaya göre doğrulanır (bilinmeyen anahtar veya yanlış tür reddedilir) ve işlem içinde yazılır. |
| Sıfırla | `manager::reset` | Yapılandırma ve konum varsayılana döner; tıklama geçirgenliği kapatılır. İçerik verisi (etkinlik, görev, not) silinmez. |
| Yok et | `manager::remove` | Pencere kapatılır, örnek silinir. İçerik verisi silinmez. |
| Geri yükle | `manager::restore_all` | Açılışta, "widget'ları göster" ayarı açıldığında ve kaybolan pencereler için çalışır. |

### Kaybolan pencereler

Explorer çöktüğünde ebeveyni olan `WorkerW` ile birlikte widget pencereleri de yok edilir. Yönetici, kendisinin kapatmadığı bir pencerenin yok edildiğini fark ederse 2 saniye sonra (ve ayrıca `TaskbarCreated` mesajı geldiğinde) eksik pencereleri yeniden oluşturur.

## Kalıcılık

- Kullanıcı widget'ı sürükleyip boyutlandırdığında konum her harekette değil, hareket bittikten **600 ms** sonra tek bir işlemle yazılır.
- Uygulama kapanırken bekleyen konumlar hemen yazılır (`RunEvent::Exit`).
- Konum ve boyut fiziksel piksel, ekran koordinatıdır.

## Ön Yüz

- Her widget penceresi `widget.html?id=<örnek>` adresini yükler.
- `WidgetHost` örneği `get_widget` ile alır, `widget:updated` olayıyla güncel tutar ve yalnızca ilgili türün kodunu yükler (kod bölme).
- `WidgetShell` tüm widget'lara ortak çerçeveyi sağlar:
  - Üstte ince sürükleme şeridi (`startDragging`), sağ altta boyutlandırma tutamacı (`startResizeDragging`).
  - Sağ tık menüsü: Widget ayarları, Tıklamaları geçir, Gizle, Kaldır (onaylı).
- Görünüm ayarları (opaklık, köşe yuvarlaklığı, gölge, hareket) CSS değişkenleriyle uygulanır.

## Komutlar

| Komut | Açıklama |
|---|---|
| `list_widget_definitions` | Tüm tanımlar. |
| `list_widgets` | Tüm örnekler. |
| `get_widget(id)` | Örnek + tanım. |
| `add_widget(kind)` | Yeni örnek. |
| `remove_widget(id)` | Örneği kaldır. |
| `set_widget_visible(id, visible)` | Göster/gizle. |
| `set_widget_click_through(id, value)` | Tıklama geçirgenliği. |
| `update_widget_config(id, patch)` | Yapılandırma yaması. |
| `set_widget_rect(id, rect)` | Konum/boyut (en küçük boyut ve görünür alan korunur). |
| `reset_widget_position(id)` / `reset_widget(id)` | Sıfırlama. |
| `open_control_center(section?, widgetId?)` | Kontrol Merkezi'ni aç ve yönlendir. |

Pencere oluşturan veya taşıyan komutlar `async` tanımlıdır ve işi `state::on_main` ile ana iş parçacığına devreder. Windows'ta senkron komut içinde WebView oluşturmak kilitlenmeye yol açar (geliştirme sırasında gözlendi).

## Yerleşik Widget'lar

| Tür | Ad | Çoklu örnek | Öne çıkanlar |
|---|---|---|---|
| `calendar` | Takvim Kedisi | Evet | Pazartesiyle başlayan Türkçe ay görünümü, bugün vurgusu, hafta sonu rengi, etkinlik noktaları, hızlı ekleme/düzenleme, yaklaşanlar. |
| `tasks` | Görev Kedisi | Evet | Hızlı ekleme, öncelik/son tarih/kategori/hatırlatıcı, gecikme vurgusu, ilerleme çubuğu, tamamlama kutlaması, kategori filtresi. |
| `note` | Not Kedisi | Evet | Her örnek bir nota bağlı; 600 ms gecikmeli otomatik kayıt; sabitleme; renk; basit biçimlendirme. |
| `clock` | Saat Kedisi | Evet | Dijital/analog, saniye ayarı, 12/24 saat, tarih, ek saat dilimleri. Güncelleme saniye/dakika sınırına hizalı. |
| `focus` | Odak Kedisi | Hayır | Rust'taki zamanlayıcının bitiş anına göre kalan süre; yalnızca çalışırken saniyede bir çizim. |
| `weather` | Hava Durumu Kedisi | Evet | Konum arama (Türkçe sonuçlar), anlık durum, 5 günlük tahmin, önbellek ve çevrim dışı gösterim. Sağlayıcı arayüzü: `WeatherProvider`. |
| `music` | Müzik Kedisi | Hayır | Windows medya oturumu (GSMTC): parça, sanatçı, kapak, oynat/duraklat, önceki/sonraki. |
| `system` | Sistem Kedisi | Hayır | CPU, bellek, GPU (3D), disk, açık kalma süresi. |

### Abonelik modeli

Sistem izleme ve medya oturumu gibi arka plan hizmetleri yalnızca onları gösteren bir pencere varken çalışır. Pencere `subscribe('system' | 'media')` çağırır; pencere kapanınca abonelik otomatik silinir. Son abone gidince hizmet durur (iş parçacığı uyur, WinRT olay kayıtları kaldırılır).

## Yeni Bir Widget Eklemek

1. `registry.rs` içindeki `DEFINITIONS` dizisine bir tanım ekleyin. `default_config` içindeki her anahtar, kabul edilen yapılandırmayı belirler.
2. `src/widgets/<tür>/` altında bileşeni oluşturun. Bileşen `{ id, config, settings }` alır.
3. `src/widgets/registry.js` içine `tür: lazy(() => import(...))` satırını ekleyin.
4. Kullanıcıya görünen metinleri `src/lib/i18n.js` içine Türkçe ekleyin.

## Doğrulama Durumu

| Davranış | Durum |
|---|---|
| Ekle / gizle / göster / kaldır | Çalışma zamanında doğrulandı (geliştirme derlemesi: ekleme ≈116 ms, gizleme ≈28 ms, gösterme ≈92 ms, kaldırma ≈27 ms; resmi ölçümler Aşama 9'da). |
| Yeniden başlatmada konumun geri yüklenmesi | Doğrulandı (sol monitörde, negatif koordinatla). |
| Fareyle sürükleme/boyutlandırma ve gecikmeli kayıt | Kod yolu mevcut; kullanıcının faresi kullanılmadan otomatik test edilemedi. |
| Explorer çökmesinden sonra yeniden oluşturma | Kod yolu mevcut; çalışma zamanında test edilmedi. |
