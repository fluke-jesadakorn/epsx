# 2026-04-16 Vercel Hybrid Deployment

> **DEPRECATED 2026-06-11** — This plan was written for the pre-migration
> Next.js apps. After Wave 1–4 the user-facing apps (`apps/frontend`,
> `apps/admin-frontend` → `apps/admin`) are Rust binaries
> (`bff-frontend`, `bff-admin`) — Dioxus 0.7 fullstack SSR + Axum.
> Vercel is no longer a valid target.
>
> The replacement plan is
> [`2026-06-11-rust-bff-deployment.md`](./2026-06-11-rust-bff-deployment.md).
> The cutover runbook is
> [`../runbooks/2026-06-11-rust-bff-cutover.md`](../runbooks/2026-06-11-rust-bff-cutover.md).
>
> This file is kept for historical reference only. Do not follow it.

## Target topology

| Component | Runtime | Deployment target | Public entrypoint | Notes |
| --- | --- | --- | --- | --- |
| `apps/frontend` | Next.js | Vercel | `https://epsx.io` | Main user-facing web app |
| `apps/admin-frontend` | Next.js | Vercel | `https://admin.epsx.io` | Admin dashboard |
| `apps/backend` | Rust/Axum | Local Kubernetes | `https://api.epsx.io` | Public API stays self-hosted |
| `apps/contracts` | Foundry | Local + on-chain | n/a | Local Anvil/dev plus chain deployments |

## Repo changes already in place

- Frontend and admin `dev`, `build`, and `start` scripts now load the merged repo root env stack: `.env`, `.env.<environment>`, `.env.local`, and `.env.<environment>.local`.
- Server-side backend URL lookups in both Next apps now use the shared resolver instead of hardcoded localhost fallbacks.
- Media rewrites in both Next apps now prefer `MINIO_ENDPOINT`, then `MINIO_PUBLIC_URL`, then `NEXT_PUBLIC_CDN_URL`, and only fall back to `http://localhost:9100` for local development.

These changes make the two Next.js apps compatible with Vercel builds without breaking local development.

## Vercel project setup

Create two separate Vercel projects from this monorepo:

1. `epsx-frontend`
   Root directory: `apps/frontend`
   Production domain: `epsx.io`
2. `epsx-admin`
   Root directory: `apps/admin-frontend`
   Production domain: `admin.epsx.io`

Recommended project settings:

- Framework preset: `Next.js`
- Install command: use the repo default package manager (`bun install --frozen-lockfile` if overridden manually)
- Build command: `bun run build`
- Development command: `bun run dev`

## Required Vercel environment variables

Set these for both Vercel projects, with values adjusted per app where noted.

| Variable | Frontend value | Admin value | Why |
| --- | --- | --- | --- |
| `BACKEND_URL` | `https://api.epsx.io` | `https://api.epsx.io` | Server-side fetches and middleware |
| `NEXT_PUBLIC_BACKEND_URL` | `https://api.epsx.io` | `https://api.epsx.io` | Client-side API access |
| `FRONTEND_URL` | `https://epsx.io` | `https://epsx.io` | Canonical user app URL |
| `NEXT_PUBLIC_APP_URL` | `https://epsx.io` | `https://admin.epsx.io` | App self URL in each Next app |
| `ADMIN_FRONTEND_URL` | `https://admin.epsx.io` | `https://admin.epsx.io` | Canonical admin URL |
| `NEXT_PUBLIC_ADMIN_URL` | `https://admin.epsx.io` | `https://admin.epsx.io` | Client-side admin links |
| `NEXT_PUBLIC_OAUTH_CLIENT_ID` | `epsx-frontend` | `epsx-admin` | Auth client identity |
| `NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID` | project-specific | project-specific | WalletConnect |
| `NEXT_PUBLIC_BLOCKCHAIN_NETWORK` | `mainnet` or `testnet` | `mainnet` or `testnet` | Web3 config |
| `NEXT_PUBLIC_CHAIN_ID` | `56` or `97` | `56` or `97` | Web3 config |
| `MINIO_ENDPOINT` or `MINIO_PUBLIC_URL` or `NEXT_PUBLIC_CDN_URL` | public storage/CDN URL | public storage/CDN URL | Vercel cannot reach local MinIO on `localhost:9100` |

Use Vercel environment targets deliberately:

- `production` for `epsx.io` and `admin.epsx.io`
- `preview` for pull requests or branch previews
- `development` for `vercel dev`

## Backend requirements

The backend remains the system of record and must stay reachable from Vercel over public HTTPS.

Required backend environment values:

- `BACKEND_URL=https://api.epsx.io`
- `FRONTEND_URL=https://epsx.io`
- `ADMIN_FRONTEND_URL=https://admin.epsx.io`

If the backend is deployed in local Kubernetes, keep the `api.epsx.io` ingress/tunnel pointed at that cluster.

## Important follow-up: preview deployment CORS

The current Rust CORS configuration is based on explicit frontend origins such as `FRONTEND_URL` and `ADMIN_FRONTEND_URL`. Vercel preview deployments use different hostnames, so browser requests from preview URLs will need one of these follow-ups:

- add an allowlist strategy for known preview domains
- add a controlled wildcard/predicate-based preview origin policy
- restrict previews to flows that do not require browser calls to the backend

Production custom domains are not blocked by this issue as long as the backend values above are updated.

## CI/CD ownership after migration

Desired steady state:

- Vercel owns build and deploy for `frontend` and `admin`
- GitLab CI owns build/test/deploy for `backend`
- Contract deployment stays outside the Vercel pipeline

The current `.gitlab-ci.yml` still contains Docker-based frontend/admin deploy jobs. Treat those jobs as legacy until Vercel projects and domains are live, then remove or disable them in a follow-up change.
