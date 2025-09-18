# EPSX Scripts Directory

This directory contains all deployment, testing, and utility scripts for the EPSX platform. All scripts use environment-driven configuration for flexible deployment across development, staging, and production.

## 📁 Directory Structure

```
scripts/
├── deploy/          # Environment-driven deployment system
│   ├── deploy.sh           # Deploy all services
│   ├── deploy-service.sh   # Deploy individual service
│   └── services/           # YAML templates with env variables
│       ├── production/     # Production configurations
│       ├── staging/        # Staging configurations
│       └── development/    # Development configurations
├── test/            # Testing and validation scripts
│   ├── notification-e2e.sh
│   └── verify-asset-loading.js
├── utils/           # Utility and maintenance scripts
│   ├── validate-env.js     # Environment validation
│   ├── validate-permissions.js
│   ├── promote-admin.sh
│   └── revoke-admin.sh
└── README.md        # This documentation
```

## 🚀 Quick Start

### Prerequisites

1. **Environment Files**: Configure `.env` files for each environment
2. **Google Cloud CLI**: `gcloud` installed and authenticated
3. **Docker Images**: Pre-built images in artifact registry
4. **Node.js**: For validation scripts

### Basic Workflow

```bash
# 1. Validate environment
./scripts/utils/validate-env.js

# 2. Deploy all services to production
./scripts/deploy/deploy.sh production

# 3. Deploy individual service
./scripts/deploy/deploy-service.sh backend production
```

## 🌍 Environment-Driven Configuration

### Key Features

- **No Hardcoding**: All YAML files use environment variables (`${VAR_NAME}`)
- **Environment Files**: Load configuration from `.env.production`, `.env.staging`, etc.
- **Template Processing**: Scripts substitute variables before deployment
- **Flexible Deployment**: Same templates work across all environments

### Environment Files

```bash
# Production
production/deployment/environments/production.env

# Staging  
.env.staging

# Development
.env.development
```

### YAML Template Example

**Before (Hardcoded):**
```yaml
env:
- name: DATABASE_URL
  value: "postgres://hardcoded-connection"
```

**After (Environment-Driven):**
```yaml
env:
- name: DATABASE_URL
  value: "${DATABASE_URL}"
```

## 🚀 Deployment Scripts (`/deploy/`)

### Available Scripts

| Script | Description | Usage |
|--------|-------------|-------|
| `deploy.sh` | Deploy all services | `./scripts/deploy/deploy.sh production` |
| `deploy-service.sh` | Deploy individual service | `./scripts/deploy/deploy-service.sh backend production` |

### Usage

#### Deploy All Services
```bash
# Deploy all services to production
./scripts/deploy/deploy.sh production

# Deploy all services to staging
./scripts/deploy/deploy.sh staging

# Deploy all services to development
./scripts/deploy/deploy.sh development
```

#### Deploy Individual Services
```bash
# Deploy backend to production
./scripts/deploy/deploy-service.sh backend production

# Deploy frontend to staging
./scripts/deploy/deploy-service.sh frontend staging

# Deploy admin to development
./scripts/deploy/deploy-service.sh admin development
```

### Deployment Process

1. **Load Environment**: Reads variables from appropriate `.env` file
2. **Process Templates**: Substitutes `${VAR_NAME}` placeholders in YAML
3. **Deploy Service**: Uses `gcloud run services replace` with processed YAML
4. **Health Check**: Validates deployment success
5. **Cleanup**: Removes temporary processed YAML files

### Environment Configuration

Each environment has specific configuration:

- **Production**: `epsx-469400`, `api.epsx.io`, `epsx.io`, `admin.epsx.io`
- **Staging**: `epsx-staging`, `staging-api.epsx.io`, etc.
- **Development**: `epsx-development`, `dev-api.epsx.io`, etc.

## 🧪 Test Scripts (`/test/`)

Testing and validation scripts for platform components.

### Available Scripts

| Script | Description |
|--------|-------------|
| `notification-e2e.sh` | End-to-end notification testing |
| `verify-asset-loading.js` | Frontend asset loading validation |

### Usage

```bash
# Run notification tests
./scripts/test/notification-e2e.sh

# Verify asset loading
node ./scripts/test/verify-asset-loading.js
```

## 🛠️ Utility Scripts (`/utils/`)

Maintenance, validation, and troubleshooting utilities.

### Available Scripts

| Script | Description | Language |
|--------|-------------|----------|
| `validate-env.js` | Environment variable validation | Node.js |
| `validate-permissions.js` | Permission system validation | Node.js |
| `promote-admin.sh` | Promote user to admin | Bash |
| `revoke-admin.sh` | Revoke admin privileges | Bash |

### Usage

#### Environment Validation

```bash
# Validate all environment variables
./scripts/utils/validate-env.js

# Expected output:
# ✅ All required variables are set. Ready to start EPSX!
```

#### Permission Management

```bash
# Validate permission system
./scripts/utils/validate-permissions.js

# Promote user to admin
./scripts/utils/promote-admin.sh user@example.com

# Revoke admin privileges
./scripts/utils/revoke-admin.sh user@example.com
```

## 📋 Common Workflows

