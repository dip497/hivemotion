#!/usr/bin/env bash
# Stills of this video's own scenes for the cards that fly in the render scene: takes/fly_0..9.jpg.
set -euo pipefail
times=(FiredScene@60% HiveScene@70% PromptScene@80% OriginScene@40% CanvasScene@80% AskScene@60% IslandScene@50% WidgetScene@90% PluginsScene@70% AnywhereScene@80%)
mkdir -p build/stills
cargo run --release -q -- frame "$(IFS=,; echo "${times[*]}")" -o build/stills
i=0
for f in $(ls build/stills/*.png | sort -V); do
  ffmpeg -v error -y -i "$f" -vf scale=640:360 -q:v 3 "takes/fly_$i.jpg"
  i=$((i+1))
done
echo "stills: $i"
