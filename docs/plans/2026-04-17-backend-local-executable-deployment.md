# 2026-04-17 Backend Local Executable Deployment

## Goal

Temporarily stop using Docker and Kubernetes for the Rust backend.

Run three local backend executables directly on the Mac host:

| Environment | Public hostname | Local bind |
| --- | --- | --- |
| Development | `dev-api.epsx.io` | `127.0.0.1:18080` |
| Staging | `staging-api.epsx.io` | `127.0.0.1:28080` |
| Production | `api.epsx.io` | `127.0.0.1:38080` |

Cloudflare Tunnel publishes those local ports through one named tunnel using [infrastructure/cloudflare/cloudflared-config.backend-local.yml](/Users/fluke/Desktop/Work/epsx/infrastructure/cloudflare/cloudflared-config.backend-local.yml).

On the current Mac host, that tunnel is remotely managed and still targets legacy local ports:

| Environment | Legacy local port | Forward target |
| --- | --- | --- |
| Development | `127.0.0.1:8080` | `127.0.0.1:18080` |
| Staging | `127.0.0.1:4810` | `127.0.0.1:28080` |
| Production | `127.0.0.1:9180` | `127.0.0.1:38080` |

Keep `com.epsx.port-bridge` loaded so those legacy ports forward into the three backend executables.

## Files

- Environment templates: `infrastructure/local-backend/*.env.template`
- Build command: `./infrastructure/scripts/build-backend-local.sh`
- Runtime wrapper: `./infrastructure/scripts/run-backend-instance.sh`
- Boot-time install: `sudo ./infrastructure/scripts/install-backend-local-services.sh`

## Secret layout

Do not commit runtime secrets. Create these files locally:

```bash
mkdir -p .secret/backend
cp infrastructure/local-backend/dev.env.template .secret/backend/dev.env
cp infrastructure/local-backend/staging.env.template .secret/backend/staging.env
cp infrastructure/local-backend/prod.env.template .secret/backend/prod.env
```

Each env file is self-contained and should define at minimum:

- `DATABASE_URL`
- `REDIS_URL`
- `BACKEND_URL`
- `FRONTEND_URL`
- `ADMIN_FRONTEND_URL`
- `JWT_SECRET`
- `BLOCKCHAIN_NETWORK` and `NEXT_PUBLIC_BLOCKCHAIN_NETWORK`

Add `MINIO_*` values if media upload endpoints are required.

## Install on macOS boot

1. Build the binary:

```bash
./infrastructure/scripts/build-backend-local.sh
```

2. Install the backend LaunchDaemons plus the Cloudflare Tunnel service:

```bash
sudo ./infrastructure/scripts/install-backend-local-services.sh
```

This script does five things:

- validates `.secret/backend/dev.env`, `staging.env`, and `prod.env`
- installs three LaunchDaemons under `/Library/LaunchDaemons`
- installs the `com.epsx.port-bridge` LaunchDaemon for `8080` / `4810` / `9180`
- copies the tunnel config to `/etc/cloudflared/config.yml`
- installs or restarts `com.cloudflare.cloudflared`

## Restart after backend changes

After code changes:

```bash
./infrastructure/scripts/build-backend-local.sh
sudo launchctl kickstart -k system/com.epsx.backend.dev
sudo launchctl kickstart -k system/com.epsx.backend.staging
sudo launchctl kickstart -k system/com.epsx.backend.prod
```

Restart the tunnel only when ingress rules or tunnel credentials change:

```bash
sudo launchctl kickstart -k system/com.cloudflare.cloudflared
```

## Verification

Local:

```bash
curl http://127.0.0.1:18080/health
curl http://127.0.0.1:28080/health
curl http://127.0.0.1:38080/health
```

Public:

```bash
curl https://dev-api.epsx.io/health
curl https://staging-api.epsx.io/health
curl https://api.epsx.io/health
```

Launchd state:

```bash
sudo launchctl print system/com.epsx.backend.dev
sudo launchctl print system/com.epsx.backend.staging
sudo launchctl print system/com.epsx.backend.prod
sudo launchctl print system/com.epsx.port-bridge
sudo launchctl print system/com.cloudflare.cloudflared
```
