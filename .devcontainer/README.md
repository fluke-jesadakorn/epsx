# EPSX Development Container

This development container provides a production-like environment with real domains for local development.

## Features

- **Real Domains**: Access applications via `epsx.io`, `admin.epsx.io`, and `api.epsx.io`
- **No Ports**: Clean URLs without port numbers
- **Traefik Proxy**: Automatic reverse proxy and load balancing
- **Isolated Environment**: Complete development stack in containers
- **VS Code Integration**: Full IDE support with extensions

## Quick Start

### Using VS Code Dev Containers

1. Install the "Dev Containers" extension in VS Code
2. Open the project in VS Code
3. Run **"Dev Containers: Reopen in Container"** from Command Palette
4. Wait for container to build and start

### Using Docker Compose Directly

```bash
# Start development environment
pnpm dev:container

# Stop development environment  
pnpm dev:container:down

# View logs
docker-compose -f .devcontainer/docker-compose.dev.yml logs -f
```

## Access Applications

| Service | URL | Description |
|---------|-----|-------------|
| Frontend | http://epsx.io | Main trading platform |
| Admin | http://admin.epsx.io | Administrative dashboard |
| Backend API | http://api.epsx.io | REST API server |
| Traefik Dashboard | http://localhost:8090 | Proxy configuration |

## Development Workflow

1. **Start Environment**: `pnpm dev:container`
2. **Access Applications**: Use real domain URLs above
3. **Make Changes**: Edit code normally, hot reload works
4. **View Logs**: `docker-compose logs -f [service_name]`
5. **Stop Environment**: `pnpm dev:container:down`

## Architecture

```
┌─────────────────┐    ┌──────────────┐
│   epsx.io       │────│   Frontend   │
│                 │    │   (port 3000)│
├─────────────────┤    ├──────────────┤
│ admin.epsx.io   │────│   Admin      │
│                 │    │   (port 3000)│
├─────────────────┤    ├──────────────┤
│  api.epsx.io    │────│   Backend    │
│                 │    │   (port 8080)│
└─────────────────┘    └──────────────┘
       │
   ┌───▼────┐
   │ Traefik│ (port 80)
   │ Proxy  │
   └────────┘
```

## Environment Variables

Each application has its own `.env` file with consolidated variables:

- `apps/frontend/.env` - Frontend configuration
- `apps/admin-frontend/.env` - Admin configuration  
- `apps/backend/.env` - Backend configuration

No shared environment files are used.

## Troubleshooting

### Container Issues

```bash
# Rebuild containers
pnpm dev:container:down
docker-compose -f .devcontainer/docker-compose.dev.yml build --no-cache
pnpm dev:container

# View specific service logs
docker-compose -f .devcontainer/docker-compose.dev.yml logs frontend
```

### Domain Resolution

If domains don't work:

1. Check Traefik dashboard: http://localhost:8090
2. Verify services are healthy: `docker-compose ps`
3. Check container networking: `docker network ls`

### Performance

- Use named volumes for `node_modules` to improve performance
- Enable BuildKit for faster Docker builds
- Use `--parallel` flag for concurrent builds

## Benefits

- **Production Parity**: Same domain structure as production
- **CORS Resolved**: No cross-origin issues between apps
- **Team Consistency**: Same environment for all developers
- **Easy Testing**: Test authentication flows with real domains
- **Clean Development**: No system-level changes required