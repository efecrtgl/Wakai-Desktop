# Otomatik güncellemeler

Wakai Desktop 1.1.0'dan itibaren yeni sürümleri kendisi bulur. Güncellemeler resmi **Tauri Updater** eklentisiyle, **GitHub Releases** üzerinden dağıtılır ve **dijital olarak imzalanır**.

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
- **Ayarlar → Güncellemeler:** otomatik denetimi kapatma, elle "Güncellemeleri denetle", şu anki sürüm ve son denetim zamanı.

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
2. Commit atın ve etiketleyin: `git tag v1.2.0 && git push origin v1.2.0`.
3. `.github/workflows/release.yml` testleri çalıştırır, NSIS yükleyicisini derler, imzalar ve GitHub Release'e şunları yükler:
   - `Wakai-Desktop_<sürüm>_x64-setup.exe` ve `.sig` imza dosyası,
   - `latest.json` (güncelleyici bildirimi).
4. Uygulamalar `https://github.com/efecrtgl/Wakai-Desktop/releases/latest/download/latest.json` adresini denetler. Bu yüzden yayın **taslak ya da ön sürüm olmamalıdır** ("latest" yalnızca tam sürümleri gösterir).

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