### Complete Deployment

```bash
# 1. Validate environment
./scripts/utils/validate-env.js

# 2. Deploy all services to production
./scripts/deploy/deploy.sh production

# 3. Verify deployments
gcloud run services list --project=epsx-469400 --region=us-central1
```

### Quick Service Update

```bash
# Deploy specific service
./scripts/deploy/deploy-service.sh backend production

# Check logs
gcloud logging read "resource.type=cloud_run_revision resource.labels.service_name=backend" --project=epsx-469400 --limit=10
```

### Environment Setup

```bash
# Create environment file
cp production/deployment/environments/production.env.example .env.production

# Edit with your values
nano .env.production

# Validate configuration
./scripts/utils/validate-env.js
```

## ✅ Benefits of New System

### Environment-Driven

- ✅ **No Hardcoding**: All configuration in environment files
- ✅ **Single Source of Truth**: Environment variables drive everything
- ✅ **Easy Updates**: Change `.env` file instead of editing YAML
- ✅ **Environment Parity**: Same templates across all environments

### Simplified Structure

- ✅ **Single Directory**: All scripts in `/scripts/` 
- ✅ **Clear Organization**: `deploy/`, `test/`, `utils/` subdirectories
- ✅ **No Duplicates**: Removed redundant validation and documentation
- ✅ **Consistent Naming**: Unified script conventions

### Flexible Deployment

- ✅ **Environment Selection**: Easy switching between environments
- ✅ **Service Selection**: Deploy individual services or all at once
- ✅ **Template Processing**: Automatic variable substitution
- ✅ **Health Checks**: Built-in deployment validation

## 🔧 Configuration

### Required Environment Variables

All environments need these core variables:

```bash
# Infrastructure
DATABASE_URL=postgresql://...
BACKEND_URL=https://api.epsx.io
FRONTEND_URL=https://epsx.io
ADMIN_FRONTEND_URL=https://admin.epsx.io

# Authentication
NEXTAUTH_SECRET=your-32-char-secret
OIDC_CLIENT_SECRET=your-oidc-secret
OIDC_ADMIN_CLIENT_SECRET=your-admin-secret

# Firebase
FIREBASE_PROJECT_ID=your-project-id
FIREBASE_PRIVATE_KEY=your-private-key
FIREBASE_CLIENT_EMAIL=your-service-account@project.iam.gserviceaccount.com
```

### Environment-Specific Values

Each environment uses different URLs and project IDs:

| Environment | Project ID | Backend URL | Frontend URL |
|-------------|------------|-------------|--------------|
| Production | `epsx-469400` | `https://api.epsx.io` | `https://epsx.io` |
| Staging | `epsx-staging` | `https://staging-api.epsx.io` | `https://staging.epsx.io` |
| Development | `epsx-development` | `https://dev-api.epsx.io` | `https://dev.epsx.io` |

## 🔍 Troubleshooting

### Common Issues

#### 1. Environment Variable Missing

```bash
# Error: Variable not found during substitution
❌ Environment validation FAILED

# Fix: Check your .env file
./scripts/utils/validate-env.js
```

#### 2. YAML Template Not Found

```bash
# Error: Service YAML template not found
❌ Service YAML template not found: scripts/deploy/services/production/backend.yaml

# Fix: Verify service and environment names
ls scripts/deploy/services/production/
```

#### 3. Authentication Issues

```bash
# Error: Not authenticated with gcloud
❌ Not authenticated with gcloud. Run: gcloud auth login

# Fix: Authenticate with Google Cloud
gcloud auth login
```

#### 4. Health Check Failures

```bash
# Warning: Service health check failed
⚠️  backend health check failed

# Check logs
gcloud logging read "resource.type=cloud_run_revision resource.labels.service_name=backend" --project=epsx-469400 --limit=10
```

### Debug Commands

```bash
# Check environment loading
cat production/deployment/environments/production.env

# Verify YAML processing
envsubst < scripts/deploy/services/production/backend.yaml

# Test service deployment
./scripts/deploy/deploy-service.sh backend production

# View deployment logs
gcloud logging read "resource.type=cloud_run_revision" --project=epsx-469400 --limit=20
```

## 📚 Additional Resources

- **CLAUDE.md**: Complete platform documentation
- **Environment Architecture**: See CLAUDE.md - Environment Architecture
- **Deployment Guide**: See CLAUDE.md - Cloud Run Deployment
- **Troubleshooting**: See CLAUDE.md - Troubleshooting section

## 🚨 Security Notes

1. **Never commit** `.env` files with production secrets
2. **Use environment variables** for all sensitive configuration
3. **Validate environment** before deployment
4. **Use service accounts** with minimal required permissions
5. **Monitor logs** for security events

---

**✅ Complete Environment-Driven Deployment System**

**Current Status**: All scripts consolidated and environment-driven. No hardcoded values in YAML files.

**Service URLs**:
- Backend: https://api.epsx.io ✅ Ready for deployment
- Frontend: https://epsx.io ✅ Ready for deployment  
- Admin: https://admin.epsx.io ✅ Ready for deployment

**Quick Deploy**: `./scripts/deploy/deploy.sh production`