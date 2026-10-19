# AGENTS.md

How to build, test and change hivemotion. Read this before editing.

## Build and check

```bash
cargo build --release      # first build ~10 min: fframes downloads a prebuilt FFmpeg
scripts/check.sh           # fmt --check, clippy -D warnings, all tests (needs ffmpeg + ffprobe)
```

There is no CI. `scripts/check.sh` is the gate; run it before every commit and say in the commit
or PR that it passed.

The tests encode and measure real files, so they need `ffmpeg` and `ffprobe` (with libx264) on
PATH. A full run renders frames with fframes and encodes a few minutes of H.264; give it a quiet
machine. If heavy work runs on another machine, write its details in `hosts.local.yaml` (gitignored;
copy `hosts.example.yaml`). Never commit host names, addresses or personal paths.

## Layout

| crate | job |
|---|---|
| `hivemotion-spec` | film.yaml + brand.yaml types, JSON Schema, validation naming the field, beats resolved to seconds (`Timeline`) |
| `hivemotion-render` | `prep` (cut takes with ffmpeg) and the one fframes `Video` that draws any film |
| `hivemotion-audio` | music laid on the beats (fframes mixer), two-pass loudnorm, limiter, loudness gate |
| `hivemotion-deliver` | posters as frame 0, cutdowns, platform presets (system ffmpeg) |
| `hivemotion-check` | stills at beats and mid-transition, contact sheets, read time, loudness |
| `hivemotion-ffmpeg` | runs the system `ffmpeg`/`ffprobe` |
| `hivemotion` | the CLI; every command has `--json` |

`examples/hivemind-launch/` is the benchmark film. Its takes (`takes/`) and music (`music/`) are
not in git; a missing take renders as a named slate.

## Rules

- **Time has one owner.** Every time in film.yaml is a beat or a beat with an offset. Never add a
  field that holds a second of film time without going through `TimeRef`.
- **Only fields a real film uses.** Add a spec field when a film needs it, with the film.
- **Plain names.** Fields, functions and errors read as plain English; no abbreviations.
- **Errors name the field** (`shots[3].from: no beat named 'drp'`).
- **Licence:** never turn on fframes' `h264`, `libav-agree-gpl` or other GPL/nonfree features.
  H.264 comes from the system ffmpeg as a separate program.
- **Colour and sound defaults are explicit:** BT.709 and 48 kHz (fframes defaults to BT.601 and
  44.1 kHz).
- **fframes is pinned** to one git revision in the workspace `Cargo.toml`. Upgrading it means
  re-rendering the example and comparing stills.
- **Tests guard behaviour at a real boundary** (a rendered pixel, an ffprobe field, a measured
  loudness, a CLI's JSON). No test reads source code or checks structure only. Each new test is
  shown to fail once by breaking the behaviour it guards.
- **No secrets, hostnames, IPs or personal paths in the repo.** Machine details go in an
  untracked `*.local.*` file.
