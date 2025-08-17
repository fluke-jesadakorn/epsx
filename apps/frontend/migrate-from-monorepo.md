# Migration Guide: From Monorepo to Standalone Frontend

This guide helps you migrate from the EPSX monorepo to the standalone frontend application.

## Pre-Migration Checklist

- [ ] Backup your current environment variables
- [ ] Document current build and deployment processes
- [ ] Test the existing frontend application functionality
- [ ] Ensure backend is compatible with standalone deployment
- [ ] Plan for zero-downtime deployment

## Migration Steps

### 1. Repository Setup

```bash
# Create new repository
git clone https://github.com/your-org/epsx-frontend
cd epsx-frontend

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
cp ../monorepo/apps/frontend/.env.local .env.local

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

### 5. Payment Integration Migration

Update payment webhook endpoints:

```bash
# Update Stripe webhook URLs
# Point webhooks to new domain: https://epsx.io/api/webhooks/*
# Update MusePay notification URLs
# Test payment flows in sandbox environment
```

### 6. Update CI/CD Pipeline

Update your deployment scripts to use the new repository:

```yaml
# Example GitHub Actions update
- name: Checkout code
  uses: actions/checkout@v4
  # Remove any monorepo-specific paths

# Update deployment targets
- name: Deploy to Cloud Run
  run: |
    gcloud run deploy epsx-frontend \
      --image gcr.io/${{ secrets.GCP_PROJECT_ID }}/epsx-frontend:latest
```

### 7. Backend Configuration

Ensure your backend is configured to accept requests from the new domain:

```rust
// Update CORS settings in backend
let cors = CorsLayer::new()
    .allow_origin("https://epsx.io".parse::<HeaderValue>().unwrap())
    .allow_origin("https://www.epsx.io".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(vec![AUTHORIZATION, CONTENT_TYPE]);
```

### 8. Domain and DNS Configuration

```bash
# Update your domain configuration
# Point epsx.io -> new standalone deployment
# Point www.epsx.io -> new standalone deployment
# Update SSL certificates if needed
# Configure CDN settings
```

### 9. Analytics and Monitoring

```bash
# Update Google Analytics configuration
# Update error tracking (Sentry, etc.)
# Configure performance monitoring
# Update uptime monitoring
```

## Post-Migration Testing

### Functional Testing

```bash
# Run unit tests
npm run test

# Run E2E tests
npm run test:e2e

# Manual testing checklist:
# - [ ] User registration and login
# - [ ] Stock rankings and analytics
# - [ ] Payment and subscription flows
# - [ ] Dashboard functionality
# - [ ] Mobile responsiveness
# - [ ] Performance on various devices
```

### Payment Testing

```bash
# Test payment flows:
# - [ ] Credit card payments
# - [ ] Subscription upgrades
# - [ ] Subscription cancellations
# - [ ] Refund processing
# - [ ] Webhook deliveries
```

### Performance Testing

```bash
# Build and analyze bundle
npm run analyze

# Check Core Web Vitals
# Monitor loading times
# Test with slow network conditions
# Verify caching behavior
```

### Security Testing

```bash
# Run security audit
npm audit

# Verify authentication flows
# Test session management
# Check for XSS vulnerabilities
# Verify data encryption
```

## Zero-Downtime Deployment Strategy

### Blue-Green Deployment

```bash
# Deploy to staging environment first
gcloud run deploy epsx-frontend-staging \
  --image gcr.io/project/epsx-frontend:latest

# Test staging thoroughly
# Switch traffic gradually using Cloud Run traffic splitting
gcloud run services update-traffic epsx-frontend \
  --to-revisions=REVISION-ID=50 \
  --region=us-central1

# Monitor metrics and gradually increase traffic
```

### Rollback Plan

```bash
# In case of issues, rollback to previous version
gcloud run services update-traffic epsx-frontend \
  --to-revisions=PREVIOUS-REVISION=100 \
  --region=us-central1

# Restore original DNS settings if needed
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

**Issue**: Payment processing failures
```bash
# Solution: Update webhook endpoints
# Verify API keys and secrets
# Check payment provider configurations
```

**Issue**: API connection failures
```bash
# Solution: Update NEXT_PUBLIC_BACKEND_URL
# Verify network connectivity
# Check firewall and security group settings
```

### Performance Issues

**Issue**: Slow loading times
```bash
# Solution: Enable CDN
# Optimize images and assets
# Implement proper caching headers
```

**Issue**: High memory usage
```bash
# Solution: Analyze bundle size
# Remove unused dependencies
# Optimize component rendering
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

## User Communication

### Pre-Migration

```markdown
# Email to users
Subject: Platform Improvement - Scheduled Maintenance

Dear EPSX Users,

We're improving our platform infrastructure to provide better performance and reliability. 

Scheduled Maintenance: [Date/Time]
Expected Duration: 2-4 hours
Impact: Minimal service interruption

What to expect:
- Faster loading times
- Improved reliability
- Enhanced mobile experience

Thank you for your patience.
```

### Post-Migration

```markdown
# Follow-up email
Subject: Platform Update Complete

The platform update is complete! You should now experience:
- Faster page loads
- Better mobile performance
- Improved reliability

If you experience any issues, please contact support@epsx.io
```

## Monitoring and Maintenance

After successful migration:

1. **Monitor Application Health**
   - Set up uptime monitoring (99.9% SLA target)
   - Configure error tracking and alerting
   - Monitor Core Web Vitals
   - Track conversion rates

2. **Performance Monitoring**
   - Monitor response times
   - Track user engagement metrics
   - Monitor payment success rates
   - Track mobile vs desktop usage

3. **Security Monitoring**
   - Monitor authentication failure rates
   - Track suspicious activity
   - Monitor for security vulnerabilities
   - Regular security audits

4. **Business Metrics**
   - Monitor subscription conversion rates
   - Track user retention
   - Monitor revenue metrics
   - Track feature usage

## Support

If you encounter issues during migration:

- Check the troubleshooting section in README.md
- Review GitHub Issues for similar problems
- Contact the development team for assistance
- Emergency hotline: [Your emergency contact]

## Cleanup

After successful migration and verification:

```bash
# Remove monorepo references from CI/CD
# Update any external service configurations
# Archive old monorepo documentation
# Update team documentation
# Update customer support documentation
```

---

This migration should be performed during a planned maintenance window with proper user communication to minimize impact on trading activities.