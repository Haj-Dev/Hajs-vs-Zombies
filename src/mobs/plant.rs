#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlantKind {
    Ikehaj,
    HajShooter,
    HajOLantern,
}

pub struct PlantStats {
    pub hp: f32,
    pub cost: u32,
    pub cooldown: u16,
    pub cooldown_reduction_per_level: u16,
    pub max_level: u8,
}

impl PlantKind {
    pub const ALL: [PlantKind; 3] = [Self::HajShooter, Self::Ikehaj, Self::HajOLantern];

    pub const fn stats(self) -> PlantStats {
        match self {
            Self::Ikehaj => PlantStats {
                hp: 300.0,
                cost: 50,
                cooldown: 5 * 60,
                cooldown_reduction_per_level: 30,
                max_level: 5,
            },
            Self::HajShooter => PlantStats {
                hp: 300.0,
                cost: 100,
                cooldown: 90,
                cooldown_reduction_per_level: 9,
                max_level: 5,
            },
            Self::HajOLantern => PlantStats {
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
            Self::Ikehaj => "IKEA Staff",
            Self::HajShooter => "Haj Shooter",
            Self::HajOLantern => "Haj'O-Lantern",
        }
    }
}

pub struct Plant {
    pub kind: PlantKind,
    pub hp: f32,
    pub timer: u16,
    pub level: u8,
}

impl Plant {
    pub fn new(kind: PlantKind) -> Self {
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

    /// Whether this plant has been upgraded at least once.
    pub fn is_evolved(&self) -> bool {
        self.level > 0
    }
}
