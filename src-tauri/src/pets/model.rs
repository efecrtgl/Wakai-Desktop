//! Kedi alan modeli: kişilikler, ihtiyaçlar ve kataloglar (yiyecek, oyuncak, aksesuar).
//! Bu dosyadaki her şey saf (yan etkisiz) ve birim testlidir.

use serde::{Deserialize, Serialize};

/// Kişilik davranış olasılıklarını VE ihtiyaçların değişim hızını etkiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Personality {
    Playful,
    Calm,
    Curious,
    Sleepy,
    Affectionate,
    Independent,
}

impl Personality {
    pub const ALL: [Personality; 6] = [
        Self::Playful,
        Self::Calm,
        Self::Curious,
        Self::Sleepy,
        Self::Affectionate,
        Self::Independent,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Playful => "playful",
            Self::Calm => "calm",
            Self::Curious => "curious",
            Self::Sleepy => "sleepy",
            Self::Affectionate => "affectionate",
            Self::Independent => "independent",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.as_str() == s)
    }

    /// Saatlik ihtiyaç değişim çarpanları: (tokluk azalması, mutluluk azalması, enerji harcaması).
    pub fn rates(self) -> (f64, f64, f64) {
        match self {
            Self::Playful => (1.15, 1.1, 1.3),
            Self::Calm => (0.9, 0.8, 0.8),
            Self::Curious => (1.0, 1.0, 1.1),
            Self::Sleepy => (0.85, 0.9, 1.2),
            Self::Affectionate => (1.0, 1.35, 1.0),
            Self::Independent => (0.95, 0.55, 0.95),
        }
    }
}

/// 0–100 arası ihtiyaçlar; yüksek değer iyi demektir.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Needs {
    /// Tokluk (düşükse aç)
    pub satiety: f64,
    /// Enerji (etkinlikle azalır, dinlenince dolar)
    pub energy: f64,
    pub happiness: f64,
    /// Uyku (uyanıkken azalır, uyurken dolar)
    pub rest: f64,
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            satiety: 80.0,
            energy: 80.0,
            happiness: 80.0,
            rest: 80.0,
        }
    }
}

/// Bir zaman aralığında kedinin ne yaptığı.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Activity {
    /// Uyanık; `intensity` 0 (oturuyor) – 1 (koşuyor/oynuyor)
    Awake {
        intensity: f64,
    },
    Sleeping,
    /// Uygulama kapalıyken geçen süre: rahatlatıcı varsayım — kedi dinlenmiştir.
    Offline,
}

/// Uygulama kapalıyken ihtiyaçlar bu sınırın altına düşmez (ceza yok, rahat bir deneyim).
pub const OFFLINE_FLOOR: f64 = 35.0;
/// Hesaba katılan en uzun kapalı kalma süresi.
const MAX_OFFLINE_SECS: f64 = 24.0 * 3600.0;

fn clamp(v: f64) -> f64 {
    v.clamp(0.0, 100.0)
}

impl Needs {
    /// İhtiyaçları geçen süreye göre ilerletir.
    pub fn advance(&mut self, secs: f64, activity: Activity, personality: Personality) {
        if secs <= 0.0 || !secs.is_finite() {
            return;
        }
        let (hunger_k, joy_k, energy_k) = personality.rates();
        let h = secs / 3600.0;
        match activity {
            Activity::Awake { intensity } => {
                let i = intensity.clamp(0.0, 1.0);
                self.satiety = clamp(self.satiety - 6.0 * hunger_k * h * (1.0 + i));
                self.happiness = clamp(self.happiness - 3.0 * joy_k * h);
                self.energy = clamp(self.energy - (2.0 + 14.0 * i) * energy_k * h);
                self.rest = clamp(self.rest - 5.0 * h);
            }
            Activity::Sleeping => {
                self.satiety = clamp(self.satiety - 3.0 * hunger_k * h);
                self.energy = clamp(self.energy + 30.0 * h);
                self.rest = clamp(self.rest + 35.0 * h);
            }
            Activity::Offline => {
                let h = secs.min(MAX_OFFLINE_SECS) / 3600.0;
                let floor = |v: f64, d: f64| {
                    if v <= OFFLINE_FLOOR {
                        v
                    } else {
                        (v - d).max(OFFLINE_FLOOR)
                    }
                };
                self.satiety = floor(self.satiety, 3.0 * hunger_k * h);
                self.happiness = floor(self.happiness, 1.5 * joy_k * h);
                self.energy = clamp(self.energy + 20.0 * h);
                self.rest = clamp(self.rest + 25.0 * h);
            }
        }
    }

