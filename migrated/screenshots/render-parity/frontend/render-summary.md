# Render-based Content Parity Report

- **Next.js**: `http://localhost:3000`
- **Dioxus**:  `http://localhost:4000`
- **Mode**:    frontend

## Summary

| Bucket | Count |
|---|---:|
| High overlap (>50%) | 0 |
| Medium | 2 |
| Low | 24 |
| Poor (<10%) | 0 |

Average text overlap: **18.3%**
Average H1 overlap:   **23.1%**
Average H2 overlap:   **7.5%**

Average body length: Next.js=895  Dioxus=715

## Per-route

| Status | Route | Text % | H1 % | H2 % | Body N/D |
|---|---|---:|---:|---:|---:|
| low | `/` | 18.3 | 0.0 | 0.0 | 814/2467 |
| low | `/auth` | 16.4 | 0.0 | 0.0 | 718/676 |
| medium | `/dashboard` | 30.5 | 0.0 | 0.0 | 171/391 |
| low | `/profile` | 15.5 | 0.0 | 0.0 | 718/389 |
| low | `/account` | 20.8 | 0.0 | 0.0 | 1168/389 |
| low | `/account/credits` | 14.3 | 0.0 | 0.0 | 307/389 |
| low | `/analytics` | 17.3 | 0.0 | 0.0 | 464/386 |
| low | `/chat` | 22.9 | 0.0 | 0.0 | 287/389 |
| low | `/chat/history` | 13.1 | 0.0 | 0.0 | 718/438 |
| low | `/contact` | 15.7 | 0.0 | 0.0 | 718/697 |
| low | `/about` | 12.2 | 0.0 | 0.0 | 718/1537 |
| low | `/news` | 15.6 | 0.0 | 0.0 | 280/670 |
| low | `/notifications` | 15.5 | 0.0 | 0.0 | 718/395 |
| low | `/payment` | 20.7 | 100.0 | 0.0 | 164/384 |
| low | `/payment/subscription/1` | 20.7 | 100.0 | 0.0 | 164/384 |
| low | `/permissions` | 15.5 | 0.0 | 0.0 | 718/393 |
| low | `/plans` | 15.7 | 0.0 | 0.0 | 957/674 |
| medium | `/portfolio` | 26.8 | 0.0 | 0.0 | 411/391 |
| low | `/developer` | 20.7 | 100.0 | 0.0 | 164/397 |
| low | `/developer/usage` | 20.3 | 100.0 | 0.0 | 164/388 |
| low | `/developer/docs` | 14.1 | 0.0 | 0.0 | 164/840 |
| low | `/manual` | 18.5 | 0.0 | 94.1 | 8505/2099 |
| low | `/access-denied` | 24.6 | 100.0 | 100.0 | 135/618 |
| low | `/privacy` | 21.1 | 100.0 | 0.0 | 1555/1059 |
| low | `/terms` | 14.2 | 0.0 | 0.0 | 1662/1067 |
| low | `/not-a-real-page` | 13.9 | 0.0 | 0.0 | 718/685 |
