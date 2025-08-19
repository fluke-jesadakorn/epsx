# EPSX Google Cloud Run Deployment Scripts

Simple deployment scripts for building containers locally and deploying the EPSX monorepo to Google Cloud Run.

## 🚀 Quick Start

### Prerequisites

1. **Google Cloud CLI** configured
   ```bash
   gcloud auth login
   gcloud config set project epsx-469400
   ```

2. **Docker** running locally
   ```bash
   docker info
   ```

3. **Environment file** `.env.shared` in project root

### Deploy in 3 Steps

```bash
# 1. Build containers locally
./scripts/build-all.sh

# 2. Push to Google Cloud
./scripts/push-all.sh

# 3. Deploy to Cloud Run
./scripts/deploy-cloudrun.sh
```

## 📋 Scripts Overview

### `build-all.sh` - Build Containers Locally
Builds Docker containers for all three applications using existing Dockerfiles.

**What it does:**
- Builds frontend, admin, and backend containers in parallel
- Tags for Google Artifact Registry
- Uses AMD64 platform for Cloud Run compatibility

**Usage:**
```bash
./scripts/build-all.sh
```

**Environment Variables:**
- `GOOGLE_CLOUD_PROJECT` - Project ID (default: epsx-469400)
- `GOOGLE_CLOUD_REGION` - Region (default: us-central1)
- `BUILD_VERSION` - Version tag (default: latest)

---

### `push-all.sh` - Push to Registry
Pushes built containers to Google Artifact Registry.

**What it does:**
- Configures Docker authentication
- Creates repository if needed
- Pushes all three containers

**Usage:**
```bash
./scripts/push-all.sh
```

---

### `deploy-cloudrun.sh` - Deploy to Cloud Run
Deploys services to Google Cloud Run with optimal configuration.

**Service Configuration:**

| Service | Memory | CPU | Min/Max Instances | Port |
|---------|--------|-----|------------------|------|
| Frontend | 512Mi | 1 | 1-10 | 3000 |
| Admin | 512Mi | 1 | 0-5 | 3000 |
| Backend | 1Gi | 1 | 1-10 | 8080 |

**What it does:**
- Deploys backend first (other services depend on it)
- Configures environment variables
- Sets up auto-scaling
- Returns service URLs

**Usage:**
```bash
./scripts/deploy-cloudrun.sh
```

## 🏗️ Architecture

```
Internet
    ↓
Google Cloud Run Services
┌─────────────────────────────┐
│  Frontend Service (Port 3000) │ ← https://epsx-frontend-*.run.app
├─────────────────────────────┤
│  Admin Service (Port 3000)    │ ← https://epsx-admin-*.run.app
├─────────────────────────────┤
│  Backend Service (Port 8080)  │ ← https://epsx-backend-*.run.app
└─────────────────────────────┘
```

Each service gets its own Cloud Run URL and can scale independently.

## ⚙️ Configuration

### Environment Variables

Set these in your shell or `.env` file:

```bash
# Required
GOOGLE_CLOUD_PROJECT=epsx-469400
GOOGLE_CLOUD_REGION=us-central1

# Optional
ARTIFACT_REGISTRY_REPO=epsx
BUILD_VERSION=latest
```

### `.env.shared` File

Required file in project root with shared environment variables:

```bash
# Authentication
NEXTAUTH_SECRET=your-jwt-secret
COOKIE_ENCRYPTION_KEY=your-encryption-key

# Firebase Configuration
FIREBASE_PROJECT_ID=epsx-469400
FIREBASE_CLIENT_EMAIL=your-service-account@epsx.iam.gserviceaccount.com
FIREBASE_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n"

# Other service configurations...
```

## 🔧 Development Workflow

### Local Development
```bash
# Test locally first
pnpm dev

# Build and test containers
./scripts/build-all.sh
docker run -p 3000:3000 us-central1-docker.pkg.dev/epsx-469400/epsx/frontend:latest
```

### Production Deployment
```bash
# Full deployment pipeline
./scripts/build-all.sh && \
./scripts/push-all.sh && \
./scripts/deploy-cloudrun.sh
```

### Update Existing Services
```bash
# Just rebuild and redeploy
./scripts/build-all.sh
./scripts/push-all.sh
./scripts/deploy-cloudrun.sh
```

## 📊 Service Management

### Check Service Status
```bash
# List all services
gcloud run services list --region=us-central1

# Get service details
gcloud run services describe epsx-frontend --region=us-central1
```

### View Logs
```bash
# View logs
gcloud run services logs read epsx-frontend --region=us-central1

# Follow logs
gcloud run services logs tail epsx-frontend --region=us-central1
```

### Update Service Configuration
```bash
# Update memory/CPU
gcloud run services update epsx-frontend \
  --memory=1Gi \
  --cpu=2 \
  --region=us-central1

# Update environment variables  
gcloud run services update epsx-frontend \
  --set-env-vars="NEW_VAR=value" \
  --region=us-central1
```

## 🚨 Troubleshooting

### Build Issues
```bash
# Check Docker
docker info

# Clean Docker cache
docker system prune -f

# Rebuild from scratch
docker build --no-cache -f apps/frontend/Dockerfile apps/frontend
```

### Push Issues
```bash
# Re-authenticate
gcloud auth login
gcloud auth configure-docker us-central1-docker.pkg.dev

# Check repository exists
gcloud artifacts repositories list --location=us-central1
```

### Deployment Issues
```bash
# Check service status
gcloud run services describe epsx-frontend --region=us-central1

# View deployment logs
gcloud run services logs read epsx-frontend --region=us-central1 --limit=50

# Test service health
curl https://epsx-frontend-PROJECT_ID.a.run.app
```

### Common Fixes

**"Image not found" error:**
```bash
# Make sure you pushed the image
./scripts/push-all.sh
```

**"Service timeout" error:**
```bash
# Check if .env.shared exists
ls -la .env.shared

# Verify environment variables
gcloud run services describe SERVICE_NAME --region=us-central1
```

**"Authentication failed" error:**
```bash
# Re-authenticate with gcloud
gcloud auth login
gcloud config set project epsx-469400
```

## 🔍 Monitoring

### Service Health
Each service automatically gets:
- Health checks
- Request/response metrics
- Error rate monitoring
- CPU/Memory usage tracking

### View Metrics
```bash
# Open Cloud Console monitoring
gcloud console --project=epsx-469400

# Or use CLI to get basic stats
gcloud run services describe epsx-frontend \
  --region=us-central1 \
  --format="table(metadata.name,status.url,status.traffic[].percent)"
```

## 💡 Tips & Best Practices

### Performance
- **Use latest version** of containers for better performance
- **Set appropriate memory/CPU** based on usage patterns
- **Configure min instances** for services that need low latency

### Cost Optimization
- **Admin service scales to 0** when not in use
- **Frontend/Backend have min instances** for availability
- **Monitor usage** and adjust scaling parameters

### Security
- **Environment variables** are automatically secured by Cloud Run
- **HTTPS** is automatically provided
- **Private container registry** keeps images secure

### Scaling
- Services auto-scale based on incoming requests
- **Min instances** prevent cold starts
- **Max instances** prevent runaway costs

---

## ✅ Success!

After successful deployment, you'll have three independent Cloud Run services:

- **Frontend**: Full trading platform
- **Admin**: Administrative dashboard  
- **Backend**: High-performance API

Each service can scale independently and has its own URL for testing and access.

Your EPSX platform is now running on Google Cloud Run! 🚀