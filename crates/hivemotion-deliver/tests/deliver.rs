//! Delivers the Hivemind example from stand-in masters (a moving test pattern with a tone) and
//! checks every file with ffprobe.

use std::path::{Path, PathBuf};
use std::process::Command;

use hivemotion_deliver::{deliver, PresetName, README_LIMIT_BYTES};
use hivemotion_ffmpeg::probe;

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/hivemind-launch")
}

/// A short film (12 s; its vertical cut keeps 0–6 s and 9–12 s) with the example's brand, so
/// the presets encode in seconds.
const FILM: &str = r#"
title: Short
brand: brand.yaml
fps: 60
beats: { start: 0.0, demo: 3.0, cut: 6.0, ending: 9.0, end: 12.0 }
length: end
aspects:
  "16:9":
    caption: { left: 96, bottom: 88, size: 78 }
  "9:16":
    caption: { left: 64, top: 150, size: 70 }
    keep:
      - { from: start, to: cut }
      - { from: ending, to: end }
shots:
  - { id: opening, clip: takes/opening.mp4, from: start }
finish: { fps: 30, poster: demo+0.6 }
cutdowns:
  - { name: demo-clip, from: demo, length: 4 }
  - { name: demo-loop, from: start+1, length: 3, loop_blend: 0.5, sound: false }
"#;

fn short_film(dir: &Path, with_vertical: bool) {
    for f in ["brand.yaml", "mark.svg"] {
        std::fs::copy(example().join(f), dir.join(f)).unwrap();
    }
    std::fs::create_dir_all(dir.join("fonts")).unwrap();
    for f in std::fs::read_dir(example().join("fonts")).unwrap() {
        let f = f.unwrap();
        std::fs::copy(f.path(), dir.join("fonts").join(f.file_name())).unwrap();
    }
    let film = if with_vertical {
        FILM.to_string()
    } else {
        FILM.replace("  \"9:16\":\n    caption: { left: 64, top: 150, size: 70 }\n    keep:\n      - { from: start, to: cut }\n      - { from: ending, to: end }\n", "")
    };
    std::fs::write(dir.join("film.yaml"), film).unwrap();
}

/// A stand-in master: 60 fps, 4:4:4, BT.709, PCM sound like `hivemotion master` writes, but
/// lossy-compressed so the test does not write gigabytes of ProRes.
fn stand_in_master(file: &Path, size: &str, seconds: f64) {
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    let ok = Command::new("ffmpeg")
        .args(["-hide_banner", "-v", "error", "-y", "-f", "lavfi", "-i"])
        .arg(format!("testsrc2=s={size}:r=60:d={seconds}"))
        .args(["-f", "lavfi", "-i"])
        .arg(format!("sine=f=220:r=48000:d={seconds}"))
        .args([
            "-af",
            "volume=-12dB",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-crf",
            "16",
            "-pix_fmt",
            "yuv444p",
            "-colorspace",
            "bt709",
            "-color_primaries",
            "bt709",
            "-color_trc",
            "bt709",
            "-c:a",
            "pcm_s24le",
        ])
        .arg(file)
        .status()
        .unwrap()
        .success();
    assert!(ok);
}

/// Mean difference between two frames, shrunk to 64×36 RGB (pictures read as BT.709).
fn difference(a: (&Path, f64), b: (&Path, f64)) -> f64 {
    let grab = |(file, at): (&Path, f64)| {
        Command::new("ffmpeg")
            .args(["-hide_banner", "-v", "error", "-ss", &format!("{at}"), "-i"])
            .arg(file)
            .args([
                "-frames:v",
                "1",
                "-vf",
                "scale=64:36:in_color_matrix=bt709:in_range=tv,format=rgb24",
                "-f",
                "rawvideo",
                "-",
            ])
            .output()
            .unwrap()
            .stdout
    };
    let (x, y) = (grab(a), grab(b));
    x.iter()
        .zip(&y)
        .map(|(p, q)| (*p as f64 - *q as f64).abs())
        .sum::<f64>()
        / x.len() as f64
}

