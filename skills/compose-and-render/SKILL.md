---
name: compose-and-render
description: Lay a launch film's shots, drawn graphics, captions, insets, transitions and end card on its named beats in film.yaml, then render a ProRes 4444 master per aspect with hivemotion and fframes - on this machine or on another one when this one is busy. Use when editing film.yaml, adding a recording, 3D or motion-graphics clip to a film, framing a shot with pan and zoom, making a vertical or 4:5 version, checking stills, or when a render is slow, out of memory or the colours look wrong.
---

# Compose and render

`film.yaml` is the edit. Every time in it is a beat or a beat with an offset, so the picture, the
captions, the music and the cutdowns move together.

## Steps

1. **Place shots on beats.** Each shot runs from its `from` until the next shot starts. It is a
   recording (`clip`, trimmed with `clip_start` and `speed`) or a drawn `graphic`. Field by
   field: [references/film-yaml.md](references/film-yaml.md); the schema is
   `schema/film.schema.json` (`hivemotion validate --write-schema <dir>` writes it).
2. **Bring in what fframes can't draw as clips.** 3D, particles, shader effects and real app
   footage are rendered or recorded elsewhere and enter as `clip:` (ProRes 4444 or lossless; with
   alpha if they sit over something). A one-off motion-graphics scene that `film.yaml` can't
   express is made the same way: a small fframes project of its own (the `fframes-video` skill,
   `npx skills add https://fframes.studio`) rendered to ProRes 4444. See
   [references/fframes-gaps.md](references/fframes-gaps.md).
3. **Frame each aspect.** `pan_zoom` per aspect gives a slow move from one view to another;
   an aspect left out shows the clip centred and covering the frame. A 9:16 or 4:5 cut gets its
   own caption place and may `keep` only some spans of the film; lay it out again, don't crop.
4. **Cut the takes.** `hivemotion prep film.yaml` trims every take to its shot (start, speed,
   length, frame rate) into `build/media/`. Missing takes are listed and render as named slates.
5. **Look at stills before the full render.**
   `hivemotion render film.yaml --stills start+1,drop,drop+0.2` (seconds or beats). Look at every
   beat and the middle of every transition for overflow, collisions, contrast and anything private.
6. **Render the masters.** `hivemotion render film.yaml` (or `--aspect 9:16`) writes
   `build/render-<aspect>.mov`: ProRes 4444, BT.709, no sound. Read the JSON report:
   `milliseconds_per_frame`, `missing_takes`, and notes about text shrunk to fit.
7. **If this machine is busy,** render on another one: [references/remote-machine.md](references/remote-machine.md).

Then [sound-and-loudness](../sound-and-loudness/SKILL.md) puts the sound in, and
[deliver-and-publish](../deliver-and-publish/SKILL.md) makes the files people see.

## Rules

- Never write a second of film time anywhere but `beats:`. A drop written in three places drifts.
- Frame rate: 24, 25, 30, 50 or 60. Render at 60 and deliver at 30 (`finish.fps`) for smooth
  motion blur; delivery blends frame pairs.
- Colour is BT.709 and audio 48 kHz everywhere; hivemotion sets both because fframes defaults to
  BT.601 and 44.1 kHz. If you add a clip made elsewhere, tag it BT.709 too.
- Fonts are TTF or OTF files beside `brand.yaml`; woff2 is not read.
- Rendering costs: see the numbers in [references/fframes-gaps.md](references/fframes-gaps.md#cost).

For example, `examples/hivemind-film/film.yaml` composes a drawn graphic, real takes and an
end card on one set of beats.
