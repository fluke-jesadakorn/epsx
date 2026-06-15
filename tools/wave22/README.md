# tools/wave22/

Visual harness for the dioxus-microservices wave-22 port.

## Scripts

- `snap.sh <url> [out.png] [user-data-dir-name]` — single screenshot via playwright
  headless shell (`chromium_headless_shell-1208`).
  Exit 0 on PNG > 1KB, 1 on timeout / network error / too-small PNG.
- `pixel-diff.sh <A.png> <B.png> <diff.png>` — ImageMagick `compare -metric AE`
  with PIL fallback. Prints `PIXEL_DIFF=<N>` on stdout.
- `capture-all.sh` — driver that walks the 28-route list in `ROUTES.md` and
  dumps to `prod-baseline/`.

## Baseline

`ROUTES.md` lists the 28 marketing routes captured from `https://epsx.io` for
the wave-22 visual parity baseline. PNGs themselves live outside this worktree
at `<repo-root>/.wave22/prod-baseline/<slug>.png` (10 MB total — not committed).

## Re-running

```bash
bash tools/wave22/capture-all.sh
# or
bash tools/wave22/snap.sh https://epsx.io/ /tmp/x.png
bash tools/wave22/pixel-diff.sh prod-baseline/a.png prod-baseline/b.png /tmp/d.png
```

## Headless shell binary

```
/Users/fluke/Library/Caches/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-mac-arm64/chrome-headless-shell
```

If you don't have it:

```bash
npx playwright install chromium
# then look for the actual version under
#   ~/Library/Caches/ms-playwright/chromium_headless_shell-<N>/
# and update `CHROME=` in snap.sh.
```
