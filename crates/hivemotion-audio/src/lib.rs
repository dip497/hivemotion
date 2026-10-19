//! Sound for a film: the music laid out on its beats (mixed by fframes), mastered with ffmpeg's
//! two-pass loudnorm in linear mode and a limiter, then measured again and held to the gate:
//! −15 to −13 LUFS integrated and at most −1 dBTP true peak.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use fframes::{
    AudioMap, AudioMixOptions, AudioMixer, AudioTimelineSamples, AudioTimestamp, AudioTrack,
    MediaDirectory, MediaProvider, TimeBase,
};
use hivemotion_ffmpeg::{ffmpeg, last_json_object, probe, secs};
use hivemotion_spec::film::{Aspect, Loudness};
use hivemotion_spec::timeline::MusicPiece;
use hivemotion_spec::Project;
use serde::Serialize;

pub const SAMPLE_RATE: usize = 48_000;
/// Sample ceiling of the final limiter, about −1 dBFS.
pub const CEILING: f64 = 0.891;

/// The loudness the delivered sound must measure.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Gate {
    pub min_integrated: f64,
    pub max_integrated: f64,
    pub max_true_peak: f64,
    /// When false, `judge` reports what it measured but passes everything.
    pub enforced: bool,
}

impl Gate {
    pub const STANDARD: Gate = Gate {
        min_integrated: -15.0,
        max_integrated: -13.0,
        max_true_peak: -1.0,
        enforced: true,
    };

    pub fn disabled() -> Gate {
        Gate {
            enforced: false,
            ..Gate::STANDARD
        }
    }

