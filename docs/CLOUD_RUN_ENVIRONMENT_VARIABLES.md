# Cloud Run Environment Variables Guide

## Overview

EPSX uses **runtime-only environment variables** for `NEXT_PUBLIC_*` configuration. This approach enables:

- ✅ **Single Docker Image**: Same image works across all environments
- ✅ **Dynamic Configuration**: Change URLs without rebuilds  
- ✅ **True Environment Separation**: Configure per revision in Cloud Run Console
- ✅ **Simplified Deployment**: No build-time environment coupling

## Architecture

### Traditional Approach (❌ Removed)
```bash
# Build-time coupling (no longer used)
docker build --build-arg NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io
```

### New Approach (✅ Implemented)
```bash
# Build once, configure at runtime
docker build -t image:latest .
# Set environment variables in Cloud Run Console
```

## Required Environment Variables

### For All Services (Backend, Frontend, Admin)

#### **Core URLs** (Required)
```bash
# Backend Service
BACKEND_URL=https://api.epsx.io
FRONTEND_URL=https://epsx.io  
ADMIN_FRONTEND_URL=https://admin.epsx.io

# Client-Accessible URLs  
NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io
NEXT_PUBLIC_APP_URL=https://epsx.io
NEXT_PUBLIC_ADMIN_URL=https://admin.epsx.io
```

#### **Authentication** (Required)
```bash
# Server-side secrets
NEXTAUTH_SECRET=your-32-char-minimum-secret
OIDC_CLIENT_SECRET=your-frontend-client-secret
OIDC_ADMIN_CLIENT_SECRET=your-admin-client-secret

# Client-side OAuth
NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-frontend  # or epsx-admin for admin frontend
```

#### **Database** (Backend Only)
```bash
DATABASE_URL=postgresql://user:password@host:5432/database
```

#### **Firebase** (Required)
```bash
# Server-side Firebase (Backend)
FIREBASE_PROJECT_ID=your-project-id
FIREBASE_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----"
FIREBASE_CLIENT_EMAIL=service-account@project.iam.gserviceaccount.com

# Client-side Firebase (Frontend/Admin) - Optional
NEXT_PUBLIC_FIREBASE_API_KEY=your-firebase-api-key
NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN=your-project.firebaseapp.com
NEXT_PUBLIC_FIREBASE_PROJECT_ID=your-project-id
NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET=your-project.appspot.com
NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID=your-sender-id
NEXT_PUBLIC_FIREBASE_APP_ID=your-app-id
NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID=your-measurement-id
```

## Environment-Specific Configuration

### Development Environment
```bash
# Local development URLs
NEXT_PUBLIC_BACKEND_URL=http://localhost:8080
NEXT_PUBLIC_APP_URL=http://localhost:3000
NEXT_PUBLIC_ADMIN_URL=http://localhost:3001
NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-frontend
```

### Staging Environment (Revision Tag: `staging`)
```bash
# Staging URLs
NEXT_PUBLIC_BACKEND_URL=https://staging---epsx-backend-project.us-central1.run.app
NEXT_PUBLIC_APP_URL=https://staging---epsx-frontend-project.us-central1.run.app
NEXT_PUBLIC_ADMIN_URL=https://staging---epsx-admin-project.us-central1.run.app
NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-frontend-staging
```

### Production Environment (100% Traffic)
```bash
# Production URLs
NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io
NEXT_PUBLIC_APP_URL=https://epsx.io
NEXT_PUBLIC_ADMIN_URL=https://admin.epsx.io
NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-frontend
```

## Cloud Run Configuration Steps

### 1. Deploy with Tag (No Traffic)
```bash
# Deploy staging revision
ENV=staging ./scripts/deploy/service.sh frontend

# Deploy admin staging revision  
ENV=staging ./scripts/deploy/service.sh admin

# Deploy backend staging revision
ENV=staging ./scripts/deploy/service.sh backend
```

### 2. Configure Environment Variables in Cloud Run Console

