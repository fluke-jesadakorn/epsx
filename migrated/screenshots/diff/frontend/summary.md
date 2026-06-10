# Visual Regression Report

- **Baseline**: `./screenshots/nextjs-frontend`
- **Dioxus**:   `./screenshots/dioxus-frontend`
- **Report**:   `./screenshots/diff/frontend`
- **Tolerance**: 3 (per channel)
- **Thresholds**: minor < 0.5%, major >= 5%

## Summary

| Metric | Count |
|---|---|
| Total | 29 |
| Identical | 0 |
| Minor (< 0.5%) | 0 |
| Major (>= 5%) | 29 |
| Size mismatch | 0 |
| Missing baseline | 0 |
| Missing dioxus | 0 |
| **PASS** | **0** |
| **FAIL** | **29** |

Average pixel diff: **98.99%**
Average RMSE: **39.65**

## Per-route results

| Status | Route | Diff % | RMSE | Threshold |
|---|---|---:|---:|---|
| major | `about.png` | 100.00 | 43.39 | major |
| major | `access-denied.png` | 100.00 | 37.50 | major |
| major | `account.png` | 99.99 | 59.55 | major |
| major | `account__credits.png` | 99.99 | 52.06 | major |
| major | `analytics.png` | 90.40 | 41.53 | major |
| major | `auth.png` | 99.60 | 36.04 | major |
| major | `chat.png` | 99.99 | 37.17 | major |
| major | `chat__history.png` | 100.00 | 36.71 | major |
| major | `chat__sample-1.png` | 100.00 | 38.80 | major |
| major | `contact.png` | 99.64 | 37.85 | major |
| major | `dashboard.png` | 99.99 | 38.49 | major |
| major | `developer.png` | 99.98 | 31.34 | major |
| major | `developer__docs.png` | 99.99 | 32.95 | major |
| major | `developer__usage.png` | 99.98 | 31.10 | major |
| major | `home.png` | 98.09 | 47.57 | major |
| major | `manual.png` | 99.99 | 57.01 | major |
| major | `news.png` | 99.68 | 36.89 | major |
| major | `news__welcome-to-epsx.png` | 100.00 | 38.17 | major |
| major | `not-a-real-page.png` | 99.61 | 39.21 | major |
| major | `notifications.png` | 100.00 | 38.78 | major |
| major | `offline.png` | 99.61 | 33.55 | major |
| major | `payment.png` | 99.98 | 30.58 | major |
| major | `payment__subscription__1.png` | 99.98 | 30.99 | major |
| major | `permissions.png` | 100.00 | 38.83 | major |
| major | `plans.png` | 94.27 | 40.15 | major |
| major | `portfolio.png` | 90.08 | 41.34 | major |
| major | `privacy.png` | 100.00 | 41.63 | major |
| major | `profile.png` | 100.00 | 38.69 | major |
| major | `terms.png` | 100.00 | 42.06 | major |
