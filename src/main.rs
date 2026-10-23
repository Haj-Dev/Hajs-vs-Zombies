use cabbage::prelude::*;
use citro2d::{
    prelude::*,
    render::{ReadyMode, Target},
};
use ctru::{
    prelude::*,
    services::gfx::{BottomScreen, TopScreen},
};

const CLEAR_COLOR: Rgba = 0x000000ff;

pub struct App;

impl App {
    fn draw_top_screen<'screen>(
        &self,
        target: Target<'screen, TopScreen, ReadyMode>,
    ) -> Result<Target<'screen, TopScreen, ReadyMode>, Error> {
        let mut draw_target = target.begin();
        draw_target.clear(CLEAR_COLOR);

        let text = Text::new(
            "Hello, World!",
            (50., 50.),
            TextDrawStyle::with_color(TextDrawStyle::default(), 0xffffffff),
        );
        text.blit()?;

        Ok(draw_target.end())
    }

    fn draw_bottom_screen<'screen>(
        &self,
        target: Target<'screen, BottomScreen, ReadyMode>,
    ) -> Result<Target<'screen, BottomScreen, ReadyMode>, Error> {
        let mut draw_target = target.begin();
        draw_target.clear(CLEAR_COLOR);
        Ok(draw_target.end())
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

        loop {
            self.handles.hid.scan_input();
            let keys_held = self.handles.hid.keys_held();

            if keys_held.contains(KeyPad::START) || !self.handles.apt.main_loop() {
                break;
            }

            top_target = self.app.draw_top_screen(top_target)?;
            bottom_target = self.app.draw_bottom_screen(bottom_target)?;

            self.handles.gfx.wait_for_vblank();
        }

        Ok(())
    }
}

fn main() {
    let mut apt = Apt::new().expect("failed to obtain applet service handle");
    let mut hid = Hid::new().expect("failed to obtain HID handle");
    let mut gfx = Gfx::new().expect("failed to obtain GFX handle");

    let handles = Handles {
        apt: &mut apt,
        hid: &mut hid,
        gfx: &mut gfx,
    };

    let mut state = State::new(App, handles);

    if let Err(e) = state.main() {
        let _ = state.handle_error(e);
    }
}
