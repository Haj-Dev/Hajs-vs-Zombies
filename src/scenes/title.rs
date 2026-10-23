//! The title screen.

use cabbage::{F_BOTTOM_WIDTH, prelude::Error};
use citro2d::prelude::*;
use ctru::{
    prelude::*,
    services::gfx::{BottomScreen, TopScreen},
};

use crate::{
    input::Input,
    scene::{Scene, Transition},
    scenes::game::GameScene,
};

const BACKGROUND: Rgba = 0xeff1f5ff;
const TEXT_COLOR: Rgba = 0x4c4f69ff;
const ACCENT: Rgba = 0xdf8e1dff;
const ACCENT_ALT: Rgba = 0xf9e2afff;

/// Frames between each blink toggle, at 60 fps
const BLINK_INTERVAL: u32 = 15;

/// The area of the bottom screen that starts the game when tapped.
const START_BUTTON: BoundingBox = BoundingBox::with_top_left((60., 80.), (200., 80.));

/// The game's title screen.
#[derive(Default)]
pub struct TitleScene {
    /// Frames elapsed, used to blink the prompt.
    frame: u32,
    /// Whether the start button is currently being touched.
    button_touched: bool,
}

impl TitleScene {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Scene for TitleScene {
    fn update(&mut self, input: &Input) -> Transition {
        self.frame = self.frame.wrapping_add(1);

        self.button_touched = input
            .touch()
            .is_some_and(|(x, y)| START_BUTTON.intersects_point((x as f32, y as f32)));

        if input.pressed(KeyPad::A) || input.pressed(KeyPad::START) || self.button_touched {
            Transition::Push(Box::new(GameScene::new()))
        } else if input.pressed(KeyPad::SELECT) {
            Transition::Quit
        } else {
            Transition::None
        }
    }

    fn draw_top<'s>(
        &mut self,
        target: Target<'s, TopScreen, ReadyMode>,
    ) -> Result<Target<'s, TopScreen, ReadyMode>, Error> {
        let mut draw_target = target.begin();
        draw_target.clear(BACKGROUND);

        Text::new(
            "HAJS VS ZOMBIES",
            (200., 88.),
            TextDrawStyle::default()
                .with_scalar(1.3)
                .with_color(ACCENT)
                .with_alignment(HorizAlignment::Center, VertAlignment::Center),
        )
        .blit()?;

        Text::new(
            "a tower defense game",
            (200., 132.),
            TextDrawStyle::default()
                .with_scalar(0.7)
                .with_color(TEXT_COLOR)
                .with_alignment(HorizAlignment::Center, VertAlignment::Center),
        )
        .blit()?;

        // Blinking prompt.
        if (self.frame / BLINK_INTERVAL) % 2 == 0 {
            Text::new(
                "Press A to start",
                (200., 195.),
                TextDrawStyle::default()
                    .with_scalar(0.8)
                    .with_color(TEXT_COLOR)
                    .with_alignment(HorizAlignment::Center, VertAlignment::Center),
            )
            .blit()?;
        }

        Ok(draw_target.end())
    }

    fn draw_bottom<'s>(
        &mut self,
        target: Target<'s, BottomScreen, ReadyMode>,
    ) -> Result<Target<'s, BottomScreen, ReadyMode>, Error> {
        let mut draw_target = target.begin();
        draw_target.clear(BACKGROUND);

        let fill = if self.button_touched {
            ACCENT_ALT
        } else {
            ACCENT
        };
        Rectangle::new(
            START_BUTTON,
            RectangleDrawStyle::default()
                .with_fill(fill)
                .with_border(TEXT_COLOR, 2.),
        )
        .blit()?;

        Text::new(
            "START",
            START_BUTTON.center(),
            TextDrawStyle::default()
                .with_scalar(1.2)
                .with_color(TEXT_COLOR)
                .with_alignment(HorizAlignment::Center, VertAlignment::Center),
        )
        .blit()?;

        Text::new(
            "or press A / START",
            (F_BOTTOM_WIDTH / 2., 180.),
            TextDrawStyle::default()
                .with_scalar(0.65)
                .with_color(TEXT_COLOR)
                .with_alignment(HorizAlignment::Center, VertAlignment::Center),
        )
        .blit()?;

        Text::new(
            "SELECT to quit",
            (F_BOTTOM_WIDTH / 2., 205.),
            TextDrawStyle::default()
                .with_scalar(0.6)
                .with_color(TEXT_COLOR)
                .with_alignment(HorizAlignment::Center, VertAlignment::Center),
        )
        .blit()?;

        Ok(draw_target.end())
    }
}
