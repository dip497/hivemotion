//! "When I want something new, I ask": a view, a widget and a tool, each asked for and then live.

use fframes::{Duration, FFramesContext, Frame, Scene, Svgr};

use crate::beat::*;
use crate::shaders::ground;
use crate::ui::*;

beat_scene!(AskScene, 28.0, 33.0);
beat_scene!(ViewScene, 33.0, 41.0);
beat_scene!(WidgetScene, 41.0, 50.0);
beat_scene!(ToolScene, 50.0, 56.0);

const ROWS: &[&str] = &[
    "A VIEW · A WIDGET · A TOOL · ",
    "SOMETHING NEW · ",
    "ASK FOR IT · AN AGENT BUILDS IT · ",
    "A VIEW · A WIDGET · A TOOL · ",
    "YOUR WORKSPACE · YOUR WAY · ",
    "ASK FOR IT · AN AGENT BUILDS IT · ",
    "SOMETHING NEW · ",
];
const ROW_H: f32 = 150.0;
const ROW_SIZE: usize = 132;
/// The beat inside the ask scene where the voice says "I ask".
const I_ASK: f32 = 2.9;

impl Scene for AskScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, mut frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        // only rows the card never covers light up
        let lit = [0usize, 5, 1, 6, 0][lb.max(0.0).floor() as usize % 5];
        let mut rows = Vec::new();
        for (i, text) in ROWS.iter().enumerate() {
            let w = measure(&mut frame, ctx, SANS, ROW_SIZE, 800, text) - 3.0 * text.len() as f32;
            let dir = if i % 2 == 0 { -1.0 } else { 1.0 };
            let speed = 300.0 + (i as f32 * 41.0) % 160.0;
            let enter = expo_out(prog(lb, i as f32 * 0.05, 0.6 + i as f32 * 0.05));
            let offset = (lb * BEAT * speed * dir).rem_euclid(w) - w + (1.0 - enter) * 900.0 * dir;
            let y = 120.0 + i as f32 * ROW_H;
            let is_lit = i == lit;
            let copies: Vec<Svgr> = (0..4)
                .map(|k| {
                    let x = offset + k as f32 * w;
                    let t = (*text).to_owned();
                    if is_lit {
                        fframes::svgr!(<text x={x} y={y} font-family={SANS} font-weight="800" font-size={ROW_SIZE} letter-spacing="-3" fill={HONEY}>{t}</text>)
                    } else {
                        fframes::svgr!(<text x={x} y={y} font-family={SANS} font-weight="800" font-size={ROW_SIZE} letter-spacing="-3" fill="none" stroke="#3f4246" stroke-width="1.6">{t}</text>)
                    }
                })
                .collect();
            rows.push(fframes::svgr!(<g opacity={0.7 + if is_lit { pulse(lb, 5.0) * 0.3 } else { 0.0 }}>{copies}</g>));
        }
        let card = snap(lb - 0.2);
        let first = lb < I_ASK;
        let exit = expo_in(prog(lb, 4.7, 5.0));
        fframes::svgr!(
            <g opacity={1.0 - exit}>
                {rows}
                <g transform={format!("translate({} 0)", (1.0 - card) * -700.0)}>
                    <rect x="120" y="318" width="900" height="460" fill={BG} />
                    <rect x="120" y="318" width="6" height="460" fill={HONEY} />
                    <g opacity={if first { 1.0 } else { 0.0 }}>
                        <text x="156" y="520" font-family={SANS} font-weight={HEAVY} font-size="118" letter-spacing="-5" fill={BONE}>"SOMETHING"</text>
                        <text x="156" y="640" font-family={SANS} font-weight={HEAVY} font-size="118" letter-spacing="-5" fill={BONE}>"NEW?"</text>
                    </g>
                    {Slam::new(150.0, 620.0, "I ASK.", 210.0, HONEY).draw(lb - I_ASK, 0.0, 90.0)}
                </g>
            </g>
        )
    }
}

