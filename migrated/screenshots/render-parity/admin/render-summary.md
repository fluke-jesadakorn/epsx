# Render-based Content Parity Report

- **Next.js**: `http://localhost:3001`
- **Dioxus**:  `http://localhost:4001`
- **Mode**:    admin

## Summary

| Bucket | Count |
|---|---:|
| High overlap (>50%) | 0 |
| Medium | 0 |
| Low | 25 |
| Poor (<10%) | 3 |

Average text overlap: **17.1%**
Average H1 overlap:   **2.4%**
Average H2 overlap:   **10.7%**

Average body length: Next.js=980  Dioxus=140

## Per-route

| Status | Route | Text % | H1 % | H2 % | Body N/D |
|---|---|---:|---:|---:|---:|
| low | `/` | 11.8 | 0.0 | 0.0 | 1625/140 |
| low | `/analytics` | 17.2 | 0.0 | 0.0 | 1046/144 |
| low | `/audit-log` | 19.3 | 66.7 | 0.0 | 1010/134 |
| low | `/chat` | 18.6 | 0.0 | 0.0 | 1046/140 |
| low | `/developer-portal` | 18.4 | 0.0 | 0.0 | 1024/140 |
| poor | `/developer-portal/api-keys/create` | 6.7 | 0.0 | 100.0 | 168/140 |
| low | `/media` | 21.3 | 0.0 | 0.0 | 880/140 |
| low | `/news` | 21.1 | 0.0 | 0.0 | 906/140 |
| low | `/news/create` | 19.8 | 0.0 | 0.0 | 901/140 |
| low | `/news/1/edit` | 19.5 | 0.0 | 0.0 | 903/140 |
| low | `/notifications` | 15.8 | 0.0 | 0.0 | 1101/140 |
| low | `/notifications/create` | 13.6 | 0.0 | 0.0 | 1341/140 |
| low | `/notifications/manage` | 15.8 | 0.0 | 0.0 | 1101/140 |
| low | `/payments` | 20.4 | 0.0 | 0.0 | 1092/143 |
| low | `/policies` | 20.0 | 0.0 | 0.0 | 872/140 |
| low | `/settings` | 18.4 | 0.0 | 0.0 | 1069/140 |
| poor | `/unauthorized` | 6.7 | 0.0 | 100.0 | 168/140 |
| low | `/wallet-management` | 19.8 | 0.0 | 0.0 | 1126/140 |
| low | `/wallet-management/wallets` | 19.8 | 0.0 | 0.0 | 1126/140 |
| low | `/wallet-management/0x1234` | 19.1 | 0.0 | 0.0 | 1187/140 |
| low | `/wallet-management/0x1234/disable` | 19.8 | 0.0 | 0.0 | 900/140 |
| low | `/wallet-management/credits` | 19.4 | 0.0 | 0.0 | 1106/140 |
| low | `/wallet-management/access` | 21.4 | 0.0 | 0.0 | 1047/140 |
| low | `/wallet-management/access/plans` | 21.4 | 0.0 | 0.0 | 1055/140 |
| low | `/wallet-management/access/plans/1` | 21.4 | 0.0 | 0.0 | 1044/140 |
| poor | `/access-denied` | 0.0 | 0.0 | 100.0 | 99/140 |
| low | `/auth` | 11.8 | 0.0 | 0.0 | 1625/140 |
| low | `/not-a-real-page` | 20.0 | 0.0 | 0.0 | 879/140 |
