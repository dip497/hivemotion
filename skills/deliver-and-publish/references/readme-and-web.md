# README animations and website video

## README: an animated WebP

A video file committed to a repository and referenced from the README does not play inline; it
shows as a link. GitHub plays only videos uploaded as attachments, and package registries and
mirrors that render the README play none. An animated WebP plays everywhere an image shows.

Make it from the `readme-loop` delivery:

```bash
ffmpeg -i build/deliver/readme-loop.mp4 \
  -vf "fps=15,scale=900:-2:flags=lanczos" \
  -loop 0 -c:v libwebp -lossless 0 -q:v 72 -compression_level 6 -an \
  docs/demo.webp
```

- 15 fps is enough for UI and keeps the size down; lanczos keeps small text sharp.
- Quality about 62–72: lower if the file is over budget, higher if text smears.
- Keep it under 10 MB (aim much lower; large images make pages slow and can crash phones).
- Use WebP, not GIF: GIF's 256 colours band gradients and dither UI.

## Website video

- **Serve byte ranges.** The server must answer `Range:` requests with `206 Partial Content`.
  Without it, browsers can't seek, Safari may not play at all, and chapter links or tabs that jump
  into the video break. Test: `curl -sI -H 'Range: bytes=0-99' <url>` shows `206`.
- `+faststart` (every delivered file has it) puts the index at the front so playback starts
  before the download ends.
- A hero loop: `<video autoplay muted loop playsinline poster=...>`. Make the poster a real
  `<img>` (or preload it) so it is the largest paint; never lazy-load the hero video.
- Play loops only while they are in view.
- Give autoplaying motion longer than 5 s a real pause button (WCAG 2.2.2); reduced-motion
  settings show the poster instead.
- Smaller files: AV1 in MP4 first, H.264 as the fallback `<source>`. hivemotion delivers H.264
  only; make AV1 from the master with the system ffmpeg if you need it.
- Don't use alpha video; bake the background in.

## Checking that a page plays smoothly

Average FPS hides the stutters people see. Measure the mechanism:

- **Frame times:** the 95th percentile and the longest frame, from a performance trace, not a
  frame-rate counter.
- **What the page asks the GPU for:** draw calls per frame, layers, and whether only `transform`
  and `opacity` animate.
- Measure on a real display with a real GPU. Under xvfb the browser renders in software
  (llvmpipe), and its numbers say nothing about users' machines.