    pub fn apply(&mut self, d: NeedsDelta) {
        self.satiety = clamp(self.satiety + d.satiety);
        self.energy = clamp(self.energy + d.energy);
        self.happiness = clamp(self.happiness + d.happiness);
        self.rest = clamp(self.rest + d.rest);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct NeedsDelta {
    pub satiety: f64,
    pub energy: f64,
    pub happiness: f64,
    pub rest: f64,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Food {
    pub id: &'static str,
    pub name: &'static str,
    pub satiety: f64,
    pub happiness: f64,
    /// Her gün ücretsiz yenilenen miktar.
    pub daily: i64,
}

pub const FOODS: &[Food] = &[
    Food {
        id: "kibble",
        name: "Kuru mama",
        satiety: 25.0,
        happiness: 2.0,
        daily: 10,
    },
    Food {
        id: "fish",
        name: "Balık",
        satiety: 35.0,
        happiness: 8.0,
        daily: 3,
    },
    Food {
        id: "milk",
        name: "Süt",
        satiety: 15.0,
        happiness: 6.0,
        daily: 5,
    },
    Food {
        id: "treat",
        name: "Ödül maması",
        satiety: 8.0,
        happiness: 15.0,
        daily: 3,
    },
];

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Toy {
    pub id: &'static str,
    pub name: &'static str,
    /// Oynamanın mutluluğa katkısı.
    pub fun: f64,
    pub energy_cost: f64,
    /// Bu oyuncağı açan başarım (yoksa baştan açık).
    pub unlocked_by: Option<&'static str>,
}

pub const TOYS: &[Toy] = &[
    Toy {
        id: "yarn",
        name: "Yün yumağı",
        fun: 14.0,
        energy_cost: 8.0,
        unlocked_by: None,
    },
    Toy {
        id: "ball",
        name: "Küçük top",
        fun: 12.0,
        energy_cost: 10.0,
        unlocked_by: None,
    },
    Toy {
        id: "mouse",
        name: "Oyuncak fare",
        fun: 16.0,
        energy_cost: 9.0,
        unlocked_by: Some("play_10"),
    },
    Toy {
        id: "feather",
        name: "Tüy çubuğu",
        fun: 20.0,
        energy_cost: 12.0,
        unlocked_by: Some("happy_cat"),
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Slot {
    Head,
    Neck,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct AccessoryDef {
    pub id: &'static str,
    pub slot: Slot,
    pub unlocked_by: Option<&'static str>,
}

/// Aksesuar adları/çizimleri sprites.json'dadır; burada yuva ve kilit bilgisi tutulur.
pub const ACCESSORIES: &[AccessoryDef] = &[
    AccessoryDef {
        id: "collar",
        slot: Slot::Neck,
        unlocked_by: None,
    },
    AccessoryDef {
        id: "bow",
        slot: Slot::Head,
        unlocked_by: Some("feed_10"),
    },
    AccessoryDef {
        id: "flower",
        slot: Slot::Head,
        unlocked_by: Some("pet_30"),
    },
    AccessoryDef {
        id: "party_hat",
        slot: Slot::Head,
        unlocked_by: Some("play_10"),
    },
    AccessoryDef {
        id: "scarf",
        slot: Slot::Neck,
        unlocked_by: Some("streak_3"),
    },
    AccessoryDef {
        id: "crown",
        slot: Slot::Head,
        unlocked_by: Some("collector"),
    },
];

pub fn accessory(id: &str) -> Option<&'static AccessoryDef> {
    ACCESSORIES.iter().find(|a| a.id == id)
}

pub const BREEDS: &[&str] = &[
    "pink", "white", "black", "orange", "gray", "tabby", "tuxedo", "cream",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn awake_time_drains_and_sleep_restores() {
        let mut n = Needs::default();
        n.advance(
            3600.0,
            Activity::Awake { intensity: 0.5 },
            Personality::Calm,
        );
        assert!(n.satiety < 80.0 && n.rest < 80.0 && n.energy < 80.0);
        let tired = n;
        n.advance(3600.0, Activity::Sleeping, Personality::Calm);
        assert!(n.energy > tired.energy && n.rest > tired.rest);
    }

    #[test]
    fn personality_changes_rates() {
        let mut lonely = Needs::default();
        let mut alone = Needs::default();
        lonely.advance(
            7200.0,
            Activity::Awake { intensity: 0.2 },
            Personality::Affectionate,
        );
        alone.advance(
            7200.0,
            Activity::Awake { intensity: 0.2 },
            Personality::Independent,
        );
        assert!(
            lonely.happiness < alone.happiness,
            "sevecen kedi yalnız kalınca daha çabuk üzülür"
        );
    }

    #[test]
    fn offline_is_gentle_and_capped() {
        let mut n = Needs {
            satiety: 90.0,
            energy: 20.0,
            happiness: 90.0,
            rest: 20.0,
        };
        n.advance(
            30.0 * 24.0 * 3600.0,
            Activity::Offline,
            Personality::Playful,
        );
        assert!(n.satiety >= OFFLINE_FLOOR && n.happiness >= OFFLINE_FLOOR);
        assert_eq!(n.energy, 100.0);
        // Zaten tabanın altındaysa daha da düşmez.
        let mut low = Needs {
            satiety: 10.0,
            ..Needs::default()
        };
        low.advance(3600.0, Activity::Offline, Personality::Calm);
        assert_eq!(low.satiety, 10.0);
    }

    #[test]
    fn values_stay_in_range() {
        let mut n = Needs {
            satiety: 99.0,
            energy: 99.0,
            happiness: 99.0,
            rest: 99.0,
        };
        n.apply(NeedsDelta {
            satiety: 50.0,
            energy: 50.0,
            happiness: 50.0,
            rest: 50.0,
        });
        assert_eq!(
            n,
            Needs {
                satiety: 100.0,
                energy: 100.0,
                happiness: 100.0,
                rest: 100.0
            }
        );
        n.advance(
            1e9,
            Activity::Awake { intensity: 1.0 },
            Personality::Playful,
        );
        assert!(n.satiety >= 0.0 && n.energy >= 0.0);
        n.advance(-5.0, Activity::Sleeping, Personality::Calm);
        n.advance(f64::NAN, Activity::Sleeping, Personality::Calm);
    }

    #[test]
    fn catalog_ids_are_unique_and_parse() {
        for p in Personality::ALL {
            assert_eq!(Personality::parse(p.as_str()), Some(p));
        }
        let mut ids: Vec<_> = FOODS
            .iter()
            .map(|f| f.id)
            .chain(TOYS.iter().map(|t| t.id))
            .chain(ACCESSORIES.iter().map(|a| a.id))
            .collect();
        let n = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), n);
    }
}
