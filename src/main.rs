//! Hajs vs Zombies.

mod assets;
mod frame;
mod input;
mod mobs;
mod music;
mod scene;
mod scenes;

use cabbage::prelude::*;
use citro2d::prelude::*;
use ctru::{
    prelude::*,
    services::{
        gfx::{BottomScreen, TopScreen},
        ndsp::{Ndsp, OutputMode},
    },
};

use crate::{input::Input, music::Music, scene::SceneManager, scenes::title::TitleScene};

/// Top-level game state, owned by the framework's [`State`].
pub struct App {
    scenes: SceneManager,
}

impl App {
    fn new() -> Self {
        Self {
            scenes: SceneManager::new(Box::new(TitleScene::new())),
        }
    }
}

impl StateImpl<App> for State<'_, App> {
    fn main(&mut self) -> Result<(), Error> {
        self.handles.apt.set_sleep_allowed(true);
        self.handles.apt.set_home_allowed(true);

        let mut c3d_instance = citro3d::Instance::new().expect("failed to initialize Citro3D");
        let cfgu = ctru::services::cfgu::Cfgu::new()?;
        let c2d_instance = citro2d::Instance::new(&mut c3d_instance, &cfgu)?;

        let mut top_target =
            Target::<TopScreen, _>::new(self.handles.gfx.top_screen.borrow_mut(), &c2d_instance);
        let mut bottom_target = Target::<BottomScreen, _>::new(
            self.handles.gfx.bottom_screen.borrow_mut(),
            &c2d_instance,
        );

        // Devices without DSP firmware (and emulators with no dump of it) cannot
        // play audio, so the game keeps running silently instead of failing.
        let mut ndsp = Ndsp::new().ok();
        if let Some(ndsp) = ndsp.as_mut() {
            ndsp.set_output_mode(OutputMode::Stereo);
        }
        let mut music = ndsp.as_ref().and_then(|ndsp| Music::new(ndsp).ok());

        while self.handles.apt.main_loop() {
            self.handles.hid.scan_input();
            let input = Input::new(self.handles.hid);

            self.app.scenes.update(&input);
            if self.app.scenes.is_empty() {
                break;
            }

            frame::begin();
            if let Some(music) = music.as_mut() {
                self.app.scenes.sync_music(music);
            }

            top_target = self.app.scenes.draw_top(top_target)?;
            bottom_target = self.app.scenes.draw_bottom(bottom_target)?;
            frame::end();
        }

        Ok(())
    }
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let _ = std::fs::write("sdmc:/hajs-panic.txt", info.to_string());
    }));
    let _romfs = ctru::services::romfs::RomFS::new().expect("failed to mount romfs");
    let mut apt = Apt::new().expect("failed to obtain applet service handle");
    let mut hid = Hid::new().expect("failed to obtain HID handle");
    let mut gfx = Gfx::new().expect("failed to obtain GFX handle");

    let handles = Handles {
        apt: &mut apt,
        hid: &mut hid,
        gfx: &mut gfx,
    };

    let mut state = State::new(App::new(), handles);

    if let Err(e) = state.main() {
        let _ = state.handle_error(e);
    }
}
