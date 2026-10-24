pub mod plant;
pub mod projectile;
pub mod zombie;

use cabbage::prelude::Error;
use citro2d::prelude::*;

use plant::{Plant, PlantKind};
use projectile::Projectile;
use zombie::{BITE_REACH, ZOMBIE_WIDTH, Zombie, ZombieKind, ZombieState};

pub const ROWS: usize = 5;
pub const COLS: usize = 9;
pub const CELL_W: f32 = 40.0;
pub const CELL_H: f32 = 40.0;
pub const LAWN_W: f32 = COLS as f32 * CELL_W;

pub const LAWN_X: f32 = 20.0;
pub const LAWN_Y: f32 = 30.0;

#[derive(Default)]
pub struct Economy {
    pub sun: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Playing,
    Lost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceError {
    OutOfBounds,
    Occupied,
    NotEnoughSun,
}

impl PlaceError {
    pub const fn message(self) -> &'static str {
        match self {
            Self::OutOfBounds => "Out of the lawn!",
            Self::Occupied => "This cell is taken!",
            Self::NotEnoughSun => "Not enough sun!",
        }
    }
}

#[derive(Default)]
pub struct MobManager {
    plants: [[Option<Plant>; COLS]; ROWS],
    zombies: Vec<Zombie>,
    projectiles: Vec<Projectile>,
}
