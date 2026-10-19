use std::collections::BTreeMap;
use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::time::{Beats, TimeRef};

/// One launch film: its beats, the shots and captions placed on them, the music, and the cutdowns.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Film {
    pub title: String,
    /// Path of brand.yaml, relative to this file.
    pub brand: String,
    /// Frames per second of the master render.
    pub fps: u32,
    /// Named points in time, in seconds. Every other time in this file points at one of these.
    pub beats: Beats,
    /// Where the film ends.
    pub length: TimeRef,
    /// The aspects to render, each with its own framing.
    pub aspects: BTreeMap<Aspect, Framing>,
    pub shots: Vec<Shot>,
    #[serde(default)]
    pub captions: Vec<Caption>,
    /// Smaller clips drawn over the shots, such as a phone recording.
    #[serde(default)]
    pub insets: Vec<Inset>,
    pub end_card: Option<EndCard>,
    pub music: Option<Music>,
    pub finish: Finish,
    /// Short clips cut out of the 16:9 master.
    #[serde(default)]
    pub cutdowns: Vec<Cutdown>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
pub enum Aspect {
    #[serde(rename = "16:9")]
    Landscape,
    #[serde(rename = "9:16")]
    Vertical,
    #[serde(rename = "4:5")]
    Portrait,
}

impl Aspect {
    pub const ALL: [Aspect; 3] = [Aspect::Landscape, Aspect::Vertical, Aspect::Portrait];

    pub fn size(self) -> (u32, u32) {
        match self {
            Aspect::Landscape => (1920, 1080),
            Aspect::Vertical => (1080, 1920),
            Aspect::Portrait => (1080, 1350),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Aspect::Landscape => "16:9",
            Aspect::Vertical => "9:16",
            Aspect::Portrait => "4:5",
        }
    }

    /// A name that is safe in a file name.
    pub fn file_name(self) -> &'static str {
        match self {
            Aspect::Landscape => "16x9",
            Aspect::Vertical => "9x16",
            Aspect::Portrait => "4x5",
        }
    }

    pub fn parse(text: &str) -> Option<Aspect> {
        Aspect::ALL
            .into_iter()
            .find(|a| a.name() == text || a.file_name() == text)
    }
}

impl fmt::Display for Aspect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Framing {
    pub caption: CaptionPlace,
    /// Parts of the film this aspect keeps, in order. Empty keeps the whole film.
    #[serde(default)]
    pub keep: Vec<Span>,
}

/// Where captions sit, in pixels of this aspect. Give `top` or `bottom`, not both.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionPlace {
    pub left: f64,
    pub top: Option<f64>,
    pub bottom: Option<f64>,
    /// Font size in pixels.
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Span {
    pub from: TimeRef,
    pub to: TimeRef,
}

/// A clip that fills the frame from `from` until the next shot starts.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Shot {
    pub id: String,
    /// Path of the recording, relative to film.yaml. A missing file renders as a named slate.
    pub clip: String,
    pub from: TimeRef,
    /// Seconds into the recording where the shot begins.
    #[serde(default)]
    pub clip_start: f64,
    /// Playback speed of the recording; 2.0 plays it twice as fast.
    #[serde(default = "one")]
    pub speed: f64,
    /// How this shot comes in over the one before it.
    pub transition: Option<Transition>,
    /// The curve of the pan and zoom.
    #[serde(default)]
    pub ease: Ease,
    /// Pan and zoom per aspect. An aspect left out shows the clip still, centred, covering the frame.
    #[serde(default)]
    pub pan_zoom: BTreeMap<Aspect, PanZoom>,
}

fn one() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub kind: TransitionKind,
    /// Seconds. The shot before keeps playing under this one for this long.
    pub length: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TransitionKind {
    /// The new shot grows out of the centre in a hexagon.
    HexWipe,
    /// The shot before closes into a hexagon at the centre, uncovering the new shot.
    HexIris,
    /// The new shot fades in over the one before.
    Cross,
}

impl TransitionKind {
    pub fn name(self) -> &'static str {
        match self {
            TransitionKind::HexWipe => "hex-wipe",
            TransitionKind::HexIris => "hex-iris",
            TransitionKind::Cross => "cross",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Ease {
    Linear,
    #[default]
    InOut,
    OutCubic,
}

impl Ease {
    pub fn apply(self, x: f64) -> f64 {
        let x = x.clamp(0.0, 1.0);
        match self {
            Ease::Linear => x,
            Ease::InOut if x < 0.5 => 4.0 * x * x * x,
            Ease::InOut => 1.0 - (-2.0 * x + 2.0).powi(3) / 2.0,
            Ease::OutCubic => 1.0 - (1.0 - x).powi(3),
        }
    }
}

/// A slow move from one view of the clip to another over the shot.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PanZoom {
    pub start: View,
    pub end: View,
}

/// `zoom` is the scale of the clip's own pixels (at most 1, so a clip is never blown up past its
/// recording); it grows if needed so the clip always covers the frame. `x` and `y` are the point
/// of the clip, as a fraction of its width and height, held at the centre of the frame.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct View {
    pub zoom: f64,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Caption {
    pub from: TimeRef,
    pub to: TimeRef,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Inset {
    pub id: String,
    pub clip: String,
    pub from: TimeRef,
    pub to: TimeRef,
    #[serde(default)]
    pub clip_start: f64,
}

/// The closing card: the brand mark, its name, one line and a command to copy.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndCard {
    pub from: TimeRef,
    pub line: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Music {
    /// Path of the track, relative to film.yaml.
    pub file: String,
    /// Parts of the track, in film order. Each runs until the next one starts.
    pub sections: Vec<MusicSection>,
    /// Seconds of crossfade where two sections meet.
    pub join: f64,
    pub fade_in: f64,
    pub fade_out: f64,
    pub loudness: Loudness,
}

/// A part of the track placed on the film by its cues: each cue says which second of the
/// track lands on which beat. All cues of one section must agree.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MusicSection {
    pub from: TimeRef,
    pub cues: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Loudness {
    /// Integrated loudness to aim for, in LUFS.
    pub integrated: f64,
    /// True-peak target for the normalizer, in dBTP.
    pub true_peak: f64,
    /// Loudness range the normalizer may keep, in LU.
    pub range: f64,
    /// Peak limit (linear, 0–1) applied before measuring, so one loud hit does not force the
    /// normalizer out of linear mode.
    pub pre_limit: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Finish {
    /// Frame rate of the delivered files. Below the master's rate, frames are blended in pairs.
    pub fps: u32,
    /// The frame shown as the first frame, so every player's thumbnail is a strong one.
    pub poster: TimeRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Cutdown {
    pub name: String,
    pub from: TimeRef,
    /// Seconds.
    pub length: f64,
    /// Seconds of the end blended into the start, so the clip loops without a jump.
    pub loop_blend: Option<f64>,
    #[serde(default = "yes")]
    pub sound: bool,
}

fn yes() -> bool {
    true
}
