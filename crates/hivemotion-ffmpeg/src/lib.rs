//! The system `ffmpeg` and `ffprobe`, run as separate programs so no GPL code is linked into
//! hivemotion.

use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use serde::Serialize;

/// Run ffmpeg with `args` (after `-hide_banner -nostdin -y`). Returns its stderr, where ffmpeg
/// writes filter reports such as loudnorm's JSON.
pub fn ffmpeg<I, S>(args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let args: Vec<_> = args.into_iter().map(|a| a.as_ref().to_owned()).collect();
    let out = Command::new("ffmpeg")
        .args(["-hide_banner", "-nostdin", "-y"])
        .args(&args)
        .output()
        .context("cannot start ffmpeg; is it installed and on PATH?")?;
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    if !out.status.success() {
        let tail: Vec<&str> = stderr.lines().rev().take(12).collect();
        let tail: Vec<&str> = tail.into_iter().rev().collect();
        bail!(
            "ffmpeg failed ({}): ffmpeg {}\n{}",
            out.status,
            args.iter()
                .map(|a| a.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" "),
            tail.join("\n")
        );
    }
    Ok(stderr)
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Probe {
    pub duration: f64,
    pub size_bytes: u64,
    pub video: Option<VideoStream>,
    pub audio: Option<AudioStream>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct VideoStream {
    pub codec: String,
    pub profile: String,
    pub width: u32,
    pub height: u32,
    pub pix_fmt: String,
    pub fps: f64,
    pub frames: Option<u64>,
    pub color_space: String,
    pub color_primaries: String,
    pub color_transfer: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct AudioStream {
    pub codec: String,
    pub sample_rate: u32,
    pub channels: u32,
}

fn text(v: &serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_string()
}

fn ratio(text: &str) -> f64 {
    match text.split_once('/') {
        Some((a, b)) => {
            let (a, b): (f64, f64) = (a.parse().unwrap_or(0.0), b.parse().unwrap_or(1.0));
            if b == 0.0 {
                0.0
            } else {
                a / b
            }
        }
        None => text.parse().unwrap_or(0.0),
    }
}

/// What ffprobe says about a media file.
pub fn probe(file: &Path) -> Result<Probe> {
    let out = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(file)
        .output()
        .context("cannot start ffprobe; is it installed and on PATH?")?;
    if !out.status.success() {
        bail!(
            "ffprobe cannot read {}: {}",
            file.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let json: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    let format = &json["format"];
    let mut probe = Probe {
        duration: text(format, "duration").parse().unwrap_or(0.0),
        size_bytes: text(format, "size").parse().unwrap_or(0),
        ..Default::default()
    };
    for s in json["streams"].as_array().cloned().unwrap_or_default() {
        match s["codec_type"].as_str() {
            Some("video") if probe.video.is_none() => {
                probe.video = Some(VideoStream {
                    codec: text(&s, "codec_name"),
                    profile: text(&s, "profile"),
                    width: s["width"].as_u64().unwrap_or(0) as u32,
                    height: s["height"].as_u64().unwrap_or(0) as u32,
                    pix_fmt: text(&s, "pix_fmt"),
                    fps: ratio(&text(&s, "r_frame_rate")),
                    frames: text(&s, "nb_frames").parse().ok(),
                    color_space: text(&s, "color_space"),
                    color_primaries: text(&s, "color_primaries"),
                    color_transfer: text(&s, "color_transfer"),
                })
            }
            Some("audio") if probe.audio.is_none() => {
                probe.audio = Some(AudioStream {
                    codec: text(&s, "codec_name"),
                    sample_rate: text(&s, "sample_rate").parse().unwrap_or(0),
                    channels: s["channels"].as_u64().unwrap_or(0) as u32,
                })
            }
            _ => {}
        }
    }
    Ok(probe)
}

/// The first JSON object ffmpeg printed into `log` (loudnorm writes its report this way).
pub fn last_json_object(log: &str) -> Option<serde_json::Value> {
    let start = log.rfind("\n{").map(|i| i + 1).or_else(|| {
        if log.starts_with('{') {
            Some(0)
        } else {
            None
        }
    })?;
    let end = log[start..].find("\n}").map(|i| start + i + 2)?;
    serde_json::from_str(&log[start..end]).ok()
}

/// Format seconds for an ffmpeg argument.
pub fn secs(t: f64) -> String {
    format!("{t:.6}")
}
