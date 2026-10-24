use citro2d::render::{CURRENT_TARGET, DrawingMode, ReadyMode, Target};
use ctru::services::gfx::Screen;

pub fn begin() {
    unsafe { citro3d_sys::C3D_FrameBegin(citro3d_sys::C3D_FRAME_SYNCDRAW) };
}

pub fn end() {
    unsafe {
        citro2d_sys::C2D_Flush();
        citro3d_sys::C3D_FrameEnd(0);
    }
}

pub fn begin_scene<'s, T: Screen>(target: Target<'s, T, ReadyMode>) -> Target<'s, T, DrawingMode> {
    unsafe {
        citro2d_sys::C2D_SceneBegin(target.raw);
        CURRENT_TARGET = target.raw;
        target.assume_drawing()
    }
}

pub fn end_scene<'s, T: Screen>(target: Target<'s, T, DrawingMode>) -> Target<'s, T, ReadyMode> {
    unsafe { target.assume_ready() }
}
