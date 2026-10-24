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
const BUTTON_FILL: Rgba = 0xccd0daff;
const BUTTON_FILL_SELECTED: Rgba = 0xdf8e1d70;

/// The area of the bottom screen that starts the game when tapped.
const START_BUTTON: BoundingBox = BoundingBox::with_top_left((60., 80.), (200., 80.));

/// Whether a touch point lies on the start button.
fn on_button((x, y): (u16, u16)) -> bool {
    START_BUTTON.intersects_point((x as f32, y as f32))
}

pub struct TitleScene {
    /// Frames elapsed, used to blink the prompt.
    frame: u32,
    /// Whether the current touch began on the start button.
    pressing: bool,
    /// Whether the start button is currently held down (for highlighting).
    button_touched: bool,
    /// Last position seen while the screen was held.
    last_touch: Option<(u16, u16)>,
    /// Well... the logo, obviously.
    logo: Sprite<'static>,
}

impl TitleScene {
    pub fn new() -> Self {
        let sheet: &'static SpriteSheet = Box::leak(Box::new(
            SpriteSheet::new("romfs:/logo.t3x").expect("failed to load romfs:/logo.t3x"),
        ));
        let logo = Sprite::new(sheet, 0, (200., 120.), (1., 1.));
        Self {
            frame: 0,
            pressing: false,
            button_touched: false,
            last_touch: None,
            logo,
        }
    }

    /// Tracks touches; returns true when a tap is released on the start button
    /// after having begun on it.
    fn update_touch(&mut self, input: &Input) -> bool {
        // Only a touch that begins on the button counts.
        if let Some(pos) = input.touch_started() {
            self.pressing = on_button(pos);
        }

        let touch = input.touch();
        self.button_touched = self.pressing && touch.is_some_and(on_button);

        if let Some(pos) = touch {
            self.last_touch = Some(pos);
            return false;
        }

        // Screen not touched: if it was last frame, the touch just ended.
        let released_at = self.last_touch.take();
        let pressing = std::mem::take(&mut self.pressing);
        pressing && released_at.is_some_and(on_button)
    }
}

impl Default for TitleScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for TitleScene {
    fn update(&mut self, input: &Input) -> Transition {
        self.frame = self.frame.wrapping_add(1);

        let tapped = self.update_touch(input);

        if input.pressed(KeyPad::A) || input.pressed(KeyPad::START) || tapped {
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

        self.logo.blit()?;

        Ok(draw_target.end())
    }

    fn draw_bottom<'s>(
        &mut self,
        target: Target<'s, BottomScreen, ReadyMode>,
    ) -> Result<Target<'s, BottomScreen, ReadyMode>, Error> {
        let mut draw_target = target.begin();
        draw_target.clear(BACKGROUND);

        let fill = if self.button_touched {
            BUTTON_FILL_SELECTED
        } else {
            BUTTON_FILL
        };
        let border = if self.button_touched {
            ACCENT
        } else {
            TEXT_COLOR
        };
        Rectangle::new(
            START_BUTTON,
            RectangleDrawStyle::default()
                .with_fill(fill)
                .with_border(border, 2.),
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
