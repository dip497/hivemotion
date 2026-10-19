//! The second drop: frames of this video fly out of the hex floor, next to the numbers of its
//! own render.

use fframes::{Duration, FFramesContext, Frame, Scene, Svgr};

use crate::beat::*;
use crate::facts;
use crate::shaders::floor;
use crate::ui::*;

beat_scene!(RenderScene, 132.0, 140.0);

/// Stills of this video's own scenes (scripts/stills.sh writes them).
const STILLS: usize = 10;

impl Scene for RenderScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, mut frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let t = lb * BEAT;
        // cards on a stream towards the camera, drawn far ones first
        let mut cards: Vec<(f32, Svgr)> = Vec::new();
        for k in 0..14 {
            let phase = (t * 0.42 + k as f32 / 14.0).fract();
            let z = lerp(7.0, 0.38, phase);
            let side = if k % 2 == 0 { -1.0 } else { 1.0 };
            let xw = side * (600.0 + hash(k as f32 * 3.1) * 520.0);
            let yw = -340.0 + hash(k as f32 * 7.7) * 300.0;
            let (x, y) = (960.0 + xw / z, 540.0 + yw / z);
            let w = 520.0 / z;
            let h = w * 9.0 / 16.0;
            let o = prog(phase, 0.0, 0.2) * (1.0 - prog(phase, 0.85, 1.0)) * prog(lb, 0.0, 0.3);
            let name = format!("fly_{}.jpg", k % STILLS);
            if let Some(img) = ctx.get_image(&name) {
                cards.push((
                    z,
                    fframes::svgr!(
                        <g opacity={o}>
                            <image href={img.href()} x={x - w / 2.0} y={y - h / 2.0} width={w} height={h} />
                            <rect x={x - w / 2.0} y={y - h / 2.0} width={w} height={h} fill="none" stroke={BONE} stroke-opacity="0.35" stroke-width="1" />
                        </g>
                    ),
                ));
            }
        }
        cards.sort_by(|a, b| b.0.total_cmp(&a.0));
        let cards: Vec<Svgr> = cards.into_iter().map(|(_, c)| c).collect();

        let title_w = measure(&mut frame, ctx, SANS, 120, HEAVY, "THIS VIDEO.") - 5.4 * 11.0;
        let count = (facts::FRAMES as f32 * expo_out(prog(lb, 0.0, 2.6))).round() as u64;
        let secs = facts::RENDER_SECONDS * expo_out(prog(lb, 3.0, 5.0));
        let a = snap(lb);
        let b = snap(lb - 3.0);
        let realtime = TOTAL_SECONDS / facts::RENDER_SECONDS.max(0.001);
        let c = prog(lb, 5.5, 5.8);
        let exit = expo_in(prog(lb, 7.75, 8.0));
        let speed = if realtime >= 1.0 {
            format!("{realtime:.1}× REAL TIME")
        } else {
            format!("{:.1}× SLOWER THAN REAL TIME", 1.0 / realtime.max(0.001))
        };
        fframes::svgr!(
            <g opacity={1.0 - exit}>
                {floor(&frame, 6.0, 0.5, 0.85 + pulse(lb, 6.0) * 0.35)}
                {cards}
                <defs>
                    <linearGradient id="render-fade" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0" stop-color={BG} stop-opacity="0" />
                        <stop offset="1" stop-color={BG} stop-opacity="0.92" />
                    </linearGradient>
                </defs>
                <rect x="0" y="600" width="1920" height="480" fill="url(#render-fade)" />
                <g transform={format!("translate({} 0)", (1.0 - snap(lb - 0.5)) * -200.0)} opacity={prog(lb, 0.5, 0.6)}>
                    <rect x="120" y="130" width={title_w + 60.0} height="150" fill={HONEY} />
                    <text x="150" y="246" font-family={SANS} font-weight={HEAVY} font-size="120" letter-spacing="-5.4" fill={INK}>"THIS VIDEO."</text>
                </g>
                <g transform={format!("translate(0 {})", (1.0 - a) * 70.0)} opacity={prog(lb, 0.0, 0.08)}>
                    <text x="150" y="820" font-family={SANS} font-weight={HEAVY} font-size="190" letter-spacing="-8" fill={BONE}>{thousands(count)}</text>
                    <text x="160" y="880" font-family={MONO} font-weight="700" font-size="28" letter-spacing="5" fill={BONE}>"FRAMES · 1920×1080 · 60 FPS"</text>
                </g>
                <g transform={format!("translate(0 {})", (1.0 - b) * 70.0)} opacity={prog(lb, 3.0, 3.08)}>
                    <text x="1770" y="820" text-anchor="end" font-family={SANS} font-weight={HEAVY} font-size="190" letter-spacing="-8" fill={HONEY}>{format!("{secs:.1}s")}</text>
                    <text x="1770" y="880" text-anchor="end" font-family={MONO} font-weight="700" font-size="28" letter-spacing="5" fill={BONE}>{format!("TO RENDER · {}", facts::RENDER_MACHINE)}</text>
                </g>
                <g opacity={c}>
                    <rect x="1250" y="170" width="520" height="64" fill={HONEY} />
                    <text x="1510" y="214" text-anchor="middle" font-family={MONO} font-weight="700" font-size="26" letter-spacing="3" fill={INK}>{speed}</text>
                </g>
            </g>
        )
    }
}