    pub fn judge(&self, m: &Measured) -> Verdict {
        let mut problems = Vec::new();
        if !(self.min_integrated..=self.max_integrated).contains(&m.integrated) {
            problems.push(format!(
                "integrated loudness {:.1} LUFS is outside {} to {} LUFS",
                m.integrated, self.min_integrated, self.max_integrated
            ));
        }
        if m.true_peak > self.max_true_peak {
            problems.push(format!(
                "true peak {:.1} dBTP is above {} dBTP",
                m.true_peak, self.max_true_peak
            ));
        }
        Verdict {
            passed: problems.is_empty() || !self.enforced,
            enforced: self.enforced,
            measured: m.clone(),
            problems,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Measured {
    pub file: PathBuf,
    pub integrated: f64,
    pub true_peak: f64,
    pub range: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Verdict {
    pub passed: bool,
    pub enforced: bool,
    pub measured: Measured,
    pub problems: Vec<String>,
}

/// Measure a file's sound with ffmpeg's ebur128 (true peak on).
pub fn measure(file: &Path) -> Result<Measured> {
    let log = ffmpeg([
        "-i".as_ref(),
        file.as_os_str(),
        "-map".as_ref(),
        "0:a:0".as_ref(),
        "-af".as_ref(),
        "ebur128=peak=true".as_ref(),
        "-f".as_ref(),
        "null".as_ref(),
        "-".as_ref(),
    ] as [&std::ffi::OsStr; 9])?;
    let summary = log
        .rfind("Summary:")
        .map(|i| &log[i..])
        .ok_or_else(|| anyhow!("ebur128 printed no summary for {}", file.display()))?;
    let value = |section: &str, key: &str| -> Option<f64> {
        let at = summary.find(section)?;
        summary[at..]
            .lines()
            .find(|l| l.trim_start().starts_with(key))?
            .split_whitespace()
            .nth(1)?
            .parse()
            .ok()
    };
    Ok(Measured {
        file: file.to_path_buf(),
        integrated: value("Integrated loudness", "I:")
            .ok_or_else(|| anyhow!("no integrated loudness for {}", file.display()))?,
        true_peak: value("True peak", "Peak:")
            .ok_or_else(|| anyhow!("no true peak for {}", file.display()))?,
        range: value("Loudness range", "LRA:").unwrap_or(0.0),
    })
}

/// Measure `file` and hold it to `gate`.
pub fn check(file: &Path, gate: &Gate) -> Result<Verdict> {
    Ok(gate.judge(&measure(file)?))
}

#[derive(Debug, Clone, Serialize)]
pub struct Mastered {
    pub output: PathBuf,
    pub before: Measured,
    pub normalization_type: String,
    pub after: Measured,
}

fn limiter(limit: f64) -> String {
    format!("alimiter=limit={limit}:attack=1:release=50:level=disabled:latency=true")
}

/// Two-pass loudnorm (linear) to `target`, then a limiter at −1 dBFS without its own gain.
/// Fails if loudnorm falls back to dynamic mode.
pub fn master_file(input: &Path, output: &Path, target: &Loudness) -> Result<Mastered> {
    let before = measure(input)?;
    let pre = target
        .pre_limit
        .map(|p| format!("{},", limiter(p)))
        .unwrap_or_default();
    let norm = format!(
        "loudnorm=I={}:TP={}:LRA={}",
        target.integrated, target.true_peak, target.range
    );
    let first = ffmpeg([
        "-i".as_ref(),
        input.as_os_str(),
        "-af".as_ref(),
        format!("{pre}{norm}:print_format=json").as_ref(),
        "-f".as_ref(),
        "null".as_ref(),
        "-".as_ref(),
    ] as [&std::ffi::OsStr; 7])?;
    let m = last_json_object(&first).ok_or_else(|| anyhow!("loudnorm printed no measurement"))?;
    let get = |k: &str| -> Result<String> {
        m.get(k)
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| anyhow!("loudnorm's first pass has no {k}"))
    };
    let second_filter = format!(
        "{pre}{norm}:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset={}:linear=true:print_format=json,{},aresample={SAMPLE_RATE}",
        get("input_i")?,
        get("input_tp")?,
        get("input_lra")?,
        get("input_thresh")?,
        get("target_offset")?,
        limiter(CEILING),
    );
    if let Some(dir) = output.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let second = ffmpeg([
        "-i".as_ref(),
        input.as_os_str(),
        "-af".as_ref(),
        second_filter.as_ref(),
        "-ar".as_ref(),
        SAMPLE_RATE.to_string().as_ref(),
        "-c:a".as_ref(),
        "pcm_s24le".as_ref(),
        output.as_os_str(),
    ] as [&std::ffi::OsStr; 9])?;
    let report =
        last_json_object(&second).ok_or_else(|| anyhow!("loudnorm printed no second report"))?;
    let kind = report
        .get("normalization_type")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if kind != "linear" {
        bail!(
            "loudnorm fell back to '{kind}' normalization (not linear): the source's range or peaks \
             do not fit the target; lower music.loudness.pre_limit or raise music.loudness.range"
        );
    }
    Ok(Mastered {
        output: output.to_path_buf(),
        before,
        normalization_type: kind,
        after: measure(output)?,
    })
}

/// fframes tracks for the music as `aspect` plays it.
fn tracks<'a>(
    name: &'a str,
    pieces: &[MusicPiece],
    join: f64,
    fade_in: f64,
    fade_out: f64,
    length: f64,
) -> Vec<AudioTrack<'a>> {
    let last = pieces.len().saturating_sub(1);
    pieces
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let end = if i == last { length } else { p.end + join };
            AudioTrack::new(
                name,
                AudioTimestamp::Second(p.start as f32)..AudioTimestamp::Second(end as f32),
            )
            .offset(p.track_start as f32)
            .fade_in(if i == 0 { fade_in } else { join } as f32)
            .fade_out(if i == last { fade_out } else { join } as f32)
        })
        .collect()
}

/// Lay the music out on `aspect`'s timeline with fframes' mixer: 48 kHz float WAV, unmastered.
pub fn arrange(project: &Project, aspect: Aspect) -> Result<PathBuf> {
    let t = &project.timeline;
    let music = t
        .music
        .as_ref()
        .ok_or_else(|| anyhow!("the film has no music"))?;
    let plan = t
        .aspects
        .get(&aspect)
        .ok_or_else(|| anyhow!("the film has no {aspect} aspect"))?;
    if !music.file.is_file() {
        bail!("music: no file at {}", music.file.display());
    }
    let paths = project.build_paths();
    let track_dir = paths.root.join("track");
    let _ = std::fs::remove_dir_all(&track_dir);
    std::fs::create_dir_all(&track_dir)?;
    let name = music
        .file
        .file_name()
        .ok_or_else(|| anyhow!("music: no file name"))?
        .to_string_lossy()
        .to_string();
    std::fs::copy(&music.file, track_dir.join(&name))
        .with_context(|| format!("copying {}", music.file.display()))?;
    let directory = MediaDirectory::read_folder(&track_dir)
        .map_err(|e| anyhow!("reading {}: {e:?}", track_dir.display()))?;
    let provider = directory
        .process_media_source()
        .map_err(|e| anyhow!("decoding {}: {e:?}", music.file.display()))?;

    let length = plan.length();
    let pieces = plan.music_pieces(music);
    let map = AudioMap::from(tracks(
        &name,
        &pieces,
        music.join,
        music.fade_in,
        music.fade_out,
        length,
    ));
    let out = paths.music_mix(aspect);
    let (left, right) = mix(&map, &provider, length)?;
    std::fs::create_dir_all(&paths.root)?;
    std::fs::write(&out, fframes::encode_wav(&left, &right, SAMPLE_RATE, true))?;
    Ok(out)
}

