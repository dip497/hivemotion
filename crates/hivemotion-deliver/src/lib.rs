//! Files for each platform, made from the ProRes masters with the system ffmpeg: H.264 High,
//! 4:2:0, BT.709 tags, AAC 48 kHz, `+faststart`; a chosen poster frame as frame 0 (the length is
//! unchanged); cutdowns cut from named beats.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use hivemotion_ffmpeg::{ffmpeg, probe, secs, Probe};
use hivemotion_spec::film::Aspect;
use hivemotion_spec::timeline::TimedCutdown;
use hivemotion_spec::Project;
use serde::Serialize;

/// GitHub's attachment limit on the free plan.
pub const README_LIMIT_BYTES: u64 = 10 * 1000 * 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PresetName {
    X,
    Youtube,
    Shorts,
    Linkedin,
    ReadmeLoop,
    Producthunt,
}

impl PresetName {
    pub const ALL: [PresetName; 6] = [
        PresetName::X,
        PresetName::Youtube,
        PresetName::Shorts,
        PresetName::Linkedin,
        PresetName::ReadmeLoop,
        PresetName::Producthunt,
    ];

    pub fn name(self) -> &'static str {
        match self {
            PresetName::X => "x",
            PresetName::Youtube => "youtube",
            PresetName::Shorts => "shorts",
            PresetName::Linkedin => "linkedin",
            PresetName::ReadmeLoop => "readme-loop",
            PresetName::Producthunt => "producthunt",
        }
    }

    pub fn parse(text: &str) -> Option<PresetName> {
        PresetName::ALL.into_iter().find(|p| p.name() == text)
    }
}

/// What one platform takes.
#[derive(Debug, Clone, Serialize)]
pub struct Preset {
    pub name: PresetName,
    /// The aspects it can use, best first.
    pub aspects: &'static [Aspect],
    pub crf: u32,
    pub max_rate: Option<&'static str>,
    pub audio_bitrate: Option<&'static str>,
    pub max_seconds: Option<f64>,
    pub max_bytes: Option<u64>,
    /// Width of the file; the height follows the aspect.
    pub width: Option<u32>,
}

