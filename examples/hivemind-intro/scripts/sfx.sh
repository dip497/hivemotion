#!/usr/bin/env bash
# Synthesizes the few sound effects into <dir>: soft, short, under the music.
set -euo pipefail
dir=${1:-takes}
fx() { ffmpeg -v error -y -f lavfi -i "aevalsrc=$2:s=48000:d=$3" -af "$4" -ac 2 "$dir/$1.wav"; }
# the flick: filtered noise that swells and falls
fx whoosh "0.5*(random(0)*2-1)*sin(PI*t/0.55)" 0.55 "bandpass=f=1400:width_type=o:w=2,afade=t=in:d=0.2,afade=t=out:st=0.3:d=0.25"
# the window landing in the trash
fx thud "0.9*sin(2*PI*(55+90*exp(-t*28))*t)*exp(-t*11)+0.15*(random(1)*2-1)*exp(-t*60)" 0.5 "lowpass=f=1800"
# a tile landing on the canvas
fx tick "0.5*sin(2*PI*1400*t)*exp(-t*55)+0.3*sin(2*PI*700*t)*exp(-t*35)" 0.15 "lowpass=f=5000"
# the islands rising
fx rise "0.35*sin(2*PI*(180*t+220*t*t))*sin(PI*t/1.1)+0.08*(random(2)*2-1)*sin(PI*t/1.1)" 1.1 "lowpass=f=2400"
# the CI light turning green
fx chime "0.35*(sin(2*PI*880*t)+0.6*sin(2*PI*1318.5*t)+0.3*sin(2*PI*1760*t))*exp(-t*3.5)" 1.3 "afade=t=in:d=0.005"
echo "sfx: $(ls "$dir"/*.wav | wc -l)"
