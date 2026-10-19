//! A film with every time resolved to seconds, checked as it is built.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::brand::{is_hex_color, Brand};
use crate::film::{Aspect, CaptionPlace, Ease, Film, Loudness, PanZoom, TransitionKind, View};
use crate::time::{is_beat_name, resolve, Beats, TimeRef};
use crate::SpecError;

#[derive(Debug, Clone, Serialize)]
pub struct Timeline {
    pub title: String,
    pub fps: u32,
    pub length: f64,
    pub beats: Beats,
    pub aspects: BTreeMap<Aspect, AspectPlan>,
    pub shots: Vec<TimedShot>,
    pub captions: Vec<TimedCaption>,
    pub insets: Vec<TimedInset>,
    pub end_card: Option<TimedEndCard>,
    pub music: Option<TimedMusic>,
    pub finish_fps: u32,
    pub poster: f64,
    pub cutdowns: Vec<TimedCutdown>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AspectPlan {
    pub aspect: Aspect,
    pub width: u32,
    pub height: u32,
    pub caption: CaptionPlace,
    /// Kept spans of film time, in order. Never empty: a whole-film aspect keeps `[0, length]`.
    pub keep: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimedShot {
    pub id: String,
    pub clip: PathBuf,
    pub start: f64,
    /// When the next shot starts (or the film ends).
    pub end: f64,
    /// How long this shot keeps playing under the next one while it comes in.
    pub tail: f64,
    pub clip_start: f64,
    pub speed: f64,
    pub transition: Option<(TransitionKind, f64)>,
    pub ease: Ease,
    pub pan_zoom: BTreeMap<Aspect, PanZoom>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimedCaption {
    pub start: f64,
    pub end: f64,
    pub lines: Vec<String>,
}

/// How a caption moves: each line rises in, one after another, and drops out at the end.
/// The renderer draws this and the read-time check measures it, so both use these numbers.
pub mod caption_motion {
    /// Seconds one line takes to rise in.
    pub const LINE_IN: f64 = 0.45;
    /// Delay between one line starting to rise and the next.
    pub const LINE_STAGGER: f64 = 0.07;
    /// Seconds before the caption's end when its first line starts to drop out.
    pub const OUT: f64 = 0.28;
    /// Delay between lines dropping out.
    pub const OUT_STAGGER: f64 = 0.03;
    /// Seconds a reader needs per word, at least.
    pub const READ_PER_WORD: f64 = 0.3;
}

impl TimedCaption {
    /// When every line is fully on screen: after the last line has risen in, until the first
    /// line starts to drop out.
    pub fn fully_visible(&self) -> (f64, f64) {
        use caption_motion::*;
        let n = self.lines.len().max(1) as f64;
        (
            self.start + LINE_IN + LINE_STAGGER * (n - 1.0),
            self.end - OUT,
        )
    }

    pub fn words(&self) -> usize {
        self.lines
            .iter()
            .map(|l| l.split_whitespace().count())
            .sum()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TimedInset {
    pub id: String,
    pub clip: PathBuf,
    pub start: f64,
    pub end: f64,
    pub clip_start: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimedEndCard {
    pub start: f64,
    pub line: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimedMusic {
    pub file: PathBuf,
    /// Film start, film end, and the track second that plays at film start.
    pub sections: Vec<MusicPiece>,
    pub join: f64,
    pub fade_in: f64,
    pub fade_out: f64,
    pub loudness: Loudness,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct MusicPiece {
    pub start: f64,
    pub end: f64,
    pub track_start: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimedCutdown {
    pub name: String,
    pub start: f64,
    pub length: f64,
    pub loop_blend: Option<f64>,
    pub sound: bool,
}

impl AspectPlan {
    /// Length of this aspect's render.
    pub fn length(&self) -> f64 {
        self.keep.iter().map(|(a, b)| b - a).sum()
    }

    /// Film time shown at `t` seconds into this aspect's render.
    pub fn film_time(&self, t: f64) -> f64 {
        let mut before = 0.0;
        for (a, b) in &self.keep {
            if t < before + (b - a) {
                return a + (t - before);
            }
            before += b - a;
        }
        let (_, last) = self.keep.last().copied().unwrap_or((0.0, 0.0));
        last + (t - before)
    }

    /// The kept span holding film time `t`, if any.
    pub fn span_of(&self, t: f64) -> Option<(f64, f64)> {
        self.keep.iter().copied().find(|(a, b)| t >= *a && t < *b)
    }

    /// Render time of film time `t`, when `t` is kept.
    pub fn render_time(&self, t: f64) -> Option<f64> {
        let mut before = 0.0;
        for (a, b) in &self.keep {
            if t >= *a && t <= *b {
                return Some(before + (t - a));
            }
            before += b - a;
        }
        None
    }

    /// The music pieces as this aspect plays them, in render time: every piece cut to the
    /// kept spans.
    pub fn music_pieces(&self, music: &TimedMusic) -> Vec<MusicPiece> {
        let mut out = Vec::new();
        for piece in &music.sections {
            for &(a, b) in &self.keep {
                let (s, e) = (piece.start.max(a), piece.end.min(b));
                if e - s > 1e-9 {
                    if let (Some(rs), Some(re)) = (self.render_time(s), self.render_time(e)) {
                        out.push(MusicPiece {
                            start: rs,
                            end: re,
                            track_start: piece.track_start + (s - piece.start),
                        });
                    }
                }
            }
        }
        out.sort_by(|x, y| x.start.total_cmp(&y.start));
        out
    }

    /// Captions as this aspect shows them, in render time. A caption shows only if it starts
    /// inside a kept span, and it ends with that span at the latest.
    pub fn captions(&self, captions: &[TimedCaption]) -> Vec<TimedCaption> {
        captions
            .iter()
            .filter_map(|c| {
                let (_, span_end) = self.span_of(c.start)?;
                let end = c.end.min(span_end);
                Some(TimedCaption {
                    start: self.render_time(c.start)?,
                    end: self.render_time(end)?,
                    lines: c.lines.clone(),
                })
            })
            .collect()
    }
}

struct Errors(Vec<SpecError>);

impl Errors {
    /// One error per field: the first is the cause, later ones follow from it.
    fn add(&mut self, path: impl Into<String>, message: impl Into<String>) {
        let path = path.into();
        if self.0.iter().any(|e| e.path == path) {
            return;
        }
        self.0.push(SpecError {
            path,
            message: message.into(),
        });
    }

    fn time(&mut self, beats: &Beats, path: &str, time: &TimeRef) -> f64 {
        match resolve(beats, time) {
            Ok(t) => t,
            Err(message) => {
                self.add(path, message);
                f64::NAN
            }
        }
    }
}

fn check_view(errors: &mut Errors, path: &str, view: &View) {
    if !(view.zoom > 0.0 && view.zoom <= 1.0) {
        errors.add(
            format!("{path}.zoom"),
            format!(
                "{} is outside 0–1: a clip is never shown larger than its own pixels",
                view.zoom
            ),
        );
    }
    for (name, v) in [("x", view.x), ("y", view.y)] {
        if !(0.0..=1.0).contains(&v) {
            errors.add(
                format!("{path}.{name}"),
                format!("{v} is outside 0–1 (a fraction of the clip)"),
            );
        }
    }
}

/// Lowercase letters, digits and `-`: safe as a file name and as an id in a picture.
pub fn is_plain_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Check a film and resolve every time. `dir` is the folder of film.yaml; clip paths are joined
/// to it. Every problem is returned, each naming its field.
pub fn build(film: &Film, dir: &Path) -> Result<Timeline, Vec<SpecError>> {
    let mut e = Errors(Vec::new());
    let beats = &film.beats;

    for (name, at) in beats {
        if !is_beat_name(name) {
            e.add(
                format!("beats.{name}"),
                "a beat name is lowercase letters, digits and _, starting with a letter",
            );
        }
        if !at.is_finite() || *at < 0.0 {
            e.add(format!("beats.{name}"), format!("{at} is not a time"));
        }
    }
    if film.fps == 0 || film.fps > 240 {
        e.add(
            "fps",
            format!("{} frames per second is not usable", film.fps),
        );
    }
    let length = e.time(beats, "length", &film.length);
    if length <= 0.0 {
        e.add("length", "the film must be longer than 0 s");
    }
    let inside = |t: f64| t >= 0.0 && t <= length + 1e-9;

    // aspects
    if film.aspects.is_empty() {
        e.add("aspects", "name at least one aspect: 16:9, 9:16 or 4:5");
    }
    let mut aspects = BTreeMap::new();
    for (aspect, framing) in &film.aspects {
        let path = format!("aspects.\"{aspect}\"");
        let (width, height) = aspect.size();
        let place = &framing.caption;
        if place.top.is_some() == place.bottom.is_some() {
            e.add(
                format!("{path}.caption"),
                "give `top` or `bottom`, not both and not neither",
            );
        }
        if place.size <= 0.0 {
            e.add(format!("{path}.caption.size"), "must be above 0");
        }
        let mut keep = Vec::new();
        let mut last_end = 0.0;
        for (i, span) in framing.keep.iter().enumerate() {
            let a = e.time(beats, &format!("{path}.keep[{i}].from"), &span.from);
            let b = e.time(beats, &format!("{path}.keep[{i}].to"), &span.to);
            if a >= b || a < last_end || !inside(b) {
                e.add(
                    format!("{path}.keep[{i}]"),
                    format!(
                        "{a}–{b} must be a forward span inside the film, after the span before it"
                    ),
                );
            }
            last_end = b;
            keep.push((a, b));
        }
        if keep.is_empty() {
            keep.push((0.0, length));
        }
        aspects.insert(
            *aspect,
            AspectPlan {
                aspect: *aspect,
                width,
                height,
                caption: place.clone(),
                keep,
            },
        );
    }

    // shots
    if film.shots.is_empty() {
        e.add("shots", "a film needs at least one shot");
    }
    let mut ids = BTreeSet::new();
    let mut starts = Vec::new();
    for (i, shot) in film.shots.iter().enumerate() {
        let path = format!("shots[{i}]");
        if !is_plain_name(&shot.id) || !ids.insert(shot.id.clone()) {
            e.add(
                format!("{path}.id"),
                format!(
                    "'{}' must be unique, lowercase letters, digits and -",
                    shot.id
                ),
            );
        }
        if shot.clip.trim().is_empty() {
            e.add(format!("{path}.clip"), "name the recording for this shot");
        }
        let start = e.time(beats, &format!("{path}.from"), &shot.from);
        if let Some(prev) = starts.last() {
            if start <= *prev {
                e.add(
                    format!("{path}.from"),
                    format!("{start} s is not after the shot before it ({prev} s)"),
                );
            }
        }
        if !inside(start) {
            e.add(
                format!("{path}.from"),
                format!("{start} s is outside the film"),
            );
        }
        starts.push(start);
        if shot.clip_start < 0.0 {
            e.add(format!("{path}.clip_start"), "must be 0 or more");
        }
        if !(shot.speed > 0.0 && shot.speed <= 8.0) {
            e.add(format!("{path}.speed"), "must be above 0 and at most 8");
        }
        if let Some(t) = &shot.transition {
            if t.length <= 0.0 {
                e.add(format!("{path}.transition.length"), "must be above 0");
            }
            if i == 0 {
                e.add(
                    format!("{path}.transition"),
                    "the first shot has no shot before it to come in over",
                );
            }
        }
        for (aspect, pz) in &shot.pan_zoom {
            let p = format!("{path}.pan_zoom.\"{aspect}\"");
            if !film.aspects.contains_key(aspect) {
                e.add(p.clone(), format!("the film has no {aspect} aspect"));
            }
            check_view(&mut e, &format!("{p}.start"), &pz.start);
            check_view(&mut e, &format!("{p}.end"), &pz.end);
        }
    }
    let mut shots = Vec::new();
    for (i, shot) in film.shots.iter().enumerate() {
        let start = starts[i];
        let end = starts.get(i + 1).copied().unwrap_or(length);
        let next_transition = film
            .shots
            .get(i + 1)
            .and_then(|s| s.transition.as_ref())
            .map(|t| t.length)
            .unwrap_or(0.0);
        if let Some(t) = &shot.transition {
            if start.is_finite() && end.is_finite() && t.length >= end - start {
                e.add(
                    format!("shots[{i}].transition.length"),
                    format!("{} s does not fit in a {:.2} s shot", t.length, end - start),
                );
            }
        }
        shots.push(TimedShot {
            id: shot.id.clone(),
            clip: dir.join(&shot.clip),
            start,
            end,
            tail: next_transition,
            clip_start: shot.clip_start,
            speed: shot.speed,
            transition: shot.transition.as_ref().map(|t| (t.kind, t.length)),
            ease: shot.ease,
            pan_zoom: shot.pan_zoom.clone(),
        });
    }

    // captions
    let mut captions = Vec::new();
    for (i, c) in film.captions.iter().enumerate() {
        let path = format!("captions[{i}]");
        let start = e.time(beats, &format!("{path}.from"), &c.from);
        let end = e.time(beats, &format!("{path}.to"), &c.to);
        if start >= end || !inside(start) || !inside(end) {
            e.add(
                path.clone(),
                format!("{start}–{end} s must be a forward span inside the film"),
            );
        }
        if c.lines.is_empty() || c.lines.iter().any(|l| l.trim().is_empty()) {
            e.add(
                format!("{path}.lines"),
                "give at least one line, none empty",
            );
        }
        captions.push(TimedCaption {
            start,
            end,
            lines: c.lines.clone(),
        });
    }

    // insets
    let mut insets = Vec::new();
    for (i, inset) in film.insets.iter().enumerate() {
        let path = format!("insets[{i}]");
        // shots and insets share one media folder, so their ids share one namespace
        if !is_plain_name(&inset.id) || !ids.insert(inset.id.clone()) {
            e.add(
                format!("{path}.id"),
                format!(
                    "'{}' must be unique among shots and insets, lowercase letters, digits and -",
                    inset.id
                ),
            );
        }
        let start = e.time(beats, &format!("{path}.from"), &inset.from);
        let end = e.time(beats, &format!("{path}.to"), &inset.to);
        if start >= end || !inside(end) {
            e.add(
                path.clone(),
                format!("{start}–{end} s must be a forward span inside the film"),
            );
        }
        insets.push(TimedInset {
            id: inset.id.clone(),
            clip: dir.join(&inset.clip),
            start,
            end,
            clip_start: inset.clip_start,
        });
    }

    let end_card = film.end_card.as_ref().map(|card| {
        let start = e.time(beats, "end_card.from", &card.from);
        if !inside(start) {
            e.add("end_card.from", format!("{start} s is outside the film"));
        }
        TimedEndCard {
            start,
            line: card.line.clone(),
            command: card.command.clone(),
        }
    });

    // music
    let music = film.music.as_ref().map(|m| {
        let mut pieces: Vec<MusicPiece> = Vec::new();
        for (i, section) in m.sections.iter().enumerate() {
            let path = format!("music.sections[{i}]");
            let start = e.time(beats, &format!("{path}.from"), &section.from);
            if section.cues.is_empty() {
                e.add(
                    format!("{path}.cues"),
                    "give at least one cue: a beat and the second of the track that lands on it",
                );
            }
            let mut offset: Option<(String, f64)> = None;
            for (beat, track_at) in &section.cues {
                let cue_path = format!("{path}.cues.{beat}");
                let Some(beat_at) = beats.get(beat) else {
                    e.add(cue_path, format!("no beat named '{beat}'"));
                    continue;
                };
                let this = track_at - beat_at;
                match &offset {
                    None => offset = Some((beat.clone(), this)),
                    Some((first, o)) if (o - this).abs() > 0.002 => e.add(
                        cue_path,
                        format!(
                            "puts the track {:.3} s away from cue '{first}': the cues of one section must agree",
                            this - o
                        ),
                    ),
                    _ => {}
                }
            }
            let track_start = start + offset.map(|(_, o)| o).unwrap_or(0.0);
            if track_start < 0.0 {
                e.add(
                    path.clone(),
                    "these cues put the start before the beginning of the track",
                );
            }
            if let Some(prev) = pieces.last_mut() {
                if start <= prev.start {
                    e.add(
                        format!("{path}.from"),
                        "must be after the section before it",
                    );
                }
                prev.end = start;
            }
            pieces.push(MusicPiece {
                start,
                end: length,
                track_start,
            });
        }
        if m.sections.is_empty() {
            e.add("music.sections", "give at least one section");
        }
        if m.join < 0.0 || m.fade_in < 0.0 || m.fade_out < 0.0 {
            e.add("music", "join, fade_in and fade_out are 0 or more");
        }
        if let Some(p) = m.loudness.pre_limit {
            if !(p > 0.0 && p <= 1.0) {
                e.add("music.loudness.pre_limit", "a linear level above 0 and at most 1");
            }
        }
        if m.loudness.true_peak > -1.0 {
            e.add(
                "music.loudness.true_peak",
                "above −1 dBTP the master cannot pass the loudness gate",
            );
        }
        TimedMusic {
            file: dir.join(&m.file),
            sections: pieces,
            join: m.join,
            fade_in: m.fade_in,
            fade_out: m.fade_out,
            loudness: m.loudness.clone(),
        }
    });

    // finish and cutdowns
    if film.finish.fps == 0 || film.finish.fps > film.fps {
        e.add(
            "finish.fps",
            format!(
                "{} must be above 0 and at most the master's {} fps",
                film.finish.fps, film.fps
            ),
        );
    }
    let poster = e.time(beats, "finish.poster", &film.finish.poster);
    if !inside(poster) {
        e.add("finish.poster", format!("{poster} s is outside the film"));
    }
    if !film.cutdowns.is_empty() && !film.aspects.contains_key(&Aspect::Landscape) {
        e.add(
            "cutdowns",
            "cutdowns are cut from the 16:9 master, and the film has no 16:9 aspect",
        );
    }
    let mut names = BTreeSet::new();
    let mut cutdowns = Vec::new();
    for (i, c) in film.cutdowns.iter().enumerate() {
        let path = format!("cutdowns[{i}]");
        if !is_plain_name(&c.name) || !names.insert(c.name.clone()) {
            e.add(
                format!("{path}.name"),
                format!(
                    "'{}' must be unique, lowercase letters, digits and -",
                    c.name
                ),
            );
        }
        let start = e.time(beats, &format!("{path}.from"), &c.from);
        if c.length <= 0.0 || !inside(start + c.length) {
            e.add(
                format!("{path}.length"),
                format!("{} s from {start} s runs past the film", c.length),
            );
        }
        if let Some(blend) = c.loop_blend {
            if blend <= 0.0 || blend >= c.length / 2.0 {
                e.add(
                    format!("{path}.loop_blend"),
                    "must be above 0 and under half the length",
                );
            }
        }
        cutdowns.push(TimedCutdown {
            name: c.name.clone(),
            start,
            length: c.length,
            loop_blend: c.loop_blend,
            sound: c.sound,
        });
    }

    if !e.0.is_empty() {
        return Err(e.0);
    }
    Ok(Timeline {
        title: film.title.clone(),
        fps: film.fps,
        length,
        beats: beats.clone(),
        aspects,
        shots,
        captions,
        insets,
        end_card,
        music,
        finish_fps: film.finish.fps,
        poster,
        cutdowns,
    })
}

pub fn check_brand(brand: &Brand) -> Vec<SpecError> {
    let mut e = Errors(Vec::new());
    let c = &brand.colors;
    for (name, value) in [
        ("background", &c.background),
        ("text", &c.text),
        ("text_dim", &c.text_dim),
        ("text_faint", &c.text_faint),
    ] {
        if !is_hex_color(value) {
            e.add(
                format!("colors.{name}"),
                format!("'{value}' is not a #rrggbb colour"),
            );
        }
    }
    for (name, font) in [("sans", &brand.fonts.sans), ("mono", &brand.fonts.mono)] {
        if font.files.is_empty() {
            e.add(format!("fonts.{name}.files"), "list at least one font file");
        }
    }
    if !(100..=900).contains(&brand.caption.weight) {
        e.add("caption.weight", "a font weight is 100–900");
    }
    e.0
}
