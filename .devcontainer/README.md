# EPSX Production Docker Dev Container

This dev container setup uses the **same production Docker configurations** for development, ensuring consistency between development and production environments.

## Architecture

### Multi-Service Container Setup

The dev container runs multiple specialized containers based on production Dockerfiles:

- **Frontend Container**: Node.js 22-alpine (same as production)
- **Backend Container**: Rust 1.85-slim (same as production)  
- **Database Container**: PostgreSQL 16-alpine
- **Redis Container**: Redis 7.2-alpine
- **Proxy Container**: Caddy reverse proxy for production domains

### Key Benefits

✅ **Production Parity**: Same base images as production (Node.js 22-alpine, Rust 1.85-slim)  
✅ **Multi-stage Optimization**: Production build patterns in development  
✅ **Container Isolation**: Each service in its own optimized container  
✅ **Security**: Non-root users and production security patterns  
✅ **Performance**: Optimized dependency caching and build layers  

## Quick Start

1. **Open in VS Code Dev Container**
   ```bash
   # VS Code Command Palette -> "Dev Containers: Reopen in Container"
   ```

2. **Start All Services**
   ```bash
   dev-start
   ```

3. **Start Development Servers**
   ```bash
   # Frontend (port 3000)
   pnpm dev:frontend
   
   # Admin (port 3001)  
   pnpm dev:admin
   
   # Backend (port 8080)
   rs-run
   ```

4. **Access Applications**
   - Frontend: https://epsx.io (production domain)
   - Admin: https://admin.epsx.io (production domain)
   - Backend: https://api.epsx.io (production domain)

## Development Commands

### Frontend Development
```bash
pnpm dev                    # Start both frontend and admin
pnpm dev:frontend          # Frontend only
pnpm dev:admin             # Admin only
pnpm build                 # Build all applications
```

### Backend Development
```bash
rs-run                     # Run backend
rs-watch                   # Watch mode with auto-reload
rs-check                   # Type check
rs-test                    # Run tests
rs-fmt                     # Format code
```

### Container Management
```bash
dev-start                  # Start all services
dev-stop                   # Stop all services
dev-rebuild                # Rebuild and restart services
dc-logs                    # View all logs
```

### Container Access
```bash
exec-frontend              # Shell into frontend container
exec-backend               # Shell into backend container
exec-db                    # Shell into database container
```

### Service Logs
```bash
logs-frontend              # Frontend logs
logs-backend               # Backend logs
logs-db                    # Database logs
logs-proxy                 # Proxy logs
```

## Container Details

### Frontend Container (Node.js 22-alpine)
- **Base**: `node:22-alpine` (same as production)
- **User**: `nextjs:nodejs` (same as production)
- **Tools**: pnpm 10.14.0, development tools
- **Ports**: 3000 (frontend), 3001 (admin)
- **Volumes**: Source code + optimized node_modules caching

### Backend Container (Rust 1.85-slim)
- **Base**: `rust:1.85-slim` (same as production) 
- **User**: `nonroot:nonroot` (same as production)
- **Tools**: Rust 1.85, diesel CLI, cargo-watch
- **Port**: 8080
- **Volumes**: Source code + cargo registry/target caching

### Database Container
- **Image**: `postgres:16-alpine`
- **Database**: `epsx_db`
- **User**: `epsx_user` / `epsx_password`
- **Port**: 5432

### Redis Container  
- **Image**: `redis:7.2-alpine`
- **Port**: 6379

## Environment Configuration

### Development Environment Variables
The containers use development-optimized environment variables:

```bash
# Node.js containers
NODE_ENV=development
NEXT_TELEMETRY_DISABLED=1

# Rust container  
RUST_ENV=development
RUST_LOG=debug
RUST_BACKTRACE=1

# Database connections
DATABASE_URL=postgresql://epsx_user:epsx_password@database:5432/epsx_db
REDIS_URL=redis://redis:6379
```

### Production Domain URLs
```bash
# Production domains via proxy
FRONTEND_URL=https://epsx.io
ADMIN_FRONTEND_URL=https://admin.epsx.io
BACKEND_URL=https://api.epsx.io

# Client-side URLs
NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io
NEXT_PUBLIC_APP_URL=https://epsx.io
NEXT_PUBLIC_ADMIN_URL=https://admin.epsx.io
```

## Volume Optimization

### Development Volume Strategy
- **Source Code**: Live-mounted for hot reloading
- **Dependencies**: Cached in named volumes for performance
  - `frontend_node_modules` - Frontend dependencies  
  - `admin_node_modules` - Admin dependencies
  - `cargo_registry` - Rust crate registry
  - `cargo_target` - Rust build artifacts

### Performance Benefits
- **Fast Container Startup**: Pre-cached dependencies
- **Efficient Rebuilds**: Layered dependency caching
- **Hot Reloading**: Live source code updates
- **Isolated Builds**: Container-specific optimizations

## Production Comparison

| Aspect | Development Container | Production Container |
|--------|----------------------|---------------------|
| Base Image | `node:22-alpine` | `node:22-alpine` ✅ |
| User | `nextjs:nodejs` | `nextjs:nodejs` ✅ |
| Dependencies | Volume cached | Multi-stage optimized |
| Environment | Development vars | Production vars |
| Source Code | Live mounted | Built-in optimized |

## Troubleshooting

### Container Issues
```bash
# Check container status
docker compose ps

# View service logs
dc-logs

# Restart specific service  
docker compose restart frontend
docker compose restart backend
```

### Database Issues
```bash
# Check database connection
psql-dev

# Run migrations
db-migrate

# Reset database
db-reset
```

### Build Issues
```bash
# Rebuild all containers
dev-rebuild

# Force rebuild specific service
docker compose up -d --build frontend
docker compose up -d --build backend
```

## File Structure

```
.devcontainer/
├── devcontainer.json          # Main dev container configuration
├── docker-compose.yml         # Multi-service setup
├── Dockerfile                 # Legacy support container
├── Dockerfile.frontend-dev    # Frontend dev container (Node.js 22-alpine)
├── Dockerfile.backend-dev     # Backend dev container (Rust 1.85-slim)
├── Caddyfile                  # Reverse proxy configuration
├── setup.sh                   # Container setup script
├── post-start.sh              # Post-start automation
└── README.md                  # This file
```

## Migration from Legacy Setup

### Before (Single Container)
- Generic Ubuntu base with all tools
- Single container with Node.js + Rust
- Slower startup and builds
- Less production parity

### After (Multi-Service Production Docker)
- Specialized containers per service
- Same base images as production
- Optimized builds and caching  
- High production parity

The new setup provides better isolation, performance, and production consistency while maintaining the same development workflow.