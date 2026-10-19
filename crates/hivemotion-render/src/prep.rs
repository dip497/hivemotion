//! `hivemotion prep`: cut every take to its shot with the system ffmpeg, so the renderer reads
//! clips that start at the shot, run at the shot's speed and the film's frame rate, and last as
//! long as the shot is on screen (the last frame held if the take is short).

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use hivemotion_ffmpeg::{ffmpeg, probe, secs};
use hivemotion_spec::timeline::Picture;
use hivemotion_spec::Project;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PreparedClip {
    pub id: String,
    pub source: PathBuf,
    pub output: Option<PathBuf>,
    /// made, kept (already made from the same take and settings), or missing (renders as a slate)
    pub status: &'static str,
    pub seconds: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrepReport {
    pub media: PathBuf,
    pub clips: Vec<PreparedClip>,
    pub missing: Vec<String>,
    pub fonts: Vec<PathBuf>,
}

struct Want {
    id: String,
    source: PathBuf,
    clip_start: f64,
    speed: f64,
    seconds: f64,
}

/// `<media>/<id>.<extension>`, refusing any id that could name a file outside the media folder.
fn media_file(media: &Path, id: &str, extension: &str) -> Result<PathBuf> {
    if !hivemotion_spec::timeline::is_plain_name(id) {
        bail!(
            "'{id}' is not a plain name; refusing to write outside {}",
            media.display()
        );
    }
    Ok(media.join(format!("{id}.{extension}")))
}

fn settings_text(w: &Want, fps: u32, modified: u64) -> String {
    format!(
        "source={} modified={modified} start={} speed={} seconds={} fps={fps} v=1",
        w.source.display(),
        w.clip_start,
        w.speed,
        w.seconds
    )
}

pub fn prep(project: &Project) -> Result<PrepReport> {
    let t = &project.timeline;
    let media = project.build_paths().media();
    std::fs::create_dir_all(&media).with_context(|| format!("creating {}", media.display()))?;

    let mut wants: Vec<Want> = t
        .shots
        .iter()
        .filter_map(|s| match &s.picture {
            Picture::Clip(clip) => Some((s, clip)),
            Picture::Graphic(_) => None,
        })
        .map(|(s, clip)| Want {
            id: s.id.clone(),
            source: clip.clone(),
            clip_start: s.clip_start,
            speed: s.speed,
            seconds: s.end + s.tail - s.start + 0.25,
        })
        .collect();
    wants.extend(t.insets.iter().map(|i| Want {
        id: i.id.clone(),
        source: i.clip.clone(),
        clip_start: i.clip_start,
        speed: 1.0,
        seconds: i.end - i.start + 0.25,
    }));

    let mut clips = Vec::new();
    let mut missing = Vec::new();
    for w in &wants {
        let out = media_file(&media, &w.id, "mp4")?;
        let note = media_file(&media, &w.id, "prep")?;
        let Ok(meta) = std::fs::metadata(&w.source) else {
            let _ = std::fs::remove_file(&out);
            let _ = std::fs::remove_file(&note);
            missing.push(w.source.display().to_string());
            clips.push(PreparedClip {
                id: w.id.clone(),
                source: w.source.clone(),
                output: None,
                status: "missing",
                seconds: w.seconds,
            });
            continue;
        };
        let modified = meta
            .modified()
            .ok()
            .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let settings = settings_text(w, t.fps, modified);
        if out.is_file() && std::fs::read_to_string(&note).ok().as_deref() == Some(&settings) {
            clips.push(PreparedClip {
                id: w.id.clone(),
                source: w.source.clone(),
                output: Some(out),
                status: "kept",
                seconds: w.seconds,
            });
            continue;
        }
        cut(&w.source, &out, w.clip_start, w.speed, w.seconds, t.fps)?;
        std::fs::write(&note, settings)?;
        clips.push(PreparedClip {
            id: w.id.clone(),
            source: w.source.clone(),
            output: Some(out),
            status: "made",
            seconds: w.seconds,
        });
    }

    // clips of shots that no longer exist would be picked up by id; remove them
    let ids: Vec<String> = wants.iter().map(|w| format!("{}.mp4", w.id)).collect();
    for entry in std::fs::read_dir(&media)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if name.ends_with(".mp4") && !ids.contains(&name) {
            std::fs::remove_file(&path)?;
        }
    }

    let mut fonts = Vec::new();
    for f in project.font_files() {
        let to = media.join(f.file_name().unwrap_or_default());
        std::fs::copy(&f, &to).with_context(|| format!("copying {}", f.display()))?;
        fonts.push(to);
    }
    Ok(PrepReport {
        media,
        clips,
        missing,
        fonts,
    })
}

fn cut(source: &Path, out: &Path, start: f64, speed: f64, seconds: f64, fps: u32) -> Result<()> {
    let p = probe(source)?;
    if p.video.is_none() {
        anyhow::bail!("{} has no video stream", source.display());
    }
    let filter = format!(
        "setpts=(PTS-STARTPTS)/{speed},fps={fps},tpad=stop_mode=clone:stop_duration={},format=yuv420p",
        secs(seconds)
    );
    let mut args: Vec<std::ffi::OsString> = Vec::new();
    for a in ["-v", "error", "-ss", &secs(start), "-i"] {
        args.push(a.into());
    }
    args.push(source.into());
    for a in [
        "-t",
        &secs(seconds * speed + 0.5),
        "-an",
        "-vf",
        &filter,
        "-t",
        &secs(seconds),
        "-c:v",
        "libx264",
        "-preset",
        "fast",
        "-crf",
        "12",
        "-g",
        &fps.to_string(),
        "-colorspace",
        "bt709",
        "-color_primaries",
        "bt709",
        "-color_trc",
        "bt709",
    ] {
        args.push(a.into());
    }
    args.push(out.into());
    ffmpeg(args)?;
    Ok(())
}
