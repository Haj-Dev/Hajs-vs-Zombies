//! A per-frame snapshot of the player's input.

use ctru::services::hid::{Hid, KeyPad};

/// The state of the controls for a single frame.
///
/// Scenes receive one of these instead of talking to the [`Hid`] service
/// directly, which keeps them decoupled from `ctru` and easy to reason about.
pub struct Input {
    pressed: KeyPad,
    touch: Option<(u16, u16)>,
    touch_started: Option<(u16, u16)>,
}

impl Input {
    /// Capture the current input state.
    ///
    /// [`Hid::scan_input`] must have been called earlier in the frame.
    pub fn new(hid: &Hid) -> Self {
        let held = hid.keys_held();
        let down = hid.keys_down();
        Self {
            pressed: down,
            touch: held.contains(KeyPad::TOUCH).then(|| hid.touch_position()),
            touch_started: down.contains(KeyPad::TOUCH).then(|| hid.touch_position()),
        }
    }

    /// Whether any of the given buttons were pressed this frame.
    pub fn pressed(&self, keys: KeyPad) -> bool {
        self.pressed.intersects(keys)
    }

    /// The touchscreen position, if the screen is currently being touched.
    pub fn touch(&self) -> Option<(u16, u16)> {
        self.touch
    }

    /// The position of a touch that began this frame, if any.
    pub fn touch_started(&self) -> Option<(u16, u16)> {
        self.touch_started
    }
}
