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

pub mod anywhere;
pub mod api;
pub mod canvas;
pub mod hook;
pub mod island;
pub mod outro;
pub mod plugins;
pub mod prompt;
pub mod origin;
pub mod render;
pub mod the_loop;
pub mod widgets;

pub use anywhere::AnywhereScene;
pub use api::{ApiScene, FanoutScene};
pub use canvas::CanvasScene;
pub use hook::{FiredScene, HiveScene};
pub use island::{AskScene, IslandScene};
pub use origin::OriginScene;
pub use outro::{OutroScene, SilentScene};
pub use plugins::PluginsScene;
pub use prompt::PromptScene;
pub use render::RenderScene;
pub use the_loop::LoopScene;
pub use widgets::{ToolScene, WidgetScene};
