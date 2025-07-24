# EPSX Deployment Guide

This guide covers deploying the EPSX trading platform to production environments.

## Overview

The EPSX platform consists of:
- **Backend**: Rust API server with PostgreSQL database
- **Frontend**: Next.js SSR application
- **Admin Frontend**: React admin dashboard
- **Database**: PostgreSQL with automated migrations
- **Reverse Proxy**: Nginx for SSL termination and load balancing

## Prerequisites

- Docker and Docker Compose installed
- Domain name configured with DNS
- SSL certificates (Let's Encrypt recommended)
- Environment variables configured

## Quick Start

1. **Clone and prepare the repository:**
```bash
git clone <repository-url>
cd epsx
cp .env.prod.example .env.prod
# Edit .env.prod with your configuration
```

2. **Deploy to production:**
```bash
docker-compose -f docker-compose.prod.yml --env-file .env.prod up -d
```

3. **Verify deployment:**
```bash
# Check all services are running
docker-compose -f docker-compose.prod.yml ps

# Check logs
docker-compose -f docker-compose.prod.yml logs -f backend-prod
```

## Detailed Setup

### 1. Environment Configuration

Copy the example environment file and configure:

```bash
cp .env.prod.example .env.prod
```

Key configurations:

- **Database**: Set strong `POSTGRES_PASSWORD`
- **API Keys**: Configure `SENDGRID_API_KEY` for emails
- **Domains**: Set `NEXT_PUBLIC_API_URL` and `NEXTAUTH_URL`
- **Security**: Generate strong `NEXTAUTH_SECRET` and `SESSION_SECRET`

### 2. SSL Certificate Setup

#### Option A: Let's Encrypt (Recommended)

```bash
# Install certbot
sudo apt install certbot

# Generate certificates
sudo certbot certonly --standalone -d yourdomain.com -d api.yourdomain.com

# Copy certificates
sudo mkdir -p ssl
sudo cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem ssl/cert.pem
sudo cp /etc/letsencrypt/live/yourdomain.com/privkey.pem ssl/key.pem
```

#### Option B: Self-signed (Development only)

```bash
mkdir -p ssl
openssl req -x509 -newkey rsa:4096 -keyout ssl/key.pem -out ssl/cert.pem -days 365 -nodes
```

### 3. Nginx Configuration

Create `nginx.conf`:

```nginx
events {
    worker_connections 1024;
}

http {
    upstream backend {
        server backend-prod:8080;
    }
    
    upstream frontend {
        server frontend-prod:3000;
    }
    
    # Redirect HTTP to HTTPS
    server {
        listen 80;
        server_name yourdomain.com api.yourdomain.com;
        return 301 https://$server_name$request_uri;
    }
    
    # Main application
    server {
        listen 443 ssl http2;
        server_name yourdomain.com;
        
        ssl_certificate /etc/nginx/ssl/cert.pem;
        ssl_certificate_key /etc/nginx/ssl/key.pem;
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers HIGH:!aNULL:!MD5;
        
        location / {
            proxy_pass http://frontend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
    
    # API endpoints
    server {
        listen 443 ssl http2;
        server_name api.yourdomain.com;
        
        ssl_certificate /etc/nginx/ssl/cert.pem;
        ssl_certificate_key /etc/nginx/ssl/key.pem;
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers HIGH:!aNULL:!MD5;
        
        location / {
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
```

### 4. Database Initialization

The backend automatically runs migrations on startup, but you can also run them manually:

```bash
# Run migrations manually
docker-compose -f docker-compose.prod.yml exec backend-prod ./migrate up

# Check migration status
docker-compose -f docker-compose.prod.yml exec backend-prod ./migrate status
```

### 5. Monitoring and Logging

#### View logs:
```bash
# All services
docker-compose -f docker-compose.prod.yml logs -f

# Specific service
docker-compose -f docker-compose.prod.yml logs -f backend-prod
docker-compose -f docker-compose.prod.yml logs -f frontend-prod
```

#### Health checks:
```bash
# Check backend health
curl https://api.yourdomain.com/health

# Check frontend health
curl https://yourdomain.com/api/health
```

## CI/CD Integration

### GitHub Actions

The repository includes a CI/CD pipeline (`.github/workflows/ci.yml`) that:

1. **Tests**: Runs backend and frontend tests
2. **Security**: Scans for vulnerabilities
3. **Build**: Creates Docker images
4. **Deploy**: Deploys to staging/production

### Manual Deployment

```bash
# Pull latest images
docker-compose -f docker-compose.prod.yml pull

# Update services with zero downtime
docker-compose -f docker-compose.prod.yml up -d --no-deps backend-prod
docker-compose -f docker-compose.prod.yml up -d --no-deps frontend-prod
```

## Backup and Recovery

### Database Backup

```bash
# Create backup
docker-compose -f docker-compose.prod.yml exec postgres-prod pg_dump -U epsx epsx > backup_$(date +%Y%m%d_%H%M%S).sql

# Restore backup
docker-compose -f docker-compose.prod.yml exec -T postgres-prod psql -U epsx epsx < backup_file.sql
```

### Automated Backups

Add to crontab:
```bash
# Daily backup at 2 AM
0 2 * * * cd /path/to/epsx && docker-compose -f docker-compose.prod.yml exec postgres-prod pg_dump -U epsx epsx > /backups/epsx_$(date +\%Y\%m\%d).sql
```

## Security Considerations

### 1. Environment Variables
- Never commit `.env.prod` to version control
- Use strong, unique passwords for all services
- Rotate secrets regularly

### 2. Network Security
- Use firewall to restrict access to ports
- Only expose necessary ports (80, 443)
- Consider using a VPN for administrative access

### 3. Database Security
- Use strong passwords
- Enable connection encryption
- Regular security updates

### 4. SSL/TLS
- Use strong cipher suites
- Enable HSTS headers
- Regular certificate renewal

## Scaling

### Horizontal Scaling

```yaml
# docker-compose.prod.yml
services:
  backend-prod:
    deploy:
      replicas: 3
  
  nginx:
    # Configure load balancing
```

### Database Scaling
- Read replicas for read-heavy workloads
- Connection pooling with PgBouncer
- Database partitioning for large datasets

## Troubleshooting

### Common Issues

1. **Service won't start:**
```bash
docker-compose -f docker-compose.prod.yml logs service-name
```

2. **Database connection errors:**
- Check `DATABASE_URL` format
- Verify PostgreSQL service is healthy
- Check network connectivity

3. **SSL certificate issues:**
- Verify certificate files exist and are readable
- Check certificate expiration
- Validate certificate chain

4. **Migration failures:**
```bash
# Check migration status
docker-compose -f docker-compose.prod.yml exec backend-prod ./migrate status

# Manual migration
docker-compose -f docker-compose.prod.yml exec backend-prod ./migrate up
```

### Performance Issues

1. **High memory usage:**
- Monitor with `docker stats`
- Adjust memory limits in compose file
- Check for memory leaks in logs

2. **Slow database queries:**
- Enable query logging in PostgreSQL
- Use `EXPLAIN ANALYZE` for slow queries
- Consider adding indexes

3. **High CPU usage:**
- Scale horizontally
- Optimize code hot paths
- Use profiling tools

## Maintenance

### Regular Tasks

1. **Weekly:**
   - Review logs for errors
   - Check disk space
   - Monitor performance metrics

2. **Monthly:**
   - Update Docker images
   - Review security logs
   - Database maintenance (VACUUM, ANALYZE)

3. **Quarterly:**
   - Security audit
   - Backup restore testing
   - Disaster recovery testing

### Updates

```bash
# Update to new version
git pull origin main
docker-compose -f docker-compose.prod.yml pull
docker-compose -f docker-compose.prod.yml up -d
```

## Support

For deployment issues:
1. Check logs first
2. Review this documentation
3. Check GitHub issues
4. Contact the development team

## Appendix

### Environment Variables Reference

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| `POSTGRES_PASSWORD` | Database password | Yes | - |
| `SENDGRID_API_KEY` | Email service API key | No | - |
| `NEXTAUTH_SECRET` | Authentication secret | Yes | - |
| `RUST_LOG` | Backend log level | No | info |
| `USE_MOCK_EMAIL` | Use mock email service | No | false |

### Port Reference

| Service | Internal Port | External Port | Protocol |
|---------|---------------|---------------|----------|
| Frontend | 3000 | 80/443 | HTTP/HTTPS |
| Backend | 8080 | 8080 | HTTP |
| PostgreSQL | 5432 | 5432 | TCP |
| Admin | 3000 | 3001 | HTTP |

### Health Check Endpoints

- Backend: `https://api.yourdomain.com/health`
- Frontend: `https://yourdomain.com/api/health`
- Database: PostgreSQL `pg_isready` command