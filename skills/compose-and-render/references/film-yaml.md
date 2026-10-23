# film.yaml by field

The schema (`schema/film.schema.json`, `schema/brand.schema.json`) is the full contract; this page
says what each part is for. Times are a beat name, a number of seconds, or a beat with an offset
(`drop+0.4`, `end-1.2`). Beat names are lowercase letters, digits and `_`, starting with a
letter.

```yaml
title: Acme launch
brand: brand.yaml
fps: 60                         # 24, 25, 30, 50 or 60
beats: { start: 0.0, demo: 3.0, drop: 10.0, phone_in: 30.0, phone_out: 36.0, card: 52.0, end: 58.0 }
length: end
aspects:
  "16:9": { caption: { left: 96, bottom: 88, size: 78 } }
  "9:16":
    caption: { left: 64, top: 150, size: 70 }
    keep: [ { from: start, to: drop+8 }, { from: card, to: end } ]   # this cut keeps these spans
shots: [...]
captions:
  - { from: demo+0.4, to: drop-0.3, lines: ["One line", "per beat."] }
insets:
  - { id: phone, clip: takes/phone.mp4, from: phone_in, to: phone_out }
end_card: { from: card, line: "Open source. No account.", command: "install acme" }
music: {...}                    # see sound-and-loudness
finish: { fps: 30, poster: demo+1.5 }
cutdowns:                       # cut from the 16:9 master
  - { name: demo-clip, from: demo, length: 12 }
  - { name: hero-loop, from: drop+0.6, length: 8, loop_blend: 0.6, sound: false }
```

## Shots

| Field | Meaning |
|---|---|
| `id` | A plain name; also the slate label when the take is missing |
| `clip` | Path of the take, relative to film.yaml |
| `graphic` | SVG layers moved by keys, instead of a clip (see capture-real-app's stand-ins) |
| `from` | When the shot starts; it runs until the next shot starts |
| `clip_start` | Seconds into the take where the shot begins |
| `speed` | 2.0 plays the take twice as fast |
| `transition` | `{ kind: hex-wipe \| hex-iris \| cross, length: seconds }`; the shot before keeps playing under it |
| `ease` | Curve of the pan and zoom: `in-out` (default), `linear`, `out-cubic`, `in-cubic` |
| `pan_zoom` | Per aspect, `{ start: View, end: View }` |

A `View` is `{ zoom, x, y }`: `zoom` is the scale of the clip's own pixels, at most 1, raised
only if needed to cover the frame; `x`, `y` (0–1) is the point of the clip held at the frame's
centre. A 2× take at `zoom: 0.5` fills a 1080p frame exactly; `zoom: 1` is a sharp 2× push-in.

## Captions, insets, end card

- Captions sit where the aspect's `caption` place says, in the brand's sans font. Text that would
  not fit is shrunk, and the render report says so.
- An inset is a smaller clip drawn over the shots (a phone beside a desktop).
- The end card draws the brand mark, its name, one line and a command to copy.

## brand.yaml

Name, four colours (`background`, `text`, `text_dim`, `text_faint` as `#rrggbb`), a sans and a mono
font (TTF or OTF files), an optional SVG mark, and caption weight, tracking and line height. One
brand file serves every film of a product.
