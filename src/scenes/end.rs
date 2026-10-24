//! the ending scene.

use cabbage::{F_BOTTOM_HEIGHT, F_BOTTOM_WIDTH, prelude::Error};
use citro2d::prelude::*;
use ctru::{
    prelude::*,
    services::gfx::{BottomScreen, TopScreen},
};

use crate::{
    input::Input,
    scene::{Scene, Transition},
    scenes::title::TitleScene,
};

const BACKGROUND: Rgba = 0xffffffff;
const TEXT_RED_COLOR: Rgba = 0xff0000ff;
const TEXT_GREY_COLOR: Rgba = 0x222222ff;

pub struct EndScene {
    logo: Sprite<'static>,
    survived_time_seconds: u32,
}

impl EndScene {
    pub fn new(survived_time_seconds: u32) -> Self {
        let sheet: &'static SpriteSheet = Box::leak(Box::new(
            SpriteSheet::new("romfs:/end.t3x").expect("failed to load romfs:/end.t3x"),
        ));
        let logo = Sprite::new(
            sheet,
            0,
            (F_BOTTOM_WIDTH / 2., F_BOTTOM_HEIGHT / 2.),
            (1., 1.),
        );
        Self {
            logo,
            survived_time_seconds,
        }
    }
}

fn survival_time(seconds: u32) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

impl Scene for EndScene {
    fn update(&mut self, input: &Input) -> Transition {
        let screen_touched: bool = input.touch().is_some();

        if input.pressed(KeyPad::A) || input.pressed(KeyPad::START) || screen_touched {
            Transition::Push(Box::new(TitleScene::new()))
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
            "YOU LOST\nTHE GAME",
            (F_BOTTOM_WIDTH / 2. + 45., F_BOTTOM_HEIGHT / 2. - 50.),
            TextDrawStyle::default()
                .with_scalar(2.5)
                .with_color(TEXT_RED_COLOR)
                .with_alignment(HorizAlignment::Center, VertAlignment::Center),
        )
        .blit()?;

        Text::new(
            &format!(
                "You survived for {}",
                survival_time(self.survived_time_seconds)
            ),
            (F_BOTTOM_WIDTH / 2. + 40., F_BOTTOM_HEIGHT / 2. + 57.),
            TextDrawStyle::default()
                .with_scalar(0.8)
                .with_color(TEXT_GREY_COLOR)
                .with_alignment(HorizAlignment::Center, VertAlignment::Center),
        )
        .blit()?;

        Text::new(
            "press anywhere or press\nA / START to continue",
            (F_BOTTOM_WIDTH / 2. + 40., F_BOTTOM_HEIGHT / 2. + 95.),
            TextDrawStyle::default()
                .with_scalar(0.8)
                .with_color(TEXT_GREY_COLOR)
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

        self.logo.blit()?;

        Ok(draw_target.end())
    }
}