#### Navigate to Cloud Run Console
1. Go to [Google Cloud Console → Cloud Run](https://console.cloud.google.com/run)
2. Select your service (e.g., `epsx-frontend`)
3. Click **"Edit & Deploy New Revision"**

#### Set Environment Variables
1. Go to **"Variables & Secrets"** tab
2. Click **"Add Variable"** for each required variable:

**Frontend Service Variables:**
```
NEXT_PUBLIC_BACKEND_URL = https://api.epsx.io
NEXT_PUBLIC_APP_URL = https://epsx.io  
NEXT_PUBLIC_ADMIN_URL = https://admin.epsx.io
NEXT_PUBLIC_OAUTH_CLIENT_ID = epsx-frontend
NEXT_PUBLIC_FIREBASE_API_KEY = your-firebase-api-key
NEXT_PUBLIC_FIREBASE_PROJECT_ID = your-project-id
# ... other Firebase vars as needed
```

**Admin Service Variables:**
```
NEXT_PUBLIC_BACKEND_URL = https://api.epsx.io
NEXT_PUBLIC_APP_URL = https://epsx.io
NEXT_PUBLIC_ADMIN_URL = https://admin.epsx.io  
NEXT_PUBLIC_OAUTH_CLIENT_ID = epsx-admin
NEXT_PUBLIC_FIREBASE_API_KEY = your-firebase-api-key
NEXT_PUBLIC_FIREBASE_PROJECT_ID = your-project-id
# ... other Firebase vars as needed
```

**Backend Service Variables:**
```
DATABASE_URL = postgresql://user:password@host:5432/database
BACKEND_URL = https://api.epsx.io
FRONTEND_URL = https://epsx.io
ADMIN_FRONTEND_URL = https://admin.epsx.io
NEXTAUTH_SECRET = your-32-char-secret
OIDC_CLIENT_SECRET = your-client-secret
OIDC_ADMIN_CLIENT_SECRET = your-admin-secret
FIREBASE_PROJECT_ID = your-project-id
FIREBASE_PRIVATE_KEY = your-private-key
FIREBASE_CLIENT_EMAIL = service-account@project.iam.gserviceaccount.com
```

### 3. Deploy and Test Tagged Revision
1. Click **"Deploy"** to create the revision
2. Test at tagged URL: `https://staging---epsx-frontend-project.us-central1.run.app`
3. Verify environment variables are working correctly

### 4. Promote to Production
```bash
# Promote staging to 100% traffic
./scripts/deploy/promote.sh frontend staging 100
./scripts/deploy/promote.sh admin staging 100
./scripts/deploy/promote.sh backend staging 100
```

## Deployment Workflow Examples

### Scenario 1: Deploy New Feature
```bash
# 1. Deploy feature branch with tag
ENV=feature-auth ./scripts/deploy/service.sh frontend

# 2. Configure environment variables in Cloud Run Console
#    Use same production values but with feature tag URLs

# 3. Test at tagged URL
curl https://feature-auth---epsx-frontend-project.us-central1.run.app

# 4. Promote when ready
./scripts/deploy/promote.sh frontend feature-auth 100
```

### Scenario 2: Staging Environment
```bash
# 1. Deploy all services with staging tag
ENV=staging ./scripts/deploy/service.sh backend
ENV=staging ./scripts/deploy/service.sh frontend  
ENV=staging ./scripts/deploy/service.sh admin

# 2. Configure staging environment variables in Cloud Run Console
#    Use staging URLs for cross-service communication

# 3. Test complete staging environment
# 4. Promote to production when validated
```

### Scenario 3: Production Hotfix
```bash
# 1. Deploy hotfix with dedicated tag
ENV=hotfix-auth ./scripts/deploy/service.sh backend

# 2. Configure production environment variables
# 3. Test with 0% traffic
# 4. Gradual rollout
./scripts/deploy/promote.sh backend hotfix-auth 10  # 10% traffic
./scripts/deploy/promote.sh backend hotfix-auth 50  # 50% traffic  
./scripts/deploy/promote.sh backend hotfix-auth 100 # Full promotion
```

## Environment Variable Validation

### Runtime Validation
The applications automatically validate environment variables on startup:

```typescript
// Automatically called in app/layout.tsx
initializeRuntimeEnvironment();
```

**Validation Output:**
```
🔍 Validating runtime environment variables...
✅ Runtime environment validation passed
🌐 Backend URL: https://api.epsx.io
🎯 App URL: https://epsx.io
👥 Admin URL: https://admin.epsx.io
🔑 OAuth Client ID: epsx-frontend
🔥 Firebase configuration detected
```

### Error Handling
**Missing Required Variables:**
```
❌ Runtime environment validation failed:
NEXT_PUBLIC_BACKEND_URL is required in production environment
NEXT_PUBLIC_APP_URL is required in production environment
```

**Invalid URLs:**
```
❌ Runtime environment validation failed:  
NEXT_PUBLIC_BACKEND_URL must be a valid URL (current: invalid-url)
NEXT_PUBLIC_APP_URL must use HTTPS in production (current: http://epsx.io)
```

## Troubleshooting

### Common Issues

#### 1. Environment Variables Not Available
**Problem:** `NEXT_PUBLIC_BACKEND_URL is undefined`
**Solution:** 
- Verify variables are set in Cloud Run Console
- Check variable names match exactly (case-sensitive)
- Redeploy service after setting variables

#### 2. Build Failures After Migration  
**Problem:** Build fails looking for build args
**Solution:**
- Ensure Dockerfile doesn't have `ARG NEXT_PUBLIC_*` declarations
- Remove `--build-arg NEXT_PUBLIC_*` from deployment scripts
- Only server-side variables should be build args

#### 3. Firebase Configuration Warnings
**Problem:** `Firebase partially configured (2/7 variables)`
**Solution:**
- Set all Firebase client variables or none
- Check variable names match the expected format
- Verify Firebase project configuration

### Debug Commands

```bash
# View current environment variables
./scripts/deploy/traffic.sh frontend

# List all revisions
./scripts/deploy/revisions.sh frontend

# Check service logs
gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=epsx-frontend" --limit=20
```

## Security Best Practices

### ✅ Safe for Client-Side
- `NEXT_PUBLIC_BACKEND_URL` - API endpoint URL
- `NEXT_PUBLIC_APP_URL` - Frontend URL
- `NEXT_PUBLIC_ADMIN_URL` - Admin panel URL  
- `NEXT_PUBLIC_OAUTH_CLIENT_ID` - OAuth client identifier
- `NEXT_PUBLIC_FIREBASE_*` - Firebase client configuration

### ❌ Never Client-Side
- `DATABASE_URL` - Database connection string
- `NEXTAUTH_SECRET` - JWT signing secret
- `OIDC_CLIENT_SECRET` - OAuth client secret
- `FIREBASE_PRIVATE_KEY` - Firebase service account key
- `FIREBASE_CLIENT_EMAIL` - Firebase service account email

## Migration Benefits

### Before (Build-Time Coupling)
- ❌ Different Docker images per environment
- ❌ Build-time environment dependencies  
- ❌ Complex CI/CD with environment-specific builds
- ❌ Cannot change URLs without rebuild

### After (Runtime Configuration)
- ✅ Single Docker image for all environments
- ✅ Runtime environment configuration
- ✅ Simplified deployment pipeline
- ✅ Dynamic configuration via Cloud Run Console
- ✅ Faster deployments and rollbacks
- ✅ True revision-based environment separation

## Next Steps

1. **Set up staging environment** with tagged revisions
2. **Configure production environment variables** in Cloud Run Console
3. **Test the deployment workflow** with the new scripts
4. **Monitor application startup** for environment validation logs
5. **Document environment-specific configurations** for your team

---

**🎉 Your EPSX deployment now supports runtime-only environment variables with Cloud Run Console configuration!**