//! The Hivemind intro: about 73 seconds drawn in code on the beat grid of its soundtrack.
//! `cargo run --release -- timeline` lists the scenes with their seconds.

pub mod beat;
pub mod facts;
pub mod scenes;
pub mod shaders;
pub mod ui;

use fframes::{
    AudioMap, AudioTimestamp::*, AudioTrack, Color, Duration, FFramesContext, Frame, Scene, Scenes,
    ShaderUniforms, Svgr, Video, include_media_dir,
};

use beat::*;
use scenes::*;
use shaders::SHADERS;
use ui::*;

include_media_dir!(pub struct IntroMedia, "media");

pub const WIDTH: usize = 1920;
pub const HEIGHT: usize = 1080;

#[derive(Debug)]
pub struct IntroVideo;

/// The cut clips, the music, the commit log and the stills: made by scripts/prep.sh, not in git.
pub fn takes_dir() -> std::path::PathBuf {
    std::env::var_os("INTRO_TAKES")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("takes"))
}

/// The section in the bottom-right corner, by the beat it starts on.
const SECTIONS: &[(f32, &str)] = &[
    (0.0, "00 / HOOK"),
    (8.0, "01 / PROMPT"),
    (16.0, "02 / ORIGIN"),
    (32.0, "03 / CANVAS"),
    (48.0, "04 / VIEWS"),
    (68.0, "05 / WIDGETS"),
    (84.0, "06 / TOOLS"),
    (88.0, "07 / API"),
    (104.0, "08 / ANYWHERE"),
    (112.0, "09 / PLUGINS"),
    (120.0, "10 / THIS VIDEO"),
    (144.0, "11 / HIVEMIND"),
];

impl Video for IntroVideo {
    const FPS: usize = FPS;
    const WIDTH: usize = WIDTH;
    const HEIGHT: usize = HEIGHT;
    const BACKGROUND_COLOR: Color = Color::BLACK;

    fn duration(&self) -> Duration<'_> {
        Duration::Auto
    }

    fn audio(&self) -> AudioMap<'_> {
        AudioMap::from([
            AudioTrack::new("music.mp3", Second(0.0)..Second(beat_time(SILENCE_FROM)))
                .offset(MUSIC_INTRO_AT)
                .fade_out(0.04),
            AudioTrack::new("music.mp3", Second(beat_time(ENDING_FROM))..Second(TOTAL_SECONDS))
                .offset(MUSIC_ENDING_AT)
                .fade_out(2.2),
        ])
    }

    fn define_scenes(&self) -> Scenes<'_> {
        Scenes::from(vec![
            &FiredScene as &dyn Scene,
            &HiveScene,
            &PromptScene,
            &OriginScene,
            &CanvasScene,
            &AskScene,
            &IslandScene,
            &WidgetScene,
            &ToolScene,
            &ApiScene,
            &FanoutScene,
            &AnywhereScene,
            &PluginsScene,
            &LoopScene,
            &RenderScene,
            &SilentScene,
            &OutroScene,
        ])
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let b = gbeat(&frame);
        let grain = SHADERS.grain.draw(
            &frame,
            ShaderUniforms::new().float("uGrain", 0.06).float("uVignette", 0.5),
        );
        fframes::svgr!(
            <svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080" viewBox="0 0 1920 1080">
                <rect width="1920" height="1080" fill={BG} />
                {energy(&frame, ctx, b)}
                <image href={grain.href()} x="0" y="0" width="1920" height="1080" />
                {hud(&frame, b)}
            </svg>
        )
    }
}

fn hud(frame: &Frame, b: f32) -> Svgr<'static> {
    // the HUD sits out the silent bar and the first beat of the end card
    if (SILENCE_FROM..ENDING_FROM + 1.0).contains(&b) {
        return Svgr::empty();
    }
    let appear = prog(gsec(frame), 0.15, 0.6);
    let section = SECTIONS.iter().rev().find(|(start, _)| b >= *start).map_or("", |(_, n)| *n);
    let bar = (b / 4.0).floor() as i32 + 1;
    let beat_in_bar = b.rem_euclid(4.0).floor() as i32 + 1;
    let bpm = format!("136 BPM   BAR {bar:03}.{beat_in_bar}");
    let fnum = format!("F {:05}", frame.global_index);
    let progress = (gsec(frame) / TOTAL_SECONDS * 1788.0).max(0.5);
    fframes::svgr!(
        <g opacity={appear * 0.9} style="mix-blend-mode:difference">
            {corners(36.0, 36.0, 1848.0, 1008.0, 26.0, FG2, 2.0)}
            {label(66.0, 78.0, "HIVEMIND — INTRO".to_owned(), FG2, 17.0, "start")}
            {label(1854.0, 78.0, bpm, FG2, 17.0, "end")}
            <rect x="1840" y="92" width="14" height="14" fill={HONEY} opacity={0.25 + pulse(b, 7.0) * 0.75} />
            {label(66.0, 1018.0, timecode(gsec(frame)), FG2, 17.0, "start")}
            {label(290.0, 1018.0, fnum, GREY, 17.0, "start")}
            {label(1854.0, 1018.0, section.to_owned(), FG2, 17.0, "end")}
            <rect x="66" y="1034" width="1788" height="1" fill={FG2} opacity="0.25" />
            <rect x="66" y="1033" width={progress} height="3" fill={FG2} />
        </g>
    )
}

/// Where the drums play: the picture breathes with the kick.
const PUMPING: &[(f32, f32)] = &[(32.0, 96.0), (132.0, 140.0), (144.0, 152.0)];
/// Cuts that tear for three frames.
const GLITCH_CUTS: &[f32] = &[4.0, 8.0, 16.0, 48.0, 52.0, 68.0, 84.0, 88.0, 104.0, 112.0, 120.0];
/// Drops that flash.
const DROPS: &[f32] = &[32.0, 132.0, 144.0];

fn energy<'a>(frame: &Frame, ctx: &FFramesContext<'a, '_>, b: f32) -> Svgr<'a> {
    let pumping = PUMPING.iter().any(|(s, e)| (*s..*e).contains(&b));
    let s = 1.0 + if pumping { pulse(b, 9.0) * 0.006 } else { 0.0 };
    let flash = DROPS
        .iter()
        .map(|d| if b >= *d { (-(b - d) * 7.0).exp() } else { 0.0 })
        .fold(0.0_f32, f32::max);
    let since_cut = GLITCH_CUTS
        .iter()
        .map(|c| (b - c) * BEAT * FPS as f32)
        .find(|f| *f >= 0.0 && *f < 3.0);
    let scene = if let Some(f) = since_cut {
        let bands: Vec<Svgr> = (0..6)
            .map(|i| {
                let y = i as f32 * 180.0;
                let dx = (hash(i as f32 * 13.0 + f.floor() * 7.0) - 0.5) * 120.0 * (1.0 - f / 3.0);
                let id = format!("tear-{i}");
                let url = format!("url(#{id})");
                fframes::svgr!(
                    <g>
                        <clipPath id={id.clone()}><rect x="0" y={y} width="1920" height="180" /></clipPath>
                        <g clip-path={url} transform={format!("translate({dx} 0)")}>{ctx.render_scenes(frame)}</g>
                    </g>
                )
            })
            .collect();
        fframes::svgr!(<g>{bands}</g>)
    } else {
        ctx.render_scenes(frame)
    };
    fframes::svgr!(
        <g>
            <g transform={format!("translate(960 540) scale({s}) translate(-960 -540)")}>{scene}</g>
            <rect width="1920" height="1080" fill={HONEY} opacity={flash * 0.35} />
        </g>
    )
}
