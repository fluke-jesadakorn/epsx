# EPSX Docker + Traefik Development Setup

This guide shows you how to develop EPSX using Docker Compose with Traefik reverse proxy for clean domain URLs.

## Overview

Your EPSX development environment uses:
- **Docker Compose** - Containerized services with hot reload
- **Traefik** - Reverse proxy with automatic SSL and domain routing
- **Clean URLs** - https://epsx.io, https://admin.epsx.io, https://api.epsx.io
- **Hot Reload** - All services rebuild automatically on file changes

## Architecture

```
Host Machine (/etc/hosts override)
↓
https://epsx.io → localhost:80
↓  
Docker Compose + Traefik
├── Traefik (ports 80, 443, 8080)
│   ├── epsx.io → frontend:3000
│   ├── admin.epsx.io → admin:3000  
│   └── api.epsx.io → backend:8080
├── Frontend (Next.js with hot reload)
├── Admin (Next.js with hot reload)
├── Backend (Rust with hot reload)
├── PostgreSQL (persistent data)
└── Redis (cache)
```

## Quick Start

### Prerequisites
```bash
# Install Docker Desktop, OrbStack, or Podman
# Install Homebrew (for mkcert)
```

### Start Development

```bash
# One command setup + start
pnpm dev

# This will:
# 1. Setup domain override (/etc/hosts)
# 2. Generate SSL certificates
# 3. Start all Docker services
# 4. Make apps available at https://epsx.io, etc.
```

### Access Applications
- **Frontend**: https://epsx.io
- **Admin**: https://admin.epsx.io
- **API**: https://api.epsx.io
- **Traefik Dashboard**: http://localhost:8080
- **Database**: localhost:5432
- **Redis**: localhost:6379

## Available Commands

### Primary Development
```bash
# Setup domains + start services (recommended)
pnpm dev

# Just start services (domains already setup)
pnpm dev:up

# Stop services
pnpm dev:down

# View logs
pnpm dev:logs

# Clean stop + restore DNS + optional cleanup
pnpm dev:clean
```

### Alternative Development
```bash
# Use localhost:port URLs (fallback)
pnpm dev:localhost

# DevContainer mode (optional)
pnpm dev:container
```

## How It Works

### 1. Domain Override Setup
The `setup-domains.sh` script:
- Backs up your `/etc/hosts` file
- Adds domain mappings pointing to `127.0.0.1`
- Installs mkcert and generates SSL certificates
- Creates Traefik configuration for HTTPS

### 2. Docker Compose Services
- **Traefik**: Routes domains to correct containers with SSL
- **Frontend**: Next.js app with volume mount for hot reload
- **Admin**: Next.js admin app with volume mount for hot reload
- **Backend**: Rust API with volume mount for hot reload
- **Database**: PostgreSQL with persistent data volume
- **Redis**: Cache with persistent data volume

### 3. Environment Variables
All services use the epsx.io domains:
- `NEXTAUTH_URL=https://epsx.io`
- `BACKEND_URL=https://api.epsx.io`
- `OIDC_ISSUER=https://api.epsx.io`

## Development Workflow

### Daily Development
```bash
# Start everything
pnpm dev

# Make code changes (hot reload works automatically)
# Access: https://epsx.io, https://admin.epsx.io, https://api.epsx.io

# When done
pnpm dev:down
```

### Full Cleanup (occasionally)
```bash
# Stop + restore DNS + clean Docker volumes
pnpm dev:clean
```

## File Structure

### Docker Configuration
```
docker/
├── traefik/
│   ├── certs/           # SSL certificates
│   │   ├── epsx.crt
│   │   └── epsx.key
│   └── dynamic/         # Traefik dynamic config
│       └── ssl.yml      # SSL certificate configuration
```

### Scripts
```
scripts/
├── setup-domains.sh    # Setup domains + SSL
└── cleanup-domains.sh  # Cleanup domains + Docker
```

### Key Files
- `docker-compose.yml` - Main Docker services definition
- `.env` files - Environment variables for each service
- `scripts/setup-domains.sh` - Domain and SSL setup
- `scripts/cleanup-domains.sh` - Complete cleanup

## Troubleshooting

### Domain Not Resolving
```bash
# Check hosts file
cat /etc/hosts | grep epsx

# Test DNS resolution
ping epsx.io  # Should resolve to 127.0.0.1

# Flush DNS cache
sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder
```

### SSL Certificate Issues
```bash
# Reinstall mkcert
mkcert -uninstall && mkcert -install

# Regenerate certificates
rm -rf docker/traefik/certs/
./scripts/setup-domains.sh
```

### Docker Issues
```bash
# View container status
docker-compose ps

# View logs
docker-compose logs -f [service]

# Restart specific service
docker-compose restart [service]

# Rebuild containers
docker-compose up --build
```

### Port Conflicts
```bash
# Find what's using ports 80/443
lsof -i :80
lsof -i :443

# Stop conflicting services
sudo brew services stop nginx  # Example
```

### Hot Reload Not Working
```bash
# Check volume mounts
docker-compose ps

# Restart containers with build
pnpm dev:up
```

## Performance Tips

### Use OrbStack (Recommended)
```bash
# 15x faster than Docker Desktop on Apple Silicon
# Install from: https://orbstack.dev
```

### Clean Up Regularly
```bash
# Remove unused containers/images
docker system prune

# Clean up EPSX volumes
pnpm dev:clean  # (select 'y' to remove volumes)
```

## Team Setup

### New Team Member Setup
1. **Clone repository**
2. **Install Docker** (preferably OrbStack)
3. **Start development**: `pnpm dev`
4. **Access applications**: https://epsx.io, https://admin.epsx.io, https://api.epsx.io

### Switching Between Projects
Each EPSX project instance gets its own domain override, so multiple projects can coexist.

## Benefits

### ✅ Advantages
- **Single Command** - `pnpm dev` starts everything
- **Clean URLs** - Production-like domain structure
- **HTTPS Everywhere** - SSL certificates with trusted CA
- **Hot Reload** - All services with live reloading
- **Team Consistency** - Same Docker setup for everyone
- **Easy Cleanup** - Complete environment reset with one command
- **Port Management** - No port conflicts, everything routes through 80/443

### ⚠️ Considerations
- **Domain Override** - Affects entire machine while active
- **Docker Dependency** - Requires Docker to be running
- **Initial Setup** - First run sets up domains and certificates

## Migration from Other Setups

### From localhost:port Development
- Your existing `.env` files already use epsx.io domains
- Just run `pnpm dev` instead of `pnpm dev:localhost`

### From DevContainer
- DevContainer setup is still available with `pnpm dev:container`
- Main development now uses `pnpm dev` for Docker + Traefik

## Quick Reference

### Essential Commands
- `pnpm dev` - Setup + start (daily command)
- `pnpm dev:up` - Start without domain setup
- `pnpm dev:down` - Stop services
- `pnpm dev:clean` - Complete cleanup
- `pnpm dev:logs` - View logs

### URLs
- Frontend: https://epsx.io
- Admin: https://admin.epsx.io
- API: https://api.epsx.io
- Traefik Dashboard: http://localhost:8080

### Cleanup Checklist
- [ ] Stop containers: `pnpm dev:down`
- [ ] Restore DNS: `pnpm dev:clean`
- [ ] Remove volumes: Select 'y' when prompted
- [ ] Remove SSL certs: Select 'y' when prompted