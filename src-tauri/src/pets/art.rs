//! Kedi sprite'larının yüklenmesi ve piksel arabelleğine çizilmesi.
//! Sprite verisi `scripts/gen-pet-sprites.mjs` ile üretilen `assets/pets/sprites.json`
//! dosyasıdır (ön yüz de aynı dosyayı kullanır). Çıktı, `UpdateLayeredWindow` için
//! önceden çarpılmış (premultiplied) BGRA 32 bit piksellerdir.

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Aksesuarlar başın üstüne taşabilir; tuvalin üstüne ve yanlarına boşluk bırakılır.
pub const PAD_TOP: i32 = 5;
pub const PAD_SIDE: i32 = 2;

#[derive(Deserialize)]
pub struct Anchors {
    pub head: [i32; 2],
    pub neck: [i32; 2],
}

#[derive(Deserialize)]
pub struct Frame {
    pub rows: Vec<String>,
    pub anchors: Anchors,
}

#[derive(Deserialize)]
pub struct Breed {
    pub name: String,
    #[serde(flatten)]
    pub colors: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct Accessory {
    pub name: String,
    pub anchor: String,
    pub dx: i32,
    pub dy: i32,
    pub rows: Vec<String>,
    pub palette: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct SpriteFile {
    pub width: i32,
    pub height: i32,
    pub animations: HashMap<String, Vec<Frame>>,
    pub breeds: HashMap<String, Breed>,
    pub accessories: HashMap<String, Accessory>,
}

pub fn sprites() -> &'static SpriteFile {
    static S: OnceLock<SpriteFile> = OnceLock::new();
    S.get_or_init(|| {
        serde_json::from_str(include_str!("../../assets/pets/sprites.json"))
            .expect("sprites.json geçerli olmalı")
    })
}

/// Tuval boyutu (sprite pikseli cinsinden).
pub fn canvas_size() -> (i32, i32) {
    let s = sprites();
    (s.width + PAD_SIDE * 2, s.height + PAD_TOP)
}

fn hex(c: &str) -> Option<u32> {
    let c = c.strip_prefix('#')?;
    u32::from_str_radix(c, 16).ok().filter(|_| c.len() == 6)
}

/// Kedinin başının üstünde çizilen küçük efektler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    None,
    /// Uyku: iki kareli "z"
    Zz(u8),
    /// Sevilince kalp
    Heart,
}

const ZZ: [&str; 4] = ["zzz.", "..z.", ".z..", "zzz."];
const HEART: [&str; 4] = [".h.h.", "hhhhh", ".hhh.", "..h.."];

pub struct Render<'a> {
    pub anim: &'a str,
    pub frame: usize,
    pub breed: &'a str,
    pub accessories: &'a [&'a str],
    pub effect: Effect,
    pub flip: bool,
    /// Ekran pikseli / sprite pikseli
    pub scale: i32,
}

/// Animasyondaki kare sayısı (bilinmeyen animasyon: 0).
pub fn frame_count(anim: &str) -> usize {
    sprites().animations.get(anim).map_or(0, Vec::len)
}

/// Sprite'ı tuval ızgarasına (renk kodu, 0 = saydam) yerleştirir.
fn grid(r: &Render) -> Vec<u32> {
    let s = sprites();
    let (cw, ch) = canvas_size();
    let mut g = vec![0u32; (cw * ch) as usize];
    let Some(frames) = s
        .animations
        .get(r.anim)
        .or_else(|| s.animations.get("idle"))
    else {
        return g;
    };
    let f = &frames[r.frame % frames.len()];
    let breed = s.breeds.get(r.breed).or_else(|| s.breeds.get("pink"));
    let color = |ch: char| -> u32 {
        breed
            .and_then(|b| b.colors.get(&ch.to_string()))
            .and_then(|c| hex(c))
            .map_or(0, |c| c | 0xFF00_0000)
    };
    let mut put = |x: i32, y: i32, c: u32| {
        let (gx, gy) = (x + PAD_SIDE, y + PAD_TOP);
        if c != 0 && gx >= 0 && gy >= 0 && gx < cw && gy < ch {
            g[(gy * cw + gx) as usize] = c;
        }
    };
    for (y, row) in f.rows.iter().enumerate() {
        for (x, chr) in row.chars().enumerate() {
            if chr != '.' {
                put(x as i32, y as i32, color(chr));
            }
        }
    }
    for id in r.accessories {
        let Some(a) = s.accessories.get(*id) else {
            continue;
        };
        let [ax, ay] = if a.anchor == "neck" {
            f.anchors.neck
        } else {
            f.anchors.head
        };
        for (y, row) in a.rows.iter().enumerate() {
            for (x, chr) in row.chars().enumerate() {
                if let Some(c) = a.palette.get(&chr.to_string()).and_then(|c| hex(c)) {
                    put(ax + a.dx + x as i32, ay + a.dy + y as i32, c | 0xFF00_0000);
                }
            }
        }
    }
    let [hx, hy] = f.anchors.head;
    let (glyph, color, dy): (&[&str], u32, i32) = match r.effect {
        Effect::None => (&[], 0, 0),
        Effect::Zz(k) => (&ZZ, 0xFFB4_A2E3, if k % 2 == 0 { -4 } else { -5 }),
        Effect::Heart => (&HEART, 0xFFF0_7AA0, -5),
    };
    for (y, row) in glyph.iter().enumerate() {
        for (x, chr) in row.chars().enumerate() {
            if chr != '.' {
                put(hx + 2 + x as i32, hy + dy + y as i32, color);
            }
        }
    }
    g
}

