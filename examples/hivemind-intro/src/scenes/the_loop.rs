//! The breakdown: the loop the agent ran while it made this video, with the real output of this
//! project's command line, then the confession.

use fframes::{Duration, FFramesContext, Frame, Scene, Svgr};

use crate::beat::*;
use crate::shaders::ground;
use crate::ui::*;

beat_scene!(LoopScene, 120.0, 132.0);

const NODES: &[&str] = &["WRITE", "TIMELINE", "INSPECT", "STRIP", "FRAME", "RENDER"];
/// Beats inside the scene where a new card comes in.
const CARD_BEATS: &[f32] = &[1.5, 3.0, 4.5];
const CONFESSION_AT: f32 = 6.0;

struct Card {
    node: usize,
    command: &'static str,
    lines: &'static [(&'static str, &'static str)],
}

/// Output of the commands, as they printed for this video (see facts.md).
const CARDS: &[Card] = &[
    Card {
        node: 1,
        command: "cargo run -- timeline",
        lines: &[
            ("1920x1080 @ 60 fps, 4394 frames (73.23s)", BONE),
            ("#0  FiredScene    0.00s..1.77s", GREY),
            ("#4  CanvasScene  14.12s..21.18s", GREY),
            ("#6  IslandScene  22.93s..30.00s", GREY),
        ],
    },
    Card {
        node: 2,
        command: "cargo run -- inspect",
        lines: &[
            ("checked 323 frames: 7 findings", BONE),
            ("Warning [AskScene]: text is cut off", GREY),
            ("  by the canvas edge", GREY),
        ],
    },
    Card {
        node: 3,
        command: "cargo run -- strip all -n 24",
        lines: &[
            ("all.png (24 frames: 0, 191, 382, 573,", BONE),
            ("  764, 955, 1146, 1337, 1528, ...)", GREY),
        ],
    },
];

const RING_X: f32 = 1380.0;
const RING_Y: f32 = 600.0;
const RING_R: f32 = 250.0;

impl Scene for LoopScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let bg = ground(&frame, RING_X / 1920.0, RING_Y / 1080.0, 0.3 * prog(lb, 0.0, 1.0) + 0.1, 0.1);
        if lb >= CONFESSION_AT {
            return fframes::svgr!(<g>{bg}{confession(lb - CONFESSION_AT)}</g>);
        }
        let active = CARD_BEATS.iter().rposition(|b| lb >= *b);
        let card_svg = match active {
            Some(i) => card(&CARDS[i], lb - CARD_BEATS[i]),
            None => Svgr::empty(),
        };
        let lit = active.map(|i| CARDS[i].node).or(if lb > 0.6 { Some(0) } else { None });
        let exit = expo_in(prog(lb, CONFESSION_AT - 0.3, CONFESSION_AT));
        fframes::svgr!(
            <g>
                {bg}
                <g opacity={1.0 - exit}>
                    {Slam::new(146.0, 250.0, "MADE IN", 100.0, BONE).draw(lb, 0.0, 60.0)}
                    {Slam::new(146.0, 350.0, "THE AGENT'S LOOP.", 100.0, HONEY).draw(lb - 0.5, 0.0, 60.0)}
                    {ring(lb, lit)}
                    {card_svg}
                </g>
            </g>
        )
    }
}

