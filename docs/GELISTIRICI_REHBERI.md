# Geliştirici Rehberi

## Proje yapısı

```
Wakai-Desktop/
├─ index.html              Kontrol Merkezi giriş sayfası
├─ src/
│  ├─ control-center/      Kontrol Merkezi (React)
│  ├─ ui/                  Tasarım sistemi: belirteçler, bileşenler, piksel kediler
│  └─ lib/                 IPC, Türkçe metinler, kancalar
├─ src-tauri/
│  ├─ src/
│  │  ├─ commands/         Tauri komutları (girdi doğrulama burada)
│  │  ├─ db/               SQLite, göçler, depolar
│  │  ├─ settings.rs       Türlü ayarlar
│  │  └─ lib.rs            Başlatma
│  ├─ capabilities/        Tauri izinleri (asgari)
│  └─ tauri.conf.json
├─ scripts/                Yardımcı betikler
└─ docs/                   Türkçe belgeler
```

## Kurallar

- **TypeScript kullanılmaz.** Ön yüz yalnızca `.js` / `.jsx` dosyalarından oluşur.
- Kod tanımlayıcıları İngilizce, kullanıcıya görünen tüm metinler Türkçedir. Birden çok yerde kullanılan metinler `src/lib/i18n.js` içindedir; yalnızca tek bir bileşene özgü metinler o bileşende durabilir.
- Renkler, boşluklar ve köşe değerleri yalnızca `src/ui/tokens.css` değişkenleriyle kullanılır.
- Kalıcı veri yalnızca Rust üzerinden SQLite'a yazılır; `localStorage` kullanılmaz.
- Veri değiştiren her komut ilgili olayı yayınlar (`src-tauri/src/events.rs`).
- `unsafe` kod yalnızca Windows entegrasyon modülünde bulunur.

## Komutlar

| Komut | Açıklama |
|---|---|
| `npm run tauri dev` | Geliştirme modunda çalıştırır. |
| `npm run build` | Ön yüzü derler. |
| `npm run lint` | ESLint. |
| `npm test` | Ön yüz birim testleri (Vitest). |
| `cargo test` (src-tauri içinde) | Rust testleri. |
| `cargo clippy --all-targets` | Rust lint. |
| `cargo fmt` | Rust biçimlendirme. |

## Yeni bir komut eklemek

1. `src-tauri/src/commands/` altında ilgili dosyaya `#[tauri::command]` fonksiyonu ekleyin; dönüş türü `AppResult<T>` olsun.
2. Girdiyi doğrulayın; hatalar `AppError::invalid("Türkçe mesaj")` ile döndürülür.
3. `lib.rs` içindeki `generate_handler!` listesine ekleyin.
4. Ön yüzde `call('komut_adi', { ... })` ile çağırın.

## Tasarım sistemi

| Dosya | İçerik |
|---|---|
| `src/ui/tokens.css` | Tüm renk, tipografi, boşluk, köşe, gölge ve hareket belirteçleri. Yeni tema = aynı değişkenlerin `:root[data-theme]` altında yeniden tanımlanması. |
| `src/ui/components.jsx` | Düğme, kart, anahtar, kaydırıcı, seçim, alan, rozet, boş durum, hata notu. |
| `src/ui/widget.jsx` | Widget yapı taşları: başlık (kediyle), ilerleme çubuğu, renk noktası, simge düğmesi. |
| `src/ui/PixelIcon.jsx` | 12×12 tek renkli piksel simgeler (`currentColor`). |
| `src/ui/cats/` | Özgün piksel kedi sprite'ları ve `PixelCat` bileşeni. |

### Piksel kediler

Pozlar: `face`, `sitting`, `happy`, `curious`, `relaxed`, `working`, `sleeping`. Her sprite 16×16 katmanlardan oluşur (`base`, `blink`, `tailA`/`tailB`, `zz`, `sparkle`, `question`). Renkler `--cat-*` belirteçlerinden gelir. Önizleme:

```powershell
node scripts/preview-sprites.mjs sprites.png base,tailA,zz
```

### Animasyon kuralları

- Yalnızca `opacity` ve `transform` canlandırılır; `steps()` ile düşük kare hızı kullanılır.
- Göz kırpma ~6 saniyede bir ve her kedide farklı gecikmeyle; kuyruk yalnızca üzerine gelince sallanır.
- Kutlama (`useCelebration`) tek seferliktir.
- `<html data-motion="full|reduced|off">` uygulama ayarlarından gelir; `reduced`/`off` ve işletim sisteminin “hareketi azalt” tercihi tüm animasyonları durdurur.
