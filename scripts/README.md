# EPSX Build & Deploy Scripts

✅ **COMPLETE BUILD & DEPLOY PIPELINE** - Fresh images from latest source code

## 🚀 Quick Deploy Latest Version

```bash
# Build fresh images + deploy (recommended)
./scripts/build-and-deploy.sh

# Or step by step:
./scripts/build-all.sh      # Build fresh images
./scripts/deploy.sh         # Deploy with latest images
```

## 📋 Available Scripts

### **Master Scripts**

#### **✅ `build-and-deploy.sh`** - Complete Pipeline
- **Builds fresh images** from current source code in each app directory
- **Deploys immediately** with the new images
- **One command** for complete update

#### **✅ `build-all.sh`** - Build All Services
- Builds backend, frontend, and admin from their directories
- Creates fresh Docker images with timestamp tags
- Tags as `:latest` for deployment

#### **✅ `deploy.sh`** - Deploy All Services
- Deploys all services using `:latest` images
- Backend (4GB RAM, 4 CPUs) → https://api.epsx.io
- Frontend (2GB RAM, 2 CPUs) → https://epsx.io
- Admin (1GB RAM, 1 CPU) → https://admin.epsx.io

### **Individual Service Scripts**

#### **Backend** (`apps/backend/`)
- **`build.sh`** - Build backend from latest Rust source
- **`deploy.sh`** - Deploy backend service
- Uses `Dockerfile.standalone` (self-contained)

#### **Frontend** (`apps/frontend/`)
- **`build.sh`** - Build frontend from latest Next.js source
- **`deploy.sh`** - Deploy frontend service
- Uses `Dockerfile.standalone` + copies `pnpm-lock.yaml`

#### **Admin Frontend** (`apps/admin-frontend/`)
- **`build.sh`** - Build admin from latest Next.js source
- **`deploy.sh`** - Deploy admin service
- Uses `Dockerfile.standalone` + copies `pnpm-lock.yaml`

## 🏗️ Build Architecture

```
Project Structure:
├── scripts/
│   ├── build-and-deploy.sh ← Master build + deploy
│   ├── build-all.sh        ← Build all services
│   └── deploy.sh           ← Deploy all services
├── apps/backend/
│   ├── Dockerfile.standalone ← Self-contained build
│   ├── build.sh            ← Build from this directory
│   └── deploy.sh           ← Deploy backend
├── apps/frontend/
│   ├── Dockerfile.standalone ← Self-contained build
│   ├── pnpm-lock.yaml      ← Copied from root
│   ├── build.sh            ← Build from this directory
│   └── deploy.sh           ← Deploy frontend
└── apps/admin-frontend/
    ├── Dockerfile.standalone ← Self-contained build
    ├── pnpm-lock.yaml      ← Copied from root
    ├── build.sh            ← Build from this directory
    └── deploy.sh           ← Deploy admin
```

## 🎯 Build Process

### **Each Service Builds Independently:**
1. **Backend**: Uses `Cargo.lock` and `Cargo.toml` from `apps/backend/`
2. **Frontend**: Uses `package.json` from `apps/frontend/` + copied `pnpm-lock.yaml`
3. **Admin**: Uses `package.json` from `apps/admin-frontend/` + copied `pnpm-lock.yaml`

### **Image Tagging:**
- **Timestamp tags**: `backend:20250828-143052` (for versioning)
- **Latest tags**: `backend:latest` (for deployment)

## ⚙️ Service Configuration

| Service | Memory | CPU | Min/Max Instances | Port | Custom Domain |
|---------|--------|-----|------------------|------|---------------|
| Backend | 4Gi | 4 | 0-10 | 8080 | api.epsx.io |
| Frontend | 2Gi | 2 | 0-10 | 3000 | epsx.io |
| Admin | 1Gi | 1 | 0-5 | 3000 | admin.epsx.io |

## 🔧 Individual Commands

### **Build Individual Services:**
```bash
cd apps/backend && ./build.sh           # Build backend only
cd apps/frontend && ./build.sh          # Build frontend only  
cd apps/admin-frontend && ./build.sh    # Build admin only
```

### **Deploy Individual Services:**
```bash
cd apps/backend && ./deploy.sh          # Deploy backend only
cd apps/frontend && ./deploy.sh         # Deploy frontend only
cd apps/admin-frontend && ./deploy.sh   # Deploy admin only
```

### **Management Commands:**
```bash
# Check service status
gcloud run services list --region=us-central1

# View service logs
gcloud run services logs read epsx-backend --region=us-central1
gcloud run services logs read epsx-frontend --region=us-central1
gcloud run services logs read epsx-admin --region=us-central1
```

## 📊 After Deployment

Your EPSX platform will be available at:

- **Main Platform**: https://epsx.io 🚀
- **API Backend**: https://api.epsx.io ⚡
- **Admin Panel**: https://admin.epsx.io 🔧

## 🔥 Key Features

### ✅ **Fresh Source Code**
- **No more legacy images** - builds from your current code
- Each service builds from its own directory
- Standalone Dockerfiles with no external dependencies

### ⚡ **Flexible Workflow**
- Build all services at once or individually
- Deploy all services or individual ones
- Combined pipeline or step-by-step

### 🎯 **Production Ready**
- Optimized Docker builds (no BuildKit cache issues)
- Proper resource allocation for each service
- Production environment variables
- Health checks and custom domains

### 🧹 **Clean & Simple**
- Self-contained builds from each app directory
- Clear separation between build and deploy
- Comprehensive logging and error handling

## 🎉 Success!

Your EPSX platform now deploys with **fresh images from your latest source code** instead of old cached versions. No more legacy content issues! 🚀