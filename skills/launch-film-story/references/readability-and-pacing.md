# Readability and pacing

Motion, layout, type sizes and the "no slop" rules are in the fframes skill (`fframes-video`,
`references/design.md`; install with `npx skills add https://fframes.studio`). This page holds
only what a launch film over real UI adds; where the two differ, see
[the table](../../README.md#with-the-fframes-skill).

## The read-time rule

A caption a viewer must read stays **fully on screen and settled for at least 0.3 s per word**,
counted from the moment the whole line is visible (not from when it starts to animate in).
`hivemotion check` enforces this on every caption and every aspect and fails the film when one
is short; for example a 5-word hook shown for 0.97 s fails because it needs 1.5 s.

- Keep captions to about 6–8 words. Split longer thoughts across two beats.
- Pace comes from cuts and motion, never from pulling text away early.
- Text that is only texture (a terminal scrolling, a file list) does not need to be read; text
  that carries the claim does.
- Burn captions in. Most feeds autoplay muted.

## Text on real UI

- Capture the product at device scale 2 so its own text is crisp at 100% (see
  [capture-real-app](../../capture-real-app/SKILL.md)).
- 4:2:0 video blurs small coloured text. Make status visible as fills and borders, not thin
  coloured glyphs.
- When a shot zooms out so far that the UI text is illegible, the shot must say its point another
  way: colour, motion, a caption.

## Transitions and looking

- One claim per beat. The trick may hold longer than other shots if it moves.
- A plain crossfade between two busy UI frames reads as mud. Use a `hex-wipe`, a `hex-iris`, or
  dip through the background.
- Look at stills at every beat and halfway through every transition before rendering the whole
  film (`hivemotion render film.yaml --stills <beats and times>`), and again on the master
  (`hivemotion check` makes them and a contact sheet per aspect).
