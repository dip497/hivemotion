//! The hook. "I FIRED MY IDE." while an editor window is struck through and flicked into the
//! trash, then "HIRED A HIVE." as agent tiles land on the canvas.

use fframes::{Duration, FFramesContext, Frame, Scene, Svgr};

use crate::beat::*;
use crate::shaders::ground;
use crate::ui::*;

beat_scene!(FiredScene, 0.0, 7.0);
beat_scene!(HiveScene, 7.0, 14.0);

/// A plain editor window: tab row, file list, line numbers, code as grey bars. No product.
fn editor() -> Svgr<'static> {
    let mut rows = Vec::new();
    for i in 0..13 {
        let y = 128.0 + i as f32 * 22.0;
        let indent = [0.0, 22.0, 22.0, 44.0, 44.0, 22.0, 0.0, 0.0, 22.0, 44.0, 22.0, 0.0, 22.0][i];
        let w = 60.0 + hash(i as f32 * 3.3) * 230.0;
        let tone = if i % 4 == 1 { "#4d5157" } else { "#3a3d41" };
        rows.push(fframes::svgr!(
            <g>
                <rect x="178" y={y} width="14" height="8" rx="2" fill="#2c2e31" />
                <rect x={208.0 + indent} y={y} width={w} height="8" rx="4" fill={tone} />
            </g>
        ));
    }
    fframes::svgr!(
        <g>
            <rect x="0" y="0" width="560" height="420" rx="14" fill="#141517" stroke={LINE2} stroke-width="2" />
            <circle cx="24" cy="22" r="6" fill="#3a3d41" />
            <circle cx="44" cy="22" r="6" fill="#3a3d41" />
            <circle cx="64" cy="22" r="6" fill="#3a3d41" />
            <rect x="0" y="44" width="560" height="1.5" fill={LINE2} />
            <rect x="160" y="56" width="120" height="26" rx="5" fill="#202225" />
            <rect x="176" y="66" width="70" height="7" rx="3" fill="#55595e" />
            <rect x="294" y="66" width="62" height="7" rx="3" fill="#33363a" />
            <rect x="372" y="66" width="80" height="7" rx="3" fill="#33363a" />
            <rect x="148" y="44" width="1.5" height="376" fill={LINE2} />
            <rect x="20" y="66" width="90" height="7" rx="3" fill="#44474c" />
            <rect x="32" y="88" width="80" height="7" rx="3" fill="#33363a" />
            <rect x="32" y="108" width="66" height="7" rx="3" fill="#33363a" />
            <rect x="32" y="128" width="92" height="7" rx="3" fill="#33363a" />
            <rect x="20" y="148" width="74" height="7" rx="3" fill="#44474c" />
            <rect x="32" y="168" width="58" height="7" rx="3" fill="#33363a" />
            {rows}
            <rect x="0" y="392" width="560" height="28" fill="#191a1c" />
            <rect x="16" y="402" width="120" height="7" rx="3" fill="#33363a" />
        </g>
    )
}

/// Where the trash can stands: the middle of its mouth.
const BIN: (f32, f32) = (1640.0, 760.0);

/// A plain trash can in line art; the lid swings open by `lid` degrees.
fn trash(lid: f32) -> (Svgr<'static>, Svgr<'static>) {
    let (bx, by) = BIN;
    let body = format!("M{} {by} L{} {by} L{} {} L{} {} Z", bx - 78.0, bx + 78.0, bx + 62.0, by + 190.0, bx - 62.0, by + 190.0);
    let stripes: Vec<Svgr> = [-30.0, 0.0, 30.0]
        .iter()
        .map(|dx| fframes::svgr!(<rect x={bx + dx - 1.5} y={by + 34.0} width="3" height="122" rx="1.5" fill={LINE2} />))
        .collect();
    let lid_t = format!("rotate({} {} {})", -lid, bx + 86.0, by - 8.0);
    (
        fframes::svgr!(
            <g>
                <path d={body} fill="#121315" stroke={GREY} stroke-width="3" stroke-linejoin="round" />
                {stripes}
            </g>
        ),
        fframes::svgr!(
            <g transform={lid_t}>
                <rect x={bx - 88.0} y={by - 22.0} width="176" height="16" rx="5" fill="#121315" stroke={GREY} stroke-width="3" />
                <rect x={bx - 22.0} y={by - 36.0} width="44" height="14" rx="4" fill="none" stroke={GREY} stroke-width="3" />
            </g>
        ),
    )
}

