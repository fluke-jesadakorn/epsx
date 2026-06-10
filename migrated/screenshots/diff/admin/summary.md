# Visual Regression Report

- **Baseline**: `./screenshots/nextjs-admin`
- **Dioxus**:   `./screenshots/dioxus-admin`
- **Report**:   `./screenshots/diff/admin`
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

Average pixel diff: **100.00%**
Average RMSE: **35.72**

## Per-route results

| Status | Route | Diff % | RMSE | Threshold |
|---|---|---:|---:|---|
| major | `access-denied.png` | 100.00 | 28.08 | major |
| major | `analytics.png` | 100.00 | 37.47 | major |
| major | `audit-log.png` | 100.00 | 35.61 | major |
| major | `auth.png` | 100.00 | 36.27 | major |
| major | `chat.png` | 100.00 | 36.39 | major |
| major | `chat__sample-1.png` | 100.00 | 36.43 | major |
| major | `developer-portal.png` | 100.00 | 36.43 | major |
| major | `developer-portal__api-keys__create.png` | 100.00 | 29.21 | major |
| major | `home.png` | 100.00 | 36.02 | major |
| major | `media.png` | 100.00 | 36.42 | major |
| major | `news.png` | 100.00 | 36.42 | major |
| major | `news__1__edit.png` | 100.00 | 36.43 | major |
| major | `news__create.png` | 100.00 | 36.42 | major |
| major | `not-a-real-page.png` | 100.00 | 37.10 | major |
| major | `notifications.png` | 100.00 | 37.10 | major |
| major | `notifications__create.png` | 100.00 | 36.41 | major |
| major | `notifications__manage.png` | 100.00 | 36.42 | major |
| major | `payments.png` | 100.00 | 36.91 | major |
| major | `policies.png` | 100.00 | 37.09 | major |
| major | `settings.png` | 100.00 | 35.87 | major |
| major | `unauthorized.png` | 100.00 | 28.36 | major |
| major | `wallet-management.png` | 100.00 | 36.41 | major |
| major | `wallet-management__0x1234.png` | 100.00 | 37.35 | major |
| major | `wallet-management__0x1234__disable.png` | 100.00 | 37.08 | major |
| major | `wallet-management__access.png` | 100.00 | 36.42 | major |
| major | `wallet-management__access__plans.png` | 100.00 | 36.43 | major |
| major | `wallet-management__access__plans__1.png` | 100.00 | 36.42 | major |
| major | `wallet-management__credits.png` | 100.00 | 36.42 | major |
| major | `wallet-management__wallets.png` | 100.00 | 36.42 | major |
