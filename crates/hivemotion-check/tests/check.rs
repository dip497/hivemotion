use std::path::{Path, PathBuf};

use hivemotion_check::{reading, still_times, stills_from_master, StillAt};
use hivemotion_spec::film::Aspect;

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/hivemind-launch/film.yaml")
}

#[test]
fn the_hook_caption_is_on_screen_too_briefly_to_read() {
    let project = hivemotion_spec::load(&example()).unwrap();
    let r = reading(&project.timeline);
    // "My agents built this screen.": 5 words need 1.5 s; it is fully up from 0.70 s to 1.67 s
    let hook: Vec<_> = r
        .iter()
        .filter(|c| c.text.starts_with("My agents"))
        .collect();
    assert_eq!(hook.len(), 2, "one per aspect");
    for c in hook {
        assert!(!c.ok);
        assert!(
            (c.fully_visible_seconds - 0.97).abs() < 1e-6,
            "{}",
            c.fully_visible_seconds
        );
        assert!((c.needed_seconds - 1.5).abs() < 1e-9);
    }
    // the next caption has time to spare
    assert!(r
        .iter()
        .filter(|c| c.text.starts_with("Every coding"))
        .all(|c| c.ok));
}

#[test]
fn stills_cover_every_kept_beat_and_the_middle_of_every_transition() {
    let project = hivemotion_spec::load(&example()).unwrap();
    let t = &project.timeline;
    let vertical = still_times(t, &t.aspects[&Aspect::Vertical]);
    let at = |s: f64| vertical.iter().any(|x| (x.seconds - s).abs() < 1e-6);
    assert!(at(10.0), "the drop");
    // halfway through the wipe into the board is 2.225 s, inside frame 133 at 60 fps
    assert!(at(133.0 / 60.0), "halfway through the wipe into the board");
    // the hit (61.9 s in the master) is 36.9 s into the vertical cut
    assert!(vertical
        .iter()
        .any(|x| x.label.contains("hit") && (x.seconds - 36.9).abs() < 1e-6));
    // the tool beat (40 s) is cut out of the vertical
    assert!(!vertical.iter().any(|x| x.label == "tool"));
}

#[test]
fn filter_characters_in_a_label_or_font_path_stay_text() {
    // a film's labels and folders come from whoever wrote it; none of this may become a filter option
    let dir = tempfile::tempdir().unwrap();
    let odd = dir.path().join("fonts ':,;[]\\ %{n}");
    std::fs::create_dir_all(&odd).unwrap();
    let font = odd.join("f'o:n,t.ttf");
    std::fs::copy(
        example().parent().unwrap().join("fonts/Geist-500.ttf"),
        &font,
    )
    .unwrap();
    let master = dir.path().join("master.mov");
    let made = std::process::Command::new("ffmpeg")
        .args(["-hide_banner", "-v", "error", "-y", "-f", "lavfi", "-i"])
        .arg("testsrc2=s=640x360:r=30:d=2")
        .arg(&master)
        .status()
        .unwrap()
        .success();
    assert!(made);
    let at = [StillAt {
        seconds: 1.0,
        label: "x':fontsize=300,drawbox;[a]\\ %{pts}".into(),
    }];
    let stills = stills_from_master(
        &master,
        &at,
        &dir.path().join("check"),
        Aspect::Landscape,
        Some(&font),
    )
    .unwrap();
    assert!(stills.contact_sheet.is_file());
}

#[test]
fn every_still_is_at_the_start_of_a_frame_the_master_has() {
    // the end beat sits at the very end of the film; a seek past the last frame's start finds no
    // picture, and the whole check failed on it
    let project = hivemotion_spec::load(&example()).unwrap();
    let t = &project.timeline;
    for aspect in [Aspect::Landscape, Aspect::Vertical] {
        let plan = &t.aspects[&aspect];
        let frames = (plan.length() * t.fps as f64).round();
        for s in still_times(t, plan) {
            let frame = s.seconds * t.fps as f64;
            assert!(
                (frame - frame.round()).abs() < 1e-6,
                "{aspect}: {} s is mid-frame",
                s.seconds
            );
            assert!(
                frame.round() < frames,
                "{aspect}: {} s is past the last frame",
                s.seconds
            );
        }
    }
}
