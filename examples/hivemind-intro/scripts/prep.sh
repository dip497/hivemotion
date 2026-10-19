#!/usr/bin/env bash
# Cuts the recordings into the clips the scenes play, and gathers the music and the commit log
# into takes/. Run from the project folder:
#   FILM=<folder with takes/workspace-60 and takes/island> MUSIC=<track.mp3> \
#   HIVEMIND=<a hivemind checkout> scripts/prep.sh
# HIVEMIND can be replaced by COMMITS=<a file from `git log --reverse --format='%h|%ad|%s' --date=short`>.
set -euo pipefail
: "${FILM:?FILM is the folder with takes/workspace-60 and takes/island}"
: "${MUSIC:?MUSIC is the soundtrack mp3}"
out=takes
mkdir -p "$out"
ws="$FILM/takes/workspace-60"
isl="$FILM/takes/island"
enc=(-an -c:v libx264 -preset slow -crf 14 -pix_fmt yuv420p -r 60 -movflags +faststart)

# cut <in> <crop w:h:x:y or -> <out w:h> <out> <segment>... where a segment is start:end:speed
cut() {
  local in=$1 crop=$2 size=$3 dst=$4; shift 4
  local filters="" labels="" i=0
  for seg in "$@"; do
    IFS=: read -r a b speed <<<"$seg"
    local c=""; [[ $crop != - ]] && c="crop=$crop,"
    filters+="[0:v]trim=start=$a:end=$b,setpts=(PTS-STARTPTS)/$speed,${c}scale=$size:flags=lanczos,fps=60[s$i];"
    labels+="[s$i]"; i=$((i+1))
  done
  ffmpeg -v error -y -i "$in" -filter_complex "${filters}${labels}concat=n=$i:v=1:a=0[v]" -map "[v]" "${enc[@]}" "$out/$dst"
  echo "$dst $(ffprobe -v error -show_entries format=duration -of csv=p=0 "$out/$dst")s"
}

cut "$ws/W1-board.mp4"  2240:1400:800:360  1600:1000 board.mp4     0:7.4:1
cut "$isl/I1-hook.mp4"  -                  1920:1200 island.mp4    4.0:9.6:1
cut "$isl/I2-trick.mp4" -                  1920:1200 fireworks.mp4 9.9:11.9:1
cut "$ws/W2-widget.mp4" 2400:1500:790:110  1600:1000 widget.mp4    0.6:2.85:1.5 3.0:6.3:2 6.3:8.9:1 12.4:14.0:1
cut "$ws/W3-tool.mp4"   2240:1400:800:320  1600:1000 tool.mp4      1.8:3.9:1
cut "$ws/W5-sdk.mp4"    2520:1576:680:180  1600:1000 code.mp4      1.6:3.8:1
cut "$ws/W4-fanout.mp4" 1760:1100:1040:560 1600:1000 fanout.mp4    9.0:12.35:1
cut "$ws/W5-sdk.mp4"    1760:1100:1040:560 1600:1000 sdk.mp4       4.6:8.3:1

cp "$MUSIC" "$out/music.mp3"
if [[ -n ${COMMITS:-} ]]; then
  cp "$COMMITS" "$out/commits.txt"
else
  : "${HIVEMIND:?HIVEMIND is a hivemind checkout (or set COMMITS)}"
  git -C "$HIVEMIND" log --reverse --format='%h|%ad|%s' --date=short origin/main > "$out/commits.txt"
fi
echo "commits $(wc -l < "$out/commits.txt")"
