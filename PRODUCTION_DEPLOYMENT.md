# EPSX Production Deployment Guide

## 🎯 Overview
Complete production deployment guide for EPSX trading platform on Google Cloud Run using Neon PostgreSQL database.

**Production Database**: `postgresql://neondb_owner:npg_UYc6GMDJfPk8@ep-sweet-wave-a1fnijbf-pooler.ap-southeast-1.aws.neon.tech/neondb?sslmode=require&channel_binding=require`

## ✅ Pre-Deployment Checklist

### 1. Production Environment Files
- [x] `apps/backend/.env.production` - Backend API configuration
- [x] `apps/frontend/.env.production` - Frontend configuration  
- [x] `apps/admin-frontend/.env.production` - Admin dashboard configuration

### 2. Database Setup
- [x] Production PostgreSQL database (Neon) configured
- [x] Database migrations ready (`001_initial_schema.sql`)
- [x] Production seed data ready (`002_seed_data.sql`) with admin user `info@epsx.io`

### 3. Build Scripts
- [x] `scripts/build-prod.sh` - Production-optimized builds
- [x] `scripts/push-all.sh` - Push containers to Artifact Registry
- [x] `scripts/deploy-cloudrun.sh` - Deploy to Google Cloud Run

## 🚀 Step-by-Step Deployment

### Step 1: Google Cloud Setup (15 minutes)

```bash
# 1. Authenticate with Google Cloud
gcloud auth login

# 2. Set project configuration
export GOOGLE_CLOUD_PROJECT="epsx-469400"
export GOOGLE_CLOUD_REGION="us-central1"
export ARTIFACT_REGISTRY_REPO="epsx"

gcloud config set project epsx-469400

# 3. Create Artifact Registry repository
gcloud artifacts repositories create epsx \
  --repository-format=docker \
  --location=us-central1

# 4. Configure Docker authentication
gcloud auth configure-docker us-central1-docker.pkg.dev
```

### Step 2: Database Migration (5 minutes)

The database migrations will run automatically when the backend container starts in production. The migrations include:
- Complete database schema (10 tables)
- Admin modules and permissions system
- Production admin user: `info@epsx.io`
- Firebase UID: `prod_admin_uid_epsx_2024`

### Step 3: Production Build (20 minutes)

```bash
# Set required environment variables for build validation
export DATABASE_URL="postgresql://neondb_owner:npg_UYc6GMDJfPk8@ep-sweet-wave-a1fnijbf-pooler.ap-southeast-1.aws.neon.tech/neondb?sslmode=require&channel_binding=require"
export NEXTAUTH_SECRET="prod-epsx-jwt-secret-2024-ultra-secure-32-chars-minimum"
export FIREBASE_PRIVATE_KEY="[Your Firebase Private Key]"
export FIREBASE_CLIENT_EMAIL="firebase-adminsdk-fbsvc@epsx-449804.iam.gserviceaccount.com"
export FRONTEND_URL="https://epsx.io"
export BACKEND_URL="https://api.epsx.io"
export ADMIN_FRONTEND_URL="https://admin.epsx.io"

# Build production containers
BUILD_TARGET=production ./scripts/build-prod.sh
```

**Expected Output:**
- Frontend container: Production-optimized Next.js
- Admin container: Admin dashboard with authentication
- Backend container: Rust API with database connections

### Step 4: Push to Registry (10 minutes)

```bash
# Push all containers to Google Artifact Registry
./scripts/push-all.sh
```

**Expected Output:**
- `us-central1-docker.pkg.dev/epsx-469400/epsx/frontend:latest`
- `us-central1-docker.pkg.dev/epsx-469400/epsx/admin:latest`
- `us-central1-docker.pkg.dev/epsx-469400/epsx/backend:latest`

### Step 5: Deploy to Cloud Run (15 minutes)

```bash
# Deploy all services to Google Cloud Run
./scripts/deploy-cloudrun.sh
```

