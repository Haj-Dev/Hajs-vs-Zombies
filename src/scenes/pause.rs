//! A simple pause menu pushed on top of the level.

use cabbage::{F_BOTTOM_WIDTH, prelude::Error};
use citro2d::prelude::*;
use ctru::{
    prelude::*,
    services::gfx::{BottomScreen, TopScreen},
};

use crate::{
    frame,
    input::Input,
    scene::{Scene, Transition},
};

const BACKGROUND: Rgba = 0xeff1f5ff;
const TEXT_COLOR: Rgba = 0x4c4f69ff;
const ACCENT: Rgba = 0xdf8e1dff;
const BUTTON_FILL: Rgba = 0xccd0daff;
const BUTTON_FILL_SELECTED: Rgba = 0xdf8e1d70;

const ITEMS: [&str; 2] = ["Resume", "Quit to title"];

const BUTTON_W: f32 = 200.;
const BUTTON_H: f32 = 40.;
const BUTTON_GAP: f32 = 20.;
const BUTTON_TOP: f32 = 70.;

/// Top-left corner of the button at `index` on the bottom screen.
fn button_origin(index: usize) -> (f32, f32) {
    let x = (F_BOTTOM_WIDTH - BUTTON_W) / 2.;
    let y = BUTTON_TOP + index as f32 * (BUTTON_H + BUTTON_GAP);
    (x, y)
}

/// Which button, if any, contains the touch point.
fn button_at(touch: (u16, u16)) -> Option<usize> {
    let (tx, ty) = (touch.0 as f32, touch.1 as f32);
    (0..ITEMS.len()).find(|&i| {
        let (x, y) = button_origin(i);
        tx >= x && tx < x + BUTTON_W && ty >= y && ty < y + BUTTON_H
    })
}

pub struct PauseScene {
    selected: usize,
    /// Button the current touch began on, if it began on one.
    press: Option<usize>,
    /// Last position seen while the screen was held.
    last_touch: Option<(u16, u16)>,
}

impl PauseScene {
    pub fn new() -> Self {
        Self {
            selected: 0,
            press: None,
            last_touch: None,
        }
    }

    fn activate(&self, index: usize) -> Transition {
        match index {
            0 => Transition::Pop,
            // Drop the pause menu and the level beneath it.
            _ => Transition::PopMany(2),
        }
    }

    /// Tracks touches; returns a button index when a tap is released on the
    /// same button it began on.
    fn update_touch(&mut self, input: &Input) -> Option<usize> {
        // Only a touch that *begins* in this scene counts, so a finger already
        // down when the menu opened can't trigger anything.
        if let Some(pos) = input.touch_started() {
            self.press = button_at(pos);
            if let Some(index) = self.press {
                self.selected = index; // visual feedback while pressed
            }
        }

        if let Some(pos) = input.touch() {
            self.last_touch = Some(pos);
            return None;
        }

        // Screen not touched: if it was last frame, the touch just ended.
        let released_at = self.last_touch.take()?;
        let pressed = self.press.take()?;
        (button_at(released_at) == Some(pressed)).then_some(pressed)
    }
}

impl Default for PauseScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for PauseScene {
    fn update(&mut self, input: &Input) -> Transition {
        // Touch: a button activates when the tap is released on it.
        if let Some(index) = self.update_touch(input) {
            return self.activate(index);
        }

        let count = ITEMS.len();
        if input.pressed(KeyPad::DPAD_UP) {
            self.selected = (self.selected + count - 1) % count;
        }
        if input.pressed(KeyPad::DPAD_DOWN) {
            self.selected = (self.selected + 1) % count;
        }

        // START or B always resumes.
        if input.pressed(KeyPad::START | KeyPad::B) {
            return Transition::Pop;
        }

        if input.pressed(KeyPad::A) {
            return self.activate(self.selected);
        }

        Transition::None
    }

    fn draw_top<'s>(
        &mut self,
        target: Target<'s, TopScreen, ReadyMode>,
    ) -> Result<Target<'s, TopScreen, ReadyMode>, Error> {
        let mut draw_target = frame::begin_scene(target);
        draw_target.clear(BACKGROUND);

        Text::new(
            "Paused",
            (200., 120.),
            TextDrawStyle::default()
                .with_scalar(1.2)
                .with_color(TEXT_COLOR)
                .with_alignment(HorizAlignment::Center, VertAlignment::Center),
        )
        .blit()?;

        Ok(frame::end_scene(draw_target))
    }

    fn draw_bottom<'s>(
        &mut self,
        target: Target<'s, BottomScreen, ReadyMode>,
    ) -> Result<Target<'s, BottomScreen, ReadyMode>, Error> {
        let mut draw_target = frame::begin_scene(target);
        draw_target.clear(BACKGROUND);

        for (i, &item) in ITEMS.iter().enumerate() {
            let selected = i == self.selected;
            let (x, y) = button_origin(i);

            Rectangle::new(
                BoundingBox::with_top_left((x, y), (BUTTON_W, BUTTON_H)),
                RectangleDrawStyle::default()
                    .with_fill(if selected {
                        BUTTON_FILL_SELECTED
                    } else {
                        BUTTON_FILL
                    })
                    .with_border(if selected { ACCENT } else { TEXT_COLOR }, 2.),
            )
            .blit()?;

            Text::new(
                item,
                (x + BUTTON_W / 2., y + BUTTON_H / 2.),
                TextDrawStyle::default()
                    .with_scalar(0.8)
                    .with_color(TEXT_COLOR)
                    .with_alignment(HorizAlignment::Center, VertAlignment::Center),
            )
            .blit()?;
        }

        Text::new(
            "Tap a button, or D-Pad + A. START: resume",
            (F_BOTTOM_WIDTH / 2., 220.),
            TextDrawStyle::default()
                .with_scalar(0.5)
                .with_color(TEXT_COLOR)
                .with_alignment(HorizAlignment::Center, VertAlignment::Center),
        )
        .blit()?;

        Ok(frame::end_scene(draw_target))
    }
}
