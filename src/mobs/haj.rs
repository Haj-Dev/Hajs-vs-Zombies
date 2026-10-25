#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HajKind {
    Ikehaj,
    Mhaj,
    HajOLantern,
}

pub struct HajStats {
    pub hp: f32,
    pub cost: u32,
    pub cooldown: u16,
    pub cooldown_reduction_per_level: u16,
    pub max_level: u8,
}

impl HajKind {
    pub const ALL: [HajKind; 3] = [Self::Mhaj, Self::Ikehaj, Self::HajOLantern];

    pub const fn stats(self) -> HajStats {
        match self {
            Self::Ikehaj => HajStats {
                hp: 300.0,
                cost: 50,
                cooldown: 5 * 60,
                cooldown_reduction_per_level: 30,
                max_level: 5,
            },
            Self::Mhaj => HajStats {
                hp: 300.0,
                cost: 100,
                cooldown: 90,
                cooldown_reduction_per_level: 9,
                max_level: 5,
            },
            Self::HajOLantern => HajStats {
                hp: 1200.0,
                cost: 50,
                cooldown: 0,
                cooldown_reduction_per_level: 0,
                max_level: 5,
            },
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Ikehaj => "Ikehaj",
            Self::Mhaj => "Mhaj",
            Self::HajOLantern => "Haj'O-Lantern",
        }
    }
}

pub struct Haj {
    pub kind: HajKind,
    pub hp: f32,
    pub timer: u16,
    pub level: u8,
}

impl Haj {
    pub fn new(kind: HajKind) -> Self {
        let stats = kind.stats();
        Self {
            kind,
            hp: stats.hp,
            timer: stats.cooldown,
            level: 0,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }

    /// Whether this haj has been upgraded at least once.
    pub fn is_evolved(&self) -> bool {
        self.level > 0
    }
}
