# Veritabanı

Wakai Desktop tüm kalıcı verileri yerel bir **SQLite** veritabanında saklar. Hiçbir veri buluta gönderilmez.

## Kütüphane seçimi: `rusqlite`

`SQLx` ve `rusqlite` değerlendirildi. `rusqlite` seçildi çünkü:

- Uygulama tek bir yerel dosya ve tek bir bağlantı kullanır; SQLx'in asenkron bağlantı havuzu bu senaryoda ek fayda sağlamaz.
- Derleme zamanında veritabanı gerektirmez, bağımlılık ağacı daha küçüktür.
- `bundled` özelliği SQLite'ı uygulamanın içine gömer; kullanıcının sisteminde SQLite gerekmez.

## Bağlantı ayarları

| PRAGMA | Değer | Neden |
|---|---|---|
| `journal_mode` | `WAL` | Okuma ve yazmalar birbirini engellemez; çökme sonrası güvenli. |
| `synchronous` | `NORMAL` | WAL ile güvenli ve daha az disk yazması. |
| `foreign_keys` | `ON` | İlişkisel bütünlük. |
| `busy_timeout` | `3000` | Kısa kilit çakışmalarında hata yerine bekleme. |

## Göçler (migrations)

- Göçler `src-tauri/src/db/migrations/` altında sıralı SQL dosyalarıdır.
- Uygulanan son göç `PRAGMA user_version` ile izlenir.
- Her göç ayrı bir işlem (transaction) içinde çalışır; hata olursa geri alınır.
- **Mevcut göç dosyaları asla değiştirilmez**; şema değişikliği her zaman yeni bir dosya ile yapılır.

## Yedekleme

- Mevcut bir veritabanında bekleyen göç varsa, göçten **önce** `VACUUM INTO` ile tutarlı bir yedek alınır: `wakai.db.bak-v<eski sürüm>`.
- `Db::backup_to` aynı yöntemle istenen konuma yedek alır (Kontrol Merkezi'nden kullanılacak).

## Şema

### `settings`
| Sütun | Tür | Açıklama |
|---|---|---|
| `key` | TEXT PK | Ayar anahtarı (şu an yalnızca `app`). |
| `value` | TEXT | JSON değer. |
| `updated_at` | TEXT | Yerel zaman. |

### `widget_instances`
| Sütun | Tür | Açıklama |
|---|---|---|
| `id` | TEXT PK | Örnek kimliği. |
| `kind` | TEXT | Widget türü (kayıt defterindeki kimlik). |
| `x`, `y`, `width`, `height` | INTEGER | Fiziksel piksel cinsinden konum ve boyut. |
| `visible`, `click_through` | INTEGER (0/1) | Görünürlük ve tıklama geçirgenliği. |
| `config` | TEXT | Örneğe özel JSON yapılandırması. |

Widget yapılandırması widget **içeriğinden** (etkinlikler, görevler, notlar) ayrı tutulur; bir widget örneğini silmek içerik verisini silmez.

### 002 — İçerik tabloları

| Tablo | İçerik | Notlar |
|---|---|---|
|  | Etkinlik kategorileri (Kişisel, İş, Sağlık, Sosyal varsayılan) | Kategori silinirse etkinlik kalır (). |
|  | Takvim etkinlikleri |  dizinli; bekleyen hatırlatıcılar için kısmi dizin. Başlangıç ya da hatırlatıcı değişince  sıfırlanır. |
|  | Görevler | Öncelik 0–2, son tarih, kategori, hatırlatıcı;  dizini. |
|  | Notlar | İçerik değişmediyse otomatik kayıt diske yazmaz. |
|  | Odak oturumları geçmişi | Tamamlanan/yarıda kalan her oturum. |

İçerik tabloları  tablosuna bağlı değildir. Not Kedisi örneği yapılandırmasında yalnızca  tutar; widget silinince not silinmez.

### Zaman biçimi

Etkinlik ve görev zamanları yerel saatle  biçiminde saklanır (saat dilimi bilgisi yoktur). Takvim uygulaması kullanıcının yerel saatine göre çalışır.

### Geliştirme aracı

 uygulama veritabanında salt okunur sorgu çalıştırır (Node 24 ).
