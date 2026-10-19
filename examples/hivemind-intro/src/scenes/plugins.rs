//! The one full-bleed honey card: everything is a plugin, and how many are on HiveHub today.

use fframes::{Duration, FFramesContext, Frame, Scene, Svgr};

use crate::beat::*;
use crate::facts;
use crate::ui::*;

beat_scene!(PluginsScene, 112.0, 120.0);

const KINDS: &[&str] = &["AGENTS", "VIEWS", "WIDGETS", "TOOLS"];

impl Scene for PluginsScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        // the card wipes up from the bottom edge
        let wipe = expo_out(prog(lb, 0.0, 0.6));
        let top = 1080.0 * (1.0 - wipe);
        let count = (facts::PLUGINS as f32 * expo_out(prog(lb, 0.4, 2.0))).round() as u64;
        let kinds: Vec<Svgr> = KINDS
            .iter()
            .enumerate()
            .map(|(i, k)| {
                let at = 2.0 + i as f32 * 0.35;
                let s = snap(lb - at);
                let y = 340.0 + i as f32 * 92.0;
                fframes::svgr!(
                    <g opacity={prog(lb, at, at + 0.1)} transform={format!("translate({} 0)", (1.0 - s) * 60.0)}>
                        <path d={hex_path(1236.0, y - 24.0, 24.0)} fill={INK} />
                        <text x="1286" y={y} font-family={SANS} font-weight="800" font-size="72" letter-spacing="-2" fill={INK}>{*k}</text>
                    </g>
                )
            })
            .collect();
        let line = prog(lb, 4.5, 4.9);
        let exit = expo_in(prog(lb, 7.7, 8.0));
        fframes::svgr!(
            <g opacity={1.0 - exit}>
                <rect x="0" y={top} width="1920" height="1080" fill={HONEY} />
                <g opacity={prog(lb, 0.3, 0.5)}>
                    <rect x="130" y="130" width="1660" height="2" fill={INK} opacity="0.5" />
                    <text x="130" y="112" font-family={MONO} font-weight="700" font-size="20" letter-spacing="3" fill={INK}>"HIVEHUB · THE PLUGIN REGISTRY"</text>
                    <text x="1790" y="112" text-anchor="end" font-family={MONO} font-weight="700" font-size="20" letter-spacing="3" fill={INK}>"19 OCT 2026"</text>
                    <text x="110" y="700" font-family={SANS} font-weight={HEAVY} font-size="560" letter-spacing="-30" fill={INK}>{count.to_string()}</text>
                    <text x="130" y="800" font-family={SANS} font-weight={HEAVY} font-size="84" letter-spacing="-3" fill={INK}>"PLUGINS ON HIVEHUB"</text>
                    {kinds}
                </g>
                <g opacity={line} transform={format!("translate(0 {})", (1.0 - line) * 30.0)}>
                    <rect x="130" y="868" width="1660" height="2" fill={INK} opacity="0.5" />
                    <text x="130" y="936" font-family={MONO} font-weight="700" font-size="34" letter-spacing="2" fill={INK}>"EVERYTHING IS A PLUGIN. EVEN THE AGENTS."</text>
                </g>
            </g>
        )
    }
}
