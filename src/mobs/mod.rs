pub mod plant;
pub mod projectile;
pub mod zombie;

use cabbage::prelude::Error;
use citro2d::prelude::*;

use crate::assets::{load_sheet, place};
use plant::{Plant, PlantKind};
use projectile::{BULLET_SPIN_SPEED, EXPLOSION_FRAMES, Projectile, ProjectileState};
use zombie::{BITE_REACH, ZOMBIE_WIDTH, Zombie, ZombieKind, ZombieState};

pub const ROWS: usize = 5;
pub const COLS: usize = 9;

/// Size of a cell in the simulation's logical pixels. Zombie speeds, bite
/// reach, projectile speeds, etc. are all expressed in these units.
pub const UNIT: f32 = 40.0;
/// Logical width of the lawn.
pub const LAWN_W: f32 = COLS as f32 * UNIT;

pub const CELL_W: f32 = 26.0;
pub const CELL_H: f32 = 32.0;
/// Logical -> screen scale applied when drawing.
const SCALE: f32 = CELL_W / UNIT;

pub const LAWN_X: f32 = 36.0;
pub const LAWN_Y: f32 = 72.0;

const PLANTING_FRAMES: usize = 5;
const PLANTING_FRAME_DELAY: u8 = 3;

const PLANT_REGENERATION_RATE: f32 = 0.1;

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
            Self::NotEnoughSun => "Not enough ikea points!",
        }
    }
}

pub enum RemoveError {
    OutOfBounds,
    NotOccupied,
}

impl RemoveError {
    pub const fn message(self) -> &'static str {
        match self {
            Self::OutOfBounds => "Out of the lawn!",
            Self::NotOccupied => "The cell is empty!",
        }
    }
}

struct PlantingEffect {
    row: usize,
    col: usize,
    frame: u8,
    timer: u8,
}

impl PlantingEffect {
    fn new(row: usize, col: usize) -> Self {
        Self {
            row,
            col,
            frame: 0,
            timer: PLANTING_FRAME_DELAY,
        }
    }

    fn tick(&mut self) -> bool {
        if self.timer > 0 {
            self.timer -= 1;
            return true;
        }

        self.frame += 1;
        if self.frame as usize >= PLANTING_FRAMES {
            return false;
        }
        self.timer = PLANTING_FRAME_DELAY;
        true
    }
}

pub struct MobSprites {
    sunflower: Sprite<'static>,
    peashooter: Sprite<'static>,
    wall: Sprite<'static>,
    bullet: Sprite<'static>,
    explosions: [Sprite<'static>; EXPLOSION_FRAMES],
    planting: [Sprite<'static>; PLANTING_FRAMES],
}

impl MobSprites {
    pub fn load() -> Self {
        let sunflower_sheet = load_sheet("romfs:/hajs/haj-tireur.t3x");
        let peashooter_sheet = load_sheet("romfs:/hajs/haj-tireur-mk2.t3x");
        let wall_sheet = load_sheet("romfs:/hajs/pumpkin-haj.t3x");
        let bullet_sheet = load_sheet("romfs:/bullet.t3x");
        let planting_sheet = load_sheet("romfs:/planting-anim.t3x");

        Self {
            sunflower: Sprite::new(sunflower_sheet, 0, (0., 0.), (1., 1.)),
            peashooter: Sprite::new(peashooter_sheet, 0, (0., 0.), (1., 1.)),
            wall: Sprite::new(wall_sheet, 0, (0., 0.), (1., 1.)),
            bullet: Sprite::new(bullet_sheet, 0, (0., 0.), (1., 1.)),
            explosions: [
                Sprite::new(bullet_sheet, 1, (0., 0.), (1., 1.)),
                Sprite::new(bullet_sheet, 2, (0., 0.), (1., 1.)),
                Sprite::new(bullet_sheet, 3, (0., 0.), (1., 1.)),
            ],
            planting: std::array::from_fn(|index| {
                Sprite::new(planting_sheet, index, (0., 0.), (1., 1.))
            }),
        }
    }

    pub fn plant(&mut self, kind: PlantKind) -> &mut Sprite<'static> {
        match kind {
            PlantKind::Sunflower => &mut self.sunflower,
            PlantKind::Peashooter => &mut self.peashooter,
            PlantKind::Wall => &mut self.wall,
        }
    }
}

pub struct MobManager {
    plants: [[Option<Plant>; COLS]; ROWS],
    zombies: Vec<Zombie>,
    projectiles: Vec<Projectile>,
    planting_effects: Vec<PlantingEffect>,
    sprites: MobSprites,
}

impl MobManager {
    pub fn new() -> Self {
        Self {
            plants: std::array::from_fn(|_| std::array::from_fn(|_| None)),
            zombies: Vec::new(),
            projectiles: Vec::new(),
            planting_effects: Vec::new(),
            sprites: MobSprites::load(),
        }
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
        self.planting_effects.push(PlantingEffect::new(row, col));
        Ok(())
    }

