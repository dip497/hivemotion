//! The first drop: every coding agent in a real terminal, on one canvas. The recording runs at
//! its own speed; each agent is ticked off as its tile opens.

use fframes::{Duration, FFramesContext, Frame, Scene, Svgr};

use crate::beat::*;
use crate::shaders::ground;
use crate::ui::*;

beat_scene!(CanvasScene, 32.0, 48.0);

/// The agents of the recording and the second their tile opens (takes/workspace-60/W1-board.json).
const AGENTS: &[(&str, &str, f32)] = &[
    ("claude", "cart-page", 0.0),
    ("codex", "refunds", 1.9),
    ("opencode", "cart-tests", 4.2),
    ("pi", "todo-sweep", 6.5),
];

const BOX: (f32, f32, f32, f32) = (850.0, 350.0, 920.0, 575.0);

impl Scene for CanvasScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let t = frame.seconds();
        let (x, y, w, h) = BOX;
        let enter = expo_out(prog(lb, 0.0, 0.8));
        let rows: Vec<Svgr> = AGENTS
            .iter()
            .enumerate()
            .map(|(i, (agent, tile, at))| {
                let s = snap((t - at) / BEAT);
                let ry = 500.0 + i as f32 * 80.0;
                let on = t >= *at;
                let tick = if on { HONEY } else { LINE2 };
                fframes::svgr!(
                    <g opacity={0.35 + 0.65 * prog(t, *at, *at + 0.1)} transform={format!("translate({} 0)", (1.0 - s) * -40.0 * f32::from(on))}>
                        <path d={hex_path(168.0, ry - 12.0, 16.0)} fill={if on { tick } else { "none" }} stroke={tick} stroke-width="2" />
                        <text x="204" y={ry} font-family={MONO} font-weight="700" font-size="30" fill={BONE}>{*agent}</text>
                        <text x="610" y={ry} text-anchor="end" font-family={MONO} font-weight="500" font-size="22" fill={GREY}>{*tile}</text>
                    </g>
                )
            })
            .collect();
        let exit = expo_in(prog(lb, 15.7, 16.0));
        fframes::svgr!(
            <g opacity={1.0 - exit}>
                {ground(&frame, 0.66, 0.5, 0.28, 0.1)}
                {Slam::new(146.0, 214.0, "EVERY CODING AGENT,", 92.0, BONE).draw(lb, 0.0, 70.0)}
                {Slam::new(146.0, 306.0, "IN A REAL TERMINAL.", 92.0, HONEY).draw(lb - 0.75, 0.0, 70.0)}
                <g transform={format!("translate({} 0)", (1.0 - enter) * 160.0)} opacity={enter}>
                    {framed_clip(&frame, ctx, "board.mp4", 0.0, x, y, w, h, "THE CANVAS · RECORDED IN THE APP".to_owned())}
                </g>
                {rows}
            </g>
        )
    }
}
