# The loudness chain

What `hivemotion master` runs, and why each step is there.

```
music laid on the beats (48 kHz)
  → optional pre-limit (alimiter at music.loudness.pre_limit)
  → loudnorm pass 1: measure
  → loudnorm pass 2: linear=true with the measured values
  → alimiter limit=0.891 (≈ −1 dBFS) level=disabled
  → measure again with ebur128 (true peak on)
  → gate: −15 ≤ integrated ≤ −13 LUFS, true peak ≤ −1 dBTP
```

## Why each step

- **Two passes, linear.** One-pass loudnorm rides the gain up and down (dynamic mode) and pumps
  music. Pass 1 measures; pass 2 applies one constant gain. ffmpeg **silently falls back to
  dynamic** when the target range is below the source's range or the gain would push the true
  peak over the target, so hivemotion reads `normalization_type` from pass 2 and fails unless it
  is `linear`.
- **A limiter after loudnorm.** Linear gain does not back off for one loud transient. `alimiter`
  catches it, with `level=disabled`, because otherwise the limiter adds its own make-up gain and
  the result ends louder than the target.
- **A pre-limit before loudnorm** (optional). One loud hit can force dynamic mode; limiting it
  first keeps linear mode possible.
- **Measure the file that ships.** The gate measures the mastered file, and `check` measures
  every delivered file with sound again.
- **48 kHz, set explicitly.** loudnorm upsamples internally and fframes defaults to 44.1 kHz.
- **fframes' own limiter is off;** mastering owns the peaks.

## Targets

One master serves every platform: about −14 LUFS integrated, true peak at most −1 dBTP. YouTube
turns louder files down and never turns quieter ones up; most feeds publish no target. Aim
`loudness.integrated` at −14 and `true_peak` a little under the gate (−1.5 to −2.5) so the limiter
has room. As an example of what the chain does, one film's mix went from −12.2 LUFS / +0.4 dBTP
(too loud, clipping) to −13.8 LUFS / −4.0 dBTP.

## When master fails

| Message | Do |
|---|---|
| loudnorm fell back to dynamic | Lower `pre_limit` (for example 0.6), or raise `range` (for example 14) |
| integrated outside −15..−13 | Move `integrated` toward −14; check the track isn't mostly silence |
| true peak above −1 dBTP | Lower `true_peak`, or add/lower `pre_limit` |
| music not loaded / no file | The `music.file` path is relative to film.yaml |

Never "fix" a failing gate with `--no-gate`.
