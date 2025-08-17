# Migration Guide: From Monorepo to Standalone Admin Frontend

This guide helps you migrate from the EPSX monorepo to the standalone admin frontend application.

## Pre-Migration Checklist

- [ ] Backup your current environment variables
- [ ] Document current build and deployment processes
- [ ] Test the existing admin application functionality
- [ ] Ensure backend is compatible with standalone deployment

## Migration Steps

### 1. Repository Setup

```bash
# Create new repository
git clone https://github.com/your-org/epsx-admin-frontend
cd epsx-admin-frontend

# Or if migrating existing repo
git checkout -b standalone-migration
```

### 2. Update Dependencies

```bash
# Remove old package-lock.json and node_modules
rm -rf node_modules package-lock.json

# Install dependencies with new standalone package.json
npm install

# Verify all dependencies are properly installed
npm run type-check
```

### 3. Environment Configuration

```bash
# Copy your existing environment variables
cp ../monorepo/apps/admin-frontend/.env.local .env.local

# Update any monorepo-specific paths or references
# Remove any package references like @epsx/*
```

### 4. Build Verification

```bash
# Test the build process
npm run build

# Verify standalone output is generated
ls -la .next/standalone
```

### 5. Update CI/CD Pipeline

Update your deployment scripts to use the new repository:

```yaml
# Example GitHub Actions update
- name: Checkout code
  uses: actions/checkout@v4
  # Remove any monorepo-specific paths
```

### 6. Database and Backend Configuration

Ensure your backend is configured to accept requests from the new domain:

```rust
// Update CORS settings in backend
let cors = CorsLayer::new()
    .allow_origin("https://admin.your-domain.com".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(vec![AUTHORIZATION, CONTENT_TYPE]);
```

### 7. Domain and DNS Configuration

```bash
# Update your domain configuration
# Point admin.epsx.io -> new standalone deployment
# Update SSL certificates if needed
```

## Post-Migration Testing

### Functional Testing

```bash
# Run unit tests
npm run test

# Run E2E tests
npm run test:e2e

# Manual testing checklist:
# - [ ] Admin login functionality
# - [ ] User management features
# - [ ] Analytics dashboard
# - [ ] Billing management
# - [ ] Settings configuration
```

### Performance Testing

```bash
# Build and analyze bundle
npm run analyze

# Check for any performance regressions
# Monitor loading times and resource usage
```

### Security Testing

```bash
# Run security audit
npm audit

# Verify authentication flows
# Test admin permission validation
# Check for any exposed sensitive data
```

## Rollback Plan

In case of issues, you can rollback to the monorepo version:

```bash
# Switch back to monorepo deployment
# Restore original DNS settings
# Monitor for any data consistency issues
```

## Common Migration Issues

### Build Errors

**Issue**: Module not found errors
```bash
# Solution: Check for any remaining @epsx/* imports
grep -r "@epsx/" src/
# Replace with relative imports or npm packages
```

**Issue**: TypeScript errors
```bash
# Solution: Update tsconfig.json paths
# Remove monorepo-specific path mappings
```

### Runtime Errors

**Issue**: Authentication failures
```bash
# Solution: Verify backend CORS settings
# Check JWT token validation
# Confirm OAuth client configurations
```

**Issue**: API connection failures
```bash
# Solution: Update NEXT_PUBLIC_BACKEND_URL
# Verify network connectivity
# Check firewall and security group settings
```

### Deployment Issues

**Issue**: Docker build failures
```bash
# Solution: Update Dockerfile
# Remove monorepo-specific COPY commands
# Verify all dependencies are in package.json
```

**Issue**: Cloud Run deployment failures
```bash
# Solution: Check environment variables
# Verify service account permissions
# Review resource allocation settings
```

## Monitoring and Maintenance

After successful migration:

1. **Monitor Application Health**
   - Set up uptime monitoring
   - Configure error tracking
   - Monitor performance metrics

2. **Update Documentation**
   - Update deployment documentation
   - Update team runbooks
   - Update incident response procedures

3. **Team Training**
   - Train team on new repository structure
   - Update development workflows
   - Update code review processes

## Support

If you encounter issues during migration:

- Check the troubleshooting section in README.md
- Review GitHub Issues for similar problems
- Contact the development team for assistance

## Cleanup

After successful migration and verification:

```bash
# Remove monorepo references from CI/CD
# Update any external service configurations
# Archive old monorepo documentation
```

---

This migration should be performed during a maintenance window to minimize disruption to admin users.