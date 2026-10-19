//! The board: each coding agent close up in its real terminal, then all of them on one canvas.

use fframes::{Duration, FFramesContext, Frame, Scene, Svgr};

use crate::beat::*;
use crate::shaders::ground;
use crate::ui::*;

beat_scene!(BoardScene, 14.0, 28.0);

/// Each agent's close-up: the agent, its tile on the board and the clip of its terminal.
const SHOTS: &[(&str, &str, &str)] = &[
    ("claude", "cart-page", "tile-claude.mp4"),
    ("codex", "refunds", "tile-codex.mp4"),
    ("opencode", "cart-tests", "tile-opencode.mp4"),
    ("pi", "todo-sweep", "tile-pi.mp4"),
];
/// Beats each close-up holds; the wide shot follows.
const SHOT_BEATS: f32 = 2.0;
const WIDE_AT: f32 = 8.0;
const BOX: (f32, f32, f32, f32) = (660.0, 270.0, 1140.0, 641.0);

impl Scene for BoardScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let exit = expo_in(prog(lb, 13.7, 14.0));
        if lb >= WIDE_AT {
            let l = lb - WIDE_AT;
            let start = WIDE_AT * BEAT;
            let push = 1.06 - 0.06 * expo_out(prog(l, 0.0, 6.0));
            let picture = clip(&frame, ctx, "board.mp4", start, 0.0, -60.0, 1920.0, 1200.0);
            return fframes::svgr!(
                <g opacity={1.0 - exit}>
                    <g transform={format!("translate(960 540) scale({push}) translate(-960 -540)")}>{picture}</g>
                    <defs>
                        <linearGradient id="board-top" x1="0" y1="0" x2="0" y2="1">
                            <stop offset="0" stop-color={BG} stop-opacity="0.95" />
                            <stop offset="1" stop-color={BG} stop-opacity="0" />
                        </linearGradient>
                    </defs>
                    <rect width="1920" height="380" fill="url(#board-top)" />
                    {Slam::new(146.0, 290.0, "ONE BOARD.", 150.0, HONEY).draw(l, 0.0, 80.0)}
                </g>
            );
        }
        let k = ((lb / SHOT_BEATS).floor() as usize).min(SHOTS.len() - 1);
        let (agent, tile, file) = SHOTS[k];
        let l = lb - k as f32 * SHOT_BEATS;
        let (x, y, w, h) = BOX;
        // each close-up starts a little further into the recording
        let start = k as f32 * SHOT_BEATS * BEAT - k as f32 * 0.9;
        let push = 1.0 + 0.03 * prog(l, 0.0, SHOT_BEATS);
        let enter = snap(l);
        let heading = if lb < 4.0 { ("EVERY AGENT,", BONE) } else { ("REAL TERMINALS.", HONEY) };
        let heading_l = if lb < 4.0 { lb } else { lb - 4.0 };
        let ticks: Vec<Svgr> = SHOTS
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let on = i <= k;
                fframes::svgr!(<path d={hex_path(166.0 + i as f32 * 44.0, 820.0, 15.0)} fill={if on { HONEY } else { "none" }} stroke={if on { HONEY } else { LINE2 }} stroke-width="2" />)
            })
            .collect();
        fframes::svgr!(
            <g opacity={1.0 - exit}>
                {ground(&frame, 0.66, 0.5, 0.28, 0.1)}
                {Slam::new(146.0, 200.0, heading.0, 92.0, heading.1).draw(heading_l, 0.0, 60.0)}
                <g transform={format!("translate(0 {})", (1.0 - enter) * 40.0)} opacity={prog(l, 0.0, 0.1)}>
                    <text x="146" y="560" font-family={MONO} font-weight="700" font-size="72" fill={BONE}>{agent}</text>
                    {label(150.0, 616.0, format!("{} · IN ITS OWN TERMINAL", tile.to_uppercase()), GREY, 20.0, "start")}
                </g>
                {ticks}
                <g transform={format!("translate({} {}) scale({push}) translate({} {})", x + w / 2.0, y + h / 2.0, -(x + w / 2.0), -(y + h / 2.0))}
                   opacity={prog(l, 0.0, 0.08)}>
                    {framed_clip(&frame, ctx, file, start, x, y, w, h, format!("{agent} · RECORDED IN THE APP").to_uppercase())}
                </g>
            </g>
        )
    }
}
