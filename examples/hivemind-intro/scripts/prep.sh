#!/usr/bin/env bash
# Cuts the recordings into the clips the scenes play and gathers the stills, the music, the voice
# and the sound effects into takes/. Run from the project folder:
#   FILM=<folder with takes/workspace-60 and island-look> MUSIC=<track.mp3> scripts/prep.sh
set -euo pipefail
: "${FILM:?FILM is the folder with takes/workspace-60 and island-look}"
: "${MUSIC:?MUSIC is the soundtrack mp3}"
out=takes
mkdir -p "$out"
ws="$FILM/takes/workspace-60"
enc=(-an -c:v libx264 -preset slow -crf 14 -pix_fmt yuv420p -r 60 -movflags +faststart)

# cut <in> <crop w:h:x:y> <out w:h> <out> <start:end:speed>
cut() {
  local in=$1 crop=$2 size=$3 dst=$4 a b speed
  IFS=: read -r a b speed <<<"$5"
  ffmpeg -v error -y -i "$in" -vf "trim=start=$a:end=$b,setpts=(PTS-STARTPTS)/$speed,crop=$crop,scale=$size:flags=lanczos,fps=60" "${enc[@]}" "$out/$dst"
  echo "$dst $(ffprobe -v error -show_entries format=duration -of csv=p=0 "$out/$dst")s"
}

# the tiles landing, sped up, and the whole board
cut "$ws/W1-board.mp4" 2240:1400:800:360  1600:1000 hive.mp4         0:8.0:2.6
cut "$ws/W1-board.mp4" 2176:1360:836:320  1920:1200 board.mp4        8.0:11.4:1
# one tile each, its top-left corner, so the terminal text reads
cut "$ws/W1-board.mp4" 640:360:874:1030   1280:720  tile-claude.mp4   6.6:11.4:1
cut "$ws/W1-board.mp4" 640:360:1610:494   1280:720  tile-codex.mp4    6.6:11.4:1
cut "$ws/W1-board.mp4" 640:360:1586:1030  1280:720  tile-opencode.mp4 6.6:11.4:1
cut "$ws/W1-board.mp4" 640:360:2286:1030  1280:720  tile-pi.mp4       6.6:11.4:1
# the CI light going green, and a log tail
cut "$ws/W2-widget.mp4" 2560:1440:640:140 1920:1080 widget.mp4 10.7:14.6:1
cut "$ws/W3-tool.mp4"   2200:1238:820:380 1920:1080 tool.mp4   0.3:3.2:1

for still in wide close; do
  ffmpeg -v error -y -i "$FILM/island-look/after-$still.png" -vf scale=2400:-1:flags=lanczos -q:v 2 "$out/island-$still.jpg"
done
cp "$MUSIC" "$out/music.mp3"
cp voice/vo_*.mp3 "$out/"
bash scripts/sfx.sh "$out"
