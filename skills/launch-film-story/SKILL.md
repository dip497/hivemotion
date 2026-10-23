---
name: launch-film-story
description: Plan a product launch film before anything is recorded - the angle, the hook in the first two seconds, the beats, captions that can be read, the length and the cutdowns - and write it into a hivemotion film.yaml. Use when someone wants a launch video, demo film, announcement video, teaser or trailer for a product, app, library or feature, or asks what the film should say, how it should open, or how long it should be. Start here before capture-real-app or compose-and-render.
---

# Launch film story

The story decides most of a film's reach; polish decides little. Plan on paper first, then put
the plan into `film.yaml` so every later step reads one timeline.

## Steps

1. **Answer the brief in one line each.** What is it? Who is it for? What does only this product
   do? What is the most surprising true thing it does? How does someone get it? If "only this
   product" has no answer, stop and find one: being different predicts reach; being clear does
   not. See [references/evidence.md](references/evidence.md).
2. **Pick the hook first.** The first 2 seconds are a face, or the product doing something
   surprising. Never a logo or a title card. Write three candidate hooks and pick one; see
   [references/hooks-and-beats.md](references/hooks-and-beats.md).
3. **Plan the beats.** One claim per beat, shown by the product, not stated. A master of about
   52–90 s, 16:9. Write a table: time, beat name, what is on screen, the caption. Every 3D or
   decorative shot must carry information in the same shot.
4. **Write captions that can be read.** At least 0.3 s per word, counted from when the whole line
   is on screen; at most about 6–8 words per caption. The film must work with the sound off. See
   [references/readability-and-pacing.md](references/readability-and-pacing.md).
5. **List the cutdowns** from the master: a 10–15 s clip per strong beat, a 6–10 s silent loop for
   a README or site hero, an optional vertical cut laid out again (not cropped).
6. **Write it into `film.yaml`.**
   ```bash
   hivemotion new my-film --name "Product"     # film.yaml, brand.yaml, fonts, takes/
   hivemotion validate my-film/film.yaml       # every problem names its field
   ```
   Put each beat in `beats:` once, by name; shots, captions, music cues, the poster and the
   cutdowns all point at beat names (`demo+0.4`). Moving a beat later moves everything on it.
   Set `brand.yaml` from the product's real colours and fonts (TTF or OTF).
7. **Validate early, before any recording exists.** A shot whose take is missing renders as a
   named slate, so `hivemotion render film.yaml --stills <beat>,<beat>` already shows the timing
   and the captions.

## Rules

- Show the real product, recorded, not rebuilt as a mock-up.
- Never show another company's product, logo or screenshot. If the story needs "the old way",
  draw an unbranded stand-in (see capture-real-app).
- Every frame postable: any paused frame should be worth sharing.
- The fframes skill's "no slop" rules apply in full (no invented numbers, generic copy, beat
  spam, one motion for everything); install it with `npx skills add https://fframes.studio`.
- Write post copy by hand, short, first person. Readers punish copy that reads as machine-written.

Next: [capture-real-app](../capture-real-app/SKILL.md) for the takes, then
[compose-and-render](../compose-and-render/SKILL.md).

For example, `examples/hivemind-film/film.yaml` is a planned film with every beat named.
