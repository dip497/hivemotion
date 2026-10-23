---
name: deliver-and-publish
description: Turn a mastered launch film into the files each place needs - a poster baked in as frame 0, cutdowns and loops, X, YouTube, Shorts, LinkedIn and Product Hunt files, a README animation, and video for a website that seeks and plays smoothly - then run the final check and write the credits and licences. Use after hivemotion master, when posting or uploading a film, adding a demo to a README or landing page, when a site video won't seek or stutters, or when asked what the platform limits are or how to credit music, fonts and art.
---

# Deliver and publish

## Steps

1. **Choose the poster frame.** `finish.poster` in `film.yaml` points at the strongest settled
   frame (text fully in, not mid-transition). It is baked in as frame 0 of every platform file,
   replacing that frame, so the length and sync are unchanged and every thumbnail shows it.
2. **Deliver.** `hivemotion deliver film.yaml` makes, from each master, the posters (PNG and
   JPEG per aspect), the cutdowns, and every preset the film's aspects allow; or name some:
   `--preset x,youtube,readme-loop`. Files go to `build/deliver/`. Each comes out H.264 High,
   4:2:0, BT.709, AAC 48 kHz, `+faststart`, at `finish.fps`. What each preset is for and its
   limits: [references/platforms.md](references/platforms.md).
3. **Check.** `hivemotion check film.yaml` pulls stills at every beat and halfway through every
   transition, makes a contact sheet per aspect, checks every caption's read time, and measures
   loudness on every file with sound. Read the contact sheets for anything private, clipped,
   overlapping or hard to read. It exits 1 on any problem; fix it in `film.yaml` and run the
   chain again.
4. **README and website.** A README needs an animated WebP made from the `readme-loop` file; a
   website needs a server that answers byte ranges. See
   [references/readme-and-web.md](references/readme-and-web.md).
5. **Credits and licences.** Write them from the asset record before posting; see
   [references/credits.md](references/credits.md).
6. **Post the same day everywhere you post,** with copy written by hand (see
   [launch-film-story](../launch-film-story/references/evidence.md)).
7. **Clean up.** Delete the multi-GB ProRes files in `build/` once nothing needs re-delivering.

## Rules

- Never ship a file that failed `check`. Never ship with `--no-gate`.
- Upload the delivered file, not the master: platforms re-encode ProRes badly or refuse it.
- Captions stay burned in for feeds; most autoplay muted.

For example, `examples/hivemind-launch/film.yaml` defines a 12 s and a 15 s clip and an 8 s silent
loop for a README.
