# EPSX Platform - Cloudflare Tunnel Deployment Guide

## Overview

This guide walks you through deploying the EPSX platform using Docker Compose with Cloudflare Tunnel for secure, zero-trust access to your services.

### Architecture

```
Internet → Cloudflare Edge → Cloudflare Tunnel → Docker Network
                                                  ├─ Frontend (port 3000)
                                                  ├─ Admin Frontend (port 3000)
                                                  ├─ Backend (port 8080)
                                                  ├─ PostgreSQL (port 5432)
                                                  └─ Redis (port 6379)
```

### Key Benefits

- ✅ **No Exposed Ports**: Services never exposed directly to the internet
- ✅ **Automatic HTTPS**: Cloudflare manages all SSL/TLS certificates
- ✅ **DDoS Protection**: Built-in Cloudflare protection
- ✅ **Zero Trust Security**: Can add Cloudflare Access policies
- ✅ **Global CDN**: Edge caching for better performance
- ✅ **Web Application Firewall**: Built-in WAF protection

## Prerequisites

### Required

1. **Cloudflare Account** with a domain configured
2. **Docker & Docker Compose** installed on deployment server
3. **Domain in Cloudflare**: `epsx.io` added to Cloudflare DNS
4. **Server Requirements**:
   - 4+ CPU cores
   - 8GB+ RAM
   - 50GB+ disk space
   - Ubuntu 22.04 LTS or similar Linux distribution

### Optional

- Cloudflare Zero Trust account for access policies
- Monitoring tools (Grafana, Prometheus)

## Step 1: Create Cloudflare Tunnel

### Option A: Cloudflare Dashboard (Recommended)

