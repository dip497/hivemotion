//! The Hivemind intro: about 32 seconds, voiced, drawn in code on the beat grid of its soundtrack.
//! `cargo run --release -- timeline` lists the scenes with their seconds.

pub mod beat;
pub mod scenes;
pub mod shaders;
pub mod ui;

use fframes::{
    AudioMap, AudioTimestamp::*, AudioTrack, Color, Ducking, Duration, FFramesContext, Frame, Scene,
    Scenes, ShaderUniforms, Svgr, Video, include_media_dir,
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

/// The cut clips, the stills, the music, the voice and the effects: made by scripts/prep.sh, not in git.
pub fn takes_dir() -> std::path::PathBuf {
    std::env::var_os("INTRO_TAKES")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("takes"))
}

/// The section in the bottom-right corner, by the beat it starts on.
const SECTIONS: &[(f32, &str)] = &[
    (0.0, "00 / HOOK"),
    (14.0, "01 / THE BOARD"),
    (28.0, "02 / ASK"),
    (56.0, "03 / HIVEMIND"),
];

/// Each voice line and the beat it starts on (voice/script.tsv has the words).
pub const VOICE: &[(&str, f32)] = &[
    ("vo_01.mp3", 0.45),
    ("vo_02.mp3", 7.5),
    // "on one board" lands on the wide shot at beat 22
    ("vo_03.mp3", 16.8),
    // "I ask" lands at beat 30.9, a beat before the drop
    ("vo_04.mp3", 27.5),
    ("vo_05.mp3", 35.0),
    ("vo_06.mp3", 46.2),
    ("vo_07.mp3", 50.75),
    ("vo_08.mp3", 57.0),
];

/// Effects on the actions they belong to: (file, beat, gain).
const EFFECTS: &[(&str, f32, f32)] = &[("thud.wav", 4.55, -16.0), ("chime.wav", 46.2, -18.0)];

/// The calm bed's level, set so it sits about 8 dB under the voice between lines.
const MUSIC_GAIN_DB: f32 = -9.0;

/// `INTRO_MUSIC=0` leaves the music out: the voice and the two effects only.
fn with_music() -> bool {
    std::env::var("INTRO_MUSIC").map_or(true, |v| v != "0")
}

impl Video for IntroVideo {
    const FPS: usize = FPS;
    const WIDTH: usize = WIDTH;
    const HEIGHT: usize = HEIGHT;
    const BACKGROUND_COLOR: Color = Color::BLACK;

    fn duration(&self) -> Duration<'_> {
        Duration::Auto
    }

    fn audio(&self) -> AudioMap<'_> {
        let mut tracks = Vec::new();
        if with_music() {
            tracks.push(
                AudioTrack::new("calm.wav", Second(0.0)..Second(TOTAL_SECONDS))
                    .gain_db(MUSIC_GAIN_DB)
                    .fade_out(beat_time(TOTAL_BEATS - ENDING_FROM))
                    .duck(Ducking { depth_db: -5.0, ..Default::default() }),
            );
        }
        tracks.extend(VOICE.iter().map(|(file, at)| AudioTrack::new(*file, Second(beat_time(*at))..Eof).voice()));
        tracks.extend(EFFECTS.iter().map(|(file, at, gain)| AudioTrack::new(*file, Second(beat_time(*at))..Eof).gain_db(*gain)));
        AudioMap::from(tracks)
    }

    fn define_scenes(&self) -> Scenes<'_> {
        Scenes::from(vec![
            &FiredScene as &dyn Scene,
            &HiveScene,
            &BoardScene,
            &AskScene,
            &ViewScene,
            &WidgetScene,
            &ToolScene,
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
    // the HUD sits out the widget (it covers the light) and the first beat of the end card
    if (41.0..50.0).contains(&b) || (ENDING_FROM..ENDING_FROM + 1.0).contains(&b) {
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

/// Cuts that tear for three frames.
const GLITCH_CUTS: &[f32] = &[7.0, 14.0, 22.0, 28.0, 41.0, 50.0];

fn energy<'a>(frame: &Frame, ctx: &FFramesContext<'a, '_>, b: f32) -> Svgr<'a> {
    let since_cut = GLITCH_CUTS
        .iter()
        .map(|c| (b - c) * BEAT * FPS as f32)
        .find(|f| *f >= 0.0 && *f < 3.0);
    if let Some(f) = since_cut {
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
    }
}
