# Frame-stepped capture

hivemotion has no capture command; this is the recipe its example films were made with. You write
the capture script in the app's own test or automation tooling.

## Why

A real-time recorder samples whatever the app drew when the timer fired. On a loaded machine the
app draws late, so frames repeat or skip; a heavy view can fall to about 1 fps. Under xvfb the
GPU is a software rasterizer (llvmpipe), so frame rates measured there are not the frame rates
users see. Neither problem exists if the capture owns the clock.

## The loop

```
for frame in 0..N:
    advance the clock by 1/fps          # one clock for everything that moves
    wait until the app has drawn         # a rendered frame, fonts and images loaded
    grab the window at device scale 2    # lossless PNG
```

**One clock moves everything:**
- the page's time: before the page's own script runs, replace `performance.now()` with a clock
  the script owns and queue `requestAnimationFrame` callbacks; each step adds 1/fps to the clock
  and runs the queued callbacks with it (add `Date.now()` if the app animates from it);
- CSS animations and transitions run on the browser's own clock. Over the Chrome DevTools
  Protocol (`Animation.enable`, then `Animation.setPlaybackRate`), set the rate each frame to
  `(1/fps) / (real seconds the last frame took)`, so they advance one frame's worth per captured
  frame however slow the machine is; reset it to 1 after the take;
- anything scripted outside the page (stand-in users or agents, fake servers) reads the same
  clock, for example from a file the capture script rewrites atomically each step, never wall time;
- typing and clicks are events placed at clock times in the take's script, with a little
  unevenness between keystrokes so typing looks human.

If any one of these runs on wall time, it moves faster or slower than the rest and the take looks
wrong only in that part. Look for it in stills.

**Settle, then keep.** Draw a second or two before the frames you keep, so anything that settles
when the take starts is not in it. Reject any captured frame whose size is not the expected one
(a window that resized mid-take).

**Grab exact frames, not a screen:**
- Electron: `webContents.capturePage()` per frame, with the window at device scale factor 2
  (launch with `--force-device-scale-factor=2`), piping each bitmap into a lossless ffmpeg
  encode. Proven: smooth 60 fps on a machine that drew the view at about 1 fps in real time; a
  27 s take took about 30 minutes to capture.
- Web app in Chromium: a headless browser with `deviceScaleFactor: 2` and a screenshot per frame
  after advancing the clock. (The same mechanism; not yet proven with hivemotion.)
- Native desktop app: only if the app can be driven by a clock you control; otherwise record in
  real time on an idle machine with a GPU and check every frame.

Do not use a browser automation tool's built-in video recorder: some hardcode a low bitrate.

## Frame sizes

| Film aspect | Film size | Capture at 2× |
|---|---|---|
| 16:9 | 1920×1080 | 3840×2160 |
| 9:16 | 1080×1920 | 2160×3840 |
| 4:5 | 1080×1350 | 2160×2700 |

A 9:16 cut can reuse a 16:9 take only where the important part fits a vertical window; otherwise
capture again with the app laid out for a tall window.

## Encode

```bash
ffmpeg -framerate 60 -i frames/%06d.png -c:v ffv1 -pix_fmt yuv444p takes/<beat>.mkv
# or ProRes 4444: -c:v prores_ks -profile:v 4 -pix_fmt yuva444p10le takes/<beat>.mov
```

Never encode a take to H.264 before it reaches the film: each lossy step blurs small text.
