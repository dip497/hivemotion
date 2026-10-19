//! One file per part of the video. Every scene declares the beats it starts and ends on; `lb` is
//! the beat inside the scene.

macro_rules! beat_scene {
    ($name:ident, $start:expr, $end:expr) => {
        #[derive(Debug)]
        pub struct $name;
        impl $name {
            pub const START: f32 = $start;
            pub const END: f32 = $end;
            #[allow(dead_code)]
            pub fn lb(frame: &fframes::Frame) -> f32 {
                crate::beat::gbeat(frame) - Self::START
            }
            pub fn frames() -> fframes::Duration<'static> {
                fframes::Duration::Frames(crate::beat::span_frames(Self::START, Self::END))
            }
        }
    };
}

pub mod asks;
pub mod board;
pub mod hook;
pub mod outro;

pub use asks::{AskScene, ToolScene, ViewScene, WidgetScene};
pub use board::BoardScene;
pub use hook::{FiredScene, HiveScene};
pub use outro::OutroScene;
