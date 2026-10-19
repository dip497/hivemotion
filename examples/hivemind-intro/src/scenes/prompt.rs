//! The prompt that asked for this video, typed into an agent in a Hivemind tile, and the first
//! things the agent did.

use fframes::{Duration, FFramesContext, Frame, Scene, Svgr};

use crate::beat::*;
use crate::shaders::ground;
use crate::ui::*;

beat_scene!(PromptScene, 8.0, 16.0);

pub const PROMPT: [&str; 2] = ["make a launch video for hivemind.", "make it stunning."];

/// What the agent ran next: (a tool call, text).
const OUTPUT: &[(bool, &str)] = &[
    (true, "Read skills/fframes-video/SKILL.md"),
    (true, "Ran git log --reverse | head -1"),
    (false, "66337e8 2026-05-27 Initial OSS release of hivemind"),
    (true, "Fetched hivehub /api/v1/index.json"),
    (true, "Wrote src/scenes/*.rs"),
];

const SIZE: f32 = 54.0;
const ADV: f32 = SIZE * 0.6;

impl Scene for PromptScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let t = lb * BEAT;
        let total: usize = PROMPT.iter().map(|l| l.chars().count()).sum();
        // typed at 30 characters a second, Enter on beat 4
        let typed = ((t - 0.05) * 30.0).max(0.0) as usize;
        let enter = 4.0;
        let after = lb - enter;
        let n0 = PROMPT[0].chars().count();
        let line1: String = PROMPT[0].chars().take(typed.min(n0)).collect();
        let line2: String = PROMPT[1].chars().take(typed.saturating_sub(n0)).collect();
        let on_second = typed > n0;
        let caret_x = 300.0 + if on_second { line2.chars().count() } else { line1.chars().count() } as f32 * ADV;
        let caret_y = if on_second { 560.0 } else { 484.0 };
        let caret_on = after < 0.0 && (typed < total || (t * 3.0).fract() < 0.55);
        let lift = expo_out(after / 0.6) * -150.0 * f32::from(after > 0.0);

        let lines: Vec<Svgr> = OUTPUT
            .iter()
            .enumerate()
            .map(|(i, (call, text))| {
                let at = enter + 0.2 + i as f32 * 0.55;
                let o = prog(lb, at, at + 0.12);
                let y = 610.0 + i as f32 * 50.0;
                let marker = if *call {
                    fframes::svgr!(<circle cx="252" cy={y - 9.0} r="7" fill={HONEY} />)
                } else {
                    fframes::svgr!(<path d={format!("M262 {} V{} H280", y - 30.0, y - 9.0)} fill="none" stroke={GREY} stroke-width="2" />)
                };
                let x = if *call { 276.0 } else { 296.0 };
                let color = if *call { FG2 } else { GREY };
                fframes::svgr!(
                    <g opacity={o}>
                        {marker}
                        <text x={x} y={y} font-family={MONO} font-weight="500" font-size="28" fill={color}>{*text}</text>
                    </g>
                )
            })
            .collect();
        let exit = expo_in(prog(lb, 7.6, 8.0));
        let tile_border = if after > 0.0 { HONEY } else { LINE2 };
        fframes::svgr!(
            <g>
                {ground(&frame, 0.78, 0.6, 0.3 + (-after.max(0.0) * 3.0).exp() * f32::from(after > 0.0) * 0.4, 0.12)}
                <g opacity={1.0 - exit}>
                    <g transform={format!("translate(0 {lift})")}>
                        <rect x="200" y="330" width="1520" height="300" rx="12" fill="#0f1012" fill-opacity="0.92" stroke={tile_border} stroke-width="2" />
                        <rect x="200" y="330" width="1520" height="52" rx="12" fill="#15161a" />
                        <rect x="200" y="381" width="1520" height="1.5" fill={LINE2} />
                        <circle cx="232" cy="356" r="7" fill={if after > 0.0 { HONEY } else { GREY }} />
                        <text x="254" y="364" font-family={MONO} font-weight="700" font-size="21" fill={BONE}>"hivemind-intro"</text>
                        <text x="446" y="364" font-family={MONO} font-weight="500" font-size="21" fill={GREY}>"claude"</text>
                        <text x="1690" y="364" text-anchor="end" font-family={MONO} font-weight="500" font-size="19" letter-spacing="2" fill={GREY}>"A TILE ON THE HIVEMIND CANVAS"</text>
                        <text x="248" y="484" font-family={MONO} font-weight="700" font-size={SIZE} fill={HONEY}>"›"</text>
                        <text x="300" y="484" font-family={MONO} font-weight="500" font-size={SIZE} fill={BONE}>{line1}</text>
                        <text x="300" y="560" font-family={MONO} font-weight="500" font-size={SIZE} fill={HONEY}>{line2}</text>
                        <rect x={caret_x + 4.0} y={caret_y - 44.0} width="28" height="56" fill={HONEY} opacity={if caret_on { 1.0 } else { 0.0 }} />
                    </g>
                    <g transform="translate(0 -40)">{lines}</g>
                </g>
            </g>
        )
    }
}
