pub mod haj;
pub mod projectile;
pub mod zombie;

use cabbage::prelude::Error;
use citro2d::prelude::*;

use crate::assets::{load_sheet, place};
use haj::{Haj, HajKind};
use projectile::{BULLET_SPIN_SPEED, EXPLOSION_FRAMES, Projectile, ProjectileState};
use zombie::{
    ATTACK_FRAMES, BITE_REACH, DIE_FRAMES, WALK_FRAMES, ZOMBIE_WIDTH, Zombie, ZombieKind,
    ZombieState,
};

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

const HAJING_FRAMES: usize = 5;
const HAJING_FRAME_DELAY: u8 = 3;

const HAJ_REGENERATION_RATE: f32 = 0.1;

const WALK_ANCHOR: (f32, f32) = (21.0, 52.0);
const ATTACK_ANCHOR: (f32, f32) = (20.0, 49.0);
const DIE_ANCHOR: (f32, f32) = (33.5, 29.0);

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
    MaxLevelReached,
}

impl PlaceError {
    pub const fn message(self) -> &'static str {
        match self {
            Self::OutOfBounds => "Out of the lawn!",
            Self::Occupied => "This cell is taken!",
            Self::NotEnoughSun => "Not enough ikea points!",
            Self::MaxLevelReached => "Haj is already at max level!",
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

struct HajingEffect {
    row: usize,
    col: usize,
    frame: u8,
    timer: u8,
}

impl HajingEffect {
    fn new(row: usize, col: usize) -> Self {
        Self {
            row,
            col,
            frame: 0,
            timer: HAJING_FRAME_DELAY,
        }
    }

    fn tick(&mut self) -> bool {
        if self.timer > 0 {
            self.timer -= 1;
            return true;
        }

        self.frame += 1;
        if self.frame as usize >= HAJING_FRAMES {
            return false;
        }
        self.timer = HAJING_FRAME_DELAY;
        true
    }
}

pub struct MobSprites {
    ikehaj: Sprite<'static>,
    hajshooter: Sprite<'static>,
    hajshooter_evolved: Sprite<'static>,
    hajolantern: Sprite<'static>,
    bullet: Sprite<'static>,
    explosions: [Sprite<'static>; EXPLOSION_FRAMES],
    hajing: [Sprite<'static>; HAJING_FRAMES],
    zombie_walk: [Sprite<'static>; WALK_FRAMES],
    zombie_attack: [Sprite<'static>; ATTACK_FRAMES],
    zombie_die: [Sprite<'static>; DIE_FRAMES],
}

impl MobSprites {
    pub fn load() -> Self {
        let ikehaj_sheet = load_sheet("romfs:/hajs/ikehaj.t3x");
        let hajshooter_sheet = load_sheet("romfs:/hajs/haj-tireur.t3x");
        let hajshooter_evolved_sheet = load_sheet("romfs:/hajs/haj-tireur-mk2.t3x");
        let hajolantern_sheet = load_sheet("romfs:/hajs/pumpkin-haj.t3x");
        let bullet_sheet = load_sheet("romfs:/bullet.t3x");
        let hajing_sheet = load_sheet("romfs:/hajing-anim.t3x");
        let zombie_sheet = load_sheet("romfs:/zombies.t3x");

        Self {
            ikehaj: Sprite::new(ikehaj_sheet, 0, (0., 0.), (1., 1.)),
            hajshooter: Sprite::new(hajshooter_sheet, 0, (0., 0.), (1., 1.)),
            hajshooter_evolved: Sprite::new(hajshooter_evolved_sheet, 0, (0., 0.), (1., 1.)),
            hajolantern: Sprite::new(hajolantern_sheet, 0, (0., 0.), (1., 1.)),
            bullet: Sprite::new(bullet_sheet, 0, (0., 0.), (1., 1.)),
            explosions: [
                Sprite::new(bullet_sheet, 1, (0., 0.), (1., 1.)),
                Sprite::new(bullet_sheet, 2, (0., 0.), (1., 1.)),
                Sprite::new(bullet_sheet, 3, (0., 0.), (1., 1.)),
            ],
            hajing: std::array::from_fn(|index| {
                Sprite::new(hajing_sheet, index, (0., 0.), (1., 1.))
            }),
            zombie_walk: std::array::from_fn(|index| {
                Sprite::new(zombie_sheet, index, (0., 0.), (1., 1.))
            }),
            zombie_attack: std::array::from_fn(|index| {
                Sprite::new(
                    zombie_sheet,
                    WALK_FRAMES + index,
                    (0., 0.),
                    (1., 1.),
                )
            }),
            zombie_die: std::array::from_fn(|index| {
                Sprite::new(
                    zombie_sheet,
                    WALK_FRAMES + ATTACK_FRAMES + index,
                    (0., 0.),
                    (1., 1.),
                )
            }),
        }
    }

    pub fn haj(&mut self, kind: HajKind, evolved: bool) -> &mut Sprite<'static> {
        match kind {
            HajKind::Mhaj if evolved => &mut self.hajshooter_evolved,
            HajKind::Mhaj => &mut self.hajshooter,
            HajKind::Ikehaj => &mut self.ikehaj,
            HajKind::HajOLantern => &mut self.hajolantern,
        }
    }
}

pub struct MobManager {
    hajs: [[Option<Haj>; COLS]; ROWS],
    zombies: Vec<Zombie>,
    projectiles: Vec<Projectile>,
    hajing_effects: Vec<HajingEffect>,
    sprites: MobSprites,
}

impl MobManager {
    pub fn new() -> Self {
        Self {
            hajs: std::array::from_fn(|_| std::array::from_fn(|_| None)),
            zombies: Vec::new(),
            projectiles: Vec::new(),
            hajing_effects: Vec::new(),
            sprites: MobSprites::load(),
        }
    }

    pub fn haj_at(&self, row: usize, col: usize) -> Option<&Haj> {
        self.hajs
            .get(row)
            .and_then(|cells| cells.get(col))
            .and_then(|cell| cell.as_ref())
    }

    pub fn place_haj(
        &mut self,
        economy: &mut Economy,
        row: usize,
        col: usize,
        kind: HajKind,
    ) -> Result<(), PlaceError> {
        let cell = self
            .hajs
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
        *cell = Some(Haj::new(kind));
        self.hajing_effects.push(HajingEffect::new(row, col));
        Ok(())
    }

    pub fn upgrade_haj(
        &mut self,
        economy: &mut Economy,
        row: usize,
        col: usize,
    ) -> Result<(), PlaceError> {
        let cell = self
            .hajs
            .get_mut(row)
            .and_then(|cells| cells.get_mut(col))
            .ok_or(PlaceError::OutOfBounds)?;

        let Some(haj) = cell.as_mut() else {
            return Err(PlaceError::Occupied);
        };

        let stats = haj.kind.stats();
        if haj.level >= stats.max_level - 1 {
            return Err(PlaceError::MaxLevelReached);
        }

        let upgrade_cost = stats.cost;
        if economy.sun < upgrade_cost {
            return Err(PlaceError::NotEnoughSun);
        }

        economy.sun -= upgrade_cost;
        haj.level = (haj.level + 1).min(stats.max_level);
        self.hajing_effects.push(HajingEffect::new(row, col));
        Ok(())
    }

    pub fn remove_haj(&mut self, row: usize, col: usize) -> Result<(), RemoveError> {
        let cell = self
            .hajs
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
        self.update_hajs(economy);
        self.update_projectiles();
        self.update_zombies();
        self.update_hajing_effects();
        self.remove_dead();

        if self
            .zombies
            .iter()
            .any(|z| z.state != ZombieState::Dying && z.x <= 0.0)
        {
            Outcome::Lost
        } else {
            Outcome::Playing
        }
    }

    fn update_hajs(&mut self, economy: &mut Economy) {
        let mut spawned = Vec::new();

        for (row, cells) in self.hajs.iter_mut().enumerate() {
            for (col, cell) in cells.iter_mut().enumerate() {
                let Some(haj) = cell else { continue };

                haj.timer = haj.timer.saturating_sub(1);
                if haj.timer > 0 {
                    continue;
                }

                let haj_x = col as f32 * UNIT;
                match haj.kind {
                    HajKind::Mhaj => {
                        let zombie_ahead = self.zombies.iter().any(|z| {
                            z.state != ZombieState::Dying
                                && z.row == row
                                && z.x < LAWN_W
                                && z.x + ZOMBIE_WIDTH > haj_x
                        });
                        if zombie_ahead {
                            spawned.push(Projectile::pea(row, haj_x + UNIT - 8.0));
                            haj.timer = haj.kind.stats().cooldown
                                - haj.level as u16 * haj.kind.stats().cooldown_reduction_per_level;
                        }
                    }
                    HajKind::Ikehaj => {
                        economy.sun += 25;
                        haj.timer = haj.kind.stats().cooldown
                            - haj.level as u16 * haj.kind.stats().cooldown_reduction_per_level;
                    }
                    HajKind::HajOLantern => {}
                }

                //passive hp regeneration for hajs
                if haj.hp < haj.kind.stats().hp {
                    match haj.kind {
                        HajKind::HajOLantern => {
                            haj.hp += HAJ_REGENERATION_RATE * haj.level as f32;
                        }
                        _ => {
                            haj.hp += HAJ_REGENERATION_RATE;
                        }
                    }
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
                .filter(|z| {
                    z.state != ZombieState::Dying
                        && z.hp > 0.0
                        && z.row == p.row
                        && p.x >= z.x
                        && p.x <= z.x + ZOMBIE_WIDTH
                })
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
            if zombie.is_dead() {
                zombie.start_dying();
                zombie.update_animation();
                continue;
            }

            let stats = zombie.kind.stats();

            let probe = zombie.x + BITE_REACH;
            let haj = if (0.0..LAWN_W).contains(&probe) {
                self.hajs[zombie.row][(probe / UNIT) as usize].as_mut()
            } else {
                None
            };

            match haj {
                Some(haj) => {
                    zombie.set_state(ZombieState::Eating);
                    haj.hp -= stats.bite;
                }
                None => {
                    zombie.set_state(ZombieState::Walking);
                    zombie.x -= stats.speed;
                }
            }
            zombie.update_animation();
        }
    }

    fn update_hajing_effects(&mut self) {
        self.hajing_effects.retain_mut(HajingEffect::tick);
    }

    fn remove_dead(&mut self) {
        self.zombies.retain(|z| !z.is_finished_dying());

        for cell in self.hajs.iter_mut().flatten() {
            if cell.as_ref().is_some_and(Haj::is_dead) {
                *cell = None;
            }
        }
    }

    pub fn draw(&mut self) -> Result<(), Error> {
        for (row, haj_row) in self.hajs.iter().enumerate() {
            let row_y = LAWN_Y + row as f32 * CELL_H;

            for (col, haj) in haj_row.iter().enumerate() {
                if let Some(haj) = haj {
                    let (x, y) = cell_origin(row, col);
                    draw_haj(haj, x, y, self.sprites.haj(haj.kind, haj.is_evolved()))?;
                }
            }

            for zombie in self.zombies.iter().filter(|z| z.row == row) {
                draw_zombie(zombie, LAWN_X + zombie.x * SCALE, row_y, &mut self.sprites)?;
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

        for effect in &self.hajing_effects {
            let (x, y) = cell_origin(effect.row, effect.col);
            let frame = (effect.frame as usize).min(HAJING_FRAMES - 1);
            let sprite = &mut self.sprites.hajing[frame];
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

fn draw_haj(haj: &Haj, x: f32, y: f32, sprite: &mut Sprite<'_>) -> Result<(), Error> {
    place(sprite, (x + CELL_W / 2.0, y + CELL_H / 2.0));
    sprite.blit()?;
    draw_hp_bar(
        x + 3.0,
        y + 1.0,
        CELL_W - 6.0,
        haj.hp as i32,
        haj.kind.stats().hp as i32,
    )?;
    Text::new(
        &format!("{}", haj.level + 1),
        (x, y + CELL_H),
        TextDrawStyle::default()
            .with_scalar(0.5)
            .with_color(0x000000ff)
            .with_alignment(HorizAlignment::Left, VertAlignment::Baseline),
    )
    .blit()?;
    Ok(())
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

fn draw_zombie(zombie: &Zombie, x: f32, row_y: f32, sprites: &mut MobSprites) -> Result<(), Error> {
    let (sprite, anchor) = match zombie.state {
        ZombieState::Walking => (
            &mut sprites.zombie_walk[zombie.animation_frame()],
            WALK_ANCHOR,
        ),
        ZombieState::Eating => (
            &mut sprites.zombie_attack[zombie.animation_frame()],
            ATTACK_ANCHOR,
        ),
        ZombieState::Dying => (
            &mut sprites.zombie_die[zombie.animation_frame()],
            DIE_ANCHOR,
        ),
    };

    let foot_x = x + ZOMBIE_WIDTH * SCALE / 2.0;
    let foot_y = row_y + CELL_H - 2.0;
    let size = sprite.bounding_box.size;
    let scale = sprite.scalar.0;
    place(
        sprite,
        (
            foot_x + size.0 / 2.0 - anchor.0 * scale,
            foot_y + size.1 / 2.0 - anchor.1 * scale,
        ),
    );
    sprite.blit()?;

    if zombie.state != ZombieState::Dying {
        draw_hp_bar(
            x,
            row_y + 4.0 * SCALE,
            ZOMBIE_WIDTH * SCALE,
            zombie.hp as i32,
            zombie.kind.stats().hp as i32,
        )?;
    }
    Ok(())
}
