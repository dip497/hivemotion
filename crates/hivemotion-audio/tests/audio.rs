use std::path::{Path, PathBuf};
use std::process::Command;

use hivemotion_audio::{arrange, check, master_file, Gate};
use hivemotion_spec::film::{Aspect, Loudness};

fn run_ffmpeg(args: &[&str]) {
    let ok = Command::new("ffmpeg")
        .args(["-hide_banner", "-v", "error", "-y"])
        .args(args)
        .status()
        .unwrap()
        .success();
    assert!(ok, "ffmpeg {args:?}");
}

/// 20 s of pink noise and a tone, far louder than any platform wants (about −7 LUFS, peaks clipped at full scale).
fn loud_track(dir: &Path) -> PathBuf {
    let out = dir.join("loud.wav");
    run_ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        "anoisesrc=color=pink:amplitude=0.8:seed=7:d=20:r=48000",
        "-f",
        "lavfi",
        "-i",
        "sine=f=110:d=20:r=48000",
        "-filter_complex",
        "[0][1]amix=inputs=2:normalize=0,volume=3.0,aformat=channel_layouts=stereo",
        "-c:a",
        "pcm_s16le",
        out.to_str().unwrap(),
    ]);
    out
}

#[test]
fn a_loud_track_is_mastered_into_the_gate() {
    let dir = tempfile::tempdir().unwrap();
    let loud = loud_track(dir.path());
    let target = Loudness {
        integrated: -14.0,
        true_peak: -2.0,
        range: 11.0,
        pre_limit: None,
    };
    let mastered = master_file(&loud, &dir.path().join("mastered.wav"), &target).unwrap();
    assert!(
        mastered.before.integrated > -10.0,
        "the input was not loud: {:?}",
        mastered.before
    );
    assert_eq!(mastered.normalization_type, "linear");
    let verdict = check(&mastered.output, &Gate::STANDARD).unwrap();
    assert!(verdict.passed, "{:?}", verdict);
}

#[test]
fn the_gate_is_what_rejects_a_loud_file() {
    let dir = tempfile::tempdir().unwrap();
    let loud = loud_track(dir.path());
    let gated = check(&loud, &Gate::STANDARD).unwrap();
    assert!(!gated.passed);
    assert!(
        gated
            .problems
            .iter()
            .any(|p| p.contains("integrated loudness")),
        "{:?}",
        gated.problems
    );
    // the same file with the gate's check turned off goes through
    let ungated = check(&loud, &Gate::disabled()).unwrap();
    assert!(ungated.passed);
}

/// Seconds at which the mix first goes above half scale, at or after `from`.
fn first_hit(wav: &Path, from: f64) -> f64 {
    let out = Command::new("ffmpeg")
        .args(["-hide_banner", "-v", "error", "-i"])
        .arg(wav)
        .args(["-ac", "1", "-f", "f32le", "-ar", "48000", "-"])
        .output()
        .unwrap();
    let samples: Vec<f32> = out
        .stdout
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect();
    let start = (from * 48000.0) as usize;
    let i = samples[start..]
        .iter()
        .position(|s| s.abs() > 0.5)
        .expect("no hit");
    (start + i) as f64 / 48000.0
}

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/hivemind-launch")
}

#[test]
fn the_music_cues_land_on_their_beats_in_every_aspect() {
    let dir = tempfile::tempdir().unwrap();
    for f in ["brand.yaml", "mark.svg"] {
        std::fs::copy(example().join(f), dir.path().join(f)).unwrap();
    }
    std::fs::create_dir_all(dir.path().join("fonts")).unwrap();
    for f in std::fs::read_dir(example().join("fonts")).unwrap() {
        let f = f.unwrap();
        std::fs::copy(f.path(), dir.path().join("fonts").join(f.file_name())).unwrap();
    }
    let film = std::fs::read_to_string(example().join("film.yaml"))
        .unwrap()
        .replace("music/cyberpunk-music-601089.mp3", "music/clicks.wav");
    std::fs::write(dir.path().join("film.yaml"), film).unwrap();
    std::fs::create_dir_all(dir.path().join("music")).unwrap();
    // a silent track with a click at the drop (28.10 s) and at the final hit (153.80 s)
    run_ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        "aevalsrc='if(between(t,28.10,28.12)+between(t,153.80,153.82),0.9,0)':s=48000:d=170",
        "-ac",
        "2",
        "-c:a",
        "pcm_f32le",
        dir.path().join("music/clicks.wav").to_str().unwrap(),
    ]);
    let project = hivemotion_spec::load(&dir.path().join("film.yaml")).unwrap();

    let master = arrange(&project, Aspect::Landscape).unwrap();
    assert!((first_hit(&master, 0.0) - 10.0).abs() < 0.002);
    assert!((first_hit(&master, 11.0) - 61.9).abs() < 0.002);

    let vertical = arrange(&project, Aspect::Vertical).unwrap();
    assert!((first_hit(&vertical, 0.0) - 10.0).abs() < 0.002);
    assert!((first_hit(&vertical, 11.0) - 36.9).abs() < 0.002);
}