#[test]
fn every_preset_has_the_codec_size_and_length_its_platform_takes() {
    let dir = tempfile::tempdir().unwrap();
    short_film(dir.path(), true);
    let build = dir.path().join("build");
    stand_in_master(&build.join("master-16x9.mov"), "1920x1080", 12.0);
    stand_in_master(&build.join("master-9x16.mov"), "1080x1920", 9.0);
    let project = hivemotion_spec::load(&dir.path().join("film.yaml")).unwrap();
    let report = deliver(&project, &PresetName::ALL).unwrap();

    let expect = [
        ("x", 1920, 1080, 12.0, true),
        ("youtube", 1920, 1080, 12.0, true),
        ("producthunt", 1920, 1080, 12.0, true),
        ("linkedin", 1920, 1080, 12.0, true),
        ("shorts", 1080, 1920, 9.0, true),
        ("readme-loop", 1280, 720, 3.0, false),
        ("demo-clip", 1920, 1080, 4.0, true),
        ("demo-loop", 1920, 1080, 3.0, false),
    ];
    for (name, w, h, seconds, sound) in expect {
        let d = report
            .files
            .iter()
            .find(|f| f.name == name)
            .unwrap_or_else(|| panic!("no {name}"));
        let p = probe(&d.file).unwrap();
        let v = p.video.as_ref().unwrap();
        assert_eq!(
            (v.codec.as_str(), v.profile.as_str()),
            ("h264", "High"),
            "{name}"
        );
        assert_eq!((v.width, v.height), (w, h), "{name}");
        assert_eq!(v.pix_fmt, "yuv420p", "{name}");
        assert_eq!(v.color_space, "bt709", "{name}");
        assert!((v.fps - 30.0).abs() < 0.01, "{name}: {} fps", v.fps);
        assert!(
            (p.duration - seconds).abs() < 0.1,
            "{name}: {} s",
            p.duration
        );
        match (&p.audio, sound) {
            (Some(a), true) => {
                assert_eq!((a.codec.as_str(), a.sample_rate), ("aac", 48000), "{name}")
            }
            (None, false) => {}
            (a, _) => panic!("{name}: sound {a:?}, expected {sound}"),
        }
    }
    let readme = report
        .files
        .iter()
        .find(|f| f.name == "readme-loop")
        .unwrap();
    assert!(
        readme.probe.size_bytes < README_LIMIT_BYTES,
        "{} bytes",
        readme.probe.size_bytes
    );
    // the moov atom comes first (+faststart): its box precedes mdat
    let bytes = std::fs::read(
        &report
            .files
            .iter()
            .find(|f| f.name == "youtube")
            .unwrap()
            .file,
    )
    .unwrap();
    let find = |tag: &[u8]| bytes.windows(4).position(|w| w == tag).unwrap();
    assert!(find(b"moov") < find(b"mdat"));
}

#[test]
fn the_poster_is_the_first_frame_and_the_length_is_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    short_film(dir.path(), false);
    let build = dir.path().join("build");
    let master = build.join("master-16x9.mov");
    stand_in_master(&master, "1920x1080", 12.0);
    let project = hivemotion_spec::load(&dir.path().join("film.yaml")).unwrap();
    let report = deliver(&project, &[PresetName::Youtube]).unwrap();
    let out = &report
        .files
        .iter()
        .find(|f| f.name == "youtube")
        .unwrap()
        .file;
    // the poster still is the master at demo+0.6 = 3.6 s
    let png = build.join("deliver/poster-16x9.png");
    assert!(difference((&png, 0.0), (&master, 3.6)) < 1.0);
    // frame 0 is that still: much nearer to it than to the master's own first frame
    let to_poster = difference((out, 0.0), (&png, 0.0));
    let to_first = difference((out, 0.0), (&master, 0.0));
    assert!(
        to_poster * 2.0 < to_first,
        "frame 0 is not the poster ({to_poster} to the poster, {to_first} to the first frame)"
    );
    // the second frame is the film again
    let second = 1.0 / 30.0;
    let to_film = difference((out, second), (&master, second));
    let to_poster = difference((out, second), (&png, 0.0));
    assert!(
        to_film * 2.0 < to_poster,
        "{to_film} to the film, {to_poster} to the poster"
    );
    assert!((probe(out).unwrap().duration - 12.0).abs() < 0.1);
}
