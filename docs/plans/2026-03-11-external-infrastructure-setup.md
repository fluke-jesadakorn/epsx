# External Infrastructure Setup Plan

This document outlines the strategy for managing GitLab and Cloudflare Tunnel independently from the main EPSX monorepo. Separating these services reduces the complexity of the EPSX deployment lifecycle and treats them as foundational infrastructure rather than application dependencies.

## 1. Cloudflare Tunnel (Ingress)

Instead of managing `cloudflared` through host-level scripts or coupling it tightly with the application manifests, it should be deployed as a standalone service.

### Option A: In-Cluster Kubernetes Deployment (Recommended)
Deploy `cloudflared` directly into the Colima/k3s cluster to route traffic directly to the internal Kubernetes services without needing NodePorts or `socat` bridges.

1. **Create a new namespace:** 
   ```bash
   kubectl create namespace ingress-cloudflare
   ```
2. **Create a Secret for credentials:** Store the tunnel credentials in a Kubernetes Secret.
3. **Deploy the Tunnel:** Use a Kubernetes Deployment for the `cloudflare/cloudflared` image.
4. **ConfigMap Routing (Example):**
   ```yaml
   ingress:
     - hostname: epsx.io
       service: http://epsx-frontend.epsx-prod.svc.cluster.local:3000
     - hostname: admin.epsx.io
       service: http://epsx-admin.epsx-prod.svc.cluster.local:3001
     - hostname: api.epsx.io
       service: http://epsx-backend.epsx-prod.svc.cluster.local:8080
     - service: http_status:404
   ```

### Option B: Host-Level Docker Service
Run `cloudflared` as a standalone Docker container on the host machine, independent of the EPSX application.

1. Create a directory (e.g., `~/cloudflared-standalone/`) on the host.
2. Place the `config.yml` and `credentials.json` in this directory.
3. Run using Docker Compose or pure Docker:
   ```bash
   docker run -d --name external-cloudflared --restart unless-stopped \
     -v ~/cloudflared-standalone:/etc/cloudflared \
     cloudflare/cloudflared:latest tunnel --config /etc/cloudflared/config.yml run
   ```

---

## 2. GitLab (VCS, CI/CD, Registry)

GitLab should be hosted entirely separately, either on a different server or in an isolated directory and Docker network on the same machine.

### Standalone Docker Compose Setup
Create a dedicated directory on the host (e.g., `~/gitlab-standalone/`) with its own `docker-compose.yml`. This setup runs completely independently of the EPSX project.

```yaml
version: '3.8'
services:
  gitlab:
    image: gitlab/gitlab-ce:latest
    container_name: standalone-gitlab
    restart: always
    hostname: 'gitlab.jesadakorn.com'
    environment:
      GITLAB_OMNIBUS_CONFIG: |
        external_url 'https://gitlab.jesadakorn.com'
        registry_external_url 'https://registry.jesadakorn.com'
        gitlab_rails['registry_enabled'] = true
    ports:
      - '8929:80'
      - '5050:5050'
      - '2222:22'
    volumes:
      - './config:/etc/gitlab'
      - './logs:/var/log/gitlab'
      - './data:/var/opt/gitlab'
```

### Integration with EPSX
1. **Source Control:** The EPSX repository simply points to the external GitLab URL as its Git remote.
2. **Container Registry:** EPSX deployment scripts will pull images from `registry.jesadakorn.com:5050`. The EPSX deployment environment (k3s) must be configured with Kubernetes `imagePullSecrets` to authenticate with this external registry.
3. **CI/CD:** You can reintroduce a `.gitlab-ci.yml` to the EPSX repo later. When you do, the GitLab Runners (managed within your new standalone GitLab setup) will pick up the jobs, build the images, and apply the Kubernetes manifests to your cluster remotely.

---

## Migration Steps for You

1. **Stand up GitLab:** Use the docker-compose setup above in a completely separate folder on your machine (e.g., `~/Infrastructure/GitLab`).
2. **Stand up Cloudflare Tunnel:** Decide between Option A (Kubernetes) or Option B (Docker) and start the tunnel pointing to your EPSX services.
3. **Update DNS:** Ensure Cloudflare routes `gitlab.jesadakorn.com` to your standalone GitLab instance.
4. **Push Code:** Push your updated, clean EPSX repository to your newly isolated GitLab instance.