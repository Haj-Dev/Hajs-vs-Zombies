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

/// Size of a cell in the simulation's logical pixels. Zombie speeds, bite
/// reach, projectile speeds, etc. are all expressed in these units.
pub const UNIT: f32 = 40.0;
/// Logical width of the lawn.
pub const LAWN_W: f32 = COLS as f32 * UNIT;

/// On-screen cell size on the 320x240 bottom screen (9 * 35 = 315 px wide).
pub const CELL_W: f32 = 35.0;
pub const CELL_H: f32 = 35.0;
/// Logical -> screen scale applied when drawing.
const SCALE: f32 = CELL_W / UNIT;

/// On-screen top-left of the lawn: centred horizontally, below the shop strip
/// (which ends at y = 40). 48 + 5 * 35 = 223.
pub const LAWN_X: f32 = (320.0 - COLS as f32 * CELL_W) / 2.0;
pub const LAWN_Y: f32 = 48.0;

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

impl MobManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn place_plant(
        &mut self,
        economy: &mut Economy,
        row: usize,
        col: usize,
        kind: PlantKind,
    ) -> Result<(), PlaceError> {
        let cell = self
            .plants
            .get_mut(row)
            .and_then(|cells| cells.get_mut(col))
            .ok_or(PlaceError::OutOfBounds)?;

        if cell.is_some() {
            return Err(PlaceError::Occupied);
        }
        let cost = kind.stats().cost;
        if economy.sun < cost {
            return Err(PlaceError::NotEnoughSun);
        }

        economy.sun -= cost;
        *cell = Some(Plant::new(kind));
        Ok(())
    }

    pub fn spawn_zombie(&mut self, row: usize, kind: ZombieKind) {
        let row = row.min(ROWS - 1);
        self.zombies.push(Zombie::new(kind, row, LAWN_W));
    }

    pub fn zombie_count(&self) -> usize {
        self.zombies.len()
    }

    pub fn update(&mut self, economy: &mut Economy) -> Outcome {
        self.update_plants(economy);
        self.update_projectiles();
        self.update_zombies();
        self.remove_dead();

        if self.zombies.iter().any(|z| z.x <= 0.0) {
            Outcome::Lost
        } else {
            Outcome::Playing
        }
    }

    fn update_plants(&mut self, economy: &mut Economy) {
        let mut spawned = Vec::new();

        for (row, cells) in self.plants.iter_mut().enumerate() {
            for (col, cell) in cells.iter_mut().enumerate() {
                let Some(plant) = cell else { continue };

                plant.timer = plant.timer.saturating_sub(1);
                if plant.timer > 0 {
                    continue;
                }

                let plant_x = col as f32 * UNIT;
                match plant.kind {
                    PlantKind::Peashooter => {
                        let zombie_ahead = self
                            .zombies
                            .iter()
                            .any(|z| z.row == row && z.x < LAWN_W && z.x + ZOMBIE_WIDTH > plant_x);
                        if zombie_ahead {
                            spawned.push(Projectile::pea(row, plant_x + UNIT - 8.0));
                            plant.timer = plant.kind.stats().cooldown;
                        }
                    }
                    PlantKind::Sunflower => {
                        economy.sun += 25;
                        plant.timer = plant.kind.stats().cooldown;
                    }
                    PlantKind::Wall => {}
                }
            }
        }

        self.projectiles.extend(spawned);
    }

    fn update_projectiles(&mut self) {
        let zombies = &mut self.zombies;

        self.projectiles.retain_mut(|p| {
            p.x += p.speed;
            if p.x > LAWN_W {
                return false;
            }

            let target = zombies
                .iter_mut()
                .filter(|z| z.row == p.row && p.x >= z.x && p.x <= z.x + ZOMBIE_WIDTH)
                .min_by(|a, b| a.x.total_cmp(&b.x));

            match target {
                Some(zombie) => {
                    zombie.hp -= p.damage;
                    false
                }
                None => true,
            }
        });
    }

    fn update_zombies(&mut self) {
        for zombie in &mut self.zombies {
            let stats = zombie.kind.stats();

            let probe = zombie.x + BITE_REACH;
            let plant = if (0.0..LAWN_W).contains(&probe) {
                self.plants[zombie.row][(probe / UNIT) as usize].as_mut()
            } else {
                None
            };

            match plant {
                Some(plant) => {
                    zombie.state = ZombieState::Eating;
                    plant.hp -= stats.bite;
                }
                None => {
                    zombie.state = ZombieState::Walking;
                    zombie.x -= stats.speed;
                }
            }
        }
    }

    fn remove_dead(&mut self) {
        self.zombies.retain(|z| !z.is_dead());

        for cell in self.plants.iter_mut().flatten() {
            if cell.as_ref().is_some_and(Plant::is_dead) {
                *cell = None;
            }
        }
    }

    pub fn draw(&self) -> Result<(), Error> {
        for row in 0..ROWS {
            for col in 0..COLS {
                let color = if (row + col) % 2 == 0 {
                    LAWN_LIGHT
                } else {
                    LAWN_DARK
                };
                let (x, y) = cell_origin(row, col);
                fill_rect(x, y, CELL_W, CELL_H, color)?;
            }
        }

        for row in 0..ROWS {
            let row_y = LAWN_Y + row as f32 * CELL_H;

            for (col, plant) in self.plants[row].iter().enumerate() {
                if let Some(plant) = plant {
                    let (x, y) = cell_origin(row, col);
                    draw_plant(plant, x, y)?;
                }
            }

            for zombie in self.zombies.iter().filter(|z| z.row == row) {
                draw_zombie(zombie, LAWN_X + zombie.x * SCALE, row_y)?;
            }

            for projectile in self.projectiles.iter().filter(|p| p.row == row) {
                fill_rect(
                    LAWN_X + (projectile.x - 3.0) * SCALE,
                    row_y + (UNIT / 2.0 - 8.0) * SCALE,
                    6.0 * SCALE,
                    6.0 * SCALE,
                    PEA_COLOR,
                )?;
            }
        }

        Ok(())
    }
}

