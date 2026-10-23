//! Every GPU shader of the video, compiled once by the Skia renderer.

use std::sync::LazyLock;

use fframes::{Color, Frame, Shader, ShaderUniforms, Svgr};

pub struct Shaders {
    pub ground: Shader,
    pub grain: Shader,
}

pub static SHADERS: LazyLock<Shaders> = LazyLock::new(|| Shaders {
    ground: Shader::sksl(include_str!("shaders/ground.sksl")),
    grain: Shader::sksl(include_str!("shaders/grain.sksl")),
});

/// The ground: a faint honeycomb, `comb` its strength.
pub fn ground(frame: &Frame, comb: f32) -> Svgr<'static> {
    let layer = SHADERS.ground.draw(frame, ShaderUniforms::new().float("uComb", comb).color("uInk", Color::hex("#7d8186")));
    fframes::svgr!(<image href={layer.href()} x="0" y="0" width="1920" height="1080" />)
}
