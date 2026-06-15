#!/usr/bin/env bash
# pixel-diff.sh — compare two PNGs and emit a diff image + pixel-difference count
#
# Usage:
#   pixel-diff.sh <A.png> <B.png> <diff.png>
#
# Behavior:
#   - Tries ImageMagick `compare -metric AE -fuzz 2%` first.
#   - Falls back to python3 + PIL (per-pixel diff, resize to A's dimensions if
#     sizes differ, output PNG with red pixels where they differ).
#   - Prints "PIXEL_DIFF=<N>" on stdout.
#   - Exits 0 even if diff is non-zero (this is a measurement, not a gate).
#
# Exit codes:
#   0  comparison done (diff may be > 0)
#   1  input file missing
#   2  usage error
set -u

A="${1:-}"
B="${2:-}"
DIFF="${3:-}"

if [[ -z "$A" || -z "$B" || -z "$DIFF" ]]; then
  echo "usage: $0 <A.png> <B.png> <diff.png>" >&2
  exit 2
fi

if [[ ! -f "$A" ]]; then echo "pixel-diff.sh: missing $A" >&2; exit 1; fi
if [[ ! -f "$B" ]]; then echo "pixel-diff.sh: missing $B" >&2; exit 1; fi

mkdir -p "$(dirname "$DIFF")"

if command -v compare >/dev/null 2>&1; then
  # ImageMagick path: fuzz 2% to ignore trivial anti-aliasing differences
  # -metric AE prints the count of pixels that differ (with fuzz applied) to stderr
  # -highlight-color red -lowlight-color white for visibility
  AE=$(compare -metric AE -fuzz 2% -highlight-color red -lowlight-color white \
       "$A" "$B" "$DIFF" 2>&1 >/dev/null || true)
  AE_INT=$(echo "$AE" | awk '{print $1}')
  echo "PIXEL_DIFF=${AE_INT:-0}"
  exit 0
fi

# PIL fallback
python3 - <<PY
from PIL import Image, ImageChops
import sys
A = Image.open("$A").convert("RGB")
B = Image.open("$B").convert("RGB")
if A.size != B.size:
    B = B.resize(A.size, Image.LANCZOS)
diff = ImageChops.difference(A, B)
# Count non-zero pixels (any channel > 0 after the 2% fuzz)
# Approximate 2% fuzz on 8-bit channel: 255*0.02 ≈ 5
threshold = 5
px = diff.load()
w, h = diff.size
n = 0
for y in range(h):
    for x in range(w):
        r, g, b = px[x, y]
        if r > threshold or g > threshold or b > threshold:
            n += 1
# Build a visualization: red where differ, white where same, save
out = Image.new("RGB", diff.size, (255, 255, 255))
opx = out.load()
for y in range(h):
    for x in range(w):
        r, g, b = px[x, y]
        if r > threshold or g > threshold or b > threshold:
            opx[x, y] = (255, 0, 0)
out.save("$DIFF")
print(f"PIXEL_DIFF={n}")
PY
exit 0
