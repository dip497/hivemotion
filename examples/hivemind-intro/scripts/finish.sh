#!/usr/bin/env bash
# The deliverable from the ProRes master: H.264 High, BT.709, AAC 48 kHz at -14 LUFS (two-pass
# loudnorm, linear), faststart. Usage: scripts/finish.sh build/intro.mov build/hivemind-intro.mp4 [audio.wav]
# An audio file as the third argument replaces the master's sound (the version without music).
set -euo pipefail
in=$1 out=$2 audio=${3:-$1}
measured=$(ffmpeg -hide_banner -nostats -i "$audio" -map 0:a -af loudnorm=I=-14:TP=-2:LRA=11:print_format=json -f null - 2>&1 | sed -n '/^{/,/^}/p')
get() { sed -n "s/.*\"$1\" : \"\(.*\)\".*/\1/p" <<<"$measured"; }
ffmpeg -v error -y -i "$in" -i "$audio" \
  -map 0:v -map 1:a \
  -c:v libx264 -profile:v high -preset slow -crf 16 -pix_fmt yuv420p \
  -colorspace bt709 -color_primaries bt709 -color_trc bt709 \
  -af "loudnorm=I=-14:TP=-2:LRA=11:measured_I=$(get input_i):measured_TP=$(get input_tp):measured_LRA=$(get input_lra):measured_thresh=$(get input_thresh):offset=$(get target_offset):linear=true,aresample=48000,alimiter=limit=0.75:attack=1:release=60:level=false" \
  -c:a aac -b:a 256k -movflags +faststart "$out"
# loudnorm stays linear and can land short of -14 when the peaks have no room: trim the rest
loudness() { ffmpeg -hide_banner -nostats -i "$1" -map 0:a -af ebur128 -f null - 2>&1 | sed -n 's/^ *I: *\(-[0-9.]*\) LUFS/\1/p' | tail -1; }
trim=$(echo "-14 - $(loudness "$out")" | bc)
ffmpeg -v error -y -i "$out" -map 0 -c:v copy -af "volume=${trim}dB,alimiter=limit=0.85:attack=1:release=60:level=false" -c:a aac -b:a 256k -movflags +faststart "$out.tmp.mp4"
mv "$out.tmp.mp4" "$out"
ffmpeg -hide_banner -nostats -i "$out" -map 0:a -af ebur128=peak=true -f null - 2>&1 | grep -E "^\s+(I|LRA|Peak):" | tr -s ' '