fn ring(lb: f32, lit: Option<usize>) -> Svgr<'static> {
    let draw = cubic_in_out(prog(lb, 0.2, 1.6));
    let circ = std::f32::consts::TAU * RING_R;
    let dash = format!("{} {}", circ * draw, circ);
    let mut nodes = Vec::new();
    for (i, name) in NODES.iter().enumerate() {
        let a = -std::f32::consts::FRAC_PI_2 + i as f32 / NODES.len() as f32 * std::f32::consts::TAU;
        let (x, y) = (RING_X + a.cos() * RING_R, RING_Y + a.sin() * RING_R);
        let at = 0.4 + i as f32 * 0.2;
        let on = lit == Some(i);
        let lx = RING_X + a.cos() * (RING_R + 44.0);
        let ly = RING_Y + a.sin() * (RING_R + 44.0) + 8.0;
        let anchor = if a.cos() > 0.3 { "start" } else if a.cos() < -0.3 { "end" } else { "middle" };
        let r = if on { 22.0 } else { 15.0 };
        nodes.push(fframes::svgr!(
            <g opacity={prog(lb, at, at + 0.1)}>
                <path d={hex_path(x, y, r * snap(lb - at).max(0.05))} fill={if on { HONEY } else { BG }} stroke={if on { HONEY } else { BONE }} stroke-width="2" />
                <text x={lx} y={ly} text-anchor={anchor} font-family={MONO} font-weight="700" font-size="22" letter-spacing="3"
                      fill={if on { HONEY } else { BONE }}>{*name}</text>
            </g>
        ));
    }
    // a comet laps the ring once a bar
    let comet: Vec<Svgr> = if lb > 1.6 {
        (0..10)
            .map(|k| {
                let t = (lb - 1.6) / 4.0 - k as f32 * 0.008;
                let a = -std::f32::consts::FRAC_PI_2 + t * std::f32::consts::TAU;
                let (x, y) = (RING_X + a.cos() * RING_R, RING_Y + a.sin() * RING_R);
                fframes::svgr!(<circle cx={x} cy={y} r={7.0 - k as f32 * 0.5} fill={HONEY} opacity={1.0 - k as f32 / 10.0} />)
            })
            .collect()
    } else {
        Vec::new()
    };
    let center = prog(lb, 1.2, 1.6);
    fframes::svgr!(
        <g>
            <circle cx={RING_X} cy={RING_Y} r={RING_R} fill="none" stroke={GREY} stroke-width="1.5"
                    stroke-dasharray={dash} transform={format!("rotate(-90 {RING_X} {RING_Y})")} />
            <circle cx={RING_X} cy={RING_Y} r={RING_R - 32.0} fill="none" stroke={LINE} stroke-width="1" stroke-dasharray="4 8" opacity={draw} />
            {comet}
            {nodes}
            <g opacity={center}>
                {mark(RING_X - 36.0, RING_Y - 92.0, 72.0, HONEY, BONE, 0.45)}
                <text x={RING_X} y={RING_Y + 40.0} text-anchor="middle" font-family={MONO} font-weight="700" font-size="22" letter-spacing="3" fill={BONE}>"AN AGENT"</text>
                <text x={RING_X} y={RING_Y + 72.0} text-anchor="middle" font-family={MONO} font-weight="500" font-size="18" letter-spacing="3" fill={GREY}>"IN A HIVEMIND TILE"</text>
            </g>
        </g>
    )
}

fn card(card: &Card, l: f32) -> Svgr<'static> {
    let (x, y, w) = (150.0, 470.0, 760.0);
    let enter = snap(l);
    let typed = ((l / 0.5) * card.command.len() as f32).min(card.command.len() as f32) as usize;
    let command: String = card.command.chars().take(typed).collect();
    let rows: Vec<Svgr> = card
        .lines
        .iter()
        .enumerate()
        .map(|(i, (text, color))| {
            let o = prog(l, 0.5 + i as f32 * 0.08, 0.55 + i as f32 * 0.08);
            fframes::svgr!(<text x={x + 28.0} y={y + 112.0 + i as f32 * 38.0} font-family={MONO} font-weight="500" font-size="23" fill={*color} opacity={o}>{*text}</text>)
        })
        .collect();
    let h = 90.0 + card.lines.len() as f32 * 38.0 + 24.0;
    fframes::svgr!(
        <g opacity={prog(l, 0.0, 0.08)} transform={format!("translate({} 0)", (1.0 - enter) * -80.0)}>
            <rect x={x} y={y} width={w} height={h} fill={SURFACE} fill-opacity="0.94" stroke={LINE2} stroke-width="1.5" />
            <rect x={x} y={y} width="4" height={h} fill={HONEY} />
            <text x={x + 28.0} y={y + 52.0} font-family={MONO} font-weight="700" font-size="25" fill={HONEY}>"$"</text>
            <text x={x + 58.0} y={y + 52.0} font-family={MONO} font-weight="500" font-size="25" fill={BONE}>{command}</text>
            <rect x={x + 28.0} y={y + 70.0} width={w - 56.0} height="1" fill={LINE2} />
            {rows}
        </g>
    )
}

/// THIS VIDEO WAS / MADE BY AN AGENT / IN HIVEMIND.
fn confession(l: f32) -> Svgr<'static> {
    let c = prog(l, 2.6, 3.0);
    let collapse = expo_in(prog(l, 5.6, 6.0));
    let s = 1.0 - collapse * 0.15;
    fframes::svgr!(
        <g opacity={1.0 - collapse} transform={format!("translate(960 540) scale({s}) translate(-960 -540)")}>
            {Slam::new(146.0, 420.0, "THIS VIDEO WAS", 132.0, BONE).draw(l, 0.0, 160.0)}
            {Slam::new(146.0, 560.0, "MADE BY AN AGENT", 132.0, BONE).draw(l - 0.6, 0.0, 160.0)}
            {Slam::new(146.0, 700.0, "IN HIVEMIND.", 132.0, HONEY).draw(l - 1.2, 0.0, 160.0)}
            <text x="152" y="790" font-family={MONO} font-weight="500" font-size="28" letter-spacing="1" fill={FG2} opacity={c}>"Written in Rust with fframes. No timeline editor was opened."</text>
        </g>
    )
}
