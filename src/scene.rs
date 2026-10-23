//! The [`Scene`] trait and the [`SceneManager`] that drives it.

use cabbage::prelude::Error;
use citro2d::render::{ReadyMode, Target};
use ctru::services::gfx::{BottomScreen, TopScreen};

use crate::input::Input;

/// What the [`SceneManager`] should do once a scene has been updated.
pub enum Transition {
    /// Keep the current scene.
    None,
    /// Push a new scene on top of the current one (e.g. a pause menu).
    Push(Box<dyn Scene>),
    /// Drop the current scene, revealing the one beneath it.
    Pop,
    /// Close the game.
    Quit,
}

/// A single screen of the game: the title, a level, a menu,...
pub trait Scene {
    /// Simulate a frame of the scene.
    fn update(&mut self, input: &Input) -> Transition;

    /// Draw the scene to the top screen.
    fn draw_top<'s>(
        &mut self,
        target: Target<'s, TopScreen, ReadyMode>,
    ) -> Result<Target<'s, TopScreen, ReadyMode>, Error>;

    /// Draw the scene to the bottom screen.
    fn draw_bottom<'s>(
        &mut self,
        target: Target<'s, BottomScreen, ReadyMode>,
    ) -> Result<Target<'s, BottomScreen, ReadyMode>, Error>;
}

/// Holds the stack of active scenes and routes updates and draws to the top one.
///
/// Only the scene on top of the stack is updated and drawn; the ones below it
/// stay alive so they can be returned to with [`Transition::Pop`].
pub struct SceneManager {
    stack: Vec<Box<dyn Scene>>,
}

impl SceneManager {
    /// Create a manager whose only scene is `initial`.
    pub fn new(initial: Box<dyn Scene>) -> Self {
        Self {
            stack: vec![initial],
        }
    }

    /// Whether every scene has been popped, meaning the game should exit.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Update the active scene and apply the [`Transition`] it returns.
    pub fn update(&mut self, input: &Input) {
        let Some(scene) = self.stack.last_mut() else {
            return;
        };
        let transition = scene.update(input);

        match transition {
            Transition::None => {}
            Transition::Push(scene) => self.stack.push(scene),
            Transition::Pop => {
                self.stack.pop();
            }
            Transition::Quit => self.stack.clear(),
        }
    }

    /// Draw the active scene to the top screen.
    pub fn draw_top<'s>(
        &mut self,
        target: Target<'s, TopScreen, ReadyMode>,
    ) -> Result<Target<'s, TopScreen, ReadyMode>, Error> {
        match self.stack.last_mut() {
            Some(scene) => scene.draw_top(target),
            None => Ok(target),
        }
    }

    /// Draw the active scene to the bottom screen.
    pub fn draw_bottom<'s>(
        &mut self,
        target: Target<'s, BottomScreen, ReadyMode>,
    ) -> Result<Target<'s, BottomScreen, ReadyMode>, Error> {
        match self.stack.last_mut() {
            Some(scene) => scene.draw_bottom(target),
            None => Ok(target),
        }
    }
}
