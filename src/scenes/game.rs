//! The level scene: a lawn, some plants, and zombies coming for them.

use cabbage::{F_BOTTOM_WIDTH, prelude::Error};
use citro2d::prelude::*;
use ctru::{
    prelude::*,
    services::gfx::{BottomScreen, TopScreen},
};

use crate::{
    input::Input,
    mobs::{
        CELL_H, CELL_W, COLS, Economy, MobManager, Outcome, ROWS, cell_origin, plant::PlantKind,
        zombie::ZombieKind,
    },
    scene::{Scene, Transition},
    scenes::end::EndScene,
};

const BACKGROUND: Rgba = 0xeff1f5ff;
const TEXT_COLOR: Rgba = 0x4c4f69ff;
const ACCENT: Rgba = 0xdf8e1dff;
const CURSOR_FILL: Rgba = 0xdf8e1d70;

const STARTING_SUN: u32 = 150;
const SKY_SUN_INTERVAL: u32 = 8 * 60;
const SKY_SUN_AMOUNT: u32 = 25;
const FIRST_ZOMBIE_DELAY: u32 = 8 * 60;
const MESSAGE_DURATION: u16 = 90;

pub struct GameScene {
    mobs: MobManager,
    economy: Economy,
    outcome: Outcome,
    cursor: (usize, usize),
    selected: usize,
    frame: u32,
    next_zombie: u32,
    rng: u32,
    message: Option<(&'static str, u16)>,
}

impl GameScene {
    pub fn new() -> Self {
        Self {
            mobs: MobManager::new(),
            economy: Economy { sun: STARTING_SUN },
            outcome: Outcome::Playing,
            cursor: (ROWS / 2, 1),
            selected: 0,
            frame: 0,
            next_zombie: FIRST_ZOMBIE_DELAY,
            rng: 0x2545_f491,
            message: None,
        }
    }

    fn selected_kind(&self) -> PlantKind {
        PlantKind::ALL[self.selected]
    }

    fn next_random(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng
    }

    fn handle_input(&mut self, input: &Input) {
        let (row, col) = &mut self.cursor;
        if input.pressed(KeyPad::DPAD_UP) {
            *row = row.saturating_sub(1);
        }
        if input.pressed(KeyPad::DPAD_DOWN) {
            *row = (*row + 1).min(ROWS - 1);
        }
        if input.pressed(KeyPad::DPAD_LEFT) {
            *col = col.saturating_sub(1);
        }
        if input.pressed(KeyPad::DPAD_RIGHT) {
            *col = (*col + 1).min(COLS - 1);
        }

        let count = PlantKind::ALL.len();
        if input.pressed(KeyPad::R) {
            self.selected = (self.selected + 1) % count;
        }
        if input.pressed(KeyPad::L) {
            self.selected = (self.selected + count - 1) % count;
        }

        if input.pressed(KeyPad::A) {
            let (row, col) = self.cursor;
            let kind = self.selected_kind();
            if let Err(error) = self.mobs.place_plant(&mut self.economy, row, col, kind) {
                self.message = Some((error.message(), MESSAGE_DURATION));
            }
        }
    }

    fn update_waves(&mut self) {
        if self.frame < self.next_zombie {
            return;
        }

        let row = self.next_random() as usize % ROWS;
        self.mobs.spawn_zombie(row, ZombieKind::Basic);

        let interval = 480u32.saturating_sub(self.frame / 15).max(150);
        self.next_zombie = self.frame + interval;
    }
}

impl Default for GameScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for GameScene {
    fn update(&mut self, input: &Input) -> Transition {
        if input.pressed(KeyPad::B) {
            return Transition::Pop;
        }

        self.frame += 1;
        self.handle_input(input);

        if self.frame % SKY_SUN_INTERVAL == 0 {
            self.economy.sun += SKY_SUN_AMOUNT;
        }
        self.update_waves();
        self.outcome = self.mobs.update(&mut self.economy);
        if self.outcome == Outcome::Lost {
            return Transition::Push(Box::new(EndScene::new()));
        }

        self.message = match self.message {
            Some((text, frames)) if frames > 1 => Some((text, frames - 1)),
            _ => None,
        };

        Transition::None
    }

    fn draw_top<'s>(
        &mut self,
        target: Target<'s, TopScreen, ReadyMode>,
    ) -> Result<Target<'s, TopScreen, ReadyMode>, Error> {
        let mut draw_target = target.begin();
        draw_target.clear(BACKGROUND);

        let kind = self.selected_kind();
        let hud = format!(
            "Sun: {}    {} ({})",
            self.economy.sun,
            kind.name(),
            kind.stats().cost
        );
        Text::new(
            &hud,
            (200., 14.),
            TextDrawStyle::default()
                .with_scalar(0.7)
                .with_color(TEXT_COLOR)
                .with_alignment(HorizAlignment::Center, VertAlignment::Center),
        )
        .blit()?;

        self.mobs.draw()?;

        let (x, y) = cell_origin(self.cursor.0, self.cursor.1);
        Rectangle::new(
            BoundingBox::with_top_left((x, y), (CELL_W, CELL_H)),
            RectangleDrawStyle::default()
                .with_fill(CURSOR_FILL)
                .with_border(ACCENT, 2.),
        )
        .blit()?;

        Ok(draw_target.end())
    }

    fn draw_bottom<'s>(
        &mut self,
        target: Target<'s, BottomScreen, ReadyMode>,
    ) -> Result<Target<'s, BottomScreen, ReadyMode>, Error> {
        let mut draw_target = target.begin();
        draw_target.clear(BACKGROUND);

        let center = F_BOTTOM_WIDTH / 2.;
        let line = |text: &str, y: f32, scalar: f32, color: Rgba| {
            Text::new(
                text,
                (center, y),
                TextDrawStyle::default()
                    .with_scalar(scalar)
                    .with_color(color)
                    .with_alignment(HorizAlignment::Center, VertAlignment::Center),
            )
            .blit()
        };

        line(
            &format!("Zombies on the lawn: {}", self.mobs.zombie_count()),
            40.,
            0.7,
            TEXT_COLOR,
        )?;
        line("D-Pad: move cursor", 100., 0.65, TEXT_COLOR)?;
        line("A: plant    L / R: change plant", 125., 0.65, TEXT_COLOR)?;
        line("B: back to the title", 150., 0.65, TEXT_COLOR)?;

        if let Some((message, _)) = self.message {
            line(message, 200., 0.8, TEXT_COLOR)?;
        }

        Ok(draw_target.end())
    }
}
