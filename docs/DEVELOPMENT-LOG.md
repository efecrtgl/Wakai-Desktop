# Geliştirme Günlüğü — 1.1.0

Her aşamanın sonunda ne yapıldığı, nasıl doğrulandığı ve commit bilgisi buraya eklenir.

## Aşama 0 — Denetim ve temel ölçüm (24 Kasım 2026)

- Depo, Rust modülleri, ön yüz, veritabanı (şema v2), Tauri yapılandırması, kurulum ve Git geçmişi incelendi. Ayrıntılar: [ARCHITECTURE.md](ARCHITECTURE.md).
- Mevcut testler: ön yüz 12/12, Rust 55 birim + 6 bütünleşme, lint ve clippy temiz.
- Temel ölçüm (kurulu 1.0.0): 8 widget ile 375,3 MB özel bellek, %0,05 CPU; widget'lar gizliyken 2,7 MB; açılış 0,8–1,0 sn. Ayrıntılar: [PERFORMANCE.md](PERFORMANCE.md).
- **Uyuşmazlık:** 1.1.0 görev tanımı mevcut bir "Alışveriş widget'ı"ndan söz ediyor; kodda böyle bir widget yok. Alışveriş Kedisi yeni bir widget olarak eklenecek (taşınacak eski veri yok).
- CI/CD yok; sürüm otomasyonu Aşama 9'da eklenecek.

## Aşama 1 — Tasarım sistemi ve görünüm modları

- Üç tam görünüm modu: **Wakai** (özgün pastel), **Siyah**, **Beyaz**. Filtre değil, her biri kendi anlamsal belirteç kümesiyle (`src/ui/tokens.css`). Ayar SQLite'ta (`appearance`) saklanır; Kontrol Merkezi ve her widget penceresi `<html data-theme>` ile uygular. Siyah modda Kontrol Merkezi'nin başlık çubuğu da koyu olur.
- Bileşenlerdeki tüm sabit renkler ve ham palet değişkenleri anlamsal belirteçlere taşındı (denetimde 15 yer).
- Tipografi ölçeği: boyut, ağırlık (`--weight-*`), satır yüksekliği (`--leading-*`); tüm CSS'teki sayısal ağırlıklar belirteçlere çevrildi.
- Yazı tipi seçimi: Varsayılan (Segoe UI Variable), Modern (Bahnschrift), Daktilo (Cascadia). Hepsi Windows 11'de yüklü; uygulama boyutu değişmedi.
- Yeni ortak bileşenler: `Dialog`, `Loading`, `Tooltip`.
- Doğrulama: üç modda Kontrol Merkezi ve widget ekran görüntüleri incelendi; ayar testi (geçersiz görünüm değeri reddedilir) eklendi.

## Aşama 2 — Widget yenilemesi

- **Takvim Kedisi yeniden tasarlandı:** ay/yıl başlığı, yalnızca başka bir güne/aya geçildiğinde görünen "Bugün" kısayolu, yuvarlak köşeli gün hücreleri (bugün: yumuşak vurgu, seçili: dolu), ok tuşlarıyla gün gezinme, renk çubuklu etkinlik satırları ve etkinlik sayısı rozeti, boş gün için "eklemek için dokun". Veriler ve işlevler aynı.
- **Alışveriş Kedisi (yeni):** göç 003 (`shopping_lists`, `shopping_items`), 10 komut, `shopping:changed` olayı. "2 kg elma" gibi girdiden miktar ayrıştırma, sepete atma, çift tıkla düzenleme, silme, sepeti temizleme, boş durum, son madde sepete girince kutlama. Widget bir listeye bağlıdır; widget silinince liste kalır.
- **Kapsayıcı sorguları:** widget içeriği kendi boyutuna göre sadeleşir (Takvim, Görev, Hava Durumu, Sistem, Müzik, Alışveriş).
- `WidgetIconButton` verilen `className`'i temel sınıfın yerine koyuyordu; birleştirilecek şekilde düzeltildi.
- Doğrulama: 2 yeni Rust testi (liste yaşam döngüsü, doğrulama ve kademeli silme), 2 yeni ön yüz testi (miktar ayrıştırma); widget'lar gerçek uygulamada ekran görüntüsüyle incelendi (Takvim tam ve kompakt boyut, Alışveriş). Test widget'ı ve örnek liste sonra kaldırıldı.

