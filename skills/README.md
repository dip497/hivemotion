# Skills

Agent Skills for making a launch film with hivemotion, for any product. Each folder is one skill:
a short `SKILL.md` with the steps, and `references/` with the detail. They load in any agent that
reads the Agent Skills format (Claude Code, Codex, Gemini CLI, Cursor, opencode).

Use them in this order. Each one says what it needs from the one before.

| Skill | Use it to | Drives |
|---|---|---|
| [launch-film-story](launch-film-story/SKILL.md) | find the angle and the hook, plan the beats, and write the script and storyboard into `film.yaml` | `new`, `validate` |
| [capture-real-app](capture-real-app/SKILL.md) | record the product itself, smooth at 60 fps on any machine, with no private data and no one else's product on screen | (writes `takes/`) `prep`, `render --stills` |
| [compose-and-render](compose-and-render/SKILL.md) | lay shots, graphics, captions and transitions on the beats and render ProRes masters, here or on another machine | `prep`, `render` |
| [sound-and-loudness](sound-and-loudness/SKILL.md) | choose and place the music, mix sound effects into it, and master it through the loudness gate | `master` |
| [deliver-and-publish](deliver-and-publish/SKILL.md) | make the poster, cutdowns and platform files, the README loop and site video, and record credits and licences | `deliver`, `check` |

## With the fframes skill

fframes ships its own skill, `fframes-video` (install with `npx skills add https://fframes.studio`).
It covers the fframes API, motion and layout basics, its "no slop" rules, the preview window,
contact sheets and loudness reports through a project's own CLI. These skills don't repeat any
of that; they cover what it lacks: the `film.yaml` workflow, frame-stepped capture of a real
app, data hygiene, mastering through a gate, delivery presets, README and website video, and
rendering on another machine. Read both. Where they disagree, for a hivemotion film follow these:

| fframes-video says | Here | Why |
|---|---|---|
| Read time: 3 words a second plus 1 s after the text settles | A hard floor of 0.3 s per word fully on screen, enforced by `hivemotion check` | Dynamic pacing drove about 20× the reach of flat pacing in the launch studies ([evidence](launch-film-story/references/evidence.md)); fframes' rule is a good target for long lines but fails short hooks that read fine. Not measured: we have no reading-speed study of our own |
| Check loudness with `audio analyze`; the master bus limiter keeps peaks under −1 dBFS | Two-pass linear loudnorm, a limiter with `level=disabled`, then a gate on the delivered file; fframes' limiter is off | A report does not stop a bad file shipping, loudnorm silently falls back to dynamic mode, and a limiter with make-up gain overshoots the target ([chain](sound-and-loudness/references/loudness-chain.md)) |
| Install x264 and encode `.mp4` from fframes | Master to ProRes 4444; H.264 from the system ffmpeg as a separate program | fframes' H.264 feature is GPL; hivemotion stays MIT ([gaps](compose-and-render/references/fframes-gaps.md)) |
| Cross-fade scenes | Wipe, iris or dip through the background between busy frames | A plain crossfade of two busy UI frames reads as a muddy double exposure |
| Use the Skia GPU backend | hivemotion renders on the CPU backend | It runs on any headless Linux machine or remote box with no GPU; the cost is in [gaps](compose-and-render/references/fframes-gaps.md#cost) |
| Write a Rust scene per video | Edit `film.yaml`; nothing to compile | One renderer reads the film at run time, so an agent changes YAML and renders again |

A worked example of all five is the Hivemind launch film in [`../examples/`](../examples/).

Every command takes `--json` and prints one object with `ok`; it exits 1 when `ok` is false. Outputs
go to `build/` beside `film.yaml`. `hivemotion <command> --help` is the source of truth for flags.
