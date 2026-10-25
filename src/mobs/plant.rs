#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlantKind {
    Sunflower,
    Peashooter,
    Wall,
}

pub struct PlantStats {
    pub hp: f32,
    pub cost: u32,
    pub cooldown: u16,
}

impl PlantKind {
    pub const ALL: [PlantKind; 3] = [Self::Peashooter, Self::Sunflower, Self::Wall];

    pub const fn stats(self) -> PlantStats {
        match self {
            Self::Sunflower => PlantStats {
                hp: 300.0,
                cost: 50,
                cooldown: 5 * 60,
            },
            Self::Peashooter => PlantStats {
                hp: 300.0,
                cost: 100,
                cooldown: 90,
            },
            Self::Wall => PlantStats {
                hp: 1200.0,
                cost: 50,
                cooldown: 0,
            },
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Sunflower => "Sunflower",
            Self::Peashooter => "Peashooter",
            Self::Wall => "Wall",
        }
    }
}

pub struct Plant {
    pub kind: PlantKind,
    pub hp: f32,
    pub timer: u16,
}

impl Plant {
    pub fn new(kind: PlantKind) -> Self {
        let stats = kind.stats();
        Self {
            kind,
            hp: stats.hp,
            timer: stats.cooldown,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }
}