fn mix<'m>(
    map: &AudioMap<'_>,
    provider: &'m fframes::DynamicMediaProvider<'m>,
    length: f64,
) -> Result<(Vec<f32>, Vec<f32>)> {
    let time_base = TimeBase {
        fps: 60,
        sample_rate: SAMPLE_RATE,
    };
    let resolved = map
        .resolve_with_scenes::<AudioTimelineSamples>(None, &time_base, |file| {
            provider
                .resolve_audio(file)
                .map(|a| f64::from(a.duration_in_seconds()))
                .ok_or_else(|| {
                    fframes::error::FFramesError::RequiredAudioNotFound(file.to_string())
                })
        })
        .map_err(|e| anyhow!("laying out the music: {e:?}"))?;
    let total = (length * SAMPLE_RATE as f64).round() as usize;
    let mut mixer = AudioMixer::new(
        resolved.as_ref(),
        Some(provider as &dyn MediaProvider),
        SAMPLE_RATE,
        0..total,
        total,
        AudioMixOptions {
            master_gain_db: 0.0,
            limiter: None,
            declick: true,
        },
    );
    if !mixer.missing_files().is_empty() {
        bail!("music not loaded: {:?}", mixer.missing_files());
    }
    Ok(mixer.render_all())
}

#[derive(Debug, Clone, Serialize)]
pub struct MasterReport {
    pub aspect: String,
    pub mix: PathBuf,
    pub mastered: Mastered,
    pub audio_gate: Verdict,
    /// The render with this sound, when the render exists.
    pub master: Option<PathBuf>,
    pub master_gate: Option<Verdict>,
}

/// Arrange, master and gate the sound of one aspect; put it into the master file if the
/// picture has been rendered.
pub fn master(project: &Project, aspect: Aspect, gate: &Gate) -> Result<MasterReport> {
    let music = project
        .timeline
        .music
        .as_ref()
        .ok_or_else(|| anyhow!("the film has no music"))?;
    let paths = project.build_paths();
    let mix = arrange(project, aspect)?;
    let mastered = master_file(&mix, &paths.audio(aspect), &music.loudness)?;
    let audio_gate = gate.judge(&mastered.after);
    if !audio_gate.passed {
        bail!(
            "the mastered sound fails the loudness gate: {}",
            audio_gate.problems.join("; ")
        );
    }
    let render = paths.render(aspect);
    let (master, master_gate) = if render.is_file() {
        let out = paths.master(aspect);
        let length = probe(&render)?.duration;
        ffmpeg([
            "-v".as_ref(),
            "error".as_ref(),
            "-i".as_ref(),
            render.as_os_str(),
            "-i".as_ref(),
            mastered.output.as_os_str(),
            "-map".as_ref(),
            "0:v:0".as_ref(),
            "-map".as_ref(),
            "1:a:0".as_ref(),
            "-c:v".as_ref(),
            "copy".as_ref(),
            "-c:a".as_ref(),
            "pcm_s24le".as_ref(),
            "-t".as_ref(),
            secs(length).as_ref(),
            out.as_os_str(),
        ] as [&std::ffi::OsStr; 17])?;
        let verdict = check(&out, gate)?;
        if !verdict.passed {
            bail!(
                "the master's sound fails the loudness gate: {}",
                verdict.problems.join("; ")
            );
        }
        (Some(out), Some(verdict))
    } else {
        (None, None)
    };
    Ok(MasterReport {
        aspect: aspect.name().into(),
        mix,
        mastered,
        audio_gate,
        master,
        master_gate,
    })
}
