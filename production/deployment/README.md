# 🚀 EPSX Production Deployment

Organized deployment structure for EPSX platform with local build + direct Cloud Run deployment.

## 📁 Directory Structure

```
production/deployment/
├── scripts/                  # Deployment scripts
│   ├── deploy-dev.sh         # Development deployment
│   ├── deploy-staging.sh     # Staging deployment  
│   ├── deploy-prod.sh        # Production deployment
│   └── deploy-service.sh     # Individual service deployment
├── environments/             # Environment configurations
│   ├── development.env.example
│   ├── staging.env.example
│   ├── production.env       # Your existing production config
│   └── staging.env          # Your existing staging config
├── configs/                 # Deployment configurations
│   └── deployment.json      # Centralized deployment config
└── docs/                    # Documentation
    └── DEPLOYMENT.md        # Detailed deployment guide
```

## 🎯 Quick Commands

### Environment Setup
```bash
# Development
cp production/deployment/environments/development.env.example .env.development
# Edit with your values

# Staging
cp production/deployment/environments/staging.env.example .env.staging
# Edit with your values
```

### Deployment Commands
```bash
# Deploy all services
pnpm deploy:dev        # Development environment
pnpm deploy:staging    # Staging environment
pnpm deploy:prod       # Production environment

# Deploy individual services
ENV=development pnpm deploy:backend
ENV=staging pnpm deploy:frontend  
ENV=production pnpm deploy:admin
```

## 🌍 Environment URLs

### Development
- Backend: `https://dev-api.epsx.io`
- Frontend: `https://dev.epsx.io`
- Admin: `https://dev-admin.epsx.io`

### Staging
- Backend: `https://staging-api.epsx.io`
- Frontend: `https://staging.epsx.io`
- Admin: `https://staging-admin.epsx.io`

### Production (Unchanged)
- Backend: `https://api.epsx.io` ✅
- Frontend: `https://epsx.io` ✅
- Admin: `https://admin.epsx.io` ✅

## 📊 Resource Allocation

### Development (Cost-Optimized)
- **Backend**: 1 CPU, 1Gi RAM, 0-3 instances
- **Frontend**: 0.5 CPU, 512Mi RAM, 0-3 instances
- **Admin**: 0.5 CPU, 512Mi RAM, 0-3 instances

### Staging (Production-Like)
- **Backend**: 2 CPU, 2Gi RAM, 1-5 instances
- **Frontend**: 1 CPU, 1Gi RAM, 1-5 instances
- **Admin**: 1 CPU, 1Gi RAM, 1-3 instances

### Production (Current Settings)
- **Backend**: 4 CPU, 4Gi RAM, 1-10 instances
- **Frontend**: 2 CPU, 2Gi RAM, 1-10 instances
- **Admin**: 2 CPU, 2Gi RAM, 1-10 instances

## ✨ Key Features

- 🏠 **Local Build + Deploy**: Full control from your machine
- ⚡ **Turborepo Integration**: Only builds changed services
- 🎯 **Environment Specific**: Optimized resources per environment
- 🔧 **Uses Existing Setup**: Leverages your Dockerfiles and env schema
- 💰 **Cost Effective**: No external CI/CD costs
- 📊 **Production Ready**: Maintains your current production settings

## 🔧 Prerequisites

1. **Google Cloud CLI** installed and authenticated
2. **Docker** installed and running
3. **Environment variables** configured
4. **Project access** to Google Cloud projects

## 📝 Configuration

All deployment settings are centralized in `configs/deployment.json`:
- Environment-specific resource allocation
- Service configurations
- URL mappings
- Required environment variables

See `docs/DEPLOYMENT.md` for detailed setup instructions.