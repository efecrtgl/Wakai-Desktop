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
- Kod tanımlayıcıları İngilizce, kullanıcıya görünen tüm metinler Türkçedir ve `src/lib/i18n.js` içinde tutulur.
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
