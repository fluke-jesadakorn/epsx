# 🚀 EPSX Local Deployment Guide

Complete guide for EPSX local build + direct Cloud Run deployment system.

## 🎯 Quick Start

1. **Set up environment**:
   ```bash
   cp production/deployment/environments/development.env.example .env
   # Edit .env with your values
   pnpm env:validate
   ```

2. **Deploy to development**:
   ```bash
   pnpm deploy:dev
   ```

3. **Deploy to production**:
   ```bash
   cp production/deployment/environments/production.env .env
   pnpm deploy:prod
   ```

## 🌍 Environment Setup

### Development Environment
```bash
# Copy template and customize
cp production/deployment/environments/development.env.example .env.development
# Edit .env.development with your development values

# Deploy to development
ENV=development pnpm deploy:dev
```

### Staging Environment  
```bash
# Copy template and customize
cp production/deployment/environments/staging.env.example .env.staging
# Edit .env.staging with your staging values

# Deploy to staging
ENV=staging pnpm deploy:staging
```

### Production Environment
```bash
# Use your existing production .env file
cp production/deployment/environments/production.env .env
# Or set production environment variables manually

# Deploy to production (with confirmation)
ENV=production pnpm deploy:prod
```

## 🔧 Available Commands

### Full Environment Deployment
```bash
pnpm deploy:dev        # Deploy all services to development
pnpm deploy:staging    # Deploy all services to staging  
pnpm deploy:prod       # Deploy all services to production
```

### Individual Service Deployment
```bash
ENV=development pnpm deploy:backend   # Deploy only backend
ENV=staging pnpm deploy:frontend      # Deploy only frontend
ENV=production pnpm deploy:admin      # Deploy only admin
```

### Build Commands
```bash
pnpm build:affected              # Build only changed services
turbo run build --affected       # Same as above
```

## 🌐 Environment URLs

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

## 🔐 Authentication Setup

1. **Install Google Cloud CLI**:
   ```bash
   # Install gcloud CLI
   curl https://sdk.cloud.google.com | bash
   
   # Authenticate
   gcloud auth login
   gcloud auth application-default login
   ```

2. **Set up projects**:
   ```bash
   # Development
   gcloud config set project epsx-development
   
   # Staging  
   gcloud config set project epsx-staging
   
   # Production
   gcloud config set project epsx-469400  # Your existing project
   ```

## 📦 Required Environment Variables

### Server-Only (15 variables)
- `DATABASE_URL` - PostgreSQL connection string
- `NEXTAUTH_SECRET` - JWT secret (32+ chars)
- `OIDC_CLIENT_SECRET` - OIDC client secret
- `OIDC_ADMIN_CLIENT_SECRET` - OIDC admin client secret
- `FIREBASE_PROJECT_ID` - Firebase project ID
- `FIREBASE_PRIVATE_KEY` - Firebase service account key
- `FIREBASE_CLIENT_EMAIL` - Firebase service account email

### Client-Safe (NEXT_PUBLIC_ prefix)
- `NEXT_PUBLIC_BACKEND_URL` - Backend API URL
- `NEXT_PUBLIC_APP_URL` - Frontend URL
- `NEXT_PUBLIC_ADMIN_URL` - Admin URL

## 🔍 Validation

Always validate your environment before deployment:
```bash
pnpm env:validate
```

This checks:
- All required variables are set
- Correct formats (URLs, email, etc.)
- Security issues (dev secrets in production)

## 🐛 Troubleshooting

### Authentication Issues
```bash
# Re-authenticate
gcloud auth login
gcloud auth application-default login

# Check active account
gcloud auth list
```

### Docker Issues
```bash
# Configure Docker for Artifact Registry
gcloud auth configure-docker us-central1-docker.pkg.dev
```

### Build Issues
```bash
# Clean and rebuild
pnpm clean
pnpm build:affected
```

### Deployment Issues
```bash
# Check service logs
gcloud logging read 'resource.type=cloud_run_revision' --limit=20

# Check service status
gcloud run services list --region=us-central1
```

## 🎯 Migration from Old System

1. **Remove GitHub Actions** ✅ (Already done)
2. **Use new scripts** - Replace old deployment commands
3. **Environment files** - Use new .env templates in `production/deployment/environments/`
4. **Validation** - Always run `pnpm env:validate`

### Old vs New Commands
```bash
# OLD: GitHub Actions based
git push origin main  # Auto-deploys via GitHub Actions

# NEW: Local build + deploy
pnpm deploy:prod      # Local build + deploy to Cloud Run
```

## 📁 File Structure

```
production/deployment/
├── scripts/
│   ├── deploy-dev.sh         # Development deployment
│   ├── deploy-staging.sh     # Staging deployment  
│   ├── deploy-prod.sh        # Production deployment
│   └── deploy-service.sh     # Individual service deployment
├── environments/
│   ├── development.env.example
│   ├── staging.env.example
│   ├── production.env
│   └── staging.env
├── configs/
│   └── deployment.json      # Centralized configuration
└── docs/
    └── DEPLOYMENT.md        # This guide
```

## ✨ Benefits

- **Full Control**: Build and deploy from your local machine
- **No External Dependencies**: No GitHub Actions or Cloud Build
- **Fast Iteration**: Leverages Turborepo --affected builds
- **Environment Consistency**: Same commands across dev/staging/prod
- **Easy Debugging**: All build logs visible locally
- **Cost Effective**: No CI/CD service costs
- **Organized Structure**: Clear separation of deployment concerns

## 🔄 Typical Workflow

```bash
# 1. Develop locally
pnpm dev

# 2. Test changes
pnpm env:validate
pnpm deploy:dev

# 3. Stage for testing
cp production/deployment/environments/staging.env .env
pnpm deploy:staging

# 4. Deploy to production
cp production/deployment/environments/production.env .env
pnpm deploy:prod  # Requires confirmation
```

## 📊 Resource Allocation

All resource configurations are defined in `production/deployment/configs/deployment.json` for easy management and consistency across environments.