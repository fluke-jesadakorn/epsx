# Google Cloud Artifact Registry Setup

## Current Status
- **Project**: `epsx-469400`
- **Account**: `info@epsx.io` (authenticated)
- **Issue**: Missing Artifact Registry permissions

## Manual Configuration Required

### 1. Create Artifact Registry Repository
1. Go to [Google Cloud Console > Artifact Registry](https://console.cloud.google.com/artifacts)
2. Click **"Create Repository"**
3. Configure:
   - **Name**: `epsx`
   - **Format**: `Docker`
   - **Location**: `us-central1 (Iowa)`
   - **Description**: `EPSX containers`
4. Click **"Create"**

### 2. Grant IAM Permissions
1. Go to [IAM & Admin > IAM](https://console.cloud.google.com/iam-admin/iam)
2. Find user `info@epsx.io`
3. Click **"Edit Principal"**
4. Add these roles:
   - `Artifact Registry Repository Administrator`
   - `Artifact Registry Writer`
5. Click **"Save"**

### 3. Verify Configuration
Run this command to test access:
```bash
gcloud artifacts repositories describe epsx --location=us-central1
```

### 4. Push Images
Once configured, run:
```bash
./scripts/build-all.sh   # Build containers
./scripts/push-all.sh    # Push to registry
```

## Container Registry URLs
After setup, your images will be available at:
- Frontend: `us-central1-docker.pkg.dev/epsx-469400/epsx/frontend:latest`
- Admin: `us-central1-docker.pkg.dev/epsx-469400/epsx/admin:latest`
- Backend: `us-central1-docker.pkg.dev/epsx-469400/epsx/backend:latest`

## Docker Integration
The scripts use Docker for reliable container builds and deployment to Google Cloud Run.