# Rendering on another machine

Rendering, mastering and delivering are heavy. When the laptop is busy, run them on another
machine over ssh. hivemotion has no remote command; this is the procedure.

Write the machine's details in `hosts.local.yaml` (copy `hosts.example.yaml`; it is gitignored).
Never commit host names, addresses, user names or personal paths.

## Before starting

1. **Own folder.** Everything (toolchain, `target/`, caches, the film) lives under one folder of
   your own on that machine. Touch nothing outside it, and never stop anything you did not start.
2. **Take a lock** so two renders don't run at once: `flock -n <your folder>/render.lock <command>`
   refuses when another render holds it. If the machine has its own lock files for other work,
   check those are free too. A wrapper that refuses should exit 75 ("busy, try later"), so the
   caller can fall back to running the job locally at low priority instead of failing.
3. **Check free memory** from `/proc/meminfo` (`MemAvailable`) and wait until there is more than
   the render needs (a 1080p render peaked at about 3.6 GB). Run at a high niceness (`nice -n 10`).

## Running

```bash
rsync -a --delete my-film/ host:<folder>/my-film/         # film, takes, music; not build/
ssh host 'cd <folder> && flock -n render.lock nice -n 10 \
  ./hivemotion render my-film/film.yaml --json' > render.json
rsync -a host:<folder>/my-film/build/deliver/ my-film/build/deliver/
```

Run long steps under `nohup` or a terminal multiplexer so a dropped connection doesn't kill them.

**If rsync or ssh hangs** after a dropped connection, the shared ssh connection (ControlMaster) is
probably stale. Close it with `ssh -O exit host` (or delete the socket file) and run again.

## After

- Copy back only what you need: the delivered files, posters, stills.
- **Delete the ProRes masters** (`build/render-*.mov`, `build/master-*.mov`) once delivery is
  done: they are several GB each. Keep them only while you may still re-deliver.
- Remove your folder when the film is finished. Leave the machine as you found it.
