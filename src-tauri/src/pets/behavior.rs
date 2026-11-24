//! Davranış seçimi: kişilik + ihtiyaçlar + ortam → bir sonraki eylem ve süresi.
//! Saf mantıktır; rastgelelik tohumlanabilir bir xorshift üretecinden gelir (test edilebilir).

use super::model::{Needs, Personality};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Walk,
    Run,
    Sit,
    Idle,
    Look,
    Sleep,
    Play,
    /// İmlece doğru yürü
    Approach,
    /// Yakındaki bir pencerenin üstüne zıpla
    Climb,
}

/// Profilden gelen genel etkinlik düzeyi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActivityLevel {
    /// Kediler yerinde durur (oturur/uyur), dolaşmaz.
    Paused,
    Low,
    #[default]
    Normal,
    High,
}

pub struct Context {
    pub personality: Personality,
    pub needs: Needs,
    pub level: ActivityLevel,
    pub cursor_near: bool,
    pub platform_reachable: bool,
}

/// Küçük, hızlı, tohumlanabilir rastgele sayı üreteci (xorshift64*).
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// [0, 1)
    pub fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.f64()
    }
    pub fn chance(&mut self, p: f64) -> bool {
        self.f64() < p
    }
}

fn base_weights(p: Personality) -> [(Action, f64); 9] {
    use Action::*;
    let w = match p {
        //                     walk run  sit  idle look sleep play appr climb
        Personality::Playful => [3.0, 2.0, 1.0, 1.0, 1.0, 1.0, 3.0, 1.0, 2.0],
        Personality::Calm => [2.0, 0.3, 3.0, 3.0, 1.0, 2.0, 0.7, 1.0, 0.5],
        Personality::Curious => [3.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 2.0, 3.0],
        Personality::Sleepy => [1.0, 0.2, 2.0, 2.0, 0.5, 5.0, 0.5, 0.5, 0.3],
        Personality::Affectionate => [2.0, 0.5, 2.0, 1.0, 1.0, 1.5, 1.0, 4.0, 0.5],
        Personality::Independent => [3.0, 1.0, 2.0, 2.0, 1.0, 2.0, 1.0, 0.2, 1.5],
    };
    [
        (Walk, w[0]),
        (Run, w[1]),
        (Sit, w[2]),
        (Idle, w[3]),
        (Look, w[4]),
        (Sleep, w[5]),
        (Play, w[6]),
        (Approach, w[7]),
        (Climb, w[8]),
    ]
}

/// Ağırlıkları ihtiyaçlara, ortama ve etkinlik düzeyine göre ayarlar.
pub fn weights(ctx: &Context) -> Vec<(Action, f64)> {
    use Action::*;
    let n = ctx.needs;
    base_weights(ctx.personality)
        .into_iter()
        .map(|(a, mut w)| {
            // Uykusu gelen kedi uyumak ister; dinlenmiş kedi pek uyumaz.
            if a == Sleep {
                w *= if n.rest < 25.0 {
                    8.0
                } else if n.rest < 50.0 {
                    2.0
                } else if n.rest > 85.0 {
                    0.15
                } else {
                    1.0
                };
            }
            // Enerjisi düşük kedi koşmaz, oynamaz.
            if matches!(a, Run | Play | Climb) {
                w *= if n.energy < 20.0 {
                    0.05
                } else if n.energy < 45.0 {
                    0.4
                } else {
                    1.0
                };
            }
            if matches!(a, Sit | Idle) && n.energy < 35.0 {
                w *= 2.0;
            }
            // Aç ya da mutsuz kedi kullanıcıya yaklaşır (ilgi ister).
            if a == Approach {
                if n.satiety < 30.0 || n.happiness < 40.0 {
                    w *= 3.0;
                }
                if !ctx.cursor_near {
                    w *= 0.2;
                }
            }
            if a == Climb && !ctx.platform_reachable {
                w = 0.0;
            }
            match ctx.level {
                ActivityLevel::Paused => {
                    if !matches!(a, Sit | Idle | Sleep | Look) {
                        w = 0.0;
                    }
                }
                ActivityLevel::Low => {
                    if matches!(a, Walk | Run | Play | Climb | Approach) {
                        w *= 0.25;
                    }
                }
                ActivityLevel::High => {
                    if matches!(a, Walk | Run | Play | Climb) {
                        w *= 1.8;
                    }
                }
                ActivityLevel::Normal => {}
            }
            (a, w.max(0.0))
        })
        .collect()
}

