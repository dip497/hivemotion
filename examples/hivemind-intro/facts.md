# Facts on screen

Every number in the video, the command that produced it and when. The hivemind numbers come from
`origin/main` at `b712697` on 2026-10-19; `src/facts.rs` holds them.

| On screen | Value | Command |
|---|---|---|
| First commit | `66337e8`, 2026-05-27, "Initial OSS release of hivemind" | `git -C hivemind log --reverse --format='%h %ad %s' --date=short origin/main \| head -1` |
| Days | 145 | 2026-05-27 to 2026-10-19: `python3 -c "import datetime as d;print((d.date(2026,10,19)-d.date(2026,5,27)).days)"` |
| Commits | 1,450 | `git -C hivemind rev-list --count origin/main` |
| Releases | 76 | `gh release list -R dip497/hivemind --limit 300 --json isPrerelease -q '[.[]\|select(.isPrerelease\|not)]\|length'` (stable releases; nightlies are pre-releases and not counted) |
| Lines | 193,437 (151,580 TypeScript, 41,857 Rust) | No tokei or cloc on the machine, so `git ls-tree -r --name-only origin/main`, keep `.ts` `.tsx` `.rs`, drop `.d.ts` and anything under `node_modules` or `dist`, then `git cat-file -p origin/main:<file> \| wc -l` summed per extension: ts 125,968, tsx 25,612, rs 41,857. Counts every line, blank and comment lines too, tests included. |
| Plugins on HiveHub | 51 (21 agents, 6 views, 11 widgets, 13 tools) | `curl -s https://hivehub.griiken.workers.dev/api/v1/index.json`, the length of `plugins`, grouped by `type` |
| Frames | 4,394 | `cargo run --release -- timeline` (73.23 s at 60 fps) |
| Render time | 204.5 s (2.8x slower than real time) | wall-clock of the `render` step in `scripts/render.sh`, first full render on 2026-10-20: 8-core Intel laptop, Intel Iris Xe, Skia on Vulkan, ProRes 422 HQ out. The final render, with this number in it, took 213.4 s. |

Left out on purpose:

- Agents detected by default: none are compiled in any more. Every agent arrives as a manifest
  from HiveHub, so the honest number is the HiveHub count above. The video does not show an agent
  count.
- Stars, installs and downloads: not measured here.

The commit log that scrolls in the origin scene is the real `git log` of hivemind's `origin/main`
(subjects cut to 52 characters), made by `scripts/prep.sh`.

Music: "cyberpunk-music-601089.mp3" from Pixabay (Pixabay Content License), the same track as
`examples/hivemind-film/music/`. `scripts/prep.sh` copies it into `takes/`; it is not committed here.