## Aşama 3–4 — Masaüstü kedi motoru, çizimler ve kişilikler

Aşama 3 ve 4 tek commit'te birleştirildi: motorun gerçek davranışla test edilebilmesi için çizimler ve kişilikler önkoşuldu.

- **Mimari:** `src-tauri/src/pets/` widget motorundan bağımsız. Kediler WebView değil, Rust'ta `UpdateLayeredWindow` ile çizilen katmanlı pencerelerdir (`WS_EX_LAYERED | TOOLWINDOW | NOACTIVATE | TOPMOST`). Saydam piksellerden tıklama geçer, odak çalınmaz.
- **Motor** tek iş parçacığı; etkinliğe göre uyur (hareket ~30 Hz, oturma saniyeler, uyku 1,6 sn, kedi yoksa tamamen bekler). Pencere üstlerine zıplama, taşınan pencereyi takip etme, düşme, sürükle-bırak-fırlat, okşama algılama, oyuncak fiziği, tam ekranda gizlenme.
- **Çizimler:** `scripts/gen-pet-sprites.mjs` parçalardan 14 animasyon / 25 kare üretir, konturu otomatik çizer; 8 cins palet; 6 aksesuar. Rust ve ön yüz aynı `sprites.json`'u kullanır.
- **Kişilik** davranış ağırlıklarını ve ihtiyaç hızlarını değiştirir (test: uykucu kedi oyuncudan 2 kat fazla uyur, sevecen kedi bağımsızdan 5 kat fazla yaklaşır).
- **İhtiyaçlar** geçen süreden hesaplanır (`PetStore`), veritabanına etkileşimlerde ve 5 dakikada bir yazılır. Göç 004.
- **Bulunan hatalar:** `WM_MOUSELEAVE` kapsamda olmadığı için Rust onu her mesajı yakalayan bir *değişken deseni* sayıyordu (sağ tık ve `WM_MOUSEACTIVATE` hiç çalışmazdı) — derleyici uyarısıyla yakalandı ve düzeltildi. `PetStore` başlangıçta kaydedilmiyordu — ilk gerçek çalıştırmada yakalandı.
- **Doğrulama (gerçek Windows 11):** iki kedi oluşturuldu; pencere stilleri doğrulandı; kediler görev çubuğu üstünde, açık tarayıcının üstünde saydam çizildi (ekran görüntüsü); bir kedi uyudu (Zz), diğeri uyandıktan sonra yürüdü; besleme (tokluk 80→100, balık 3→2), başarımlar (İlk dost, Mutlu kedi) ve oyuncak penceresinin kedi tarafından itilmesi (konum izlendi) doğrulandı. 81 Rust birim testi.
- **Elle doğrulanması gerekenler:** fareyle okşama, sürükle-bırak, sağ tık menüsü ve pencere üstüne zıplama gerçek fareyle denenmedi (kullanıcının faresi kullanılmadı).

## Aşama 5–6 — İhtiyaçlar, etkileşimler, oyuncaklar, aksesuarlar ve koleksiyon