pub fn choose(ctx: &Context, rng: &mut Rng) -> Action {
    let ws = weights(ctx);
    let total: f64 = ws.iter().map(|(_, w)| w).sum();
    if total <= 0.0 {
        return Action::Sit;
    }
    let mut r = rng.f64() * total;
    for (a, w) in &ws {
        if r < *w {
            return *a;
        }
        r -= w;
    }
    ws.last().map(|(a, _)| *a).unwrap_or(Action::Sit)
}

/// Eylemin süresi (saniye). Her kedi farklı sürelerle davranır; eşzamanlı hareket olmaz.
pub fn duration(action: Action, p: Personality, rng: &mut Rng) -> f64 {
    let base = match action {
        Action::Walk => rng.range(3.0, 9.0),
        Action::Run => rng.range(1.5, 3.5),
        Action::Sit => rng.range(6.0, 20.0),
        Action::Idle => rng.range(3.0, 10.0),
        Action::Look => rng.range(2.0, 5.0),
        Action::Sleep => rng.range(60.0, 240.0),
        Action::Play => rng.range(4.0, 8.0),
        Action::Approach => rng.range(4.0, 10.0),
        Action::Climb => 2.0,
    };
    let k = match (p, action) {
        (Personality::Sleepy, Action::Sleep) => 1.8,
        (Personality::Calm, Action::Sit | Action::Idle) => 1.5,
        (Personality::Playful, Action::Play) => 1.6,
        (Personality::Curious, Action::Look | Action::Walk) => 1.3,
        _ => 1.0,
    };
    base * k
}

#[cfg(test)]
mod tests {
    use super::*;

    fn freq(p: Personality, needs: Needs, level: ActivityLevel, action: Action) -> f64 {
        let ctx = Context {
            personality: p,
            needs,
            level,
            cursor_near: true,
            platform_reachable: true,
        };
        let mut rng = Rng::new(42);
        let n = 20_000;
        (0..n).filter(|_| choose(&ctx, &mut rng) == action).count() as f64 / n as f64
    }

    #[test]
    fn personality_affects_behaviour() {
        let n = Needs {
            rest: 60.0,
            ..Needs::default()
        };
        assert!(
            freq(Personality::Sleepy, n, ActivityLevel::Normal, Action::Sleep)
                > 2.0
                    * freq(
                        Personality::Playful,
                        n,
                        ActivityLevel::Normal,
                        Action::Sleep
                    )
        );
        assert!(
            freq(Personality::Playful, n, ActivityLevel::Normal, Action::Play)
                > 2.0 * freq(Personality::Calm, n, ActivityLevel::Normal, Action::Play)
        );
        assert!(
            freq(
                Personality::Affectionate,
                n,
                ActivityLevel::Normal,
                Action::Approach
            ) > 5.0
                * freq(
                    Personality::Independent,
                    n,
                    ActivityLevel::Normal,
                    Action::Approach
                )
        );
        assert!(
            freq(
                Personality::Curious,
                n,
                ActivityLevel::Normal,
                Action::Climb
            ) > freq(Personality::Calm, n, ActivityLevel::Normal, Action::Climb)
        );
    }

    #[test]
    fn needs_affect_behaviour() {
        let tired = Needs {
            rest: 10.0,
            energy: 15.0,
            ..Needs::default()
        };
        assert!(
            freq(
                Personality::Playful,
                tired,
                ActivityLevel::Normal,
                Action::Sleep
            ) > 0.4
        );
        assert!(
            freq(
                Personality::Playful,
                tired,
                ActivityLevel::Normal,
                Action::Run
            ) < 0.01
        );
    }

    #[test]
    fn paused_level_never_moves() {
        let ctx = Context {
            personality: Personality::Playful,
            needs: Needs::default(),
            level: ActivityLevel::Paused,
            cursor_near: true,
            platform_reachable: true,
        };
        let mut rng = Rng::new(7);
        for _ in 0..5000 {
            assert!(matches!(
                choose(&ctx, &mut rng),
                Action::Sit | Action::Idle | Action::Sleep | Action::Look
            ));
        }
    }

    #[test]
    fn climb_requires_platform_and_durations_vary() {
        let ctx = Context {
            personality: Personality::Curious,
            needs: Needs::default(),
            level: ActivityLevel::High,
            cursor_near: false,
            platform_reachable: false,
        };
        let mut rng = Rng::new(3);
        assert!((0..5000).all(|_| choose(&ctx, &mut rng) != Action::Climb));
        let a = duration(Action::Walk, Personality::Calm, &mut rng);
        let b = duration(Action::Walk, Personality::Calm, &mut rng);
        assert_ne!(a, b);
    }

    #[test]
    fn rng_is_uniform_enough() {
        let mut rng = Rng::new(99);
        let mean: f64 = (0..10_000).map(|_| rng.f64()).sum::<f64>() / 10_000.0;
        assert!((mean - 0.5).abs() < 0.02);
    }
}
