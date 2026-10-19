#!/usr/bin/env bash
# Synthesizes the two sound effects into <dir>: a soft thud and a gentle chime.
set -euo pipefail
dir=${1:-takes}
fx() { ffmpeg -v error -y -f lavfi -i "aevalsrc=$2:s=48000:d=$3" -af "$4" -ac 2 "$dir/$1.wav"; }
# the window landing in the trash
fx thud "0.6*sin(2*PI*(55+90*exp(-t*28))*t)*exp(-t*11)+0.15*(random(1)*2-1)*exp(-t*60)" 0.5 "lowpass=f=900"
# the CI light turning green
fx chime "0.35*(sin(2*PI*880*t)+0.6*sin(2*PI*1318.5*t)+0.3*sin(2*PI*1760*t))*exp(-t*3.5)" 1.3 "afade=t=in:d=0.005"
echo "sfx: $(ls "$dir"/*.wav | wc -l)"
