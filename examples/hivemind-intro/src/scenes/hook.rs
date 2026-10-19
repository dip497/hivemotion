//! The hook. "I FIRED MY IDE." while an editor window is struck through and flicked off the
//! screen, then "HIRED A HIVE." as the cells of a hive snap together.

use fframes::{Duration, FFramesContext, Frame, Scene, Svgr};

use crate::beat::*;
use crate::shaders::ground;
use crate::ui::*;

beat_scene!(FiredScene, 0.0, 4.0);
beat_scene!(HiveScene, 4.0, 8.0);

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

impl Scene for FiredScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        // the window is there on the first frame, then thrown off with a quarter turn
        let throw = prog(lb, 2.3, 3.3);
        let fly = expo_in(throw);
        let (wx, wy) = (1190.0 + fly * 820.0, 330.0 - (throw * std::f32::consts::PI).sin() * 160.0 + fly * 420.0);
        let rot = fly * 38.0 + (1.0 - snap(lb)) * -4.0;
        let ws = 1.0 - fly * 0.35 + (1.0 - snap(lb)) * 0.06;
        let window = fframes::svgr!(
            <g transform={format!("translate({} {}) rotate({rot}) scale({ws}) translate(-280 -210)", wx + 280.0, wy + 210.0)}
               opacity={1.0 - prog(throw, 0.85, 1.0)}>
                {editor()}
            </g>
        );
        // a honey strike through "IDE" just before the throw
        let strike = quart_out(prog(lb, 1.5, 2.2));
        let exit = expo_in(prog(lb, 3.6, 4.0));
        fframes::svgr!(
            <g>
                {ground(&frame, 0.72, 0.5, 0.32, 0.18)}
                {window}
                <g opacity={1.0 - exit} transform={format!("translate(0 {})", -exit * 90.0)}>
                    {Slam::new(146.0, 470.0, "I FIRED", 220.0, BONE).draw(lb + 0.12, 0.0, 120.0)}
                    {Slam::new(146.0, 690.0, "MY IDE.", 220.0, BONE).draw(lb - 0.5, 0.0, 120.0)}
                    <rect x="146" y="604" width={(strike * 840.0).max(0.5)} height="22" fill={HONEY} />
                </g>
            </g>
        )
    }
}

/// The cells of the hive: (column, row) on a pointy-top grid, and the name on each.
const CELLS: &[(f32, f32, &str)] = &[
    (0.0, 0.0, "claude"),
    (1.0, 0.0, "codex"),
    (-0.5, 1.0, "a view"),
    (0.5, 1.0, "opencode"),
    (1.5, 1.0, "a widget"),
    (0.0, 2.0, "pi"),
    (1.0, 2.0, "a tool"),
];

impl Scene for HiveScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let r = 118.0;
        let (ox, oy) = (1290.0, 300.0);
        let cells: Vec<Svgr> = CELLS
            .iter()
            .enumerate()
            .map(|(i, (c, row, name))| {
                let at = 0.4 + i as f32 * 0.5;
                let s = soft(lb - at);
                let cx = ox + c * r * 3f32.sqrt();
                let cy = oy + row * r * 1.5;
                // each cell flies in from the edge it is nearest
                let from_x = (cx - 1380.0) * 1.8 * (1.0 - s);
                let from_y = (cy - 470.0) * 1.8 * (1.0 - s);
                let agent = !name.starts_with("a ");
                let fill = if agent { "#16171a" } else { HONEY };
                let text = if agent { BONE } else { INK };
                let name = name.to_string();
                let t = format!("translate({} {})", from_x, from_y);
                fframes::svgr!(
                    <g transform={t} opacity={prog(lb, at, at + 0.15)}>
                        <path d={hex_path(cx, cy, r - 6.0)} fill={fill} stroke={if agent { LINE2 } else { HONEY }} stroke-width="2" />
                        <text x={cx} y={cy + 9.0} text-anchor="middle" font-family={MONO} font-weight="700" font-size="25" fill={text}>{name}</text>
                    </g>
                )
            })
            .collect();
        let exit = expo_in(prog(lb, 3.65, 4.0));
        fframes::svgr!(
            <g>
                {ground(&frame, 0.7, 0.45, 0.36, 0.3)}
                <g opacity={1.0 - exit}>
                    {cells}
                    {Slam::new(146.0, 470.0, "HIRED", 220.0, BONE).draw(lb, -160.0, 0.0)}
                    {Slam::new(146.0, 690.0, "A HIVE.", 220.0, HONEY).draw(lb - 0.5, -160.0, 0.0)}
                </g>
            </g>
        )
    }
}
