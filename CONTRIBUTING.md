# Katkıda Bulunma

Ivily Desktop'a (eski adıyla Wakai Desktop) katkıda bulunmak istediğiniz için teşekkürler! 🐾

## Başlamadan önce

1. [Kurulum](docs/KURULUM.md) adımlarını izleyin.
2. [Geliştirici Rehberi](docs/GELISTIRICI_REHBERI.md) ve [Mimari](docs/MIMARI.md) belgelerini okuyun.

## Değişiklik gönderirken

- Kod tanımlayıcıları İngilizce, arayüz metinleri ve belgeler Türkçe olmalıdır.
- TypeScript eklemeyin.
- Göndermeden önce şunları çalıştırın:
  ```powershell
  npm run lint
  npm test
  npm run build
  cd src-tauri; cargo fmt --check; cargo clippy --all-targets; cargo test
  ```
- Commit mesajları İngilizce ve [Conventional Commits](https://www.conventionalcommits.org/) biçiminde olmalıdır (ör. `feat(calendar): add event reminders`).
- Performansı etkileyen değişikliklerde [Performans](docs/PERFORMANS.md) belgesindeki ölçüm yöntemiyle önce/sonra değerleri ekleyin.
- API anahtarı, parola veya kişisel veri commit etmeyin.
