use std::path::{Path, PathBuf};
use std::process::Command;

fn hivemotion(args: &[&str]) -> (bool, serde_json::Value) {
    let out = Command::new(env!("CARGO_BIN_EXE_hivemotion"))
        .args(args)
        .arg("--json")
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("not JSON ({e}): {}", String::from_utf8_lossy(&out.stdout)));
    (out.status.success(), json)
}

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/hivemind-launch/film.yaml")
}

#[test]
fn validate_reports_the_example_as_json() {
    let (ok, json) = hivemotion(&["validate", example().to_str().unwrap()]);
    assert!(ok, "{json}");
    assert_eq!(json["ok"], true);
    assert_eq!(json["length"], 70.0);
}

#[test]
fn validate_names_the_broken_field_and_exits_non_zero() {
    let dir = tempfile::tempdir().unwrap();
    let (ok, _) = hivemotion(&["new", dir.path().join("film").to_str().unwrap()]);
    assert!(ok);
    let film = dir.path().join("film/film.yaml");
    let text = std::fs::read_to_string(&film)
        .unwrap()
        .replace("from: demo\n", "from: dmeo\n");
    std::fs::write(&film, text).unwrap();
    let (ok, json) = hivemotion(&["validate", film.to_str().unwrap()]);
    assert!(!ok);
    assert_eq!(json["ok"], false);
    assert_eq!(json["errors"][0]["path"], "shots[1].from", "{json}");
}

#[test]
fn a_new_film_validates_as_made() {
    let dir = tempfile::tempdir().unwrap();
    let (ok, json) = hivemotion(&[
        "new",
        dir.path().join("film").to_str().unwrap(),
        "--name",
        "Acme",
    ]);
    assert!(ok, "{json}");
    let (ok, json) = hivemotion(&[
        "validate",
        dir.path().join("film/film.yaml").to_str().unwrap(),
    ]);
    assert!(ok, "{json}");
    assert_eq!(json["title"], "Acme launch");
}