/// Kareyi ölçekleyip (en yakın komşu) BGRA premultiplied piksellere çizer.
/// Dönüş: (genişlik, yükseklik, pikseller)
pub fn render(r: &Render) -> (i32, i32, Vec<u32>) {
    let (cw, ch) = canvas_size();
    let g = grid(r);
    let s = r.scale.max(1);
    let (w, h) = (cw * s, ch * s);
    let mut px = vec![0u32; (w * h) as usize];
    for gy in 0..ch {
        for gx in 0..cw {
            let src = if r.flip { cw - 1 - gx } else { gx };
            let c = g[(gy * cw + src) as usize];
            if c == 0 {
                continue;
            }
            // Tüm pikseller tam opaktır; premultiplied = aynı değer.
            for dy in 0..s {
                let row = ((gy * s + dy) * w) as usize;
                for dx in 0..s {
                    px[row + (gx * s + dx) as usize] = c;
                }
            }
        }
    }
    (w, h, px)
}

/// Basit oyuncak sprite'ları (masaüstünde kedinin yanına bırakılır).
pub fn toy_pixels(toy: &str, scale: i32) -> (i32, i32, Vec<u32>) {
    let (rows, pal): (&[&str], &[(char, u32)]) = match toy {
        "ball" => (
            &[".oo.", "obbo", "obbo", ".oo."],
            &[('o', 0xFF5B_4A4F), ('b', 0xFF8F_C3EE)],
        ),
        "mouse" => (
            &["..oo..", ".oggo.", "oggggo", ".o..o."],
            &[('o', 0xFF5B_4A4F), ('g', 0xFFA3_A1AA)],
        ),
        "feather" => (
            &["..f", ".fo", "fo.", "o.."],
            &[('o', 0xFF7A_5A3A), ('f', 0xFFB4_A2E3)],
        ),
        _ => (
            &[".oyo.", "oyyyo", "oyoyo", ".oyo."],
            &[('o', 0xFF5B_4A4F), ('y', 0xFFF4_9AB2)],
        ),
    };
    let h = rows.len() as i32;
    let w = rows[0].len() as i32;
    let s = scale.max(1);
    let mut px = vec![0u32; (w * s * h * s) as usize];
    for (y, row) in rows.iter().enumerate() {
        for (x, chr) in row.chars().enumerate() {
            if let Some((_, c)) = pal.iter().find(|(k, _)| *k == chr) {
                for dy in 0..s {
                    for dx in 0..s {
                        px[((y as i32 * s + dy) * w * s + x as i32 * s + dx) as usize] = *c;
                    }
                }
            }
        }
    }
    (w * s, h * s, px)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opaque(px: &[u32]) -> usize {
        px.iter().filter(|p| **p != 0).count()
    }

    #[test]
    fn all_animations_and_breeds_render() {
        let s = sprites();
        assert!(s.animations.len() >= 12);
        for anim in s.animations.keys() {
            for breed in s.breeds.keys() {
                let r = Render {
                    anim,
                    frame: 0,
                    breed,
                    accessories: &[],
                    effect: Effect::None,
                    flip: false,
                    scale: 2,
                };
                let (w, h, px) = render(&r);
                assert_eq!(px.len(), (w * h) as usize);
                assert!(opaque(&px) > 50, "{anim}/{breed} boş çizildi");
            }
        }
    }

    #[test]
    fn flip_mirrors_pixels() {
        let base = Render {
            anim: "walk",
            frame: 1,
            breed: "orange",
            accessories: &[],
            effect: Effect::None,
            flip: false,
            scale: 1,
        };
        let (w, h, a) = render(&base);
        let (_, _, b) = render(&Render { flip: true, ..base });
        for y in 0..h {
            for x in 0..w {
                assert_eq!(a[(y * w + x) as usize], b[(y * w + (w - 1 - x)) as usize]);
            }
        }
    }

    #[test]
    fn accessories_and_effects_add_pixels_inside_canvas() {
        let plain = Render {
            anim: "sit",
            frame: 0,
            breed: "white",
            accessories: &[],
            effect: Effect::None,
            flip: false,
            scale: 1,
        };
        let n0 = opaque(&render(&plain).2);
        let all: Vec<&str> = sprites().accessories.keys().map(String::as_str).collect();
        for acc in &all {
            let n = opaque(
                &render(&Render {
                    accessories: std::slice::from_ref(acc),
                    ..plain
                })
                .2,
            );
            assert!(n > n0 - 6, "{acc} çizilemedi");
        }
        assert!(
            opaque(
                &render(&Render {
                    effect: Effect::Heart,
                    ..plain
                })
                .2
            ) > n0
        );
        assert!(
            opaque(
                &render(&Render {
                    effect: Effect::Zz(1),
                    anim: "sleep",
                    ..plain
                })
                .2
            ) > 0
        );
    }

    #[test]
    fn toys_render() {
        for t in ["yarn", "ball", "mouse", "feather"] {
            let (w, h, px) = toy_pixels(t, 3);
            assert!(w > 0 && h > 0 && opaque(&px) > 10);
        }
    }
}