impl Scene for FiredScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        // the window is there on the first frame, then flicked in an arc into the trash
        let throw = prog(lb, 3.4, 4.55);
        let fly = cubic_in_out(throw);
        let (x0, y0) = (1470.0, 470.0);
        let (x1, y1) = (BIN.0, BIN.1 + 60.0);
        let wx = lerp(x0, x1, fly);
        let wy = lerp(y0, y1, fly) - (throw * std::f32::consts::PI).sin() * 230.0;
        let rot = fly * 34.0 + (1.0 - snap(lb)) * -4.0;
        let ws = lerp(1.0, 0.16, expo_in(throw)) + (1.0 - snap(lb)) * 0.06;
        let window = fframes::svgr!(
            <g transform={format!("translate({wx} {wy}) rotate({rot}) scale({ws}) translate(-280 -210)")}
               opacity={1.0 - prog(throw, 0.9, 1.0)}>
                {editor()}
            </g>
        );
        // the trash steps in, opens for the window and shuts on it
        let can_in = snap(lb - 2.4);
        let lid = 62.0 * prog(lb, 3.0, 3.5) * (1.0 - snap(lb - 4.5));
        let jolt = if lb > 4.55 { (-(lb - 4.55) * 6.0).exp() * ((lb - 4.55) * 20.0).sin() * 6.0 } else { 0.0 };
        let (body, lid) = trash(lid);
        let can_t = format!("translate(0 {})", (1.0 - can_in) * 260.0 + jolt);
        // a honey strike through "IDE" just before the flick
        let strike = quart_out(prog(lb, 2.7, 3.3));
        let exit = expo_in(prog(lb, 6.6, 7.0));
        fframes::svgr!(
            <g opacity={1.0 - exit}>
                {ground(&frame, 0.18)}
                {window}
                <g transform={can_t} opacity={prog(lb, 2.4, 2.6)}>{body}{lid}</g>
                <g transform={format!("translate(0 {})", -exit * 90.0)}>
                    {Slam::new(146.0, 470.0, "I FIRED", 220.0, BONE).draw(lb - 0.35, 0.0, 120.0)}
                    {Slam::new(146.0, 690.0, "MY IDE.", 220.0, BONE).draw(lb - 1.5, 0.0, 120.0)}
                    <rect x="146" y="604" width={(strike * 840.0).max(0.5)} height="22" fill={HONEY} />
                </g>
            </g>
        )
    }
}

/// Where the recording of the tiles landing sits.
const HIVE_BOX: (f32, f32, f32, f32) = (930.0, 250.0, 860.0, 538.0);

impl Scene for HiveScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let (x, y, w, h) = HIVE_BOX;
        let enter = expo_out(prog(lb, 0.0, 0.8));
        let exit = expo_in(prog(lb, 6.65, 7.0));
        fframes::svgr!(
            <g opacity={1.0 - exit}>
                {ground(&frame, 0.3)}
                <g transform={format!("translate({} 0)", (1.0 - enter) * 160.0)} opacity={enter}>
                    {framed_clip(&frame, ctx, "hive.mp4", 0.0, x, y, w, h, "THE CANVAS · RECORDED IN THE APP".to_owned())}
                </g>
                {Slam::new(146.0, 470.0, "HIRED", 200.0, BONE).draw(lb - 0.9, -160.0, 0.0)}
                {Slam::new(146.0, 680.0, "A HIVE.", 200.0, HONEY).draw(lb - 1.8, -160.0, 0.0)}
            </g>
        )
    }
}
