# Platforms and presets

What `hivemotion deliver --preset <name>` makes. Every file: H.264 High, 4:2:0, BT.709 tags,
AAC 48 kHz, `+faststart`, poster as frame 0, at `finish.fps` (a 60 fps master delivered at 30
blends frame pairs).

| Preset | Aspects (first available is used) | Quality | Limits checked | Where it goes |
|---|---|---|---|---|
| `x` | 16:9, 4:5, 9:16 | CRF 18, ≤ 25 Mb/s, audio 256 kb/s | warns over 140 s; fails over 512 MB | X post |
| `youtube` | 16:9 | CRF 16, audio 384 kb/s | — | YouTube; also the source for Product Hunt |
| `producthunt` | 16:9 | as `youtube`, plus a 1270×760 gallery image and a 240×240 thumbnail from the poster | — | Product Hunt takes only a YouTube link: upload this file to YouTube |
| `shorts` | 9:16 | CRF 18, audio 256 kb/s | warns over 180 s | YouTube Shorts and other vertical feeds |
| `linkedin` | 4:5, 16:9 | CRF 18, ≤ 30 Mb/s, audio 192 kb/s | warns over 600 s; fails over 5 GB | LinkedIn feed |
| `readme-loop` | 16:9 | 1280 wide, no sound, CRF raised until it fits | ≤ 12 s; fails at 10 MB | Source for the README animation |

The readme loop is the first cutdown with `loop_blend`, or else 6 s from the poster with a 0.5 s
blend, so its end flows into its start.

## Aspects

hivemotion renders 16:9 (1920×1080), 9:16 (1080×1920) and 4:5 (1080×1350). There is no 1:1; 4:5
is the feed format that does as well or better. Make 9:16 and 4:5 as their own aspects in
`film.yaml`, laid out again for the shape (caption place, `pan_zoom`, `keep`), not cropped from
16:9. Vertical feeds cover about the bottom 17–20% and the right edge with their own buttons;
keep captions and the product clear of them.

## Cutdowns

Each `cutdowns:` entry in `film.yaml` is cut from the 16:9 master: `from` a beat, `length`
seconds, optional `loop_blend` for a seamless loop and `sound: false` for silent autoplay. Make a
10–15 s clip for each strong beat (replies, forum posts) and a 6–10 s silent loop for a README or
site hero.

## Platform facts (secondary sources, October 2026; check before relying on a number)

- X: 140 s for most accounts; muted autoplay.
- YouTube: plays 16:9 and vertical; upload captions as SRT/VTT for the long film if you want
  them selectable (hivemotion writes no SRT).
- LinkedIn: 4:5 and 1:1 do well in the feed; 16:9 for screens and UI.
- Product Hunt: a YouTube URL only, 16:9; gallery 1270×760; thumbnail 240×240.
- GitHub: uploaded attachments up to 10 MB on the free plan.
