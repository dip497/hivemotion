//! Every GPU shader of the video, compiled once by the Skia renderer.

use std::sync::LazyLock;

use fframes::{Color, Frame, Shader, ShaderUniforms, Svgr};

use crate::ui::HONEY;

pub struct Shaders {
    pub ground: Shader,
    pub grain: Shader,
    pub grid: Shader,
}

pub static SHADERS: LazyLock<Shaders> = LazyLock::new(|| Shaders {
    ground: Shader::sksl(include_str!("shaders/ground.sksl")),
    grain: Shader::sksl(include_str!("shaders/grain.sksl")),
    grid: Shader::sksl(include_str!("shaders/grid.sksl")),
});

/// The contour ground with its warm well at (`x`, `y`), 0..1 of the screen.
pub fn ground(frame: &Frame, x: f32, y: f32, bright: f32, comb: f32) -> Svgr<'static> {
    let layer = SHADERS.ground.draw(
        frame,
        ShaderUniforms::new()
            .float2("uWell", x, y)
            .float("uDepth", 0.8)
            .float("uDensity", 17.0)
            .float("uBright", bright)
            .float("uComb", comb)
            .color("uInk", Color::hex("#7d8186"))
            .color("uHot", Color::hex(HONEY)),
    );
    fframes::svgr!(<image href={layer.href()} x="0" y="0" width="1920" height="1080" />)
}

/// The hex floor rushing at the camera.
pub fn floor(frame: &Frame, speed: f32, horizon: f32, bright: f32) -> Svgr<'static> {
    let layer = SHADERS.grid.draw(
        frame,
        ShaderUniforms::new()
            .float("uSpeed", speed)
            .float("uHorizon", horizon)
            .float("uBright", bright)
            .color("uInk", Color::hex("#5d6166"))
            .color("uHot", Color::hex(HONEY)),
    );
    fframes::svgr!(<image href={layer.href()} x="0" y="0" width="1920" height="1080" />)
}
