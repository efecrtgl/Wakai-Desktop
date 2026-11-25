# Otomatik güncellemeler

Ivily Desktop (1.1.0'a kadar adı Wakai Desktop) yeni sürümleri kendisi bulur. Güncellemeler resmi **Tauri Updater** eklentisiyle, **GitHub Releases** üzerinden dağıtılır ve **dijital olarak imzalanır**.

## Kullanıcı için

- Açılıştan yaklaşık 20 saniye sonra, en fazla günde bir kez, arka planda denetlenir. Açılışı ve widget'ları geciktirmez; internet yoksa sessizce vazgeçer.
- Yeni sürüm bulunursa Kontrol Merkezi'nde **"Yeni bir sürüm hazır!"** penceresi açılır (bildirimler açıksa bir Windows bildirimi de gelir). Pencerede sürüm notları, **Güncellemeyi İndir** ve **Daha Sonra** düğmeleri vardır.
- Güncelleme **asla onayınız olmadan kurulmaz**. Sessiz ya da zorunlu kurulum yoktur. "Daha Sonra" denen sürüm o oturumda yeniden sorulmaz.
- İndirme sırasında ilerleme gösterilir. İndirilen dosyanın imzası uygulamaya gömülü genel anahtarla doğrulanır; imza tutmazsa hiçbir şey değiştirilmez.
- İmza doğrulandıktan sonra, kurulumdan hemen önce:
  1. bekleyen widget konumları ve kedi durumu kaydedilir,
  2. veritabanı yedeklenir (`yedekler/wakai-…-guncelleme-oncesi.db`),
  3. çalışan program geri dönüş için saklanır (`previous/<sürüm>/`, en fazla 2 sürüm).
- Kurulum yükleyicinin "pasif" kipinde yapılır: küçük bir ilerleme penceresi görünür, soru sorulmaz, uygulama kapanır ve kurulum bitince yeniden açılır.
- **Ayarlar → Güncellemeler:** otomatik denetimi kapatma, elle "Güncellemeleri denetle", şu anki sürüm, son denetim zamanı ve **güncelleme kanalı**.

### Kararlı ve ön sürüm kanalı (1.1.5)

| Kanal | Denetlenen adres | Ne gelir? |
|---|---|---|
| **Kararlı** (varsayılan) | `releases/latest/download/latest.json` | Yalnızca kararlı sürümler. GitHub'ın "latest" bağlantısı ön sürümleri ve taslakları hiç göstermez; kararlı kanaldaki kullanıcı ön sürüme zorlanamaz. |
| **Ön sürüm** | `releases/download/updater-prerelease/latest.json` | Yayımlanan en yeni sürüm (ön sürüm ya da kararlı). Güncelleme penceresinde "Ön sürüm kanalından" notu görünür. |

1.1.0 kullanıcılarının uygulamasında kanal seçimi yoktur; kararlı kanalı kullanırlar. Bu nedenle
**1.1.5 ön sürüm olarak yayımlanırsa 1.1.0 kurulumlarına önerilmez** (doğru davranış); denemek
isteyen 1.1.0 kullanıcısı 1.1.5 yükleyicisini bir kez elle kurar. Not: GitHub'da yalnızca ön
sürümler varken kararlı kanaldaki elle denetim "Güncelleme denetlenemedi" der (yayımlanmış
kararlı sürüm yok).

## 1.0.0'dan geçiş (tek seferlik, elle)

Wakai Desktop **1.0.0 güncelleyici içermez**; bu nedenle 1.1.0'a otomatik geçemez. 1.0.0 kullanıcıları 1.1.0 yükleyicisini GitHub Releases sayfasından bir kez elle indirip çalıştırmalıdır. Yükleyici mevcut kurulumun üzerine kurar; veriler (`%APPDATA%\com.wakai.desktop`) korunur ve veritabanı göçleri ilk açılışta otomatik ve geriye uyumlu biçimde uygulanır. 1.1.0 ve sonrası birbirini otomatik günceller.

## Depo sahibi için: imzalama ve sürüm yayınlama

### Anahtarlar

- **Genel anahtar** `src-tauri/tauri.conf.json` → `plugins.updater.pubkey` içindedir (gizli değildir).
- **Özel anahtar depoda değildir ve asla depoya eklenmemelidir.** Geliştirme bilgisayarında `C:\Users\<kullanıcı>\.tauri\wakai-desktop-updater.key` konumunda üretildi (parolasız). Bu dosyanın güvenli bir yedeğini alın: kaybolursa mevcut kurulumlar yeni anahtarla imzalanmış güncellemeleri kabul etmez (kullanıcılar bir kez elle kurmak zorunda kalır).
- Anahtarı yenilemek gerekirse: `npx tauri signer generate -w <yol>` → yeni genel anahtarı `tauri.conf.json`'a yazın → yeni özel anahtarı GitHub gizlisine koyun. Eski sürümler yeni imzayı kabul etmeyeceği için bu sürüm bir kez elle dağıtılmalıdır.

### Gerekli GitHub gizlileri (tek seferlik)

GitHub → depo → **Settings → Secrets and variables → Actions → New repository secret**:

| Ad | Değer |
| --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY` | `wakai-desktop-updater.key` dosyasının **tüm içeriği** |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Anahtar parolası (bu anahtar parolasız; boş değerle ekleyin ya da hiç eklemeyin) |

Ya da GitHub CLI ile (anahtar içeriği ekrana yazılmaz):

```powershell
gh secret set TAURI_SIGNING_PRIVATE_KEY < "$env:USERPROFILE\.tauri\wakai-desktop-updater.key"
```

### Sürüm yayınlama

1. `package.json`, `src-tauri/Cargo.toml` ve `src-tauri/tauri.conf.json` içindeki sürümü yükseltin, `CHANGELOG.md`'yi güncelleyin.
2. Commit atın ve etiketleyin: `git tag v1.2.0 && git push origin v1.2.0` (ön sürüm için etikete `-pre` ekleyin ya da iş akışını elle "prerelease" seçerek çalıştırın).
3. `.github/workflows/release.yml` testleri çalıştırır, NSIS yükleyicisini derler, imzalar ve **taslak** bir GitHub Release'e şunları yükler:
   - `Ivily.Desktop_<sürüm>_x64-setup.exe` ve `.sig` imza dosyası,
   - `latest.json` (güncelleyici bildirimi).
4. Taslak hiçbir kullanıcıya ulaşmaz. GitHub'da sürüm notlarını kontrol edip **Publish release** dediğinizde yayımlanır. Yayımlanınca `.github/workflows/prerelease-channel.yml` `latest.json`'u ön sürüm kanalına kopyalar.
5. Kararlı kanal `releases/latest/download/latest.json` adresini denetler; bu adres yalnızca yayımlanmış, ön sürüm olmayan en son sürümü gösterir.

> **Önemli:** Depo şu an herkese açık değil (oturum açmadan `github.com/efecrtgl/Wakai-Desktop` 404 döner). Kurulu uygulamalar oturum açmadığı için güncelleme bildirimine ve dosyalara erişemez. Otomatik güncellemenin çalışması için ya depo herkese açık yapılmalı ya da yalnızca sürümlerin yayımlandığı herkese açık bir depo kullanılmalı (bu durumda `src-tauri/src/updater.rs` içindeki iki adres değiştirilir).

Gizli tanımlı değilse iş akışı "TAURI_SIGNING_PRIVATE_KEY secret is missing" hatasıyla durur; imzasız sürüm yayınlanmaz.

### Elle (yerel) imzalı derleme

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content -Raw "$env:USERPROFILE\.tauri\wakai-desktop-updater.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
npm run tauri build -- --bundles nsis
```

Çıktı: `src-tauri/target/release/bundle/nsis/` altında `.exe` ve `.exe.sig`. `latest.json` elle hazırlanırsa biçim:

```json
{
  "version": "1.2.0",
  "notes": "Sürüm notları",
  "pub_date": "2026-12-01T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "<.sig dosyasının içeriği>",
      "url": "https://github.com/efecrtgl/Wakai-Desktop/releases/download/v1.2.0/Wakai-Desktop_1.2.0_x64-setup.exe"
    }
  }
}
```

## Güvenlik

- İmzasız ya da imzası geçersiz güncelleme reddedilir (yerel testte değiştirilmiş dosya "The signature verification failed" ile reddedildi).
- Sürüm derlemelerinde HTTPS dışı güncelleme adresleri eklenti tarafından reddedilir.
- `WAKAI_UPDATE_ENDPOINT` ortam değişkeni yalnızca test içindir; imza doğrulamasını atlatmaz.
- Ön yüze eklentinin JS izinleri verilmez; tüm güncelleme işlemleri Rust komutlarıyla (`check_for_updates`, `update_status`, `install_update`) yapılır.

## Test

Birincil kuruluma dokunmadan, hata ayıklama derlemesi + yerel HTTP sunucusuyla:

| Durum | Sonuç |
| --- | --- |
| Geçerli imzalı 9.9.9 bildirimi | Güncelleme bulundu, pencere açıldı, indirme + imza doğrulaması başarılı (kurulmadı) |
| İçeriği değiştirilmiş dosya, aynı imza | "İmza doğrulanamadı: The signature verification failed" |
| Bildirimdeki sürüm daha eski | "Güncelleme yok" |
| Sunucuya ulaşılamıyor | "Güncelleme denetlenemedi. İnternet bağlantınızı kontrol edin." |

Gerçek bir kurulumun (1.1.0 → sonraki sürüm) uçtan uca denenmesi, imzalı ilk GitHub sürümü yayınlandıktan sonra ayrı bir test kurulumunda yapılmalıdır.

## Sürüm geçmişi ve önceki sürüme dönüş

- **Sürüm geçmişi:** her açılışta sürüm `app_versions` tablosuna yazılır (göç 006, yalnızca ekleme). Ayarlar → Sürüm geçmişi kartında bu bilgisayarda çalışmış sürümler, ilk görülme tarihi, açılış sayısı ve şu anki sürümün uygulamaya gömülü yenilikleri (`src/lib/releaseNotes.js`) gösterilir.
- **Saklanan sürümler:** güncelleme kurulmadan hemen önce çalışan program `%APPDATA%\com.wakai.desktop\previous\<sürüm>\wakai-desktop.exe` olarak saklanır (en fazla 2).
- **Elle geri dönüş:** Ayarlar → Sürüm geçmişi → "<sürüm> sürümüne dön". Onaydan sonra veritabanı yedeklenir (`…-geri-donus-oncesi.db`), çalışan exe `.exe.old` olarak yeniden adlandırılır (Windows çalışan exe'nin üzerine yazmaya izin vermez ama yeniden adlandırmaya izin verir), saklanan sürüm yerine kopyalanır ve uygulama yeniden başlar. Kopyalama başarısız olursa eski exe geri konur. `.old` bir sonraki açılışta silinir.
- **Otomatik geri dönüş:** bir sürüm 15 sn sorunsuz çalışınca "sağlıklı" işaretlenir. Henüz hiç sağlıklı açılmamış bir sürüm art arda 3 kez başarısız olursa (ör. güncelleme sonrası açılışta çökme) saklanan önceki sürüme otomatik dönülür ve "Wakai Desktop önceki sürüme döndü" bildirimi gösterilir. Sağlıklı açılmış bir sürüm asla otomatik geri alınmaz; döngüye girmemek için geri alınan sürüm işaretlenir.
- **Veriler geri alınmaz:** tüm göçler yalnızca tablo/sütun ekler; eski sürümler yeni şemayı açabilir (bilinmeyen tabloları yok sayar). Gerekirse geri dönüş öncesi yedek Ayarlar → Yedekler'den geri yüklenebilir.
- **Sınırlar:** geri dönüş yalnızca programı değiştirir; Windows "Uygulamalar" listesindeki sürüm numarası yükleyicinin yazdığı değer olarak kalır. Bir sonraki güncelleme bunu düzeltir. Otomatik denetim açıksa geri dönülen yeni sürüm yeniden önerilir ("Daha Sonra" ya da denetimi kapatma).

### Geri dönüş testi (yalıtılmış ortam)

`WAKAI_DATA_DIR` ortam değişkeni uygulamanın veri klasörünü değiştirir (yalnızca test içindir). Hata ayıklama derlemesinin bir kopyası geçici bir klasörde, ayrı bir veri klasörüyle çalıştırıldı; birincil kurulum ve veriler kullanılmadı:

| Senaryo | Sonuç |
| --- | --- |
| Sağlık dosyası "2 başarısız açılış", saklı sürüm var → açılış | 3. başarısız açılış algılandı, exe saklı sürümle değişti (SHA-256 doğrulandı), uygulama aynı yoldan yeniden başladı, `.old` temizlendi, bildirim bir kez gösterildi, döngü yok |
| 15 sn sonra | Sürüm "sağlıklı" işaretlendi |
| Ayarlar'dan elle geri dönüş (0.8.0) | Yedek alındı, exe değişti (SHA-256), yeniden başladı |
| Geçersiz/yok sürüm (`../../x`, `7.7.7`) | "Bu sürümün yedeği bulunamadı."; yedek ya da dosya değişikliği yok |
