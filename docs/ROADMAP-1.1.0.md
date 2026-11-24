# Wakai Desktop 1.1.0 Yol Haritası

| Aşama | Kapsam | Durum |
|---|---|---|
| 0 | Denetim ve temel ölçüm | ✅ |
| 1 | Tasarım sistemi: yazı ölçeği, Wakai / Siyah / Beyaz görünümleri | ⏳ |
| 2 | Widget yenilemesi: Takvim yeniden tasarımı, yeni Alışveriş Kedisi, tutarlılık | ⏳ |
| 3 | Masaüstü evcil hayvan motoru (Rust, yerel katmanlı pencereler) | ⏳ |
| 4 | Kedi çizimleri, animasyonlar ve kişilikler | ⏳ |
| 5 | İhtiyaçlar ve etkileşimler (beslemek, sevmek, uyku) | ⏳ |
| 6 | Oyuncaklar, aksesuarlar, koleksiyon ve başarımlar | ⏳ |
| 7 | Kedi Evi | ⏳ |
| 8 | Masaüstü profilleri (Çalışma, Oyun, Dinlenme, Odaklanma, özel) | ⏳ |
| 9 | İmzalı otomatik güncellemeler (Tauri Updater + GitHub Releases) | ⏳ |
| 10 | Sürüm geçmişi ve güvenli kurtarma | ⏳ |
| 11 | Yedekleme ve veri yönetimi | ⏳ |
| 12 | Performans, güvenlik ve kararlılık | ⏳ |
| 13 | Test ve kalite güvencesi | ⏳ |
| 14 | Belgeler ve sürüm hazırlığı | ⏳ |

## Temel teknik kararlar

- **Kediler widget değildir.** Widget'lar duvar kağıdı katmanında WebView pencereleridir (her biri ≈26 MB). Kediler ise Rust'ta, WebView kullanmadan, `UpdateLayeredWindow` ile çizilen küçük katmanlı pencerelerdir: piksel başına saydamlık, saydam piksellerden tıklama geçer, odak çalmaz (`WS_EX_NOACTIVATE`), uygulama pencerelerinin üstünde durur.
- **Kare hızı etkinliğe göre:** yürürken/koşarken yüksek, otururken düşük, uyurken neredeyse sıfır.
- **İhtiyaçlar** geçen süreden hesaplanır; veritabanına kare başına değil, olay anında ve seyrek aralıklarla yazılır.
- **Profiller** ayarların üstünde bir katmandır; widget konumlarını ve kedi durumunu değiştirmez.
- **Güncellemeler** resmi Tauri Updater eklentisi, GitHub Releases ve imzalı yapıtlarla.
