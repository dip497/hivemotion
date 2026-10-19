//! `hivemotion check`: what an agent that cannot watch the film needs to judge it. Stills from the
//! master at every beat and halfway through every transition, one contact sheet per aspect, the
//! read-time rule (a caption stays fully on screen at least 0.3 s per word), and the loudness gate
//! on every file with sound.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::Result;
use hivemotion_audio::{check as loudness, Gate, Verdict};
use hivemotion_ffmpeg::{ffmpeg, filter_text, secs};
use hivemotion_spec::film::Aspect;
use hivemotion_spec::timeline::{caption_motion::READ_PER_WORD, AspectPlan, Timeline};
use hivemotion_spec::Project;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StillAt {
    /// Seconds into this aspect's render.
    pub seconds: f64,
    /// What is there: a beat name, or a transition into a shot.
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CaptionReading {
    pub aspect: String,
    pub text: String,
    pub words: usize,
    pub fully_visible_seconds: f64,
    pub needed_seconds: f64,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AspectStills {
    pub aspect: String,
    pub stills: Vec<(StillAt, PathBuf)>,
    pub contact_sheet: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckReport {
    pub passed: bool,
    pub stills: Vec<AspectStills>,
    pub reading: Vec<CaptionReading>,
    pub loudness: Vec<Verdict>,
    pub problems: Vec<String>,
}

/// Where the stills go: every beat the aspect keeps, and the middle of every transition.
pub fn still_times(timeline: &Timeline, plan: &AspectPlan) -> Vec<StillAt> {
    let mut out: Vec<StillAt> = Vec::new();
    // a still is taken at the start of a frame: ffmpeg seeking into the middle of a frame skips
    // it, and past the last frame's start it finds nothing
    let fps = timeline.fps as f64;
    let last = plan.length() - 1.0 / fps;
    let frame_start = |t: f64| ((t * fps + 1e-6).floor() / fps).min(last);
    for (name, at) in &timeline.beats {
        if let Some(r) = plan.render_time(*at) {
            out.push(StillAt {
                seconds: frame_start(r),
                label: name.clone(),
            });
        }
    }
    for shot in &timeline.shots {
        if let Some((kind, length)) = shot.transition {
            if let Some(r) = plan.render_time(shot.start + length / 2.0) {
                out.push(StillAt {
                    seconds: frame_start(r),
                    label: format!("{} into {} (middle)", kind.name(), shot.id),
                });
            }
        }
    }
    out.sort_by(|a, b| a.seconds.total_cmp(&b.seconds));
    out.dedup_by(|a, b| {
        (a.seconds - b.seconds).abs() < 1e-6 && {
            b.label = format!("{} / {}", b.label, a.label);
            true
        }
    });
    out
}

/// The read-time rule for every caption of every aspect.
pub fn reading(timeline: &Timeline) -> Vec<CaptionReading> {
    let mut out = Vec::new();
    for (aspect, plan) in &timeline.aspects {
        for c in plan.captions(&timeline.captions) {
            let (a, b) = c.fully_visible();
            let words = c.words();
            let visible = (b - a).max(0.0);
            let needed = words as f64 * READ_PER_WORD;
            out.push(CaptionReading {
                aspect: aspect.name().into(),
                text: c.lines.join(" / "),
                words,
                fully_visible_seconds: (visible * 1000.0).round() / 1000.0,
                needed_seconds: (needed * 1000.0).round() / 1000.0,
                ok: visible + 1e-9 >= needed,
            });
        }
    }
    out
}

fn args(list: &[&str]) -> Vec<OsString> {
    list.iter().map(OsString::from).collect()
}

/// Pull stills from `master` and tile labelled thumbnails into one contact sheet.
pub fn stills_from_master(
    master: &Path,
    at: &[StillAt],
    dir: &Path,
    aspect: Aspect,
    font: Option<&Path>,
) -> Result<AspectStills> {
    std::fs::create_dir_all(dir)?;
    let mut stills = Vec::new();
    let thumbs = dir.join(format!("thumbs-{}", aspect.file_name()));
    let _ = std::fs::remove_dir_all(&thumbs);
    std::fs::create_dir_all(&thumbs)?;
    let thumb_width = if aspect == Aspect::Landscape {
        480
    } else {
        270
    };
    for (i, s) in at.iter().enumerate() {
        let file = dir.join(format!("{}-{:07.3}s.png", aspect.file_name(), s.seconds));
        let mut a = args(&["-v", "error", "-ss", &secs(s.seconds), "-i"]);
        a.push(master.into());
        a.extend(args(&["-frames:v", "1", "-update", "1"]));
        a.push(file.clone().into());
        ffmpeg(a)?;
        let label = filter_text(&format!("{:.2}s {}", s.seconds, s.label));
        let font_arg = font
            .map(|f| format!(":fontfile={}", filter_text(&f.to_string_lossy())))
            .unwrap_or_default();
        let filter = format!(
            "scale={thumb_width}:-2,pad=iw:ih+28:0:0:0x101215,drawtext=expansion=none:text={label}{font_arg}:fontsize=15:fontcolor=0xd0d4d8:x=8:y=h-21"
        );
        let mut a = args(&["-v", "error", "-i"]);
        a.push(file.clone().into());
        a.extend(args(&["-vf", &filter, "-update", "1"]));
        a.push(thumbs.join(format!("{i:04}.png")).into());
        ffmpeg(a)?;
        stills.push((s.clone(), file));
    }
    let columns = if aspect == Aspect::Landscape { 4 } else { 6 };
    let rows = at.len().div_ceil(columns).max(1);
    let sheet = dir.join(format!("contact-sheet-{}.png", aspect.file_name()));
    let mut a = args(&["-v", "error", "-framerate", "1", "-i"]);
    a.push(thumbs.join("%04d.png").into());
    a.extend(args(&[
        "-vf",
        &format!("tile={columns}x{rows}:padding=6:margin=6:color=0x08090b"),
        "-frames:v",
        "1",
        "-update",
        "1",
    ]));
    a.push(sheet.clone().into());
    ffmpeg(a)?;
    let _ = std::fs::remove_dir_all(&thumbs);
    Ok(AspectStills {
        aspect: aspect.name().into(),
        stills,
        contact_sheet: sheet,
    })
}

/// Run every check on what has been made so far.
pub fn check(project: &Project, gate: &Gate) -> Result<CheckReport> {
    let t = &project.timeline;
    let paths = project.build_paths();
    let dir = paths.check();
    let mut problems = Vec::new();
    let font = project.font_files().into_iter().find(|f| {
        project
            .brand
            .fonts
            .mono
            .files
            .iter()
            .any(|m| f.ends_with(m))
    });

    let mut stills = Vec::new();
    for (aspect, plan) in &t.aspects {
        let master = paths.master(*aspect);
        let picture = if master.is_file() {
            master
        } else {
            paths.render(*aspect)
        };
        if !picture.is_file() {
            problems.push(format!("{aspect}: no master to take stills from"));
            continue;
        }
        stills.push(stills_from_master(
            &picture,
            &still_times(t, plan),
            &dir,
            *aspect,
            font.as_deref(),
        )?);
    }

    let reading = reading(t);
    for r in reading.iter().filter(|r| !r.ok) {
        problems.push(format!(
            "{}: \"{}\" is fully on screen {:.2} s; {} words need {:.2} s",
            r.aspect, r.text, r.fully_visible_seconds, r.words, r.needed_seconds
        ));
    }

    let mut sounds: Vec<PathBuf> = t
        .aspects
        .keys()
        .map(|a| paths.master(*a))
        .filter(|p| p.is_file())
        .collect();
    if let Ok(entries) = std::fs::read_dir(paths.deliver()) {
        let mut delivered: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "mp4"))
            .filter(|p| {
                hivemotion_ffmpeg::probe(p)
                    .map(|pr| pr.audio.is_some())
                    .unwrap_or(false)
            })
            .collect();
        delivered.sort();
        sounds.extend(delivered);
    }
    let mut verdicts = Vec::new();
    for file in sounds {
        let v = loudness(&file, gate)?;
        if !v.passed {
            problems.push(format!("{}: {}", file.display(), v.problems.join("; ")));
        }
        verdicts.push(v);
    }
    Ok(CheckReport {
        passed: problems.is_empty(),
        stills,
        reading,
        loudness: verdicts,
        problems,
    })
}
