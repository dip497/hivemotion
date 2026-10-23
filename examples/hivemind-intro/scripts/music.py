"""A calm bed for the voice: warm pad chords and a soft plucked pulse on the video's 136 BPM grid.

Writes a 48 kHz stereo WAV. Usage: python3 scripts/music.py takes/calm-dry.wav
"""
import math
import struct
import sys
import wave

RATE = 48000
BEAT = 60.0 / 136.0
SECONDS = 34.0
# two bars a chord: D major 9, B minor 11, G major 7, A6 sus4
CHORDS = [
    [146.83, 220.00, 329.63, 369.99, 554.37],
    [123.47, 185.00, 293.66, 329.63, 440.00],
    [98.00, 146.83, 246.94, 369.99, 440.00],
    [110.00, 164.81, 293.66, 369.99, 493.88],
]
CHORD_SECONDS = 8 * BEAT
# plucked notes, one every two beats, an octave up from the chord
ARP = [2, 3, 4, 3]


def pad_env(t, length):
    attack, release = 1.4, 1.6
    if t < 0 or t > length + release:
        return 0.0
    a = min(1.0, t / attack)
    r = 1.0 if t <= length else 1.0 - (t - length) / release
    return a * a * (3 - 2 * a) * max(0.0, r)


def main(path):
    n = int(SECONDS * RATE)
    left = [0.0] * n
    right = [0.0] * n
    chords = int(SECONDS / CHORD_SECONDS) + 1
    for k in range(chords):
        start = k * CHORD_SECONDS
        notes = CHORDS[k % len(CHORDS)]
        i0 = int(start * RATE)
        i1 = min(n, int((start + CHORD_SECONDS + 1.6) * RATE))
        for f in notes:
            amp = 0.05 * (220.0 / f) ** 0.35
            for i in range(i0, i1):
                t = i / RATE - start
                e = pad_env(t, CHORD_SECONDS) * amp
                if e == 0.0:
                    continue
                w = 2 * math.pi * f * t
                trem = 1.0 + 0.08 * math.sin(2 * math.pi * 0.23 * t)
                left[i] += e * trem * (math.sin(w * 0.9985) + 0.18 * math.sin(2 * w))
                right[i] += e * trem * (math.sin(w * 1.0015) + 0.18 * math.sin(2 * w))
        for j, step in enumerate(range(0, 8, 2)):
            at = start + step * BEAT
            f = notes[ARP[j % len(ARP)]] * 2
            p0 = int(at * RATE)
            for i in range(p0, min(n, p0 + int(1.6 * RATE))):
                t = i / RATE - at
                e = 0.045 * math.exp(-t * 3.2) * (1 - math.exp(-t * 300))
                s = e * (math.sin(2 * math.pi * f * t) + 0.25 * math.sin(4 * math.pi * f * t) * math.exp(-t * 6))
                left[i] += s * (0.85 if j % 2 else 1.0)
                right[i] += s * (1.0 if j % 2 else 0.85)
    fade = int(1.5 * RATE)
    with wave.open(path, "wb") as out:
        out.setnchannels(2)
        out.setsampwidth(2)
        out.setframerate(RATE)
        frames = bytearray()
        for i in range(n):
            g = min(1.0, i / fade)
            frames += struct.pack("<hh", int(max(-1, min(1, left[i] * g)) * 32767), int(max(-1, min(1, right[i] * g)) * 32767))
        out.writeframes(bytes(frames))


if __name__ == "__main__":
    main(sys.argv[1])