/// On-screen top-left of a lawn cell.
pub fn cell_origin(row: usize, col: usize) -> (f32, f32) {
    (LAWN_X + col as f32 * CELL_W, LAWN_Y + row as f32 * CELL_H)
}

const LAWN_LIGHT: Rgba = 0x8fd16bff;
const LAWN_DARK: Rgba = 0x7bc257ff;
const PEA_COLOR: Rgba = 0x2e7d32ff;
const HP_BAR_BACK: Rgba = 0x4c4f69ff;
const HP_BAR_FILL: Rgba = 0xd20f39ff;

fn fill_rect(x: f32, y: f32, w: f32, h: f32, color: Rgba) -> Result<(), Error> {
    Rectangle::new(
        BoundingBox::with_top_left((x, y), (w, h)),
        RectangleDrawStyle::default().with_fill(color),
    )
    .blit()?;
    Ok(())
}

fn draw_hp_bar(x: f32, y: f32, w: f32, hp: i32, max_hp: i32) -> Result<(), Error> {
    let fraction = (hp as f32 / max_hp as f32).clamp(0.0, 1.0);
    fill_rect(x, y, w, 3.0, HP_BAR_BACK)?;
    fill_rect(x, y, w * fraction, 3.0, HP_BAR_FILL)
}

fn draw_plant(plant: &Plant, x: f32, y: f32) -> Result<(), Error> {
    let (color, inset) = match plant.kind {
        PlantKind::Sunflower => (0xf9c74fff, 8.0 * SCALE),
        PlantKind::Peashooter => (0x2d9e3cff, 8.0 * SCALE),
        PlantKind::Wall => (0x9c6b3eff, 5.0 * SCALE),
    };
    fill_rect(
        x + inset,
        y + inset,
        CELL_W - inset * 2.0,
        CELL_H - inset * 2.0,
        color,
    )?;
    draw_hp_bar(
        x + 6.0 * SCALE,
        y + 2.0,
        CELL_W - 12.0 * SCALE,
        plant.hp,
        plant.kind.stats().hp,
    )
}

fn draw_zombie(zombie: &Zombie, x: f32, row_y: f32) -> Result<(), Error> {
    let color = match zombie.state {
        ZombieState::Walking => 0x7f8c8dff,
        ZombieState::Eating => 0xa05a5aff,
    };
    let w = ZOMBIE_WIDTH * SCALE;
    fill_rect(x, row_y + 8.0 * SCALE, w, CELL_H - 12.0 * SCALE, color)?;
    draw_hp_bar(x, row_y + 4.0 * SCALE, w, zombie.hp, zombie.kind.stats().hp)
}
