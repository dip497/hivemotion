# hivemotion

Launch films for any product, made again and again from one `film.yaml`, by you or by a coding
agent. MIT licensed, no account, runs headless on Linux.

hivemotion owns the parts of a launch film that usually live in hand-edited scripts: when things
happen (named beats), how each recording is framed (pan and zoom per aspect), the captions, the
music laid on the beats, loudness mastering with a gate, and the files each platform wants. The
picture is drawn by [fframes](https://github.com/dmtrKovalenko/fframes), a Rust video engine (SVG
and Skia), pinned to one exact revision.

## What a film is

`film.yaml` names its beats once; everything else points at them:

```yaml
beats: { start: 0.0, board: 2.0, drop: 10.0, hit: 61.9, end: 70.0 }
shots:
  - id: board
    clip: takes/W1-board.mp4
    from: board
    transition: { kind: hex-wipe, length: 0.45 }
    pan_zoom:
      "16:9": { start: { zoom: 0.75, x: 0.5, y: 0.5 }, end: { zoom: 0.6, x: 0.5, y: 0.5 } }
captions:
  - { from: board+0.3, to: ask-0.3, lines: ["Every coding agent.", "One board."] }
music:
  sections:
    - { from: start, cues: { drop: 28.10 } }   # the track's 28.10 s lands on the drop
```

Move a beat and the shots, captions, music and cutdowns move with it. `brand.yaml` holds the
colours, fonts and mark. `examples/hivemind-launch/` is a complete film (the Hivemind launch, 70 s
master and a 45 s vertical cut) and the regression benchmark.

A shot whose recording is missing renders as a named slate, so a film can be cut, timed and
checked before every take exists.

A shot can be drawn instead of recorded: a `graphic` is SVG files drawn back to front, each moved
by keys (position, turn, size, squash, opacity, an ease and an upward `arc` for a throw).
`examples/fired-ide/` flicks an editor window into a trash can in half a second:

```yaml
shots:
  - id: fired
    from: start
    graphic:
      layers:
        - svg: graphics/editor-window.svg
          width: 0.62                      # of the frame's shorter side, so it fits every aspect
          keys:
            - { at: 0.0, x: 0.5, y: 0.42 }
            - { at: 0.1, y: 0.45, rotate: -6, squash: 0.9, ease: out-cubic }
            - { at: 0.44, y: 0.72, rotate: 160, scale: 0.16, squash: 1.0, arc: 0.3 }
```

## Quick start

Needs Rust (stable), and `ffmpeg` + `ffprobe` on PATH (any build with libx264).

```bash
cargo build --release            # first build: about 10 min (fframes downloads a prebuilt FFmpeg)
hm=target/release/hivemotion

$hm new my-film --name Acme      # film.yaml, brand.yaml, fonts
$hm validate my-film/film.yaml   # every problem names its field: shots[1].from: no beat named 'dmeo'
$hm prep my-film/film.yaml       # cut takes to their shots (start, speed, length)
$hm render my-film/film.yaml --stills 2,demo+1   # look at single frames first
$hm render my-film/film.yaml     # ProRes 4444 per aspect, BT.709
$hm master my-film/film.yaml     # music on the beats, two-pass loudnorm, limiter, loudness gate
$hm deliver my-film/film.yaml    # x, youtube, shorts, linkedin, readme-loop, producthunt + cutdowns
$hm check my-film/film.yaml      # stills at every beat, contact sheet, read time, loudness
```

Every command takes `--json` and prints one JSON object with `ok`; it exits 1 when `ok` is false.
Outputs go to `build/` next to film.yaml.

## The gates

- **Loudness:** the master and every delivered file with sound must measure −15 to −13 LUFS
  integrated and at most −1 dBTP true peak (ffmpeg `ebur128`). Mastering is two-pass `loudnorm`
  in linear mode (it fails if loudnorm falls back to dynamic) and `alimiter` with `level=disabled`.
- **Read time:** a caption must be fully on screen at least 0.3 s per word.
- **Delivery:** H.264 High, 4:2:0, BT.709 tags, AAC 48 kHz, `+faststart`; the poster frame is
  frame 0 and the length is unchanged; the README loop is under 10 MB.

## Licences

hivemotion is MIT. fframes is MIT; it is built **without** its GPL features (`h264`,
`libav-agree-gpl`), so the hivemotion binary links no GPL code. H.264 files are made by the
system `ffmpeg`, run as a separate program. The example's fonts (Geist, JetBrains Mono) are under
the SIL Open Font License; their licence files sit beside them.

## Developing

`scripts/check.sh` is the one check: `cargo fmt --check`, `cargo clippy -D warnings` and every
test. See [AGENTS.md](AGENTS.md).
