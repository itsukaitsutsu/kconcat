#!/usr/bin/env python3
"""Structural report on a cue-level .srt.

No audio needed: this measures everything derivable from cue boundaries alone.
It cannot measure word drift - that needs word timestamps or the audio.
"""
import re
import statistics as st
import sys
from pathlib import Path

# Legibility threshold, not the model's floor: a clip may legitimately be as
# short as concat_project::model::MIN_CLIP_DURATION (1/60 s). Below ~0.3 s a
# cue cannot be read, which is a different question from whether it is legal.
MIN_CUE_DURATION = 0.30


def parse(path: Path):
    text = path.read_text(encoding="utf-8", errors="replace")
    text = text.replace("--&gt;", "-->")          # HTML-escaped arrow, if pasted
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    cues = []
    blocks = re.split(r"\n\s*\n", text.strip())
    for block in blocks:
        lines = [l for l in block.split("\n") if l.strip()]
        if not lines:
            continue
        head = next((i for i, l in enumerate(lines) if "-->" in l), None)
        if head is None:
            continue
        m = re.match(
            r"(\d+):(\d+):(\d+)[,.](\d+)\s*-->\s*(\d+):(\d+):(\d+)[,.](\d+)",
            lines[head].strip(),
        )
        if not m:
            continue
        g = [int(x) for x in m.groups()]
        start = g[0] * 3600 + g[1] * 60 + g[2] + g[3] / 1000
        end = g[4] * 3600 + g[5] * 60 + g[6] + g[7] / 1000
        body = " ".join(lines[head + 1:]).strip()
        cues.append({"start": start, "end": end, "text": body})
    return cues


def hhmmss(t: float) -> str:
    m, s = divmod(t, 60)
    return f"{int(m)}:{s:06.3f}"


def main(path: str) -> None:
    cues = parse(Path(path))
    dur = cues[-1]["end"]
    print(f"file            : {path}")
    print(f"cues            : {len(cues)}")
    print(f"span            : {hhmmss(cues[0]['start'])} -> {hhmmss(dur)}  ({dur:.2f} s)")
    print()

    # ---- ordering / overlaps -------------------------------------------------
    overlaps = []
    zero_len = []
    backwards = []
    for i, c in enumerate(cues):
        if c["end"] <= c["start"]:
            zero_len.append(i + 1)
        if i and c["start"] < cues[i - 1]["end"]:
            overlaps.append((i, cues[i - 1]["end"] - c["start"]))
        if i and c["start"] < cues[i - 1]["start"]:
            backwards.append(i + 1)

    print("--- structure ---")
    print(f"overlapping cues        : {len(overlaps)}"
          + (f"  {overlaps[:5]}" if overlaps else ""))
    print(f"zero/negative duration  : {len(zero_len)} {zero_len if zero_len else ''}")
    print(f"out-of-order cues       : {len(backwards)} {backwards if backwards else ''}")

    # ---- durations -----------------------------------------------------------
    durs = [c["end"] - c["start"] for c in cues]
    print()
    print("--- cue duration (s) ---")
    print(f"min {min(durs):.2f} | median {st.median(durs):.2f} | "
          f"mean {st.mean(durs):.2f} | max {max(durs):.2f}")

    long_cues = sorted(
        ((c["end"] - c["start"], i + 1, c["text"]) for i, c in enumerate(cues)),
        reverse=True,
    )[:8]
    print()
    print("longest cues (a caption over ~4 s is a wall of text on a phone):")
    for d, n, t in long_cues:
        print(f"  #{n:>3}  {d:5.2f}s  {t[:64]}")

    short = [(i + 1, d, cues[i]["text"]) for i, d in enumerate(durs)
             if d < MIN_CUE_DURATION]
    print()
    print(f"cues under {MIN_CUE_DURATION:.2f}s (check against the model's minimum clip length):")
    for n, d, t in short:
        print(f"  #{n:>3}  {d:5.2f}s  {t!r}")

    # ---- reading speed -------------------------------------------------------
    print()
    print("--- reading speed (characters/second) ---")
    cps = [(len(c["text"]) / (c["end"] - c["start"]), i + 1)
           for i, c in enumerate(cues) if c["end"] > c["start"]]
    vals = [v for v, _ in cps]
    print(f"median {st.median(vals):.2f} cps | p90 {sorted(vals)[int(len(vals)*.9)]:.2f} | "
          f"max {max(vals):.2f}")
    fast = sorted(((v, n) for v, n in cps if v > 21), reverse=True)[:6]
    if fast:
        print("too fast to read (>21 cps):")
        for v, n in fast:
            print(f"  #{n:>3}  {v:5.1f} cps  {cues[n-1]['text'][:60]!r}")

    # ---- gaps ----------------------------------------------------------------
    print()
    print("--- gaps between cues (what silence detection should find) ---")
    gaps = [(cues[i + 1]["start"] - cues[i]["end"], i + 1, i + 2)
            for i in range(len(cues) - 1)]
    total_gap = sum(g for g, _, _ in gaps)
    speech = sum(durs)
    print(f"speech {speech:.1f}s ({speech/dur*100:.0f}%) | "
          f"gap {total_gap:.1f}s ({total_gap/dur*100:.0f}%)")
    exact = [n for g, _, n in gaps if abs(g) < 0.001]
    print(f"boundaries with a 0 s gap (whisper segment joins, not real pauses): {len(exact)}")
    for n in exact:
        print(f"    #{n-1} -> #{n}   {cues[n-2]['text'][-30:]!r} | {cues[n-1]['text'][:30]!r}")

    notable = sorted(((g, a, b) for g, a, b in gaps if g >= 0.5), reverse=True)
    print(f"\ngaps >= 0.5 s ({len(notable)} of them):")
    for g, a, b in notable:
        print(f"  {g:5.2f}s   after #{a:<3} at {hhmmss(cues[a-1]['end'])}")

    # ---- edit candidates -----------------------------------------------------
    print()
    print("--- retakes / production lines that a trim would remove ---")
    for i, c in enumerate(cues):
        t = c["text"].lower()
        if any(k in t for k in ("sorry", "ulang", "kamera", "salah", "gak keambil")):
            print(f"  #{i+1:>3}  {hhmmss(c['start'])}  {c['text'][:60]!r}")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "user-video.srt")
