# EPSX Legacy Local Archive — 2026-08-30

This archive contains pre-Cloudflare local runtime artefacts moved from the working tree for Full D1/R2/KV local simulation.

Contents:
- infrastructure/kubernetes/ (base + overlays/dev|prod|staging) — 362-line kustomize, NodePorts 30000/30001/30080, hostAliases 192.168.5.1
- infrastructure/docker/docker-compose.*.yml — prod/dev/staging/local with host.docker.internal PG/Redis/MinIO
- infrastructure/docker/cloudflared-proxy/ — socks/redsocks helper
- infrastructure/scripts/com.epsx.*.plist — LaunchAgents (minio :9100, colima-start, port-bridge 8080/4810/9180, pay-port-bridge 4747-4752)

Rollback:
```bash
cp -R _archive/legacy-local-2026-08-30/infrastructure/kubernetes infrastructure/
cp _archive/legacy-local-2026-08-30/infrastructure/docker/docker-compose.*.yml infrastructure/docker/
cp _archive/legacy-local-2026-08-30/infrastructure/scripts/com.epsx.*.plist infrastructure/scripts/
cp _archive/legacy-local-2026-08-30/infrastructure/scripts/com.epsx.*.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.epsx.minio.plist
launchctl load ~/Library/LaunchAgents/com.epsx.colima-start.plist
launchctl load ~/Library/LaunchAgents/com.epsx.port-bridge.plist
launchctl load ~/Library/LaunchAgents/com.epsx.pay-port-bridge.plist
kubectl apply -k infrastructure/kubernetes/overlays/prod
```

Data migrated to Cloudflare local:
- PG dumps: .wrangler/migration/dumps/*.sql (epsx_* via pg_dump --data-only + postgres superuser for restricted DBs)
- R2 buckets: .wrangler/migration/r2/{chat,public,news,notifications,epsx-contracts} (SKIP erpx-backups)
- KV: .wrangler/migration/kv/kv.json (4 keys from redis 6379)

Preserved for other projects:
- brew postgresql@14 / redis still `started`
- DB `plf_webapp_dev` + `/Users/fluke/epsx-minio-data/erpx-backups` untouched
