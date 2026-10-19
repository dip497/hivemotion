use std::path::{Path, PathBuf};

use hivemotion_check::{reading, still_times};
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
    assert!(at(2.225), "halfway through the wipe into the board");
    // the hit (61.9 s in the master) is 36.9 s into the vertical cut
    assert!(vertical
        .iter()
        .any(|x| x.label.contains("hit") && (x.seconds - 36.9).abs() < 1e-6));
    // the tool beat (40 s) is cut out of the vertical
    assert!(!vertical.iter().any(|x| x.label == "tool"));
}
