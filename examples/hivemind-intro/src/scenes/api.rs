//! Agents drive agents. The workflow command, typed and annotated; the same API called from a
//! script; then both recordings side by side as the agents they started open.

use fframes::{Duration, FFramesContext, Frame, Scene, Svgr};

use crate::beat::*;
use crate::shaders::ground;
use crate::ui::*;

beat_scene!(ApiScene, 88.0, 96.0);
beat_scene!(FanoutScene, 96.0, 104.0);

/// The command as typed in takes/workspace-60/W4-fanout.mp4, split where it is explained.
const COMMAND: &[(&str, &str)] = &[
    ("hive ctl workflow --shape fanout", "ONE AGENT PER ITEM"),
    ("  --items 'cart||auth||search'", "THE ITEMS"),
    ("  --prompt 'Review the {item} code'", "WHAT EACH ONE DOES"),
];

fn title(lb: f32) -> Svgr<'static> {
    fframes::svgr!(
        <g>
            {Slam::new(146.0, 330.0, "EVERYTHING", 92.0, BONE).draw(lb, 0.0, 60.0)}
            {Slam::new(146.0, 422.0, "YOU CAN CLICK,", 92.0, BONE).draw(lb - 0.25, 0.0, 60.0)}
            {Slam::new(146.0, 514.0, "AN AGENT", 92.0, HONEY).draw(lb - 0.75, 0.0, 60.0)}
            {Slam::new(146.0, 606.0, "CAN CALL.", 92.0, HONEY).draw(lb - 1.0, 0.0, 60.0)}
        </g>
    )
}

impl Scene for ApiScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let body = if lb < 4.0 { command(lb) } else { script(&frame, ctx, lb - 4.0) };
        let exit = expo_in(prog(lb, 7.7, 8.0));
        fframes::svgr!(
            <g opacity={1.0 - exit}>
                {ground(&frame, 0.75, 0.5, 0.3, 0.1)}
                {title(lb)}
                {label(152.0, 690.0, "HIVE CTL · THE SDK · ANY SCRIPT".to_owned(), FG2, 19.0, "start")}
                {body}
            </g>
        )
    }
}

/// Beats 0-4: the command typed into a shell card, each part explained as it lands.
/// Characters typed per second.
const CPS: f32 = 70.0;

fn command(lb: f32) -> Svgr<'static> {
    let (x, y, w) = (880.0, 330.0, 900.0);
    let enter = snap(lb);
    let mut rows = Vec::new();
    let mut typed_before = 0.0;
    for (i, (text, note)) in COMMAND.iter().enumerate() {
        let n = text.chars().count() as f32;
        let start = 0.2 + typed_before / CPS / BEAT;
        let typed = ((lb - start) * BEAT * CPS).clamp(0.0, n) as usize;
        typed_before += n;
        let shown: String = text.chars().take(typed).collect();
        let ry = y + 116.0 + i as f32 * 84.0;
        let note_at = start + n / CPS / BEAT;
        let note_o = prog(lb, note_at, note_at + 0.3);
        rows.push(fframes::svgr!(
            <g>
                <text x={x + 34.0} y={ry} font-family={MONO} font-weight="500" font-size="27" fill={BONE}>{shown}</text>
                <g opacity={note_o} transform={format!("translate({} 0)", (1.0 - note_o) * -20.0)}>
                    <rect x={x + 58.0} y={ry + 18.0} width="18" height="2" fill={HONEY} />
                    <text x={x + 86.0} y={ry + 26.0} font-family={MONO} font-weight="700" font-size="17" letter-spacing="2" fill={HONEY}>{*note}</text>
                </g>
            </g>
        ));
    }
    fframes::svgr!(
        <g opacity={prog(lb, 0.0, 0.1)} transform={format!("translate({} 0)", (1.0 - enter) * 120.0)}>
            <rect x={x} y={y} width={w} height="360" fill={SURFACE} fill-opacity="0.94" stroke={LINE2} stroke-width="1.5" />
            <rect x={x} y={y} width="4" height="360" fill={HONEY} />
            <text x={x + 34.0} y={y + 50.0} font-family={MONO} font-weight="500" font-size="22" fill={GREY}>"storefront $"</text>
            <rect x={x + 30.0} y={y + 68.0} width={w - 60.0} height="1" fill={LINE2} />
            {rows}
        </g>
    )
}

/// Beats 4-8: the same thing from a script: `cat board.ts`, recorded.
fn script<'a>(frame: &Frame, ctx: &FFramesContext<'a, '_>, l: f32) -> Svgr<'a> {
    let (x, y, w, h) = (880.0, 200.0, 900.0, 563.0);
    let enter = snap(l);
    // `hive.call("agent.start", …)` in the recording, inside the box
    let (cx, cy) = (x + 330.0, y + 466.0);
    fframes::svgr!(
        <g opacity={prog(l, 0.0, 0.1)} transform={format!("translate({} 0)", (1.0 - enter) * 120.0)}>
            {framed_clip(frame, ctx, "code.mp4", SCRIPT_AT, x, y, w, h, "BOARD.TS · RECORDED IN THE APP".to_owned())}
            {callout(cx, cy, cx + 260.0, y + h + 90.0, "HIVE.CALL(\"AGENT.START\")".to_owned(), HONEY, prog(l, 1.2, 2.0))}
        </g>
    )
}

/// Seconds into the scene where the script's recording starts.
const SCRIPT_AT: f32 = 4.0 * BEAT;

impl Scene for FanoutScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let left = expo_out(prog(lb, 0.0, 0.8));
        let right = expo_out(prog(lb, 0.5, 1.3));
        let exit = expo_in(prog(lb, 7.7, 8.0));
        fframes::svgr!(
            <g opacity={1.0 - exit}>
                {ground(&frame, 0.5, 0.75, 0.22, 0.1)}
                {Slam::new(146.0, 240.0, "AGENTS START AGENTS.", 104.0, BONE).draw(lb, 0.0, 70.0)}
                <g opacity={left} transform={format!("translate(0 {})", (1.0 - left) * 80.0)}>
                    {framed_clip(&frame, ctx, "fanout.mp4", 0.0, 150.0, 330.0, 790.0, 494.0, "HIVE CTL WORKFLOW · FANOUT".to_owned())}
                </g>
                <g opacity={right} transform={format!("translate(0 {})", (1.0 - right) * 80.0)}>
                    {framed_clip(&frame, ctx, "sdk.mp4", 0.0, 980.0, 330.0, 790.0, 494.0, "BUN BOARD.TS · THE SDK".to_owned())}
                </g>
            </g>
        )
    }
}
