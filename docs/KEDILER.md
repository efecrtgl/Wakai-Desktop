# Masaüstü Kedileri

Wakai Desktop 1.1.0 ile kedileriniz masaüstünde dolaşabilir: yürür, koşar, oturur, etrafa bakar, uyur, açık pencerelerin üstüne zıplar, oyuncakla oynar ve sizinle etkileşir.

## Nasıl çalışır?

| Konu | Davranış |
|---|---|
| Nerede durur? | Görev çubuğunun üstünde ya da açık bir pencerenin üst kenarında. Pencere taşınırsa kedi de onunla gider; pencere kapanır ya da küçültülürse aşağı düşer. |
| Uygulamalarımı engeller mi? | Hayır. Kedi yalnızca kendi şeklinin olduğu yerde fareyi algılar; çevresindeki saydam alandan tıklamalar alttaki uygulamaya geçer. Klavye odağını asla almaz. |
| Oyun / tam ekran | Tam ekran bir uygulama öndeyken kediler gizlenir (Ayarlar'dan kapatılabilir). |
| Widget'lar | Kediler widget'lardan tamamen bağımsızdır: widget'lar masaüstü katmanındadır, kediler pencerelerin üstündedir. |

## Etkileşimler

| Eylem | Nasıl? | Sonuç |
|---|---|---|
| Sevmek | Fareyi kedinin üstünde ileri geri gezdirin ya da bir kez tıklayın | Mutluluk artar, küçük bir kalp çıkar. Uyuyan kedi **uyanmaz**, yalnızca mutlu olur. |
| Taşımak | Basılı tutup sürükleyin | Kedi havada sallanır; bırakınca düşer (hızlı bırakırsanız biraz fırlar). |
| Uyandırmak | Uyuyan kediye tıklayın | Yavaşça uyanır. |
| Menü | Sağ tıklayın | Sev, Besle, Oyna, Uyut/Uyandır, Yerinde dursun, Kedi Evi'ne gönder, Kedileri yönet. |

## İhtiyaçlar

Her kedinin dört ihtiyacı vardır (0–100, yüksek iyi): **Tokluk**, **Enerji**, **Mutluluk**, **Uyku**.

- Uyanıkken tokluk ve uyku yavaşça azalır; koşmak ve oynamak enerji harcar.
- Uyurken enerji ve uyku dolar.
- Beslemek tokluğu, sevmek ve oynamak mutluluğu artırır.
- Yorgun kedi oynamak istemez; aç ya da mutsuz kedi size yaklaşır.
- **Ceza yok:** uygulama kapalıyken kedileriniz dinlenmiş sayılır; tokluk ve mutluluk %35'in altına düşmez (en fazla 24 saat hesaba katılır). Kediler ölmez, kaçmaz.

İhtiyaçlar geçen süreden hesaplanır; veritabanına her an değil, etkileşimlerde ve 5 dakikada bir yazılır.

## Kişilikler

Kişilik bir etiket değildir; davranış olasılıklarını ve ihtiyaçların değişim hızını gerçekten değiştirir.

| Kişilik | Davranış |
|---|---|
| Oyuncu | Daha sık koşar, oynar ve pencerelere zıplar; daha çabuk acıkır ve yorulur. |
| Sakin | Uzun uzun oturur, az koşar; ihtiyaçları yavaş değişir. |
| Meraklı | Etrafa çok bakar, keşfeder, pencerelere zıplamayı sever. |
| Uykucu | Sık ve uzun uyur. |
| Sevecen | İmlece sık yaklaşır; yalnız kalınca çabuk üzülür, sevilince daha çok mutlu olur. |
| Bağımsız | Kendi başına takılır, nadiren yaklaşır; yalnızlıktan pek etkilenmez. |

## Cinsler

Pembiş, Pamuk (beyaz), Gece (siyah), Tarçın (turuncu), Duman (gri), Benekli (tekir), Frak (siyah-beyaz), Krem. Hepsi aynı özgün piksel çizim dilini paylaşır (`scripts/gen-pet-sprites.mjs`).

## Yiyecek, oyuncak ve aksesuarlar

- **Yiyecekler** her gün ücretsiz yenilenir: Kuru mama (10), Balık (3), Süt (5), Ödül maması (3). Ödeme ya da sanal para yoktur.
- **Oyuncaklar:** Yün yumağı ve Küçük top baştan açık; Oyuncak fare ve Tüy çubuğu başarımlarla açılır. Oyuncak masaüstünde kedinin yanına bırakılır, kedi onu patisiyle iter.
- **Aksesuarlar:** baş (Fiyonk, Çiçek, Parti şapkası, Taç) ve boyun (Tasma, Atkı). Her kedi bir baş ve bir boyun aksesuarı takabilir.

## Başarımlar

İlk dost, Mama ustası, Sevgi dolu, Oyun arkadaşı, Sadık dost (3 gün üst üste besleme), Mutlu kedi, Koleksiyoncu (4 farklı cins). Başarımlar bildirim göndermez; Kontrol Merkezi'nde görünür.

## Performans

Kediler WebView kullanmaz; Rust'ta doğrudan katmanlı pencerelere çizilir. Kare hızı etkinliğe göre değişir: yürürken ~30 Hz konum, otururken birkaç saniyede bir kare, uyurken ~1,6 saniyede bir. Masaüstünde aynı anda en fazla 5 kedi dolaşır (Ayarlar'dan 1–12 arası); sınırın nedeni her kedinin ayrı bir yerel pencere ve kare hesabı olmasıdır.

## Bilinen sınırlamalar

- Kediler bulunduğu monitörün içinde kalır; monitörler arasında kendiliğinden geçmez (sürükleyerek taşıyabilirsiniz).
- Pencere üstleri, başka bir pencerenin altında kalsa bile basılabilir yüzey sayılır.
- Tam ekran algılama, ön plandaki pencerenin monitörü tamamen kaplamasına dayanır; bazı "kenarlıksız pencere" oyunları algılanmayabilir.
