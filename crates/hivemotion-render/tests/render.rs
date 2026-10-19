//! Renders the Hivemind example at a few beats, with solid-colour stand-in takes so the colour of
//! a pixel says which shot is on screen.

use std::path::{Path, PathBuf};
use std::process::Command;

use hivemotion_render::{prep, render_stills};
use hivemotion_spec::film::Aspect;
use hivemotion_spec::Project;

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/hivemind-launch")
}

fn copy_example(dir: &Path, edit: impl Fn(String) -> String) {
    for f in ["brand.yaml", "mark.svg"] {
        std::fs::copy(example().join(f), dir.join(f)).unwrap();
    }
    std::fs::create_dir_all(dir.join("fonts")).unwrap();
    for f in std::fs::read_dir(example().join("fonts")).unwrap() {
        let f = f.unwrap();
        std::fs::copy(f.path(), dir.join("fonts").join(f.file_name())).unwrap();
    }
    let film = std::fs::read_to_string(example().join("film.yaml")).unwrap();
    std::fs::write(dir.join("film.yaml"), edit(film)).unwrap();
}

/// A take of one flat colour.
fn colour_take(dir: &Path, name: &str, colour: &str, size: &str) {
    std::fs::create_dir_all(dir.join("takes")).unwrap();
    let ok = Command::new("ffmpeg")
        .args(["-hide_banner", "-v", "error", "-y", "-f", "lavfi", "-i"])
        .arg(format!("color=c={colour}:s={size}:r=30:d=12"))
        .args(["-c:v", "libx264", "-crf", "10", "-pix_fmt", "yuv420p"])
        .arg(dir.join("takes").join(name))
        .status()
        .unwrap()
        .success();
    assert!(ok);
}

fn project(dir: &Path) -> Project {
    let p = hivemotion_spec::load(&dir.join("film.yaml")).unwrap();
    prep(&p).unwrap();
    p
}

fn pixel(png: &Path, x: u32, y: u32) -> [u8; 3] {
    let out = Command::new("ffmpeg")
        .args(["-hide_banner", "-v", "error", "-i"])
        .arg(png)
        .args([
            "-vf",
            &format!("crop=1:1:{x}:{y}"),
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-",
        ])
        .output()
        .unwrap();
    [out.stdout[0], out.stdout[1], out.stdout[2]]
}

#[test]
fn two_renders_of_the_same_frames_are_identical() {
    let dir = tempfile::tempdir().unwrap();
    copy_example(dir.path(), |f| f);
    colour_take(dir.path(), "I1-hook.mp4", "0x3355aa", "3200x2000");
    colour_take(dir.path(), "W1-board.mp4", "0xaa5533", "3200x2000");
    let p = project(dir.path());
    let times = [1.0, 2.2, 10.0, 62.5];
    let a = render_stills(&p, Aspect::Landscape, &times, &dir.path().join("a")).unwrap();
    let b = render_stills(&p, Aspect::Landscape, &times, &dir.path().join("b")).unwrap();
    for (x, y) in a.stills.iter().zip(&b.stills) {
        let (bx, by) = (
            std::fs::read(&x.file).unwrap(),
            std::fs::read(&y.file).unwrap(),
        );
        assert!(
            bx == by,
            "{} differs from {}",
            x.file.display(),
            y.file.display()
        );
    }
}

#[test]
fn a_hex_wipe_shows_the_new_shot_in_the_middle_and_the_old_one_at_the_corners() {
    let dir = tempfile::tempdir().unwrap();
    copy_example(dir.path(), |f| f);
    colour_take(dir.path(), "I1-hook.mp4", "red", "3200x2000");
    colour_take(dir.path(), "W1-board.mp4", "blue", "3200x2000");
    let p = project(dir.path());
    // the board shot wipes in over 0.45 s from 2.0 s; at 2.08 s the hexagon is part grown
    let r = render_stills(
        &p,
        Aspect::Landscape,
        &[1.5, 2.08, 3.0],
        &dir.path().join("s"),
    )
    .unwrap();
    let [before, middle, after] = [&r.stills[0].file, &r.stills[1].file, &r.stills[2].file];
    let red = |c: [u8; 3]| c[0] > 2 * c[2].max(10);
    let blue = |c: [u8; 3]| c[2] > 2 * c[0].max(10);
    assert!(
        red(pixel(before, 960, 400)),
        "{:?}",
        pixel(before, 960, 400)
    );
    assert!(
        blue(pixel(middle, 960, 400)),
        "{:?}",
        pixel(middle, 960, 400)
    );
    assert!(red(pixel(middle, 30, 30)), "{:?}", pixel(middle, 30, 30));
    assert!(blue(pixel(after, 30, 30)), "{:?}", pixel(after, 30, 30));
}

#[test]
fn a_shot_without_its_take_is_a_named_slate_not_the_last_take() {
    let dir = tempfile::tempdir().unwrap();
    copy_example(dir.path(), |f| f);
    colour_take(dir.path(), "I1-hook.mp4", "red", "3200x2000");
    let p = project(dir.path());
    let r = render_stills(&p, Aspect::Landscape, &[3.0], &dir.path().join("s")).unwrap();
    assert!(r.missing_takes.iter().any(|m| m.ends_with("W1-board.mp4")));
    let c = pixel(&r.stills[0].file, 960, 300);
    assert!(
        c[0] < 60 && c[1] < 60 && c[2] < 60,
        "the slate is not dark: {c:?}"
    );
}

#[test]
fn a_caption_too_wide_for_the_vertical_frame_is_shrunk_to_fit() {
    let dir = tempfile::tempdir().unwrap();
    copy_example(dir.path(), |f| {
        f.replace(
            "lines: [\"Every coding agent.\", \"One board.\"]",
            "lines: [\"Every single coding agent you use.\", \"One board.\"]",
        )
    });
    let p = project(dir.path());
    // fully on screen at 4 s; the vertical frame is 1080 wide with a 64 px margin
    let r = render_stills(&p, Aspect::Vertical, &[4.0], &dir.path().join("s")).unwrap();
    assert!(
        r.notes.iter().any(|n| n.contains("shrunk")),
        "{:?}",
        r.notes
    );
    let out = Command::new("ffmpeg")
        .args(["-hide_banner", "-v", "error", "-i"])
        .arg(&r.stills[0].file)
        .args([
            "-vf",
            "crop=40:400:1040:150,format=gray",
            "-f",
            "rawvideo",
            "-",
        ])
        .output()
        .unwrap();
    let brightest = out.stdout.iter().copied().max().unwrap();
    assert!(
        brightest < 120,
        "text reaches the right edge (brightest {brightest})"
    );
}
