# Kurulum

## Son kullanıcılar için

1. `Wakai Desktop_1.0.0_x64-setup.exe` dosyasını çalıştırın (≈ 2,5 MB).
2. Kurulum **yalnızca geçerli kullanıcıya** yapılır; yönetici izni gerekmez.
   - Program: `%LOCALAPPDATA%\Wakai Desktop\`
   - Başlat menüsü ve masaüstü kısayolu oluşturulur.
3. WebView2 Runtime Windows 11'de yüklü gelir; eksikse kurulum sessizce indirir.

**Kaldırma:** Ayarlar → Uygulamalar → Wakai Desktop → Kaldır. Program dosyaları, kısayollar ve kaldırma kaydı silinir; **verileriniz** (`%APPDATA%\com.wakai.desktop\`) korunur. Verileri de silmek isterseniz bu klasörü elle silin.

**Doğrulama (geliştirme makinesinde, 1.0.0):**

| Adım | Sonuç |
|---|---|
| Sessiz kurulum (`/S`) | Çıkış kodu 0; program, kısayollar ve kaldırma kaydı oluştu. |
| Kurulu uygulamayı çalıştırma | Mevcut verilerle açıldı, widget'ları geri yükledi. |
| "Windows ile başlat" açık | `HKCU\...\Run` altına `"...\wakai-desktop.exe" --autostart` yazıldı. |
| "Windows ile başlat" kapalı | Kayıt silindi. |
| Sessiz kaldırma (`/S`) | Program dosyaları, kısayollar ve kaldırma kaydı silindi; kullanıcı verisi korundu. |
| Gerçek oturum açılışında otomatik başlama | Denenmedi (bilgisayarın yeniden başlatılmasını gerektirir). |

## Geliştiriciler için gereksinimler

| Araç | Sürüm (doğrulanmış) | Not |
|---|---|---|
| Windows | 11 (24H2 ve üstü önerilir) | Uygulama yalnızca Windows 11'i hedefler. |
| Rust | 1.100 (stable, `x86_64-pc-windows-msvc`) | `winget install Rustlang.Rustup` |
| Visual Studio C++ derleme araçları | MSVC 14.5x | “Masaüstü C++ geliştirme” iş yükü |
| Windows SDK | 10.0.26100 | `winget install Microsoft.WindowsSDK.10.0.26100` |
| Node.js | 24.x | npm 11 ile birlikte |
| WebView2 Runtime | Windows 11'de yüklü gelir | |

## Projeyi çalıştırma

```powershell
git clone https://github.com/efecrtgl/Wakai-Desktop.git
cd Wakai-Desktop
npm install
npm run tauri dev
```

## Üretim derlemesi

```powershell
npm run tauri build
```

Çıktı: `src-tauri/target/release/bundle/nsis/Wakai Desktop_<sürüm>_x64-setup.exe`. Yükleyici Türkçe NSIS arayüzüyle ve `currentUser` kurulum kipinde üretilir (`src-tauri/tauri.conf.json` → `bundle`).

**Sürüm numarası** üç yerde aynı tutulur: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`.

## Veri konumu

Kullanıcı verileri `%APPDATA%\com.wakai.desktop\wakai.db` dosyasında saklanır. Ayrıntılar: [VERITABANI.md](VERITABANI.md).
