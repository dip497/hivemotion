#!/usr/bin/env bash
# The deliverable from the ProRes master: H.264 High, BT.709, AAC 48 kHz at -14 LUFS (two-pass
# loudnorm, linear), faststart. Usage: scripts/finish.sh build/intro.mov build/hivemind-intro.mp4
set -euo pipefail
in=$1 out=$2
measured=$(ffmpeg -hide_banner -nostats -i "$in" -map 0:a -af loudnorm=I=-14:TP=-1.5:LRA=11:print_format=json -f null - 2>&1 | sed -n '/^{/,/^}/p')
get() { sed -n "s/.*\"$1\" : \"\(.*\)\".*/\1/p" <<<"$measured"; }
ffmpeg -v error -y -i "$in" \
  -map 0:v -map 0:a \
  -c:v libx264 -profile:v high -preset slow -crf 16 -pix_fmt yuv420p \
  -colorspace bt709 -color_primaries bt709 -color_trc bt709 \
  -af "loudnorm=I=-14:TP=-1.5:LRA=11:measured_I=$(get input_i):measured_TP=$(get input_tp):measured_LRA=$(get input_lra):measured_thresh=$(get input_thresh):offset=$(get target_offset):linear=true,aresample=48000" \
  -c:a aac -b:a 256k -movflags +faststart "$out"
ffmpeg -hide_banner -nostats -i "$out" -map 0:a -af ebur128=peak=true -f null - 2>&1 | grep -E "^\s+(I|LRA|Peak):" | tr -s ' '
