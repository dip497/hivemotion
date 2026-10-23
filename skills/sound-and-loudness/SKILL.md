---
name: sound-and-loudness
description: Choose a licence-safe music track for a launch film, land its drop and final hit on the film's beats, mix sound effects into it as one piece, and master it to a fixed loudness with a gate (two-pass linear loudnorm, a limiter, -15 to -13 LUFS and at most -1 dBTP) using hivemotion master. Use when a film needs music or sound effects, when the drop doesn't land on the cut, when audio is too quiet, too loud, clipping or pumping, when loudnorm falls back to dynamic, or before uploading anything with sound.
---

# Sound and loudness

The music carries the pacing; the loudness gate makes every file sound right on every platform.

## Steps

1. **Pick the track for its licence first.** CC0, or a licence you can show, with no Content ID
   registration. Record where it came from. See [references/music-sources.md](references/music-sources.md).
2. **Find its cue points.** Note the second in the track of the drop, the final hit and any
   section you want to skip to. Listen, or use a beat tracker; write the seconds down once.
3. **Lay it on the beats** in `film.yaml`. Each section says which second of the track lands on
   which beat; all cues in one section must agree:
   ```yaml
   music:
     file: music/track.wav
     sections:
       - { from: start, cues: { drop: 28.10 } }     # the track's 28.10 s lands on beat 'drop'
       - { from: ending, cues: { hit: 153.80 } }    # jump to the track's ending for the end card
     join: 0.03          # crossfade where sections meet, seconds
     fade_in: 0.08
     fade_out: 1.0
     loudness: { integrated: -14, true_peak: -2.5, range: 14, pre_limit: 0.6 }
   ```
   Move a beat and the music follows it.
4. **Mix sound effects into the track before it reaches hivemotion.** hivemotion lays one music
   file; whooshes, clicks and hits are mixed into that file as one piece. See
   [references/mix.md](references/mix.md).
5. **Master.** `hivemotion master film.yaml` lays the music on each aspect's timeline, normalises
   it in two linear passes, limits it, measures it again and puts it into `build/master-<aspect>.mov`.
   It fails if loudness is outside the gate or if loudnorm fell back to dynamic mode. The chain
   and what to do when it fails: [references/loudness-chain.md](references/loudness-chain.md).
6. **Listen once on a laptop speaker and once on headphones.** The gate proves the numbers, not
   the mix.

`--no-gate` reports loudness without failing; use it only to look, never to ship.

For example, `examples/hivemind-launch/film.yaml` jumps from a track's first drop to its ending
section so the final hit lands on the end card.
