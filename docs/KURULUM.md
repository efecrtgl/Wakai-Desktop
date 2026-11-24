# Kurulum

## Son kullanıcılar için

Hazır yükleyici ilk sürümle birlikte (Aşama 11) yayınlanacaktır.

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

Çıktı: `src-tauri/target/release/bundle/nsis/`.

## Veri konumu

Kullanıcı verileri `%APPDATA%\com.wakai.desktop\wakai.db` dosyasında saklanır. Ayrıntılar: [VERITABANI.md](VERITABANI.md).
