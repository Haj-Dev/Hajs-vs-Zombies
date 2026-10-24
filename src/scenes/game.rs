//! The level scene: a lawn, some plants, and zombies coming for them.
//!
//! The lawn and the plant shop live on the touch (bottom) screen; the top
//! screen is a read-only dashboard.

use cabbage::{F_BOTTOM_WIDTH, prelude::Error};
use citro2d::prelude::*;
use ctru::{
    prelude::*,
    services::gfx::{BottomScreen, TopScreen},
};

use crate::{
    frame,
    input::Input,
    mobs::{
        CELL_H, CELL_W, COLS, Economy, MobManager, Outcome, ROWS, cell_origin, plant::PlantKind,
        zombie::ZombieKind,
    },
    scene::{Scene, Transition},
    scenes::end::EndScene,
    scenes::pause::PauseScene,
};

const BACKGROUND: Rgba = 0xeff1f5ff;
const TEXT_COLOR: Rgba = 0x4c4f69ff;
const ACCENT: Rgba = 0xdf8e1dff;
const CURSOR_FILL: Rgba = 0xdf8e1d70;
const LOST_COLOR: Rgba = 0xd20f39ff;
const SHOP_FILL: Rgba = 0xccd0daff;
const SHOP_FILL_PRESSED: Rgba = 0xdf8e1d70;
const SHOP_FILL_DISABLED: Rgba = 0xe6e9efff;
const DISABLED_TEXT: Rgba = 0x9ca0b0ff;

const STARTING_SUN: u32 = 150;
const SKY_SUN_INTERVAL: u32 = 8 * 60;
const SKY_SUN_AMOUNT: u32 = 25;
const FIRST_ZOMBIE_DELAY: u32 = 8 * 60;
const MESSAGE_DURATION: u16 = 90;
const FPS: u32 = 60;

const SHOP_Y: f32 = 2.;
const SHOP_H: f32 = 38.;
const SHOP_MARGIN: f32 = 3.;

fn shop_rect(index: usize) -> (f32, f32, f32, f32) {
    let n = PlantKind::ALL.len() as f32;
    let w = (F_BOTTOM_WIDTH - SHOP_MARGIN * (n + 1.)) / n;
    let x = SHOP_MARGIN + index as f32 * (w + SHOP_MARGIN);
    (x, SHOP_Y, w, SHOP_H)
}

