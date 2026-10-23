//! Placeholder for the actual gameplay scene.

use cabbage::prelude::Error;
use citro2d::prelude::*;
use ctru::{
    prelude::*,
    services::gfx::{BottomScreen, TopScreen},
};

use crate::{
    input::Input,
    scene::{Scene, Transition},
};

const BACKGROUND: Rgba = 0xeff1f5ff;
const TEXT_COLOR: Rgba = 0x4c4f69ff;
const ACCENT: Rgba = 0xdf8e1dff;

/// A stand-in for the level scene, so the title screen has somewhere to go.
#[derive(Default)]
pub struct GameScene;

impl GameScene {
    pub fn new() -> Self {
        Self
    }
}

impl Scene for GameScene {
    fn update(&mut self, input: &Input) -> Transition {
        if input.pressed(KeyPad::B) {
            Transition::Pop
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
            "Game scene",
            (200., 100.),
            TextDrawStyle::default()
                .with_scalar(1.2)
                .with_color(ACCENT)
                .with_alignment(HorizAlignment::Center, VertAlignment::Center),
        )
        .blit()?;

        Text::new(
            "coming soon...",
            (200., 140.),
            TextDrawStyle::default()
                .with_scalar(0.8)
                .with_color(TEXT_COLOR)
                .with_alignment(HorizAlignment::Center, VertAlignment::Center),
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

        Text::new(
            "Press B to return to the title",
            (160., 120.),
            TextDrawStyle::default()
                .with_scalar(0.7)
                .with_color(TEXT_COLOR)
                .with_alignment(HorizAlignment::Center, VertAlignment::Center),
        )
        .blit()?;

        Ok(draw_target.end())
    }
}