/// The prompt, typed in a terminal plate, over the first beats of an ask.
fn prompt(l: f32, text: &'static str, out_at: f32) -> Svgr<'static> {
    let n = text.chars().count();
    let typed = ((l / 1.3) * n as f32).min(n as f32) as usize;
    let shown: String = text.chars().take(typed).collect();
    let caret_on = typed < n || (l * 2.0).fract() > 0.5;
    let caret_x = 196.0 + typed as f32 * 42.0 * 0.6;
    let out = expo_in(prog(l, out_at - 0.3, out_at));
    let w = 140.0 + n as f32 * 42.0 * 0.6;
    fframes::svgr!(
        <g opacity={(1.0 - out) * prog(l, 0.0, 0.1)} transform={format!("translate({} 0)", (1.0 - snap(l)) * -80.0)}>
            <rect x="110" y="440" width={w} height="190" fill={BG} fill-opacity="0.94" stroke={LINE2} stroke-width="1.5" />
            <rect x="110" y="440" width="5" height="190" fill={HONEY} />
            {label(150.0, 494.0, "ASKED IN A TERMINAL TILE".to_owned(), GREY, 19.0, "start")}
            <text x="148" y="572" font-family={MONO} font-weight="700" font-size="42" fill={HONEY}>"›"</text>
            <text x="196" y="572" font-family={MONO} font-weight="500" font-size="42" fill={BONE}>{shown}</text>
            <rect x={caret_x + 2.0} y="538" width="20" height="44" fill={HONEY} opacity={if caret_on { 1.0 } else { 0.0 }} />
        </g>
    )
}

/// The word the voice says as a result arrives, on a plate.
fn tag(l: f32, x: f32, y: f32, text: &'static str, out: f32) -> Svgr<'static> {
    if l < 0.0 {
        return Svgr::empty();
    }
    let w = text.chars().count() as f32 * 92.0 + 90.0;
    fframes::svgr!(
        <g opacity={1.0 - out}>
            <rect x={x - 36.0} y={y - 158.0} width={(snap(l) * w).max(0.5)} height="206" fill={BG} fill-opacity="0.9" />
            <rect x={x - 36.0} y={y - 158.0} width="6" height="206" fill={HONEY} />
            {Slam::new(x, y, text, 160.0, HONEY).draw(l, -80.0, 0.0)}
        </g>
    )
}

fn gradients(id: &'static str) -> Svgr<'static> {
    let top = format!("{id}-top");
    let bottom = format!("{id}-bottom");
    fframes::svgr!(
        <g>
            <defs>
                <linearGradient id={top.clone()} x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0" stop-color={BG} stop-opacity="0.85" />
                    <stop offset="1" stop-color={BG} stop-opacity="0" />
                </linearGradient>
                <linearGradient id={bottom.clone()} x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0" stop-color={BG} stop-opacity="0" />
                    <stop offset="1" stop-color={BG} stop-opacity="0.85" />
                </linearGradient>
            </defs>
            <rect width="1920" height="170" fill={format!("url(#{top})")} />
            <rect y="880" width="1920" height="200" fill={format!("url(#{bottom})")} />
        </g>
    )
}

/// The beat inside the view scene where the islands come up.
const VIEW_AT: f32 = 2.0;
const CLOSE_AT: f32 = 5.0;

impl Scene for ViewScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let l = lb - VIEW_AT;
        let still = |name: &str, x: f32, y: f32, s: f32, o: f32| -> Svgr<'a> {
            match ctx.get_image(name) {
                Some(img) => {
                    let t = format!("translate({} {}) scale({s}) translate(-960 -540)", 960.0 + x, 540.0 + y);
                    fframes::svgr!(<g transform={t} opacity={o}><image href={img.href()} x="0" y="-60" width="1920" height="1200" preserveAspectRatio="xMidYMid slice" /></g>)
                }
                None => Svgr::empty(),
            }
        };
        // the islands rise into place, the camera keeps drifting, then it moves in close
        let rise = spring(l.max(0.0) * BEAT, 120.0, 16.0);
        let wide = still(
            "island-wide.jpg",
            -40.0 * prog(l, 0.0, 6.0),
            (1.0 - rise) * 160.0,
            1.12 - 0.08 * rise.min(1.0) + 0.04 * prog(l, 0.0, 6.0),
            prog(l, 0.0, 0.2),
        );
        let close_o = cubic_in_out(prog(lb, CLOSE_AT, CLOSE_AT + 0.6));
        let close = still("island-close.jpg", 30.0 - 60.0 * prog(lb, CLOSE_AT, 8.0), 0.0, 1.04 + 0.05 * prog(lb, CLOSE_AT, 8.0), close_o);
        let exit = expo_in(prog(lb, 7.7, 8.0));
        fframes::svgr!(
            <g opacity={1.0 - exit}>
                {ground(&frame, 0.12)}
                {if l >= 0.0 { wide } else { Svgr::empty() }}
                {close}
                {if l >= 0.0 { gradients("view") } else { Svgr::empty() }}
                {prompt(lb, "make me a view where every project is an island", VIEW_AT)}
                {tag(l, 146.0, 940.0, "A VIEW.", 0.0)}
                <g opacity={prog(l, 0.0, 0.3)}>{label(1854.0, 966.0, "THE ISLAND VIEW · IN THE APP".to_owned(), BONE, 17.0, "end")}</g>
            </g>
        )
    }
}

/// Seconds into the widget clip where the CI light turns green (W2 13.0 s, cut from 10.7 s).
const GREEN_AT: f32 = 2.3;
const WIDGET_PROMPT_OUT: f32 = 1.8;

impl Scene for WidgetScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let t = frame.seconds();
        let picture = clip(&frame, ctx, "widget.mp4", 0.0, 0.0, 0.0, 1920.0, 1080.0);
        let green = t - GREEN_AT;
        // a ring of light where the widget turns green
        let ring = if green >= 0.0 {
            let r = 40.0 + expo_out(prog(green, 0.0, 0.8)) * 260.0;
            fframes::svgr!(<circle cx="1592" cy="76" r={r} fill="none" stroke={OK} stroke-width="4" opacity={1.0 - prog(green, 0.0, 0.8)} />)
        } else {
            Svgr::empty()
        };
        let exit = expo_in(prog(lb, 8.7, 9.0));
        fframes::svgr!(
            <g opacity={1.0 - exit}>
                {picture}
                {ring}
                <rect width="1920" height="1080" fill={BG} opacity={0.55 * (1.0 - prog(lb, WIDGET_PROMPT_OUT - 0.3, WIDGET_PROMPT_OUT))} />
                {prompt(lb, "a widget for the CI light", WIDGET_PROMPT_OUT)}
                {tag(green / BEAT, 1010.0, 640.0, "A WIDGET.", 0.0)}
            </g>
        )
    }
}

impl Scene for ToolScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let picture = clip(&frame, ctx, "tool.mp4", 0.0, 0.0, 0.0, 1920.0, 1080.0);
        let push = 1.0 + 0.03 * prog(lb, 0.0, 6.0);
        let exit = expo_in(prog(lb, 5.7, 6.0));
        fframes::svgr!(
            <g opacity={1.0 - exit}>
                <g transform={format!("translate(960 540) scale({push}) translate(-960 -540)")}>{picture}</g>
                {gradients("tool")}
                {tag(lb - 0.75, 146.0, 940.0, "A TOOL.", 0.0)}
                {label(1854.0, 966.0, "A LOG TAIL · IN THE APP".to_owned(), BONE, 17.0, "end")}
            </g>
        )
    }
}