pub fn preset(name: PresetName) -> Preset {
    use Aspect::*;
    match name {
        PresetName::X => Preset {
            name,
            aspects: &[Landscape, Portrait, Vertical],
            crf: 18,
            max_rate: Some("25M"),
            audio_bitrate: Some("256k"),
            max_seconds: Some(140.0),
            max_bytes: Some(512 * 1000 * 1000),
            width: None,
        },
        PresetName::Youtube | PresetName::Producthunt => Preset {
            name,
            aspects: &[Landscape],
            crf: 16,
            max_rate: None,
            audio_bitrate: Some("384k"),
            max_seconds: None,
            max_bytes: None,
            width: None,
        },
        PresetName::Shorts => Preset {
            name,
            aspects: &[Vertical],
            crf: 18,
            max_rate: None,
            audio_bitrate: Some("256k"),
            max_seconds: Some(180.0),
            max_bytes: None,
            width: None,
        },
        PresetName::Linkedin => Preset {
            name,
            aspects: &[Portrait, Landscape],
            crf: 18,
            max_rate: Some("30M"),
            audio_bitrate: Some("192k"),
            max_seconds: Some(600.0),
            max_bytes: Some(5 * 1000 * 1000 * 1000),
            width: None,
        },
        PresetName::ReadmeLoop => Preset {
            name,
            aspects: &[Landscape],
            crf: 26,
            max_rate: None,
            audio_bitrate: None,
            max_seconds: Some(12.0),
            max_bytes: Some(README_LIMIT_BYTES),
            width: Some(1280),
        },
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Delivered {
    pub name: String,
    pub file: PathBuf,
    pub aspect: String,
    pub probe: Probe,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeliverReport {
    pub posters: Vec<PathBuf>,
    pub files: Vec<Delivered>,
}

fn args(list: &[&str]) -> Vec<OsString> {
    list.iter().map(OsString::from).collect()
}

const COLOR_OUT: &str =
    "scale=out_color_matrix=bt709:out_range=tv:flags=lanczos+accurate_rnd+full_chroma_int,format=yuv420p,setsar=1";

fn h264(crf: u32, fps: u32, max_rate: Option<&str>) -> Vec<OsString> {
    let mut a = args(&[
        "-c:v",
        "libx264",
        "-profile:v",
        "high",
        "-preset",
        "slow",
        "-crf",
        &crf.to_string(),
        "-g",
        &(fps / 2).max(1).to_string(),
        "-bf",
        "2",
        "-pix_fmt",
        "yuv420p",
        "-color_primaries",
        "bt709",
        "-color_trc",
        "bt709",
        "-colorspace",
        "bt709",
        "-color_range",
        "tv",
        "-movflags",
        "+faststart",
    ]);
    if let Some(rate) = max_rate {
        let bufsize = format!(
            "{}M",
            rate.trim_end_matches('M').parse::<u32>().unwrap_or(25) * 2
        );
        a.extend(args(&["-maxrate", rate, "-bufsize", &bufsize]));
    }
    a
}

fn aac(bitrate: &str) -> Vec<OsString> {
    args(&["-c:a", "aac", "-b:a", bitrate, "-ar", "48000", "-ac", "2"])
}

/// Blend frames down to the delivery rate (a shutter of two frames when halving).
fn blend(master_fps: u32, fps: u32) -> String {
    let n = (master_fps / fps.max(1)).max(1);
    if n > 1 {
        let weights = vec!["1"; n as usize].join(" ");
        format!("tmix=frames={n}:weights='{weights}',fps={fps}")
    } else {
        format!("fps={fps}")
    }
}

/// The poster frame of `aspect`, as a lossless PNG and a JPEG.
pub fn poster(
    project: &Project,
    aspect: Aspect,
    master: &Path,
    dir: &Path,
) -> Result<(PathBuf, PathBuf)> {
    let t = &project.timeline;
    let plan = &t.aspects[&aspect];
    let at = plan.render_time(t.poster).unwrap_or(0.0);
    let png = dir.join(format!("poster-{}.png", aspect.file_name()));
    let jpg = dir.join(format!("poster-{}.jpg", aspect.file_name()));
    let mut a = args(&["-v", "error", "-ss", &secs(at), "-i"]);
    a.push(master.into());
    a.extend(args(&["-frames:v", "1", "-update", "1"]));
    a.push(png.clone().into());
    ffmpeg(a)?;
    let mut a = args(&["-v", "error", "-i"]);
    a.push(png.clone().into());
    a.extend(args(&["-q:v", "1", "-pix_fmt", "yuvj444p", "-update", "1"]));
    a.push(jpg.clone().into());
    ffmpeg(a)?;
    Ok((png, jpg))
}

fn encode_preset(
    p: &Preset,
    master: &Path,
    poster_png: &Path,
    master_fps: u32,
    fps: u32,
    out: &Path,
) -> Result<()> {
    let m = probe(master)?;
    let v = m
        .video
        .as_ref()
        .ok_or_else(|| anyhow!("{} has no picture", master.display()))?;
    let (w, h) = match p.width {
        Some(w) => (w, (w * v.height / v.width) & !1),
        None => (v.width, v.height),
    };
    let graph = format!(
        "[0:v]{},scale={w}:{h}:flags=lanczos,format=gbrp[m];[1:v]scale={w}:{h}:flags=lanczos,format=gbrp[p];\
         [m][p]overlay=enable='eq(n,0)':shortest=1:format=gbrp,{COLOR_OUT}[v]",
        blend(master_fps, fps)
    );
    let mut a = args(&["-v", "error", "-i"]);
    a.push(master.into());
    a.extend(args(&["-loop", "1", "-i"]));
    a.push(poster_png.into());
    a.extend(args(&["-filter_complex", &graph, "-map", "[v]"]));
    match p.audio_bitrate {
        Some(rate) if m.audio.is_some() => {
            a.extend(args(&["-map", "0:a:0"]));
            a.extend(aac(rate));
        }
        _ => a.extend(args(&["-an"])),
    }
    a.extend(h264(p.crf, fps, p.max_rate));
    a.extend(args(&["-t", &secs(m.duration)]));
    a.push(out.into());
    ffmpeg(a)?;
    Ok(())
}

/// One cutdown from the 16:9 master. With `loop_blend`, the end is blended into the start so it
/// loops without a jump, and it has no sound.
pub fn cutdown(
    c: &TimedCutdown,
    master: &Path,
    master_fps: u32,
    fps: u32,
    out: &Path,
    width: Option<u32>,
    crf: u32,
) -> Result<()> {
    let scale = width
        .map(|w| format!(",scale={w}:-2:flags=lanczos"))
        .unwrap_or_default();
    let mut a = args(&["-v", "error", "-ss", &secs(c.start), "-i"]);
    a.push(master.into());
    let l = c.length;
    match c.loop_blend {
        Some(b) => {
            let graph = format!(
                "[0:v]{}{scale},format=gbrp,split[a][b];[a]trim=0:{l},setpts=PTS-STARTPTS[m];\
                 [b]trim={l}:{},setpts=PTS-STARTPTS[t];[t][m]xfade=transition=fade:duration={b}:offset=0,trim=0:{l},{COLOR_OUT}[v]",
                blend(master_fps, fps),
                l + b
            );
            a.extend(args(&[
                "-t",
                &secs(l + b + 0.1),
                "-filter_complex",
                &graph,
                "-map",
                "[v]",
                "-an",
            ]));
        }
        None => {
            let graph = format!(
                "[0:v]{}{scale},fade=t=out:st={}:d=0.4,{COLOR_OUT}[v]",
                blend(master_fps, fps),
                l - 0.4
            );
            a.extend(args(&[
                "-t",
                &secs(l),
                "-filter_complex",
                &graph,
                "-map",
                "[v]",
            ]));
            if c.sound {
                let fades = format!(
                    "afade=t=in:d=0.08,afade=t=out:st={}:d=0.6,alimiter=limit=0.89:level=disabled:latency=true",
                    l - 0.6
                );
                a.extend(args(&["-map", "0:a:0?", "-af", &fades]));
                a.extend(aac("256k"));
            } else {
                a.extend(args(&["-an"]));
            }
        }
    }
    a.extend(h264(crf, fps, None));
    a.extend(args(&["-t", &secs(l)]));
    a.push(out.into());
    ffmpeg(a)?;
    Ok(())
}

/// Make the posters, the cutdowns and every asked-for preset.
pub fn deliver(project: &Project, presets: &[PresetName]) -> Result<DeliverReport> {
    let t = &project.timeline;
    let paths = project.build_paths();
    let dir = paths.deliver();
    std::fs::create_dir_all(&dir)?;
    let mut posters = Vec::new();
    let mut poster_png = std::collections::BTreeMap::new();
    for aspect in t.aspects.keys() {
        let master = paths.master(*aspect);
        if master.is_file() {
            let (png, jpg) = poster(project, *aspect, &master, &dir)?;
            posters.push(jpg);
            poster_png.insert(*aspect, png);
        }
    }
    if poster_png.is_empty() {
        bail!("no master yet: run `hivemotion render` and `hivemotion master` first");
    }
    let mut files = Vec::new();
    let landscape = paths.master(Aspect::Landscape);
    for c in &t.cutdowns {
        if !landscape.is_file() {
            bail!(
                "cutdowns come from the 16:9 master, and {} does not exist",
                landscape.display()
            );
        }
        let out = dir.join(format!("{}.mp4", c.name));
        cutdown(c, &landscape, t.fps, t.finish_fps, &out, None, 15)?;
        files.push(Delivered {
            name: c.name.clone(),
            file: out.clone(),
            aspect: Aspect::Landscape.name().into(),
            probe: probe(&out)?,
            notes: vec![],
        });
    }
    for &name in presets {
        let p = preset(name);
        let Some(aspect) = p
            .aspects
            .iter()
            .copied()
            .find(|a| poster_png.contains_key(a))
        else {
            bail!(
                "{} needs a {} master",
                name.name(),
                p.aspects
                    .iter()
                    .map(|a| a.name())
                    .collect::<Vec<_>>()
                    .join(" or ")
            );
        };
        let master = paths.master(aspect);
        let out = dir.join(format!("{}.mp4", name.name()));
        let mut notes = Vec::new();
        if name == PresetName::ReadmeLoop {
            let c = t
                .cutdowns
                .iter()
                .find(|c| c.loop_blend.is_some())
                .cloned()
                .unwrap_or(TimedCutdown {
                    name: "readme-loop".into(),
                    start: t.poster,
                    length: 6.0_f64.min(t.length - t.poster),
                    loop_blend: Some(0.5),
                    sound: false,
                });
            notes.push(format!(
                "loop of {:.2}–{:.2} s",
                c.start,
                c.start + c.length
            ));
            let mut crf = p.crf;
            loop {
                cutdown(&c, &master, t.fps, t.finish_fps, &out, p.width, crf)?;
                let size = std::fs::metadata(&out)?.len();
                if size < README_LIMIT_BYTES || crf >= 40 {
                    break;
                }
                notes.push(format!("{size} bytes at crf {crf}; trying higher"));
                crf += 3;
            }
        } else {
            encode_preset(&p, &master, &poster_png[&aspect], t.fps, t.finish_fps, &out)?;
        }
        let pr = probe(&out)?;
        if let Some(max) = p.max_seconds {
            if pr.duration > max + 0.05 {
                notes.push(format!(
                    "{:.1} s is over {}'s {max} s limit",
                    pr.duration,
                    name.name()
                ));
            }
        }
        if let Some(max) = p.max_bytes {
            if pr.size_bytes > max {
                bail!(
                    "{} is {} bytes, over its limit of {max}",
                    out.display(),
                    pr.size_bytes
                );
            }
        }
        if name == PresetName::Producthunt {
            let png = &poster_png[&aspect];
            for (file, filter) in [
                (
                    "producthunt-gallery.jpg",
                    "scale=1270:760:force_original_aspect_ratio=increase,crop=1270:760",
                ),
                (
                    "producthunt-thumbnail.jpg",
                    "scale=240:240:force_original_aspect_ratio=increase,crop=240:240",
                ),
            ] {
                let mut a = args(&["-v", "error", "-i"]);
                a.push(png.into());
                a.extend(args(&["-vf", filter, "-q:v", "2", "-update", "1"]));
                a.push(dir.join(file).into());
                ffmpeg(a)?;
            }
            notes.push("Product Hunt takes a YouTube link: upload this file to YouTube; gallery and thumbnail images are beside it".into());
        }
        files.push(Delivered {
            name: name.name().into(),
            file: out,
            aspect: aspect.name().into(),
            probe: pr,
            notes,
        });
    }
    Ok(DeliverReport { posters, files })
}
