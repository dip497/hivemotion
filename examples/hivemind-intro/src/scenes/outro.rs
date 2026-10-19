//! The end card: the mark locks together, the name, the line the voice says, how to install.

use fframes::{Duration, FFramesContext, Frame, Scene, Svgr};

use crate::beat::*;
use crate::shaders::ground;
use crate::ui::*;

beat_scene!(OutroScene, ENDING_FROM, TOTAL_BEATS);

pub const INSTALL: &str = "bash <(curl -fsSL https://hivemind.griiken.com/install.sh)";

impl Scene for OutroScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let t_end = TOTAL_SECONDS - gsec(&frame);
        let fade_out = prog(1.6 - t_end, 0.0, 1.6);

        // the three cells of the mark fly in from three sides and lock on the downbeat
        let size = 260.0;
        let (mx, my) = (960.0 - size / 2.0, 120.0);
        let lock = spring(lb * BEAT, 260.0, 20.0);
        let s = size / 24.0;
        let cell = |dx: f32, dy: f32, d: &'static str, fill: &'static str, opacity: f32| {
            let t = format!("translate({} {}) scale({s})", mx + dx * (1.0 - lock), my + dy * (1.0 - lock));
            fframes::svgr!(<g transform={t} opacity={opacity * lock.min(1.0)}><path d={d} fill={fill} /></g>)
        };
        const TOP: &str = "M11.14 1.70 Q12 1.20 12.86 1.70 L16.51 3.80 Q17.37 4.30 17.37 5.29 L17.37 9.51 Q17.37 10.50 16.51 11 L12.86 13.10 Q12 13.60 11.14 13.10 L7.49 11 Q6.63 10.50 6.63 9.51 L6.63 5.29 Q6.63 4.30 7.49 3.80 Z";
        const LEFT: &str = "M5.60 11.30 Q6.46 10.80 7.32 11.30 L10.97 13.40 Q11.83 13.90 11.83 14.89 L11.83 19.11 Q11.83 20.10 10.97 20.60 L7.32 22.70 Q6.46 23.20 5.60 22.70 L1.95 20.60 Q1.09 20.10 1.09 19.11 L1.09 14.89 Q1.09 13.90 1.95 13.40 Z";
        const RIGHT: &str = "M16.68 11.30 Q17.54 10.80 18.40 11.30 L22.05 13.40 Q22.91 13.90 22.91 14.89 L22.91 19.11 Q22.91 20.10 22.05 20.60 L18.40 22.70 Q17.54 23.20 16.68 22.70 L13.03 20.60 Q12.17 20.10 12.17 19.11 L12.17 14.89 Q12.17 13.90 13.03 13.40 Z";

        let word = snap(lb - 1.0);
        let tag = prog(lb, 1.0, 1.4);
        let cmd = snap(lb - 5.5);
        let url = prog(lb, 7.0, 7.4);
        fframes::svgr!(
            <g>
                {ground(&frame, 0.15)}
                <g opacity={1.0 - fade_out}>
                    {cell(0.0, -500.0, TOP, HONEY, 1.0)}
                    {cell(-700.0, 300.0, LEFT, BONE, 0.45)}
                    {cell(700.0, 300.0, RIGHT, BONE, 0.45)}
                    <g opacity={prog(lb, 1.0, 1.1)} transform={format!("translate(0 {})", (1.0 - word) * 60.0)}>
                        <text x="960" y="560" text-anchor="middle" font-family={SANS} font-weight="800" font-size="168" letter-spacing="-7" fill={BONE}>"hivemind"</text>
                    </g>
                    <text x="960" y="630" text-anchor="middle" font-family={MONO} font-weight="500" font-size="28" letter-spacing="1" fill={FG2} opacity={tag}>"your agents build the workspace they work in"</text>
                    <g opacity={prog(lb, 5.5, 5.6)} transform={format!("translate(0 {})", (1.0 - cmd) * 30.0)}>
                        <rect x="400" y="716" width="1120" height="68" fill={SURFACE} fill-opacity="0.92" stroke={LINE2} stroke-width="1.5" />
                        <text x="430" y="760" font-family={MONO} font-weight="700" font-size="25" fill={HONEY}>"$"</text>
                        <text x="460" y="760" font-family={MONO} font-weight="500" font-size="25" fill={BONE}>{INSTALL}</text>
                    </g>
                    <g opacity={url}>
                        <text x="960" y="880" text-anchor="middle" font-family={MONO} font-weight="700" font-size="30" letter-spacing="3" fill={HONEY}>"HIVEMIND.GRIIKEN.COM"</text>
                    </g>
                </g>
                <rect width="1920" height="1080" fill="#000000" opacity={fade_out} />
            </g>
        )
    }
}
