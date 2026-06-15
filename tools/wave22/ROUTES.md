# Wave 22 — Prod Baseline Routes (https://epsx.io)

Captured 2026-06-15 via `snap.sh` (playwright headless shell, 1280x800, 8s virtual-time-budget).
All 28 PNGs in `prod-baseline/`. HTTP status codes from `curl -I` (no follow).

| # | Slug | Path | HTTP | PNG | Notes |
|---|------|------|------|-----|-------|
| 1 | `home` | `/` | 200 | `home.png` | landing page |
| 2 | `about` | `/about` | 307 | `about.png` | middleware redirect (locale/region gate), browser still renders HTML |
| 3 | `access-denied` | `/access-denied` | 200 | `access-denied.png` | |
| 4 | `account` | `/account` | 200 | `account.png` | |
| 5 | `account-credits` | `/account/credits` | 200 | `account-credits.png` | |
| 6 | `analytics` | `/analytics` | 200 | `analytics.png` | |
| 7 | `auth` | `/auth` | 200 | `auth.png` | |
| 8 | `chat` | `/chat` | 200 | `chat.png` | |
| 9 | `chat-sample-conv-id` | `/chat/sample-conv-id` | 307 | `chat-sample-conv-id.png` | middleware redirect, sample id still renders |
| 10 | `chat-history` | `/chat/history` | 307 | `chat-history.png` | middleware redirect |
| 11 | `contact` | `/contact` | 307 | `contact.png` | middleware redirect |
| 12 | `dashboard` | `/dashboard` | 200 | `dashboard.png` | |
| 13 | `developer` | `/developer` | 200 | `developer.png` | |
| 14 | `developer-usage` | `/developer/usage` | 200 | `developer-usage.png` | |
| 15 | `developer-docs` | `/developer/docs` | 200 | `developer-docs.png` | |
| 16 | `manual` | `/manual` | 200 | `manual.png` | |
| 17 | `news` | `/news` | 200 | `news.png` | |
| 18 | `news-sample-slug` | `/news/sample-slug` | 200 | `news-sample-slug.png` | |
| 19 | `notifications` | `/notifications` | 307 | `notifications.png` | middleware redirect |
| 20 | `offline` | `/offline` | 307 | `offline.png` | middleware redirect |
| 21 | `payment` | `/payment` | 200 | `payment.png` | |
| 22 | `payment-intent-sample-id` | `/payment/intent/sample-id` | 200 | `payment-intent-sample-id.png` | |
| 23 | `permissions` | `/permissions` | 307 | `permissions.png` | middleware redirect |
| 24 | `plans` | `/plans` | 200 | `plans.png` | |
| 25 | `portfolio` | `/portfolio` | 200 | `portfolio.png` | |
| 26 | `privacy` | `/privacy` | 200 | `privacy.png` | |
| 27 | `profile` | `/profile` | 307 | `profile.png` | middleware redirect |
| 28 | `terms` | `/terms` | 200 | `terms.png` | |

## Skipped

| Path | Reason |
|------|--------|
| `/portfolio/0x...dEaD` | brief instructs to skip — returns 307 (per-route redirect); the `portfolio` listing page above is the same surface. |
| `/admin/*` | not marketing pages; not in scope for this wave |
| `/unauthorized` | not a real route — returns 404 |

## Capture method

```bash
bash .wave22/snap.sh <url> <out.png> <user-data-dir-name>
# exit 0  -> PNG > 1KB
# exit 1  -> timeout / network error / PNG too small
```

Flags: `--no-sandbox --disable-gpu --user-data-dir=/tmp/hs-<uniq> --hide-scrollbars --window-size=1280,800 --virtual-time-budget=8000 --screenshot=<out>`.

`pkill -f chrome-headless-shell; sleep 0.5` runs before each shot to avoid zombie hangs.
