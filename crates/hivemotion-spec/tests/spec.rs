use std::path::{Path, PathBuf};

use hivemotion_spec::{load, Aspect};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/hivemind-launch")
}

/// A copy of the example with film.yaml changed by `edit`.
fn edited_example(edit: impl Fn(&str) -> String) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    for entry in ["brand.yaml", "mark.svg"] {
        std::fs::copy(example().join(entry), dir.path().join(entry)).unwrap();
    }
    std::fs::create_dir(dir.path().join("fonts")).unwrap();
    for f in std::fs::read_dir(example().join("fonts")).unwrap() {
        let f = f.unwrap();
        std::fs::copy(f.path(), dir.path().join("fonts").join(f.file_name())).unwrap();
    }
    let film = std::fs::read_to_string(example().join("film.yaml")).unwrap();
    let edited = edit(&film);
    assert_ne!(film, edited, "the edit changed nothing");
    let path = dir.path().join("film.yaml");
    std::fs::write(&path, edited).unwrap();
    (dir, path)
}

#[test]
fn the_hivemind_example_validates_with_its_beats_resolved() {
    let project = load(&example().join("film.yaml")).unwrap_or_else(|e| panic!("{e}"));
    let t = &project.timeline;
    assert_eq!(t.length, 70.0);
    let ask = t.shots.iter().find(|s| s.id == "ask").unwrap();
    assert_eq!((ask.start, ask.end), (8.0, 13.0));
    // the board shot keeps playing under the ask shot while it wipes in
    let board = t.shots.iter().find(|s| s.id == "board").unwrap();
    assert_eq!(board.tail, 0.45);
    // the track plays from 18.10 s at the start and from 152.75 s at the ending section
    let music = t.music.as_ref().unwrap();
    let starts: Vec<f64> = music.sections.iter().map(|s| s.track_start).collect();
    assert!((starts[0] - 18.10).abs() < 1e-9 && (starts[1] - 152.75).abs() < 1e-9);
    // the vertical cut is the master's first 35.85 s and then its ending: 45 s
    let vertical = &t.aspects[&Aspect::Vertical];
    assert!((vertical.length() - 45.0).abs() < 1e-9);
    assert!((vertical.film_time(36.9) - 61.9).abs() < 1e-9);
    // and its music is the track's 18.10 s on, then its ending from 152.75 s at 35.85 s
    let pieces = vertical.music_pieces(music);
    assert_eq!(pieces.len(), 2);
    assert!((pieces[1].start - 35.85).abs() < 1e-9);
    assert!((pieces[1].track_start - 152.75).abs() < 1e-9);
}

#[test]
fn a_shot_pointing_at_a_missing_beat_names_that_field() {
    let (_dir, path) =
        edited_example(|film| film.replacen("    from: ask\n", "    from: asc\n", 1));
    let err = load(&path).unwrap_err();
    let errors = err.errors();
    assert_eq!(errors.len(), 1, "{err}");
    assert_eq!(errors[0].path, "shots[2].from");
    assert!(errors[0].message.contains("no beat named 'asc'"), "{err}");
}

#[test]
fn an_inset_id_that_climbs_out_of_the_media_folder_is_rejected() {
    // prep writes <media>/<id>.mp4, so this id would write and delete outside the build
    let (_dir, path) = edited_example(|film| film.replace("{ id: phone,", "{ id: ../../x,"));
    let err = load(&path).unwrap_err();
    assert!(
        err.errors().iter().any(|e| e.path == "insets[0].id"),
        "{err}"
    );
}

#[test]
fn an_inset_cannot_share_an_id_with_a_shot() {
    // both are prepared into the same media folder by id; one would overwrite the other
    let (_dir, path) = edited_example(|film| film.replace("{ id: phone,", "{ id: hook,"));
    let err = load(&path).unwrap_err();
    assert!(
        err.errors().iter().any(|e| e.path == "insets[0].id"),
        "{err}"
    );
}

#[test]
fn music_cues_that_disagree_name_the_cue() {
    // the second drop moved 0.5 s in the film but not in the track: the music would drift
    let (_dir, path) =
        edited_example(|film| film.replace("second_drop: 52.3", "second_drop: 52.8"));
    let err = load(&path).unwrap_err();
    assert!(
        err.errors()
            .iter()
            .any(|e| e.path == "music.sections[0].cues.second_drop"),
        "{err}"
    );
}

#[test]
fn a_misspelt_field_is_named_rather_than_ignored() {
    let (_dir, path) =
        edited_example(|film| film.replacen("    ease: out-cubic\n", "    easing: out-cubic\n", 1));
    let err = load(&path).unwrap_err();
    assert_eq!(err.errors()[0].path, "shots[0].easing", "{err}");
    assert!(err.errors()[0].message.contains("easing"), "{err}");
}

#[test]
fn a_caption_crossing_the_vertical_cut_ends_at_the_cut() {
    let project = load(&example().join("film.yaml")).unwrap();
    let t = &project.timeline;
    let vertical = &t.aspects[&Aspect::Vertical];
    let shown = vertical.captions(&t.captions);
    // "Widgets and tools" runs 32.4–43.6 in the master; the vertical cuts at 35.85
    let widgets = shown
        .iter()
        .find(|c| c.lines[0].starts_with("Widgets"))
        .unwrap();
    assert!((widgets.end - 35.85).abs() < 1e-9, "{}", widgets.end);
    // the caption that starts inside the cut-out part is not shown at all
    assert!(!shown.iter().any(|c| c.lines[0].starts_with("Everything")));
}