Kurallar (Rust, Aşama 3–4 commit'inde test edildi) bu aşamada kullanıcı arayüzüne bağlandı.

- **Kontrol Merkezi → Kediler:** sahiplenme penceresi (ad, 8 cins önizlemesi, kişilik açıklaması), her kedi için kart: canlı ihtiyaç çubukları (30 sn'de bir ve `pets:changed` ile tazelenir), ruh hâli, Sev / Besle (bugünkü yiyecek sayılarıyla) / Oyna (açık oyuncaklar) / Çağır, masaüstü–Kedi Evi geçişi, yerinde dursun, baş ve boyun aksesuarı (kilitliler kilit nedeniyle gösterilir), çift tıkla yeniden adlandırma, onaylı koleksiyondan çıkarma.
- **Başarımlar** ve istatistikler (besleme, sevgi, oyun, üst üste besleme günü) — bildirim göndermez.
- **Kedi ayarları:** açık/kapalı, etkinlik düzeyi, boyut, aynı anda en fazla kedi, tam ekranda gizleme.
- Ön yüz sprite çizimi Rust ile aynı `sprites.json`'dan (`src/pets/sprite.js`); 2 yeni ön yüz testi (tüm animasyon × cins çizilir, aksesuarlar ızgarayı değiştirir).
- Doğrulama: ekran görüntüleriyle koleksiyon ve sahiplenme penceresi incelendi; aksesuar seçiminin taşma hatası bulunup düzeltildi.

## Aşama 7 — Kedi Evi

- Kontrol Merkezi'nde yeni bölüm; ayrı pencere açmaz (ek WebView yok). Tek `<canvas>`, tam sayı ölçekli piksel sanatı oda (320×180 birim): çizgili duvar, pencere (gece duvarında ay), süpürgelik, zemin.
- 10 eşya (kediyle aynı çizim dili, otomatik kontur). Eşyalar `pet_home_items` tablosunda; ilk açılışta bir kez varsayılan oda kurulur (hepsi silinirse yeniden kurulmaz); en fazla 60 eşya; konumlar sunucuda zemine sıkıştırılır.
- Simülasyon (`src/pets/home/sim.js`, saf): ihtiyaç önceliği (uyku → yatak, açlık → mama), kişilik ağırlıkları (oyuncak, tırmalama, şekerleme, arkadaşının yanına oturma), yalnızca sayfa açık ve görünürken 8 Hz.
- Düzenleme modu: sürükle, çevir, kaldır, katalogdan ekle; duvar/zemin seçimi kalıcı.
- Doğrulama: 5 yeni ön yüz testi + 1 Rust testi (tek seferlik varsayılan oda, sıkıştırma). Gerçek uygulamada oda ve kedinin dolaşması ekran görüntüsüyle incelendi; o sırada kullanıcı odayı düzenleyip top ekledi — düzenleme gerçek kullanımda da çalıştı.

## Aşama 8 — Masaüstü profilleri

- Göç 005 (`desktop_profiles`), yerleşik profiller: **Çalışma, Oyun, Dinlenme, Odaklanma** (silinemez, varsayılana döndürülebilir) + özel profiller (oluştur, adlandır, düzenle, sil).
- Profil ayarların üstünde bir katmandır: genel widget görünürlüğü, widget görünürlük kuralı (dokunma / hepsi / türler / seçili widget'lar), kediler açık/kapalı, kedi etkinliği, tam ekranda gizleme, animasyon, bildirimler. Widget konumlarına ve kedi durumuna dokunmaz.
- Hızlı geçiş: Kontrol Merkezi → Profiller ve tepsi menüsü → Profil (etkin profil işaretli; menü ayar/profil değişince yeniden kurulur). Tepsiye "Masaüstü kedileri" onay kutusu da eklendi.
- İsteğe bağlı **otomatik Oyun profili**: tam ekran uygulama algılanınca geçer, çıkınca önceki profile döner. Varsayılan kapalı; açıkken 3 sn'de bir hafif kontrol yapan iş parçacığı çalışır, kapatılınca biter.
- Doğrulama: 3 yeni Rust testi. Gerçek uygulamada Odaklanma ve Dinlenme uygulandı: görünürlük kuralları doğru, tüm widget konumları üç durumda da aynı. Test sonrası kullanıcının önceki widget görünürlüğü ve ayarları geri yüklendi.
- Elle doğrulanacaklar: tepsi profil menüsü tıklaması; otomatik geçişin gerçek bir tam ekran oyunla denenmesi.

## Aşama 9 — İmzalı otomatik güncellemeler

- Resmi `tauri-plugin-updater` (2.14.1); NSIS yükleyicisi için `createUpdaterArtifacts`, `installMode: passive`. Uç nokta: GitHub Releases `latest/download/latest.json`.
- İmzalama anahtarı depo dışında üretildi (`~/.tauri/wakai-desktop-updater.key`); depoda yalnızca genel anahtar var. Depoda anahtar dosyası olmadığı doğrulandı.
- `src-tauri/src/updater.rs`: açılıştan 20 sn sonra, en fazla 20 saatte bir, kendi iş parçacığında denetim (ayar `auto_update_check`, varsayılan açık); `update:available` olayı + bildirim; indirme ilerlemesi `update:progress` (≈256 KB'de bir); imza doğrulandıktan **sonra** kurulum öncesi hazırlık (konumlar + kediler kaydedilir, veritabanı yedeği, çalışan exe `previous/<sürüm>/` altına saklanır). Başarısız indirme hiçbir iz bırakmaz; bekleyen güncelleme yeniden denenebilir.
- Kontrol Merkezi: "Yeni bir sürüm hazır!" penceresi (Güncellemeyi İndir / Daha Sonra, notlar, ilerleme çubuğu), Ayarlar → Güncellemeler kartı (otomatik denetim, elle denetim, son denetim).
- `.github/workflows/release.yml` (etiketle imzalı sürüm, `latest.json`; gizli yoksa durur) ve `ci.yml` (lint, test, build, fmt, clippy `-D warnings`, cargo test).
- Doğrulama: yerel HTTP sunucusu + hata ayıklama derlemesi; geçerli imza kabul edildi, değiştirilmiş dosya reddedildi, eski sürüm "güncelleme yok", erişilemeyen sunucu anlaşılır hata verdi. Birincil kuruluma güncelleme kurulmadı. Ayrıntılar: `docs/GUNCELLEMELER.md`.
- **Sahip eylemi:** `TAURI_SIGNING_PRIVATE_KEY` GitHub gizlisini eklemek. 1.0.0 kullanıcıları 1.1.0'ı bir kez elle kurmalı.
- Gözlem: test sırasında Kontrol Merkezi tüm widget'ları gizli gösterdi (veriye dokunulmadı; kullanıcı bilgilendirilecek).

## Aşama 10 — Sürüm geçmişi ve güvenli geri dönüş

- Göç 006 `app_versions` (yalnızca ekleme). Her açılış kaydedilir; sürüm değişince bekleyen güncelleme kaydı temizlenir.
- `recovery.rs`: önceki sürümün saklanması (en fazla 2), sürüm başına açılış sağlığı (15 sn sağlıklı → bir daha sayılmaz), hiç sağlıklı açılmamış sürüm 3 kez başarısız olursa otomatik geri dönüş, `.exe.old` yöntemiyle güvenli exe değişimi (başarısız kopyada geri alma), tek seferlik geri dönüş bildirimi.
- Komutlar `version_history`, `rollback_to_version` (önce doğrulama, sonra yedek, sonra değişim ve yeniden başlatma). Ayarlar → Sürüm geçmişi kartı + gömülü Türkçe sürüm notları.
- Yalıtılmış test ortamı için `WAKAI_DATA_DIR`.
- Doğrulama: 3 yeni Rust testi (91 birim test). Yalıtılmış ortamda otomatik ve elle geri dönüş gerçek exe ile denendi (SHA-256 ile doğrulandı). Test sırasında bulunan hata: geçersiz sürümle çağrıda önce yedek alınıyordu — doğrulama öne alındı ve yeniden denendi.
- Not: gerçek bir NSIS güncellemesinden sonraki geri dönüş, imzalı ilk sürüm yayınlandıktan sonra ayrıca denenmelidir; değiştirilen mekanizma (exe değişimi + yeniden başlatma) burada denendi.

## Aşama 11 — Yedek iyileştirmeleri

- Atomik yedek (`.tmp` → `quick_check` → yeniden adlandırma) ve atomik tam geri yükleme (geçici kopya → doğrulama → tek adımda yerine koyma; bozuk yedek mevcut veritabanına dokunmadan atlanır).
- `inspect_backup`: bütünlük, şema uyumluluğu (eski/yeni), kategori başına kayıt sayısı.
- Seçmeli geri yükleme (9 kategori): açılışta, veriler kullanılmadan önce, tek işlemde; ortak sütunlarla şema farklarına dayanıklı; önce `-secmeli-oncesi` yedeği.
- Geri yükleme öncesi kedi durumu da kaydedilir. Kontrol Merkezi'nde yeni "Yedeği geri yükle" penceresi.
- Doğrulama: 3 yeni Rust testi (94 birim test). Yalıtılmış ortamda arayüzden seçmeli geri yükleme ve bozuk yedek reddi denendi. Ayrıntılar: `docs/YEDEKLER.md`.