**Expected Services:**
- **Backend API**: 1Gi memory, 1 CPU, 1-10 instances
- **Frontend**: 512Mi memory, 1 CPU, 1-10 instances  
- **Admin**: 512Mi memory, 1 CPU, 0-5 instances (scales to zero)

## 🌐 Production URLs

After deployment, you'll receive three service URLs:

```
Frontend:     https://epsx-frontend-[random].a.run.app
Admin:        https://epsx-admin-[random].a.run.app  
Backend API:  https://epsx-backend-[random].a.run.app
```

## 🔒 Production Admin Access

**Admin User**: `info@epsx.io`
**Firebase UID**: `prod_admin_uid_epsx_2024`
**Permissions**: Full system administrator (all modules)

### Admin Modules Available:
1. **User Operations Manager** - User CRUD operations
2. **Permission Administrator** - Permission profiles and assignments
3. **Role & Policy Manager** - Casbin roles and policies
4. **Analytics Specialist** - Reporting and dashboards
5. **Billing Administrator** - Payment and subscription management
6. **System Administrator** - Database and system settings
7. **Developer Relations** - API keys and developer tools
8. **Module Coordinator** - Feature module assignments
9. **Compliance & Audit Officer** - Security and compliance
10. **Support Specialist** - User support tools

## 📊 Monitoring & Management

### View Service Status
```bash
gcloud run services list --region=us-central1
```

### View Logs
```bash
# Backend logs
gcloud run services logs read epsx-backend --region=us-central1

# Frontend logs  
gcloud run services logs read epsx-frontend --region=us-central1

# Admin logs
gcloud run services logs read epsx-admin --region=us-central1
```

### Update Services
```bash
# Update environment variables
gcloud run services update epsx-backend \
  --set-env-vars="NEW_VAR=value" \
  --region=us-central1

# Update resources
gcloud run services update epsx-frontend \
  --memory=1Gi \
  --cpu=2 \
  --region=us-central1
```

## 🔧 Custom Domain Setup (Optional)

If you own the epsx.io domain:

```bash
# Map custom domains
gcloud run domain-mappings create --service=epsx-frontend --domain=epsx.io --region=us-central1
gcloud run domain-mappings create --service=epsx-admin --domain=admin.epsx.io --region=us-central1
gcloud run domain-mappings create --service=epsx-backend --domain=api.epsx.io --region=us-central1
```

Then update DNS records to point to Cloud Run.

## 🚨 Troubleshooting

### Build Issues
- **Docker not found**: Install Docker Desktop
- **Permission denied**: Run `chmod +x scripts/*.sh`
- **Environment validation failed**: Set required environment variables

### Deployment Issues  
- **Image not found**: Run `./scripts/push-all.sh` first
- **Authentication failed**: Run `gcloud auth login`
- **Service timeout**: Check environment variables and database connection

### Database Issues
- **Connection failed**: Verify Neon PostgreSQL credentials
- **Migration failed**: Migrations run automatically in production container
- **TLS errors**: Local SQLx may not have TLS support - works in production

## 🎉 Success Indicators

✅ **Successful Deployment:**
- All 3 services show "READY" status in Cloud Run console
- Frontend URL loads trading platform interface
- Admin URL loads administrative dashboard
- Backend API responds to health checks
- Database contains production admin user and modules

## 📝 Production Configuration Summary

**Database**: Neon PostgreSQL with SSL
**Authentication**: Firebase + JWT tokens
**Admin System**: 10 granular permission modules
**Container Engine**: Google Cloud Run
**Image Registry**: Google Artifact Registry
**Build System**: Production-optimized Docker builds

## 🔄 Update Process

To deploy updates:
1. Make code changes
2. Run `BUILD_TARGET=production ./scripts/build-prod.sh`
3. Run `./scripts/push-all.sh`
4. Run `./scripts/deploy-cloudrun.sh`

Cloud Run automatically handles zero-downtime deployments.

---

**Total Deployment Time**: ~60 minutes
**Production Ready**: ✅ All components configured for production use