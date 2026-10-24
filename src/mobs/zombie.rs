pub const ZOMBIE_WIDTH: f32 = 22.0;
pub const BITE_REACH: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZombieKind {
    Basic,
}

pub struct ZombieStats {
    pub hp: i32,
    pub speed: f32,
    pub bite: i32,
}

impl ZombieKind {
    pub const fn stats(self) -> ZombieStats {
        match self {
            Self::Basic => ZombieStats {
                hp: 200,
                speed: 0.25,
                bite: 1,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZombieState {
    Walking,
    Eating,
}

pub struct Zombie {
    pub kind: ZombieKind,
    pub row: usize,
    pub x: f32,
    pub hp: i32,
    pub state: ZombieState,
}

impl Zombie {
    pub fn new(kind: ZombieKind, row: usize, x: f32) -> Self {
        Self {
            kind,
            row,
            x,
            hp: kind.stats().hp,
            state: ZombieState::Walking,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }
}
