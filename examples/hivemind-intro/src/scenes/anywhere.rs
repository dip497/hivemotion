//! Your machines and your phone, joined through a server you run yourself.

use fframes::{Duration, FFramesContext, Frame, Scene, Svgr};

use crate::beat::*;
use crate::shaders::ground;
use crate::ui::*;

beat_scene!(AnywhereScene, 104.0, 112.0);

const Y: f32 = 640.0;
/// (x, name, note)
const NODES: &[(f32, &str, &str)] = &[
    (360.0, "YOUR COMPUTER", "THE APP · YOUR AGENTS"),
    (960.0, "YOUR SERVER", "hive-net serve --all"),
    (1560.0, "YOUR PHONE", "PAIRED WITH A QR CODE"),
];

fn laptop(x: f32, y: f32) -> Svgr<'static> {
    fframes::svgr!(
        <g fill="none" stroke={BONE} stroke-width="3" stroke-linejoin="round">
            <rect x={x - 110.0} y={y - 120.0} width="220" height="140" rx="8" />
            <path d={format!("M{} {} H{} L{} {} H{} Z", x - 140.0, y + 30.0, x + 140.0, x + 156.0, y + 50.0, x - 156.0)} />
            <rect x={x - 92.0} y={y - 102.0} width="80" height="48" fill={SURFACE} stroke={LINE2} stroke-width="2" />
            <rect x={x - 4.0} y={y - 102.0} width="96" height="48" fill={SURFACE} stroke={HONEY} stroke-width="2" />
            <rect x={x - 92.0} y={y - 44.0} width="184" height="46" fill={SURFACE} stroke={LINE2} stroke-width="2" />
        </g>
    )
}

fn server(x: f32, y: f32) -> Svgr<'static> {
    let units: Vec<Svgr> = (0..3)
        .map(|i| {
            let uy = y - 110.0 + i as f32 * 52.0;
            fframes::svgr!(
                <g>
                    <rect x={x - 90.0} y={uy} width="180" height="40" rx="6" fill={SURFACE} stroke={BONE} stroke-width="3" />
                    <circle cx={x - 64.0} cy={uy + 20.0} r="5" fill={HONEY} />
                    <rect x={x - 40.0} y={uy + 17.0} width="100" height="6" rx="3" fill={LINE2} />
                </g>
            )
        })
        .collect();
    fframes::svgr!(<g>{units}</g>)
}

fn phone(x: f32, y: f32) -> Svgr<'static> {
    fframes::svgr!(
        <g>
            <rect x={x - 62.0} y={y - 130.0} width="124" height="200" rx="18" fill={SURFACE} stroke={BONE} stroke-width="3" />
            <rect x={x - 46.0} y={y - 104.0} width="92" height="22" rx="4" fill={LINE2} />
            <rect x={x - 46.0} y={y - 74.0} width="92" height="22" rx="4" fill={HONEY} />
            <rect x={x - 46.0} y={y - 44.0} width="92" height="22" rx="4" fill={LINE2} />
            <rect x={x - 46.0} y={y - 14.0} width="92" height="22" rx="4" fill={LINE2} />
            <rect x={x - 20.0} y={y + 46.0} width="40" height="5" rx="2.5" fill={GREY} />
        </g>
    )
}

impl Scene for AnywhereScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let mut parts = Vec::new();
        for (i, (x, name, note)) in NODES.iter().enumerate() {
            let at = 0.8 + i as f32 * 0.6;
            let s = soft(lb - at);
            let icon = match i {
                0 => laptop(*x, Y - 20.0),
                1 => server(*x, Y),
                _ => phone(*x, Y),
            };
            let note_color = if i == 1 { HONEY } else { GREY };
            let note_weight = if i == 1 { "700" } else { "500" };
            parts.push(fframes::svgr!(
                <g opacity={prog(lb, at, at + 0.12)} transform={format!("translate(0 {})", (1.0 - s) * 90.0)}>
                    {icon}
                    <text x={*x} y={Y + 140.0} text-anchor="middle" font-family={MONO} font-weight="700" font-size="24" letter-spacing="3" fill={BONE}>{*name}</text>
                    <text x={*x} y={Y + 182.0} text-anchor="middle" font-family={MONO} font-weight={note_weight} font-size="22" letter-spacing="1" fill={note_color}>{*note}</text>
                </g>
            ));
        }
        // the links draw on, then packets travel both ways, one per beat
        let link = cubic_in_out(prog(lb, 2.4, 3.6));
        let mut packets = Vec::new();
        if lb > 3.6 {
            for k in 0..4 {
                let f = ((lb - 3.6) * 0.5 + k as f32 * 0.25).fract();
                let (a, b) = if k % 2 == 0 { (520.0, 1400.0) } else { (1400.0, 520.0) };
                let px = lerp(a, b, f);
                if (px - 960.0).abs() > 120.0 {
                    packets.push(fframes::svgr!(<rect x={px - 7.0} y={Y - 47.0} width="14" height="14" fill={HONEY} transform={format!("rotate(45 {px} {})", Y - 40.0)} />));
                }
            }
        }
        let exit = expo_in(prog(lb, 7.7, 8.0));
        fframes::svgr!(
            <g opacity={1.0 - exit}>
                {ground(&frame, 0.5, 0.6, 0.24, 0.14)}
                {Slam::new(146.0, 230.0, "YOUR MACHINES.", 100.0, BONE).draw(lb, 0.0, 70.0)}
                {Slam::new(146.0, 330.0, "YOUR PHONE.", 100.0, HONEY).draw(lb - 0.5, 0.0, 70.0)}
                <path d={format!("M520 {y} H{}", 520.0 + 300.0 * link, y = Y - 40.0)} stroke={LINE2} stroke-width="2" stroke-dasharray="10 8" />
                <path d={format!("M1400 {y} H{}", 1400.0 - 300.0 * link, y = Y - 40.0)} stroke={LINE2} stroke-width="2" stroke-dasharray="10 8" />
                {packets}
                {parts}
                {label(1774.0, 230.0, "NO ACCOUNT".to_owned(), FG2, 20.0, "end")}
                {label(1774.0, 266.0, "NOT OUR CLOUD".to_owned(), FG2, 20.0, "end")}
            </g>
        )
    }
}
