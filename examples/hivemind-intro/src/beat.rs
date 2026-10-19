//! The musical clock. The soundtrack is an edit of a 136 BPM track cut on its bar lines, so the
//! whole video sits on one beat grid: every scene starts on a downbeat and motion is written in
//! beats. The video's beat 0 is the track's bar 9, so its drop lands on beat 32.

use fframes::Frame;

pub const FPS: usize = 60;
/// One beat of the soundtrack, in seconds (136 BPM, measured over the whole track).
pub const BEAT: f32 = 60.0 / 136.0;
/// The last beat of the video: the end card rings out over the track's decay.
pub const TOTAL_BEATS: f32 = 72.0;
pub const TOTAL_SECONDS: f32 = TOTAL_BEATS * BEAT;

/// Where the two music sections start in the track (its downbeats of bar 9 and bar 88).
pub const MUSIC_INTRO_AT: f32 = 14.3627;
pub const MUSIC_ENDING_AT: f32 = 153.775;
/// The beat the end card and the track's ending start on.
pub const ENDING_FROM: f32 = 56.0;

pub fn beat_time(beat: f32) -> f32 {
    beat * BEAT
}

/// The frame a beat starts on, rounded so scene boundaries never drift.
pub fn beat_frame(beat: f32) -> usize {
    (beat_time(beat) * FPS as f32).round() as usize
}

/// Seconds since the start of the video, whichever scene the frame belongs to.
pub fn gsec(frame: &Frame) -> f32 {
    frame.global_index as f32 / FPS as f32
}

/// Position on the beat grid of the whole video.
pub fn gbeat(frame: &Frame) -> f32 {
    gsec(frame) / BEAT
}

pub fn span_frames(start: f32, end: f32) -> usize {
    beat_frame(end) - beat_frame(start)
}

pub fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

/// Where `x` is between `a` and `b`, clamped to 0..1.
pub fn prog(x: f32, a: f32, b: f32) -> f32 {
    clamp01((x - a) / (b - a))
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn expo_out(t: f32) -> f32 {
    let t = clamp01(t);
    if t >= 1.0 { 1.0 } else { 1.0 - 2f32.powf(-10.0 * t) }
}

pub fn expo_in(t: f32) -> f32 {
    let t = clamp01(t);
    if t <= 0.0 { 0.0 } else { 2f32.powf(10.0 * t - 10.0) }
}

pub fn cubic_in_out(t: f32) -> f32 {
    let t = clamp01(t);
    if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
}

pub fn quart_out(t: f32) -> f32 {
    1.0 - (1.0 - clamp01(t)).powi(4)
}

/// Damped spring from 0 to 1 (mass 1), `t` in seconds since release.
pub fn spring(t: f32, stiffness: f32, damping: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    let w0 = stiffness.sqrt();
    let zeta = damping / (2.0 * w0);
    if zeta < 1.0 {
        let wd = w0 * (1.0 - zeta * zeta).sqrt();
        1.0 - (-zeta * w0 * t).exp() * ((wd * t).cos() + zeta * w0 / wd * (wd * t).sin())
    } else {
        1.0 - (1.0 + w0 * t) * (-w0 * t).exp()
    }
}

/// A snappy spring, `beats` since it was released.
pub fn snap(beats: f32) -> f32 {
    spring(beats * BEAT, 320.0, 22.0)
}

/// A softer spring with a visible overshoot.
pub fn soft(beats: f32) -> f32 {
    spring(beats * BEAT, 170.0, 15.0)
}

/// A pulse on every beat: 1 on the beat, about 0 half a beat later.
pub fn pulse(beat: f32, sharpness: f32) -> f32 {
    let f = beat - beat.floor();
    (-f * sharpness).exp()
}

/// Deterministic noise, 0..1.
pub fn hash(n: f32) -> f32 {
    let x = (n * 127.1 + 311.7).sin() * 43_758.547;
    x - x.floor()
}
