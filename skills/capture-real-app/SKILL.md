---
name: capture-real-app
description: Record a real product - a web app, an Electron app or a desktop app - as smooth, crisp, frame-exact footage for a launch film, with no private data and no other company's product on screen. Use when a film needs takes of the actual UI, when a screen recording came out choppy, blurry or dropped frames, when capturing on a busy or headless machine, or when a shot needs to show a competitor or "the old way" and a stand-in must be drawn instead.
---

# Capture the real app

A launch film shows the real product. Real-time screen recording is choppy whenever the machine
is busy, and software rendering (xvfb, llvmpipe) shows frame rates that are not true. Step the app
frame by frame instead: the result is a smooth 60 fps of the app's own pixels on any machine, at
any load.

## Steps

1. **Make a clean stage.** A fresh profile and home folder, seeded with demo data written for the
   film. Then scrub before you capture, not after: see
   [references/data-hygiene.md](references/data-hygiene.md).
2. **Script the take.** One script per take drives the app (clicks, typing, scripted stand-in
   agents or users) so the take can be made again identically. Name takes by beat:
   `takes/<beat>.mp4`, matching the `clip:` of a shot in `film.yaml`.
3. **Step, don't record.** For every frame: advance one clock by 1/fps for everything that moves
   (the page's time, animation rate, scripted actors), let the app draw, grab the window at device
   scale 2 into a lossless file. See [references/frame-stepped-capture.md](references/frame-stepped-capture.md).
4. **Encode the frames losslessly** into the take (`ffmpeg -framerate 60 -i %06d.png -c:v ffv1`,
   or ProRes 4444). Keep the take at 2× the film's size: hivemotion never scales a clip past its
   own pixels (`zoom` at most 1), so a 2× take allows a 2× push-in that stays sharp.
5. **Look at it.** Pull a few stills or put the take in `film.yaml` and run
   `hivemotion prep film.yaml` then `hivemotion render film.yaml --stills <beat>,<beat>+2`.
   Check text is crisp at 100%, nothing private is visible, and the motion is even.
6. **Trim in `film.yaml`, not in an editor.** `clip_start` (seconds into the take) and `speed`
   on the shot; `prep` cuts it.

## When the story needs something that isn't yours

Never show another company's product, logo, screenshot or UI, not even blurred. Draw an
unbranded stand-in as SVG layers and animate it as a `graphic` shot; see
[references/stand-ins.md](references/stand-ins.md).

## When real-time capture is acceptable

Only for a take with no camera move, on an idle machine with a real GPU, after checking the
result frame by frame. Under load or under xvfb it drops frames, and its frame-rate counter lies.

For example, `examples/fired-ide/` is a drawn stand-in, and `examples/hivemind-film/` uses
frame-stepped takes of an Electron app.
