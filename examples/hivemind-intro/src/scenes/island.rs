//! "Ask for a view": marquee rows of what you can ask for, then the Island view, full bleed, as
//! an agent writes it around the same live terminals.

use fframes::{Duration, FFramesContext, Frame, Scene, Svgr};

use crate::beat::*;
use crate::ui::*;

beat_scene!(AskScene, 48.0, 52.0);
beat_scene!(IslandScene, 52.0, 68.0);

const ROWS: &[&str] = &[
    "YOUR VIEW · YOUR WIDGETS · YOUR TOOLS · ",
    "ASK FOR IT · AN AGENT WRITES IT · ",
    "ISLANDS · BOARDS · LISTS · MAPS · ",
    "YOUR VIEW · YOUR WIDGETS · YOUR TOOLS · ",
    "THE TERMINALS STAY LIVE · ",
    "ASK FOR IT · AN AGENT WRITES IT · ",
    "YOURS · YOURS · YOURS · YOURS · ",
];
const ROW_H: f32 = 150.0;
const ROW_SIZE: usize = 132;

impl Scene for AskScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, mut frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        // only rows the card never covers light up
        let lit = [0usize, 5, 1, 6][lb.max(0.0).floor() as usize % 4];
        let mut rows = Vec::new();
        for (i, text) in ROWS.iter().enumerate() {
            let w = measure(&mut frame, ctx, SANS, ROW_SIZE, 800, text) - 3.0 * text.len() as f32;
            let dir = if i % 2 == 0 { -1.0 } else { 1.0 };
            let speed = 300.0 + (i as f32 * 41.0) % 160.0;
            let enter = expo_out(prog(lb, i as f32 * 0.05, 0.6 + i as f32 * 0.05));
            let offset = (lb * BEAT * speed * dir).rem_euclid(w) - w + (1.0 - enter) * 900.0 * dir;
            let y = 120.0 + i as f32 * ROW_H;
            let is_lit = i == lit;
            let flash = if is_lit { pulse(lb, 5.0) } else { 0.0 };
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
            rows.push(fframes::svgr!(<g opacity={0.7 + flash * 0.3}>{copies}</g>));
        }
        let card = snap(lb - 0.3);
        fframes::svgr!(
            <g>
                {rows}
                <g transform={format!("translate({} 0)", (1.0 - card) * -700.0)}>
                    <rect x="120" y="318" width="900" height="460" fill={BG} />
                    <rect x="120" y="318" width="6" height="460" fill={HONEY} />
                    {label(160.0, 400.0, "04".to_owned(), HONEY, 24.0, "start")}
                    <text x="156" y="560" font-family={SANS} font-weight={HEAVY} font-size="118" letter-spacing="-5" fill={BONE}>"ASK FOR"</text>
                    <text x="156" y="680" font-family={SANS} font-weight={HEAVY} font-size="118" letter-spacing="-5" fill={HONEY}>"A VIEW."</text>
                </g>
            </g>
        )
    }
}

/// Seconds into the scene where the second recording (a finished turn) starts.
const FIREWORKS_AT: f32 = 12.0 * BEAT;

impl Scene for IslandScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let t = frame.seconds();
        // full bleed: the 16:10 recording fills the width, a little of the top and bottom cut
        let (name, start) = if t < FIREWORKS_AT { ("island.mp4", 0.0) } else { ("fireworks.mp4", FIREWORKS_AT) };
        let push = 1.0 + prog(lb, 0.0, 16.0) * 0.04;
        let picture = clip(&frame, ctx, name, start, 0.0, -60.0, 1920.0, 1200.0);
        // captions: what is happening in the recording, under it
        let lines: [(&str, &str, f32, f32); 2] = [
            ("IT WRITES", "THE VIEW.", 5.0, 11.6),
            ("A TURN ENDS.", "A HOUSE GOES UP.", 12.2, 16.0),
        ];
        let captions: Vec<Svgr> = lines
            .iter()
            .map(|(a, b, from, to)| {
                if lb < *from || lb >= *to {
                    return Svgr::empty();
                }
                let l = lb - from;
                let out = expo_in(prog(lb, to - 0.3, *to));
                fframes::svgr!(
                    <g opacity={1.0 - out}>
                        <rect x="110" y="742" width={(snap(l) * 860.0).max(0.5)} height="196" fill={BG} fill-opacity="0.88" />
                        {Slam::new(140.0, 826.0, *a, 76.0, BONE).draw(l, -60.0, 0.0)}
                        {Slam::new(140.0, 906.0, *b, 76.0, HONEY).draw(l - 0.4, -60.0, 0.0)}
                    </g>
                )
            })
            .collect();
        // the prompt typed in the recording, over its first beats
        let ask_out = expo_in(prog(lb, 4.4, 4.8));
        let ask_in = snap(lb);
        let ask = fframes::svgr!(
            <g opacity={(1.0 - ask_out) * prog(lb, 0.0, 0.1)} transform={format!("translate({} 0)", (1.0 - ask_in) * -80.0)}>
                <rect x="110" y="700" width="1130" height="238" fill={BG} fill-opacity="0.9" />
                <rect x="110" y="700" width="5" height="238" fill={HONEY} />
                {label(150.0, 752.0, "THE PROMPT, IN A LIVE TERMINAL".to_owned(), GREY, 19.0, "start")}
                <text x="148" y="830" font-family={MONO} font-weight="700" font-size="42" fill={HONEY}>"›"</text>
                <text x="190" y="830" font-family={MONO} font-weight="500" font-size="42" fill={BONE}>"make me a view where"</text>
                <text x="190" y="892" font-family={MONO} font-weight="500" font-size="42" fill={BONE}>"every project is an island"</text>
            </g>
        );
        fframes::svgr!(
            <g>
                <g transform={format!("translate(960 540) scale({push}) translate(-960 -540)")}>{picture}</g>
                <defs>
                    <linearGradient id="island-top" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0" stop-color={BG} stop-opacity="0.92" />
                        <stop offset="1" stop-color={BG} stop-opacity="0" />
                    </linearGradient>
                    <linearGradient id="island-bottom" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0" stop-color={BG} stop-opacity="0" />
                        <stop offset="1" stop-color={BG} stop-opacity="0.92" />
                    </linearGradient>
                </defs>
                <rect width="1920" height="190" fill="url(#island-top)" />
                <rect y="900" width="1920" height="180" fill="url(#island-bottom)" />
                {ask}
                {captions}
                {label(1854.0, 990.0, "THE ISLAND VIEW · RECORDED IN THE APP".to_owned(), BONE, 17.0, "end")}
            </g>
        )
    }
}
