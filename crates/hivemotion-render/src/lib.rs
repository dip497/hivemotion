//! hivemotion's renderer: one fframes `Video` that draws any film.yaml, and `prep`, which cuts the
//! takes for it.
//!
//! The master is ProRes 4444 (yuv444p10le) tagged BT.709, with no sound; mastering adds the sound.
//! fframes is built without its GPL features, so H.264 never comes from this crate.

pub mod film_video;
pub mod fonts;
pub mod mark;
pub mod prep;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, bail, Context, Result};
use fframes::ffmpeg_sys_fframes::AVPixelFormat;
use fframes::{EncoderOptions, MediaDirectory, RenderOptions, YuvMatrix};
use hivemotion_spec::film::Aspect;
use hivemotion_spec::Project;
use serde::Serialize;

use film_video::{layout, FilmVideo, Look, TextFonts};
use fonts::pick;
use mark::Mark;

pub use prep::{prep, PrepReport};

/// Audio sample rate everywhere in hivemotion (fframes defaults to 44.1 kHz).
pub const SAMPLE_RATE: usize = 48_000;

#[derive(Debug, Clone, Serialize)]
pub struct RenderReport {
    pub aspect: String,
    pub output: PathBuf,
    pub frames: usize,
    pub seconds: f64,
    pub milliseconds_per_frame: f64,
    pub missing_takes: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Still {
    pub seconds: f64,
    pub frame: usize,
    pub file: PathBuf,
    pub problems: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StillsReport {
    pub aspect: String,
    pub stills: Vec<Still>,
    pub milliseconds_per_still: f64,
    pub missing_takes: Vec<String>,
    pub notes: Vec<String>,
}

enum Job<'j> {
    Master { output: &'j Path },
    Stills { seconds: &'j [f64], dir: &'j Path },
}

enum Done {
    Master(RenderReport),
    Stills(StillsReport),
}

struct Inputs {
    look: Look,
    mark: Option<Mark>,
    clips: HashSet<String>,
    missing: Vec<String>,
    media: PathBuf,
}

fn inputs(project: &Project) -> Result<Inputs> {
    let media = project.build_paths().media();
    if !media.is_dir() {
        bail!(
            "{} does not exist: run `hivemotion prep` first",
            media.display()
        );
    }
    let mut clips = HashSet::new();
    let t = &project.timeline;
    let mut missing = Vec::new();
    for (id, source) in t
        .shots
        .iter()
        .map(|s| (&s.id, &s.clip))
        .chain(t.insets.iter().map(|i| (&i.id, &i.clip)))
    {
        if media.join(format!("{id}.mp4")).is_file() {
            clips.insert(id.clone());
        } else {
            missing.push(source.display().to_string());
        }
    }
    let b = &project.brand;
    Ok(Inputs {
        look: Look {
            background: b.colors.background.clone(),
            text: b.colors.text.clone(),
            text_dim: b.colors.text_dim.clone(),
            text_faint: b.colors.text_faint.clone(),
            sans: b.fonts.sans.family.clone(),
            mono: b.fonts.mono.family.clone(),
            caption_weight: b.caption.weight,
            caption_tracking: b.caption.tracking,
            caption_line_height: b.caption.line_height,
        },
        mark: project.mark_file().map(|m| Mark::load(&m)).transpose()?,
        clips,
        missing,
        media,
    })
}

fn files(project: &Project, family_files: &[String]) -> Vec<PathBuf> {
    family_files
        .iter()
        .map(|f| project.brand_dir.join(f))
        .collect()
}

fn run<const W: usize, const H: usize, const FPS: usize>(
    project: &Project,
    aspect: Aspect,
    job: Job,
) -> Result<Done> {
    let inputs = inputs(project)?;
    let t = &project.timeline;
    let plan = t
        .aspects
        .get(&aspect)
        .ok_or_else(|| anyhow!("the film has no {aspect} aspect"))?;
    let b = &project.brand;
    let sans = files(project, &b.fonts.sans.files);
    let mono = files(project, &b.fonts.mono.files);
    let fonts = TextFonts {
        caption: pick(&sans, &b.fonts.sans.family, b.caption.weight)?,
        title: pick(&sans, &b.fonts.sans.family, 700)?,
        line: pick(&sans, &b.fonts.sans.family, 500)?,
        mono: pick(&mono, &b.fonts.mono.family, 500)?,
    };
    let (captions, end_card, notes) = layout(t, plan, &inputs.look, &fonts, &b.name);
    let frames = (plan.length() * FPS as f64).round() as usize;
    let video = FilmVideo::<W, H, FPS> {
        timeline: t,
        plan,
        look: inputs.look.clone(),
        mark: inputs.mark.clone(),
        clips: inputs.clips.clone(),
        captions,
        end_card,
        frames,
    };
    let directory = MediaDirectory::read_folder(&inputs.media)
        .map_err(|e| anyhow!("reading {}: {e:?}", inputs.media.display()))?;
    let provider = directory
        .process_media_source()
        .map_err(|e| anyhow!("loading media: {e:?}"))?;
    let params: [(&str, &str); 1] = [("profile", "4444")];
    let options = RenderOptions {
        media: Some(&provider),
        load_system_fonts: false,
        default_font: &b.fonts.sans.family,
        video_encoder_options: EncoderOptions {
            preferred_encoder: Some("prores_ks"),
            pixel_format: AVPixelFormat::AV_PIX_FMT_YUV444P10LE,
            color_matrix: YuvMatrix::Bt709,
            codec_params: Some(&params),
            ..Default::default()
        },
        audio_encoder_options: EncoderOptions {
            sample_rate: SAMPLE_RATE,
            ..Default::default()
        },
        ..Default::default()
    };
    match job {
        Job::Master { output } => {
            if let Some(dir) = output.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let started = Instant::now();
            fframes::render(
                output,
                &video,
                fframes::cpu::CpuRenderingBackend::default(),
                &options,
            )
            .map_err(|e| anyhow!("render failed: {e:?}"))?;
            let seconds = started.elapsed().as_secs_f64();
            Ok(Done::Master(RenderReport {
                aspect: aspect.name().into(),
                output: output.to_path_buf(),
                frames,
                seconds,
                milliseconds_per_frame: seconds * 1000.0 / frames.max(1) as f64,
                missing_takes: inputs.missing,
                notes,
            }))
        }
        Job::Stills { seconds, dir } => {
            std::fs::create_dir_all(dir)?;
            let mut previewer = fframes::Previewer::new(&video, &options)
                .map_err(|e| anyhow!("cannot prepare the preview: {e:?}"))?;
            let mut renderer = fframes::CpuFrameRenderer::default();
            let started = Instant::now();
            let mut stills = Vec::new();
            for &s in seconds {
                let index = ((s * FPS as f64).round() as usize).min(frames.saturating_sub(1));
                let (pixels, report) = previewer
                    .render_inspected(index, &mut renderer)
                    .map_err(|e| anyhow!("frame {index}: {e:?}"))?;
                let file = dir.join(format!("{}-{:07.3}s.png", aspect.file_name(), s));
                pixels
                    .save_png(&file)
                    .map_err(|e| anyhow!("writing {}: {e:?}", file.display()))?;
                stills.push(Still {
                    seconds: s,
                    frame: index,
                    file,
                    problems: report
                        .diagnostics
                        .iter()
                        .map(|d| format!("{d:?}"))
                        .collect(),
                });
            }
            let elapsed = started.elapsed().as_secs_f64();
            Ok(Done::Stills(StillsReport {
                aspect: aspect.name().into(),
                milliseconds_per_still: elapsed * 1000.0 / stills.len().max(1) as f64,
                stills,
                missing_takes: inputs.missing,
                notes,
            }))
        }
    }
}

/// fframes takes size and frame rate as constants, so each pair is its own type.
macro_rules! dispatch {
    ($project:expr, $aspect:expr, $job:expr; $( $fps:literal ),*) => {{
        let fps = $project.timeline.fps;
        match ($aspect, fps) {
            $(
                (Aspect::Landscape, $fps) => run::<1920, 1080, $fps>($project, $aspect, $job),
                (Aspect::Vertical, $fps) => run::<1080, 1920, $fps>($project, $aspect, $job),
                (Aspect::Portrait, $fps) => run::<1080, 1350, $fps>($project, $aspect, $job),
            )*
            _ => bail!("{fps} fps is not supported; use 24, 25, 30, 50 or 60"),
        }
    }};
}

/// Render one aspect to its ProRes 4444 master picture (no sound).
pub fn render_master(project: &Project, aspect: Aspect) -> Result<RenderReport> {
    let output = project.build_paths().render(aspect);
    let job = Job::Master { output: &output };
    match dispatch!(project, aspect, job; 24, 25, 30, 50, 60)? {
        Done::Master(r) => Ok(r),
        Done::Stills(_) => unreachable!(),
    }
}

/// Render single frames at `seconds` of this aspect's timeline to PNG files in `dir`.
pub fn render_stills(
    project: &Project,
    aspect: Aspect,
    seconds: &[f64],
    dir: &Path,
) -> Result<StillsReport> {
    let job = Job::Stills { seconds, dir };
    match dispatch!(project, aspect, job; 24, 25, 30, 50, 60)? {
        Done::Stills(r) => Ok::<_, anyhow::Error>(r),
        Done::Master(_) => unreachable!(),
    }
    .context("rendering stills")
}

/// The aspects the film declares, or the one named.
pub fn aspects(project: &Project, only: Option<&str>) -> Result<Vec<Aspect>> {
    match only {
        None => Ok(project.timeline.aspects.keys().copied().collect()),
        Some(name) => {
            let a =
                Aspect::parse(name).ok_or_else(|| anyhow!("'{name}' is not 16:9, 9:16 or 4:5"))?;
            if !project.timeline.aspects.contains_key(&a) {
                bail!("the film has no {a} aspect");
            }
            Ok(vec![a])
        }
    }
}