    pub fn remove_plant(&mut self, row: usize, col: usize) -> Result<(), RemoveError> {
        let cell = self
            .plants
            .get_mut(row)
            .and_then(|cells| cells.get_mut(col))
            .ok_or(RemoveError::OutOfBounds)?;

        cell.take().map(|_| ()).ok_or(RemoveError::NotOccupied)
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
        self.update_planting_effects();
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

                //passive hp regeneration for plants
                if plant.hp < plant.kind.stats().hp {
                    plant.hp += PLANT_REGENERATION_RATE;
                }
            }
        }

        self.projectiles.extend(spawned);
    }

    fn update_projectiles(&mut self) {
        let zombies = &mut self.zombies;

        self.projectiles.retain_mut(|p| {
            if !p.is_flying() {
                return p.tick_explosion();
            }

            p.x += p.speed;
            p.spin = (p.spin + BULLET_SPIN_SPEED) % 360.0;
            if p.x > LAWN_W {
                return false;
            }

            let target = zombies
                .iter_mut()
                .filter(|z| z.row == p.row && p.x >= z.x && p.x <= z.x + ZOMBIE_WIDTH)
                .min_by(|a, b| a.x.total_cmp(&b.x));

            if let Some(zombie) = target {
                zombie.hp -= p.damage;
                p.explode();
            }
            true
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

    fn update_planting_effects(&mut self) {
        self.planting_effects.retain_mut(PlantingEffect::tick);
    }

    fn remove_dead(&mut self) {
        self.zombies.retain(|z| !z.is_dead());

        for cell in self.plants.iter_mut().flatten() {
            if cell.as_ref().is_some_and(Plant::is_dead) {
                *cell = None;
            }
        }
    }

    pub fn draw(&mut self) -> Result<(), Error> {
        for (row, plant_row) in self.plants.iter().enumerate() {
            let row_y = LAWN_Y + row as f32 * CELL_H;

            for (col, plant) in plant_row.iter().enumerate() {
                if let Some(plant) = plant {
                    let (x, y) = cell_origin(row, col);
                    draw_plant(plant, x, y, self.sprites.plant(plant.kind))?;
                }
            }

            for zombie in self.zombies.iter().filter(|z| z.row == row) {
                draw_zombie(zombie, LAWN_X + zombie.x * SCALE, row_y)?;
            }

            for projectile in self.projectiles.iter().filter(|p| p.row == row) {
                draw_projectile(
                    projectile,
                    LAWN_X + projectile.x * SCALE,
                    row_y,
                    &mut self.sprites,
                )?;
            }
        }

        for effect in &self.planting_effects {
            let (x, y) = cell_origin(effect.row, effect.col);
            let frame = (effect.frame as usize).min(PLANTING_FRAMES - 1);
            let sprite = &mut self.sprites.planting[frame];
            place(sprite, (x + CELL_W / 2.0, y + CELL_H * 0.75));
            sprite.blit()?;
        }

        Ok(())
    }
}

/// On-screen top-left of a lawn cell.
pub fn cell_origin(row: usize, col: usize) -> (f32, f32) {
    (LAWN_X + col as f32 * CELL_W, LAWN_Y + row as f32 * CELL_H)
}

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
    if hp == max_hp {
        return Ok(());
    }
    let fraction = (hp as f32 / max_hp as f32).clamp(0.0, 1.0);
    fill_rect(x, y, w, 3.0, HP_BAR_BACK)?;
    fill_rect(x, y, w * fraction, 3.0, HP_BAR_FILL)
}

fn draw_plant(plant: &Plant, x: f32, y: f32, sprite: &mut Sprite<'_>) -> Result<(), Error> {
    place(sprite, (x + CELL_W / 2.0, y + CELL_H / 2.0));
    sprite.blit()?;
    draw_hp_bar(
        x + 3.0,
        y + 1.0,
        CELL_W - 6.0,
        plant.hp as i32,
        plant.kind.stats().hp as i32,
    )
}

fn draw_projectile(
    projectile: &Projectile,
    x: f32,
    row_y: f32,
    sprites: &mut MobSprites,
) -> Result<(), Error> {
    let (sprite, rotation) = match projectile.state {
        ProjectileState::Flying => (&mut sprites.bullet, projectile.spin),
        ProjectileState::Exploding { frame, .. } => {
            let frame = (frame as usize).min(EXPLOSION_FRAMES - 1);
            (&mut sprites.explosions[frame], 0.0)
        }
    };

    place(sprite, (x, row_y + CELL_H / 2.0));
    sprite.set_rotation(rotation);
    sprite.blit()?;
    Ok(())
}

fn draw_zombie(zombie: &Zombie, x: f32, row_y: f32) -> Result<(), Error> {
    let color = match zombie.state {
        ZombieState::Walking => 0x7f8c8dff,
        ZombieState::Eating => 0xa05a5aff,
    };
    let w = ZOMBIE_WIDTH * SCALE;
    fill_rect(x, row_y + 8.0 * SCALE, w, CELL_H - 12.0 * SCALE, color)?;
    draw_hp_bar(
        x,
        row_y + 4.0 * SCALE,
        w,
        zombie.hp as i32,
        zombie.kind.stats().hp as i32,
    )
}