1. Log in to [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. Navigate to **Zero Trust** → **Access** → **Tunnels**
3. Click **Create a tunnel**
4. Name: `epsx-dev`
5. Click **Save tunnel**
6. **Copy the tunnel token** (you'll need this for `.env.docker`)
7. Click **Next**

### Option B: CLI Method

```bash
# Install cloudflared
curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 -o cloudflared
sudo mv cloudflared /usr/local/bin/
sudo chmod +x /usr/local/bin/cloudflared

# Login to Cloudflare
cloudflared tunnel login

# Create tunnel
cloudflared tunnel create epsx-dev

# Get tunnel ID
cloudflared tunnel list
```

## Step 2: Configure DNS Records

Add these CNAME records in Cloudflare DNS:

| Type  | Name              | Target                              | Proxy Status |
|-------|-------------------|-------------------------------------|--------------|
| CNAME | dev.epsx.io       | `<tunnel-id>.cfargotunnel.com`      | Proxied (🟠) |
| CNAME | dev-admin.epsx.io | `<tunnel-id>.cfargotunnel.com`      | Proxied (🟠) |
| CNAME | dev-api.epsx.io   | `<tunnel-id>.cfargotunnel.com`      | Proxied (🟠) |

**Note**: Replace `<tunnel-id>` with your actual tunnel ID from Step 1.

### DNS Configuration Steps

1. Go to **Cloudflare Dashboard** → Select your domain
2. Click **DNS** in the left sidebar
3. Click **Add record**
4. For each subdomain:
   - Type: `CNAME`
   - Name: `dev` (or `dev-admin`, `dev-api`)
   - Target: `<tunnel-id>.cfargotunnel.com`
   - Proxy status: **Proxied** (orange cloud icon)
   - TTL: Auto
5. Click **Save**

## Step 3: Configure Environment Variables

1. Copy the example environment file:
   ```bash
   cd /path/to/epsx
   cp .env.docker.example .env.docker
   ```

2. Edit `.env.docker` with your values:
   ```bash
   nano .env.docker
   ```

3. **Required Configuration**:

   ```bash
   # Cloudflare Tunnel Token (from Step 1)
   CLOUDFLARE_TUNNEL_TOKEN=your-tunnel-token-here

   # Database Credentials (change these!)
   DB_PASSWORD=create-secure-password-here
   REDIS_PASSWORD=create-secure-redis-password

   # Web3 Secrets (minimum 32 characters each)
   WEB3_APP_SECRET=generate-random-32-char-secret-here
   WALLET_SIGNATURE_SECRET=generate-random-32-char-secret-here
   WEB3_SESSION_SECRET=generate-random-32-char-secret-here
   JWT_SECRET=generate-random-32-char-jwt-secret-here
   ```

4. **Generate Secure Secrets**:
   ```bash
   # Generate random 32-character secrets
   openssl rand -base64 32
   ```

## Step 4: Deploy Services

### Build and Start All Services

```bash
# Build images
docker-compose --env-file .env.docker build

# Start all services
docker-compose --env-file .env.docker up -d

# View logs
docker-compose logs -f
```

### Verify Service Health

```bash
# Check all containers are running
docker-compose ps

# Check individual service logs
docker-compose logs backend
docker-compose logs frontend
docker-compose logs admin-frontend
docker-compose logs cloudflared

# Check service health
docker-compose ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}"
```

### Expected Output

```
NAME                STATUS              PORTS
epsx-backend        Up (healthy)
epsx-frontend       Up (healthy)
epsx-admin          Up (healthy)
epsx-postgres       Up (healthy)        0.0.0.0:5432->5432/tcp
epsx-redis          Up (healthy)        0.0.0.0:6379->6379/tcp
epsx-cloudflared    Up
```

## Step 5: Verify Deployment

### Test Public Endpoints

```bash
# Test frontend
curl -I https://dev.epsx.io

# Test admin frontend
curl -I https://dev-admin.epsx.io

# Test backend API
curl https://dev-api.epsx.io/health
```

### Expected Responses

All endpoints should return:
- Status: `200 OK`
- HTTPS enabled automatically
- Cloudflare headers present

### Test Web3 Authentication

1. Open browser to `https://dev.epsx.io`
2. Connect MetaMask or Web3 wallet
3. Sign authentication message
4. Verify successful login

## Step 6: Database Migrations

### Run Backend Migrations

```bash
# Enter backend container
docker-compose exec backend bash

# Run migrations
./epsx migrate up

# Verify migrations
./epsx migrate status

# Exit container
exit
```

### Alternative: Run Migrations Directly

```bash
# Run migrations without entering container
docker-compose exec backend ./epsx migrate up
```

## Maintenance Commands

### View Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f backend
docker-compose logs -f frontend
docker-compose logs -f cloudflared

# Last 100 lines
docker-compose logs --tail=100 backend
```

### Restart Services

```bash
# Restart all services
docker-compose restart

# Restart specific service
docker-compose restart backend
docker-compose restart frontend
```

### Update Services

```bash
# Pull latest changes from git
git pull origin development

# Rebuild and restart
docker-compose --env-file .env.docker build
docker-compose --env-file .env.docker up -d
```

### Database Backup

```bash
# Backup PostgreSQL
docker-compose exec postgres pg_dump -U epsx_user epsx_dev > backup.sql

# Restore PostgreSQL
docker-compose exec -T postgres psql -U epsx_user epsx_dev < backup.sql
```

### Redis Backup

```bash
# Backup Redis
docker-compose exec redis redis-cli --rdb /data/dump.rdb SAVE
docker cp epsx-redis:/data/dump.rdb ./redis-backup.rdb

# Restore Redis
docker cp ./redis-backup.rdb epsx-redis:/data/dump.rdb
docker-compose restart redis
```

## Troubleshooting

### Issue: Tunnel Not Connecting

**Symptoms**: cloudflared container shows connection errors

**Solution**:
```bash
# Check cloudflared logs
docker-compose logs cloudflared

# Verify tunnel token is correct
grep CLOUDFLARE_TUNNEL_TOKEN .env.docker

# Restart cloudflared
docker-compose restart cloudflared
```

### Issue: Services Not Healthy

**Symptoms**: Health checks failing, containers restarting

**Solution**:
```bash
# Check service logs
docker-compose logs backend

# Verify database connection
docker-compose exec backend env | grep DATABASE_URL

# Check database is accessible
docker-compose exec backend nc -zv postgres 5432
```

### Issue: DNS Not Resolving

**Symptoms**: Domains not resolving to tunnel

**Solution**:
1. Verify DNS records in Cloudflare Dashboard
2. Check CNAME targets match tunnel ID
3. Ensure proxy status is **Proxied** (orange cloud)
4. Wait up to 5 minutes for DNS propagation

### Issue: Database Connection Errors

**Symptoms**: Backend cannot connect to PostgreSQL

**Solution**:
```bash
# Verify PostgreSQL is healthy
docker-compose ps postgres

# Check PostgreSQL logs
docker-compose logs postgres

# Test connection manually
docker-compose exec postgres psql -U epsx_user -d epsx_dev -c "SELECT 1;"
```

### Issue: Out of Memory

**Symptoms**: Containers killed due to OOM

**Solution**:
```bash
# Check container memory usage
docker stats

# Add resource limits to docker-compose.yml
# Example:
services:
  backend:
    deploy:
      resources:
        limits:
          memory: 2G
        reservations:
          memory: 1G
```

## Security Considerations

### Cloudflare Access Policies

Add authentication layer before services:

1. Navigate to **Zero Trust** → **Access** → **Applications**
2. Click **Add an application**
3. Select **Self-hosted**
4. Configure:
   - Application name: `EPSX Dev Environment`
   - Session duration: `24 hours`
   - Application domain: `dev.epsx.io`, `dev-admin.epsx.io`, `dev-api.epsx.io`
5. Add policies:
   - **Allow**: Emails ending in `@yourcompany.com`
   - **Allow**: Specific email addresses

### Web Application Firewall (WAF)

Enable WAF rules in Cloudflare:

1. Navigate to **Security** → **WAF**
2. Enable **OWASP Core Ruleset**
3. Enable **Cloudflare Managed Ruleset**
4. Create custom rules as needed

### Rate Limiting

Configure rate limiting:

1. Navigate to **Security** → **WAF** → **Rate limiting rules**
2. Create rule:
   - Name: `API Rate Limit`
   - Match: `Hostname equals dev-api.epsx.io`
   - Requests: `100 per minute`
   - Action: `Challenge` or `Block`

## Monitoring

### Container Health

```bash
# Check container health
docker-compose ps

# Watch container stats
docker stats

# Healthcheck status
docker inspect epsx-backend | jq '.[0].State.Health'
```

### Cloudflare Analytics

1. Navigate to **Analytics** → **Traffic** in Cloudflare Dashboard
2. Filter by hostname: `dev.epsx.io`, `dev-admin.epsx.io`, `dev-api.epsx.io`
3. Monitor:
   - Requests per second
   - Bandwidth usage
   - Response codes
   - Threat detection

### Application Logs

```bash
# View real-time logs
docker-compose logs -f --tail=100

# Export logs for analysis
docker-compose logs > logs.txt
```

## Performance Optimization

### Enable Cloudflare Caching

Configure caching rules:

1. Navigate to **Caching** → **Configuration**
2. Enable **Auto Minify**: JS, CSS, HTML
3. Enable **Brotli** compression
4. Create page rules:
   - `dev.epsx.io/static/*` → Cache Level: Cache Everything
   - `dev-admin.epsx.io/_next/static/*` → Cache Level: Cache Everything

### Database Connection Pooling

Already configured in `.env.docker`:
```bash
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=2
DATABASE_ACQUIRE_TIMEOUT=30
DATABASE_IDLE_TIMEOUT=300
```

Adjust based on load:
- **Low traffic**: 5-10 max connections
- **Medium traffic**: 10-20 max connections
- **High traffic**: 20-50 max connections

## Scaling

### Horizontal Scaling

To run multiple instances of services:

```yaml
# In docker-compose.yml
services:
  backend:
    deploy:
      replicas: 3
```

### Load Balancing

Cloudflare automatically load balances traffic across tunnel instances.

## Backup Strategy

### Automated Backups

Create backup script `scripts/backup.sh`:

```bash
#!/bin/bash
BACKUP_DIR=/backups/$(date +%Y%m%d_%H%M%S)
mkdir -p $BACKUP_DIR

# Backup PostgreSQL
docker-compose exec -T postgres pg_dump -U epsx_user epsx_dev > $BACKUP_DIR/postgres.sql

# Backup Redis
docker-compose exec redis redis-cli --rdb /data/dump.rdb SAVE
docker cp epsx-redis:/data/dump.rdb $BACKUP_DIR/redis.rdb

# Backup environment config
cp .env.docker $BACKUP_DIR/.env.docker

echo "Backup completed: $BACKUP_DIR"
```

### Cron Schedule

```bash
# Add to crontab
crontab -e

# Daily backup at 2 AM
0 2 * * * /path/to/epsx/scripts/backup.sh
```

## Support

### Getting Help

- **GitHub Issues**: https://github.com/your-org/epsx/issues
- **Documentation**: https://docs.epsx.io
- **Cloudflare Support**: https://support.cloudflare.com/

### Useful Links

- [Cloudflare Tunnel Documentation](https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [EPSX Platform Documentation](../README.md)

---

**Last Updated**: 2025-10-20
**Version**: 1.0.0
