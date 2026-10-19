# Music and sound effects as one piece

## The rule

Sound effects belong to the music: in the same key, in the same space (the same reverb), blended
in, never laid on top. A launch film mix is a mix, not a track with noises over it.

## Levels and placement

Gains for music, voice, UI sounds and whooshes, ducking values and where to place each sound
relative to its visual event are in the fframes skill's `references/audio.md` (install with
`npx skills add https://fframes.studio`). Two things it doesn't say:

- Nothing harsh or spiky; soften attacks. Repeated small sounds stay in the background.
- Place each effect by its own peak, not by the start of its file, so the transient lands on the
  cut or the beat.

## How to do it with hivemotion

hivemotion places one music file and has no effect track and no gain automation. So:

1. Lay the music on the beats with `film.yaml` and run `hivemotion master film.yaml --no-gate`
   once to get `build/music-mix-<aspect>.wav`, the music as the film plays it.
2. Mix the effects into that file in an audio editor or with ffmpeg (`adelay` to place,
   `volume` to set level, `amix` with `normalize=0` to combine), at 48 kHz.
3. Point `music.file` at the mixed file with a single section whose cue maps the track's 0 s to
   the film's start (`- { from: start, cues: { start: 0.0 } }`), and run `hivemotion master` with
   the gate on.

The mixed file follows one aspect's timeline, and `music.file` is one file for every aspect, so
this route fits films whose aspects all keep the whole film. For a cut that `keep`s only some
spans, master it from a copy of `film.yaml` that points at its own mixed file. (This route
follows from what the tool does today; it has not yet been used on a shipped film.)

## Voice

If the film has a voice-over, duck the music under it (ffmpeg `sidechaincompress`) before the mix
reaches hivemotion; `film.yaml` has no voice track. Captions stay burned in either way.
