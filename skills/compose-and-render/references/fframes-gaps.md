# What fframes does and doesn't do

hivemotion draws with [fframes](https://github.com/dmtrKovalenko/fframes) (MIT, Rust, SVG and
Skia), pinned to one git revision. Know its edges before planning a shot.

## Gaps, and how hivemotion handles each

| fframes | Consequence | Handled by |
|---|---|---|
| No 3D on its main branch | 3D scenes, particles, shaders can't be drawn | Render them elsewhere (a browser, Blender, a 3D engine) to ProRes 4444, with alpha if needed, and use them as a `clip` |
| No pan or zoom of clips | — | hivemotion draws `pan_zoom` itself |
| Clips have no in-point or speed | — | `hivemotion prep` cuts each take with ffmpeg |
| Defaults to BT.601 colour and 44.1 kHz audio | Colours shift and audio resamples if left alone | hivemotion sets BT.709 and 48 kHz explicitly; tag clips made elsewhere BT.709 too |
| Width, height and frame rate are compile-time constants | One build per aspect and frame rate | hivemotion builds every aspect at 24, 25, 30, 50 and 60 fps into one binary; other rates fail with a message |
| Converting seconds to a frame truncates | A still asked for mid-frame lands on the frame after; one past the last frame's start finds nothing | `check` takes every still at the start of a frame |
| Its H.264 encoder is a GPL feature | Turning it on makes the binary GPL | hivemotion masters to ProRes 4444; H.264 comes from the system `ffmpeg` run as a separate program |
| Media folders are read one level deep | Nested folders are not found | `prep` writes every clip and font flat into `build/media/` |
| Fonts must be TTF (or OTF); woff2 is not read | Web fonts need converting | Ship TTF/OTF files beside `brand.yaml` |
| Its command line parses the process's own arguments | It can't be embedded behind another CLI | hivemotion calls the library directly |
| No audio gain automation; the master limiter is on by default | Fades and loudness need care | hivemotion turns fframes' limiter off and masters with ffmpeg (see sound-and-loudness) |
| With compile-time SVG, raw SVG can't be inserted | — | hivemotion draws graphic layers as paths |

## Cost

Measured on one 8-core machine that was busy with other work (not checked in this repo; expect
different numbers on yours):

| Step | Time |
|---|---|
| 1080p render, shots with footage | about 331 ms per frame |
| 1080p render, graphics only | about 77 ms per frame |
| Peak memory during a render | about 3.6 GB |
| `master` | about 1.5 min |
| `deliver`, six presets | about 20 min |
| First `cargo build --release` | about 10 min (fframes downloads a prebuilt FFmpeg) |

So a 60 s film at 60 fps (3,600 frames) with footage renders in about 20 minutes per aspect.
`render --stills` costs a few frames; use it for every look before a full render.

## When a render fails

- **Out of memory:** render one aspect at a time (`--aspect`), or on a machine with more free
  memory (see [remote-machine.md](remote-machine.md)).
- **Colours look washed out or shifted in a player:** a clip made elsewhere is untagged or tagged
  BT.601. Re-encode it tagged BT.709.
- **A text line came out smaller:** it did not fit and was shrunk; the report's notes say which.
  Shorten the line.
