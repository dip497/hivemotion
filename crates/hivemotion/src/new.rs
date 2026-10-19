//! `hivemotion new`: a film folder that validates as made.

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

const FONTS: [(&str, &[u8]); 6] = [
    (
        "Geist-500.ttf",
        include_bytes!("../../../examples/hivemind-launch/fonts/Geist-500.ttf"),
    ),
    (
        "Geist-700.ttf",
        include_bytes!("../../../examples/hivemind-launch/fonts/Geist-700.ttf"),
    ),
    (
        "JetBrainsMono-500.ttf",
        include_bytes!("../../../examples/hivemind-launch/fonts/JetBrainsMono-500.ttf"),
    ),
    (
        "JetBrainsMono-700.ttf",
        include_bytes!("../../../examples/hivemind-launch/fonts/JetBrainsMono-700.ttf"),
    ),
    (
        "OFL-Geist.txt",
        include_bytes!("../../../examples/hivemind-launch/fonts/OFL-Geist.txt"),
    ),
    (
        "OFL-JetBrainsMono.txt",
        include_bytes!("../../../examples/hivemind-launch/fonts/OFL-JetBrainsMono.txt"),
    ),
];

const FILM: &str = r#"title: NAME launch
brand: brand.yaml
fps: 60

# Every time in this file points at a beat.
beats:
  start: 0.0
  demo: 3.0
  card: 9.0
  end: 13.0

length: end

aspects:
  "16:9":
    caption: { left: 96, bottom: 88, size: 78 }
  "9:16":
    caption: { left: 64, top: 150, size: 70 }

shots:
  # Put recordings in takes/. A missing take renders as a named slate.
  - id: opening
    clip: takes/opening.mp4
    from: start
  - id: demo
    clip: takes/demo.mp4
    from: demo
    transition: { kind: hex-wipe, length: 0.45 }

captions:
  - { from: start+0.3, to: demo-0.2, lines: ["What it does,", "in one line."] }
  - { from: demo+0.4, to: card-0.4, lines: ["Show the real thing."] }

end_card:
  from: card
  line: Open source.
  command: install NAME

finish:
  fps: 30
  poster: demo+1.5
"#;

const BRAND: &str = r##"name: NAME
colors:
  background: "#040506"
  text: "#f4f5f6"
  text_dim: "#c4c8cd"
  text_faint: "#8d939a"
fonts:
  sans:
    family: Geist
    files: [fonts/Geist-500.ttf, fonts/Geist-700.ttf]
  mono:
    family: JetBrains Mono
    files: [fonts/JetBrainsMono-500.ttf, fonts/JetBrainsMono-700.ttf]
caption:
  weight: 700
  tracking: -0.04
  line_height: 0.98
"##;

pub fn create(dir: &Path, name: &str) -> Result<Vec<PathBuf>> {
    if dir.join("film.yaml").exists() {
        bail!("{} already has a film.yaml", dir.display());
    }
    let one_line = name.replace(['\n', '"'], " ");
    std::fs::create_dir_all(dir.join("fonts"))?;
    std::fs::create_dir_all(dir.join("takes"))?;
    let mut written = Vec::new();
    for (file, text) in [("film.yaml", FILM), ("brand.yaml", BRAND)] {
        let path = dir.join(file);
        std::fs::write(&path, text.replace("NAME", &one_line))?;
        written.push(path);
    }
    for (file, bytes) in FONTS {
        let path = dir.join("fonts").join(file);
        std::fs::write(&path, bytes)?;
        written.push(path);
    }
    Ok(written)
}