fn in_rect((px, py): (f32, f32), (x, y, w, h): (f32, f32, f32, f32)) -> bool {
    px >= x && px < x + w && py >= y && py < y + h
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TouchTarget {
    Shop(usize),
    Cell(usize, usize),
}

fn target_at(touch: (u16, u16)) -> Option<TouchTarget> {
    let p = (touch.0 as f32, touch.1 as f32);

    if let Some(i) = (0..PlantKind::ALL.len()).find(|&i| in_rect(p, shop_rect(i))) {
        return Some(TouchTarget::Shop(i));
    }

    for row in 0..ROWS {
        for col in 0..COLS {
            let (x, y) = cell_origin(row, col);
            if in_rect(p, (x, y, CELL_W, CELL_H)) {
                return Some(TouchTarget::Cell(row, col));
            }
        }
    }
    None
}

pub struct GameScene {
    mobs: MobManager,
    economy: Economy,
    outcome: Outcome,
    cursor: (usize, usize),
    cursor_visible: bool,
    selected: usize,
    frame: u32,
    next_zombie: u32,
    rng: u32,
    message: Option<(&'static str, u16)>,
    press: Option<TouchTarget>,
    last_touch: Option<(u16, u16)>,
}

impl GameScene {
    pub fn new() -> Self {
        Self {
            mobs: MobManager::new(),
            economy: Economy { sun: STARTING_SUN },
            outcome: Outcome::Playing,
            cursor: (ROWS / 2, 1),
            cursor_visible: false,
            selected: 0,
            frame: 0,
            next_zombie: FIRST_ZOMBIE_DELAY,
            rng: 0x2545_f491,
            message: None,
            press: None,
            last_touch: None,
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

    fn place(&mut self, row: usize, col: usize) {
        let kind = self.selected_kind();
        if let Err(error) = self.mobs.place_plant(&mut self.economy, row, col, kind) {
            self.message = Some((error.message(), MESSAGE_DURATION));
        }
    }

    fn update_touch(&mut self, input: &Input) -> Option<TouchTarget> {
        if let Some(pos) = input.touch_started() {
            self.press = target_at(pos);
            self.cursor_visible = false;
        }

        if let Some(pos) = input.touch() {
            self.last_touch = Some(pos);
            return None;
        }

        // Not touching now: if we were last frame, the touch just ended.
        let released_at = self.last_touch.take()?;
        let pressed = self.press.take()?;
        (target_at(released_at) == Some(pressed)).then_some(pressed)
    }

    fn handle_input(&mut self, input: &Input) {
        match self.update_touch(input) {
            Some(TouchTarget::Shop(i)) => self.selected = i,
            Some(TouchTarget::Cell(row, col)) => self.place(row, col),
            None => {}
        }

        let moved = {
            let (row, col) = &mut self.cursor;
            let before = (*row, *col);
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
            before != (*row, *col)
        };
        if moved {
            self.cursor_visible = true;
        }

        let count = PlantKind::ALL.len();
        if input.pressed(KeyPad::R) {
            self.selected = (self.selected + 1) % count;
        }
        if input.pressed(KeyPad::L) {
            self.selected = (self.selected + count - 1) % count;
        }

        if input.pressed(KeyPad::A) {
            self.cursor_visible = true;
            let (row, col) = self.cursor;
            self.place(row, col);
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

    fn survival_time(&self) -> String {
        let seconds = self.frame / FPS;
        format!("{}:{:02}", seconds / 60, seconds % 60)
    }
}

impl Default for GameScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for GameScene {
    fn update(&mut self, input: &Input) -> Transition {
        if self.outcome == Outcome::Lost {
            let leave = input.pressed(KeyPad::A | KeyPad::B | KeyPad::START)
                || input.touch_started().is_some();
            return if leave {
                Transition::Pop
            } else {
                Transition::None
            };
        }

        if input.pressed(KeyPad::START) {
            // Forget any touch in progress so it can't fire after resuming.
            self.press = None;
            self.last_touch = None;
            return Transition::Push(Box::new(PauseScene::new()));
        }

        self.frame += 1;
        self.handle_input(input);

        if self.frame.is_multiple_of(SKY_SUN_INTERVAL) {
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
        let mut draw_target = frame::begin_scene(target);
        draw_target.clear(BACKGROUND);

        let line = |text: &str, y: f32, scalar: f32, color: Rgba| {
            Text::new(
                text,
                (200., y),
                TextDrawStyle::default()
                    .with_scalar(scalar)
                    .with_color(color)
                    .with_alignment(HorizAlignment::Center, VertAlignment::Center),
            )
            .blit()
        };

        line("IKEA Points", 22., 0.7, TEXT_COLOR)?;
        line(&self.economy.sun.to_string(), 55., 1.4, ACCENT)?;

        let kind = self.selected_kind();
        let stats = kind.stats();
        line(kind.name(), 100., 0.9, TEXT_COLOR)?;
        line(
            &format!(
                "Cost: {}, Health: {}, Cooldown: {}",
                stats.cost, stats.hp, stats.cooldown
            ),
            125.,
            0.7,
            TEXT_COLOR,
        )?;
        // Add more stat lines here (health, damage, ...) once you want them.

        line(
            &format!("Survived: {}", self.survival_time()),
            165.,
            0.8,
            TEXT_COLOR,
        )?;
        line(
            &format!("Zombies on the lawn: {}", self.mobs.zombie_count()),
            190.,
            0.7,
            TEXT_COLOR,
        )?;

        if let Some((message, _)) = self.message {
            line(message, 220., 0.8, LOST_COLOR)?;
        }

        Ok(frame::end_scene(draw_target))
    }

    /// Touch screen: shop strip on top, lawn below.
    fn draw_bottom<'s>(
        &mut self,
        target: Target<'s, BottomScreen, ReadyMode>,
    ) -> Result<Target<'s, BottomScreen, ReadyMode>, Error> {
        let mut draw_target = frame::begin_scene(target);
        draw_target.clear(BACKGROUND);

        // Shop strip.
        for (i, kind) in PlantKind::ALL.iter().enumerate() {
            let (x, y, w, h) = shop_rect(i);
            let cost = kind.stats().cost;
            let affordable = cost <= self.economy.sun;
            let selected = i == self.selected;
            let pressed = self.press == Some(TouchTarget::Shop(i));

            let fill = if !affordable {
                SHOP_FILL_DISABLED
            } else if pressed {
                SHOP_FILL_PRESSED
            } else {
                SHOP_FILL
            };
            let text_color = if affordable {
                TEXT_COLOR
            } else {
                DISABLED_TEXT
            };

            Rectangle::new(
                BoundingBox::with_top_left((x, y), (w, h)),
                RectangleDrawStyle::default()
                    .with_fill(fill)
                    .with_border(if selected { ACCENT } else { text_color }, 2.),
            )
            .blit()?;

            for (text, dy) in [
                (kind.name().to_string(), h * 0.32),
                (cost.to_string(), h * 0.72),
            ] {
                Text::new(
                    &text,
                    (x + w / 2., y + dy),
                    TextDrawStyle::default()
                        .with_scalar(0.45)
                        .with_color(text_color)
                        .with_alignment(HorizAlignment::Center, VertAlignment::Center),
                )
                .blit()?;
            }
        }

        // Lawn, plants and zombies.
        self.mobs.draw()?;

        // Highlight the pressed cell, or the D-pad cursor.
        let highlight = match self.press {
            Some(TouchTarget::Cell(row, col)) => Some((row, col)),
            _ if self.cursor_visible => Some(self.cursor),
            _ => None,
        };
        if let Some((row, col)) = highlight {
            let (x, y) = cell_origin(row, col);
            Rectangle::new(
                BoundingBox::with_top_left((x, y), (CELL_W, CELL_H)),
                RectangleDrawStyle::default()
                    .with_fill(CURSOR_FILL)
                    .with_border(ACCENT, 2.),
            )
            .blit()?;
        }

        Ok(frame::end_scene(draw_target))
    }
}
