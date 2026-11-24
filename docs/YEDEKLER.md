# Yedekler

Kontrol Merkezi → Ayarlar → Yedekler.

## Yedek alma

- **Şimdi yedekle** tüm verilerin tutarlı bir kopyasını (`VACUUM INTO`) `%APPDATA%\com.wakai.desktop\backups\` klasörüne yazar: widget'lar ve konumları, takvim, görevler, notlar, alışveriş listeleri, odak geçmişi, kediler ve Kedi Evi, profiller, ayarlar, sürüm geçmişi.
- **Atomik ve doğrulanmış:** yedek önce `….db.tmp` dosyasına yazılır, `PRAGMA quick_check` ile denetlenir, ancak sonra asıl adına taşınır. Yarım kalmış ya da bozuk bir yedek listede asla görünmez.
- Otomatik yedekler: her veritabanı göçünden önce (`wakai.db.bak-vN`), uygulama güncellemesinden önce (`-guncelleme-oncesi`), geri yüklemeden önce (`-geri-yukleme-oncesi`, `-secmeli-oncesi`) ve sürüm geri dönüşünden önce (`-geri-donus-oncesi`).
- En fazla 20 yedek tutulur; en eskisi silinir.

## Geri yükleme

"Geri yükle…" yedeği önce **inceler**:

- **Bütünlük:** `quick_check` + Wakai tablolarının varlığı. Bozuk ya da yabancı dosyalar geri yüklenemez.
- **Uyumluluk:** yedeğin şema sürümü gösterilir. Eski sürümden alınan yedek geri yüklendikten sonra otomatik güncellenir; daha yeni sürümden alınan yedekte bu sürümün tanımadığı veriler korunur ama kullanılmaz.
- **İçerik:** kategori başına kayıt sayısı.

Seçenekler:

- **Tümünü geri yükle:** uygulama yeniden başlar; veritabanı açılmadan önce yedek geçici bir dosyaya kopyalanır, yeniden doğrulanır ve tek adımda yerine konur. Önceki veriler ayrıca yedeklenir.
- **Yalnızca seçtiklerim:** seçilen kategoriler (ör. yalnızca görevler ya da yalnızca kediler) yedekteki hâline döner, diğer her şey olduğu gibi kalır. Uygulama yeniden başlar ve değişiklik, veriler kullanılmadan önce **tek bir veritabanı işlemi** içinde uygulanır: bir hata olursa hiçbir şey değişmez. Yalnızca iki tarafta da bulunan sütunlar kopyalanır; bu sayede eski ya da yeni şemalı yedekler de kullanılabilir. Bu yedekte bulunmayan kategoriler (ör. 1.0.0 yedeğinde kediler) seçilemez.

Not: kategoriler arası bağlantılar (ör. bir Alışveriş widget'ının gösterdiği liste) seçmeli geri yüklemeden sonra eşleşmeyebilir; widget bu durumda ilk listeye ya da boş duruma düşer.

## Test

- Rust: atomik oluşturma (geçici dosya kalmaz), tam geri yükleme gidiş-dönüşü, bozuk/yabancı dosya reddi (bekleyen geri yükleme mevcut veritabanını bozmaz), inceleme sayıları, seçmeli geri yüklemenin yalnızca seçilen kategoriyi değiştirmesi.
- Uygulamada (yalıtılmış veri klasörü, birincil veritabanının kopyası): yedek → görev ve not ekleme → yalnızca "Görevler" geri yükleme → yeniden başlatma sonrası görevler 3'e döndü, notlar 2'de kaldı, `-secmeli-oncesi` yedeği oluştu. Ortasına rastgele bayt yazılmış bir yedek "bozuk" olarak algılandı ve geri yüklenmesi reddedildi.
