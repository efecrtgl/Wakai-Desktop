# Geliştirme Günlüğü — 1.1.0

Her aşamanın sonunda ne yapıldığı, nasıl doğrulandığı ve commit bilgisi buraya eklenir.

## Aşama 0 — Denetim ve temel ölçüm (24 Kasım 2026)

- Depo, Rust modülleri, ön yüz, veritabanı (şema v2), Tauri yapılandırması, kurulum ve Git geçmişi incelendi. Ayrıntılar: [ARCHITECTURE.md](ARCHITECTURE.md).
- Mevcut testler: ön yüz 12/12, Rust 55 birim + 6 bütünleşme, lint ve clippy temiz.
- Temel ölçüm (kurulu 1.0.0): 8 widget ile 375,3 MB özel bellek, %0,05 CPU; widget'lar gizliyken 2,7 MB; açılış 0,8–1,0 sn. Ayrıntılar: [PERFORMANCE.md](PERFORMANCE.md).
- **Uyuşmazlık:** 1.1.0 görev tanımı mevcut bir "Alışveriş widget'ı"ndan söz ediyor; kodda böyle bir widget yok. Alışveriş Kedisi yeni bir widget olarak eklenecek (taşınacak eski veri yok).
- CI/CD yok; sürüm otomasyonu Aşama 9'da eklenecek.
