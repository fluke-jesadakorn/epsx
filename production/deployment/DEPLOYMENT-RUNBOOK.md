# EPSX Production Deployment Runbook

## Executive Summary

This runbook provides comprehensive guidance for deploying the EPSX trading platform's enterprise-grade security architecture to production. It covers all aspects of deployment, validation, monitoring, and emergency procedures.

## Prerequisites

### Required Access and Permissions
- [ ] Google Cloud Platform admin access to project `epsx-production`
- [ ] Docker registry push permissions
- [ ] Kubernetes cluster admin access (if applicable)
- [ ] Database admin access (Neon PostgreSQL)
- [ ] Redis Cloud admin access
- [ ] Monitoring system access (Cloud Monitoring)
- [ ] Alerting system access (PagerDuty, Slack)

### Required Tools
- [ ] `gcloud` CLI (authenticated)
- [ ] `docker` (latest version)
- [ ] `kubectl` (if using Kubernetes)
- [ ] `curl` for API testing
- [ ] `jq` for JSON processing
- [ ] `bc` for calculations
- [ ] `openssl` for certificate validation
- [ ] `nmap` for security scanning

### Environment Setup
```bash
# Verify tool versions
gcloud version
docker --version
curl --version
jq --version

# Authenticate with Google Cloud
gcloud auth login
gcloud auth application-default login
gcloud config set project epsx-production
```

## Pre-Deployment Checklist

### 1. Security Validation
- [ ] Run security test suite: `./deployment/security/validation/security-test-suite.sh production`
- [ ] Verify all critical security controls are implemented
- [ ] Review security configuration in `deployment/security/hardening/production-security-config.yaml`

### 2. Infrastructure Readiness
- [ ] Verify database connectivity and migrations are ready
- [ ] Confirm Redis cluster is operational
- [ ] Check container registry has required images
- [ ] Validate network configuration and firewall rules
- [ ] Verify SSL certificates are valid and up-to-date

### 3. Application Readiness
- [ ] All tests pass in CI/CD pipeline
- [ ] Performance benchmarks meet SLA requirements
- [ ] Security middleware latency < 10ms validated
- [ ] Permission system thoroughly tested
- [ ] Session validation working across all applications

### 4. Monitoring and Alerting
- [ ] Monitoring dashboards configured
- [ ] Alert policies activated
- [ ] Notification channels tested
- [ ] Runbook procedures documented
- [ ] On-call team notified

## Deployment Procedure

### Phase 1: Pre-deployment Setup (15 minutes)

```bash
# 1. Navigate to project root
cd /path/to/epsx

# 2. Load production environment
source deployment/environments/production.env

# 3. Verify environment configuration
echo "Backend URL: $BACKEND_URL"
echo "Frontend URL: $FRONTEND_URL"
echo "Admin URL: $ADMIN_FRONTEND_URL"

# 4. Run pre-deployment validation
./deployment/scripts/validate-deployment.sh production --pre-check
```

### Phase 2: Security Validation (10 minutes)

```bash
# 1. Run comprehensive security tests
./deployment/security/validation/security-test-suite.sh production

# 2. Verify security configuration
echo "Security validation completed in Phase 1"

# 3. Verify results - NO CRITICAL FAILURES allowed
# Check exit codes: 0 = pass, 1 = warnings, 2 = critical failures
```

### Phase 3: Deployment Execution (30 minutes)

```bash
# Option A: Full automated deployment
./deployment/scripts/deploy-production.sh

# Option B: Dry run first (recommended)
./deployment/scripts/deploy-production.sh --dry-run

# Option C: Deployment with specific options
./deployment/scripts/deploy-production.sh \
  --skip-tests \
  --force \
  --no-rollback
```

### Phase 4: Post-deployment Validation (15 minutes)

```bash
# 1. Run full validation suite
./deployment/scripts/validate-deployment.sh production

# 2. Verify all services are healthy
curl -f $BACKEND_URL/health
curl -f $FRONTEND_URL
curl -f $ADMIN_FRONTEND_URL

# 3. Check security controls
curl -I $FRONTEND_URL | grep -i "strict-transport-security"
curl -I $FRONTEND_URL | grep -i "content-security-policy"

# 4. Verify authentication system
curl -f $BACKEND_URL/.well-known/openid_configuration
curl -f $BACKEND_URL/oauth/jwks
```

### Phase 5: Monitoring Activation (10 minutes)

```bash
# 1. Verify monitoring is collecting metrics
gcloud monitoring metrics list --filter="metric.type:custom.googleapis.com/epsx/*"

# 2. Test alert notifications
# (This would trigger test alerts to verify notification channels)

# 3. Confirm dashboards are populated
# Check Cloud Monitoring console for EPSX dashboards

# 4. Verify log aggregation
gcloud logging read "resource.type=cloud_run_revision" --limit=10
```

## Validation Checklist

### Functional Validation
- [ ] **API Endpoints**: All critical endpoints responding correctly
- [ ] **Authentication**: OIDC discovery and JWT validation working
- [ ] **Authorization**: Permission system enforcing access controls
- [ ] **Database**: Connectivity and query performance within SLA
- [ ] **Cache**: Redis performance and hit rates optimal
- [ ] **Sessions**: Cross-application session management working

### Security Validation
- [ ] **HTTPS**: All traffic encrypted, proper redirects
- [ ] **Headers**: Security headers present and configured correctly
- [ ] **Rate Limiting**: Protection against brute force attacks
- [ ] **Input Validation**: XSS and injection protection active
- [ ] **CORS**: Cross-origin requests properly controlled
- [ ] **Audit Logging**: All security events being logged

### Performance Validation
- [ ] **Response Times**: API latency < 100ms (P95)
- [ ] **Throughput**: System handling expected load
- [ ] **Resource Usage**: Memory and CPU within limits
- [ ] **Database Performance**: Query times < 10ms average
- [ ] **Cache Performance**: Hit rates > 95%


## Emergency Procedures

### Immediate Rollback (5 minutes)
```bash
# Emergency rollback (fastest)
./deployment/scripts/rollback-production.sh emergency

# Controlled rollback (recommended if time permits)
./deployment/scripts/rollback-production.sh controlled

# Targeted service rollback
./deployment/scripts/rollback-production.sh targeted latest-stable backend
```

### Critical Issue Response
1. **Security Breach Detected**
   - Execute emergency rollback immediately
   - Contact security team: security@epsx.io
   - Preserve logs and evidence
   - Activate incident response procedures

2. **Performance Degradation**
   - Check monitoring dashboards
   - Scale services if needed: `gcloud run services update --max-instances=20`
   - Monitor error rates and response times
   - Consider rollback if issues persist

3. **Service Unavailability**
   - Check service health: `gcloud run services describe epsx-backend`
   - Review logs: `gcloud logs read --limit=50`
   - Restart services if needed
   - Escalate to on-call engineer

### Escalation Contacts
- **Security Team**: security@epsx.io (PagerDuty: security-alerts)
- **Engineering Team**: engineering@epsx.io (PagerDuty: engineering-alerts)
- **Infrastructure Team**: infrastructure@epsx.io (Slack: #infrastructure)

## Common Troubleshooting

### Deployment Failures

**Problem**: Container build fails
```bash
# Check build logs
gcloud builds log <BUILD_ID>

# Verify Dockerfile
docker build -t test-image -f deployment/security/hardening/container-security.dockerfile .
```

**Problem**: Service deployment fails
```bash
# Check service status
gcloud run services describe epsx-backend --region=us-central1

# Review service logs
gcloud logs read "resource.type=cloud_run_revision" --limit=10
```

**Problem**: Database connection issues
```bash
# Test database connectivity
curl -f $BACKEND_URL/api/v1/health/database

# Check connection pool status
# Review database logs in Neon console
```

### Security Issues

**Problem**: Security validation fails
```bash
# Re-run specific security tests
./deployment/security/validation/security-test-suite.sh production --verbose

# Check security configuration
cat deployment/security/hardening/production-security-config.yaml
```

**Problem**: Certificate issues
```bash
# Check certificate validity
openssl s_client -servername epsx.io -connect epsx.io:443 -verify_hostname epsx.io

# Verify certificate expiration
curl -vI https://epsx.io 2>&1 | grep -i "expire"
```

### Performance Issues

**Problem**: High response times
```bash
# Check service metrics
gcloud monitoring metrics list --filter="metric.type:run.googleapis.com/request_latencies"

# Review database performance
curl -s $BACKEND_URL/api/v1/admin/metrics/database | jq
```

**Problem**: High error rates
```bash
# Check error logs
gcloud logs read "severity>=ERROR" --limit=20

# Review service health
curl -f $BACKEND_URL/health
```

## Post-Deployment Tasks

### Immediate (0-2 hours)
- [ ] Monitor all services for stability
- [ ] Verify user access and functionality
- [ ] Check error rates and performance metrics
- [ ] Confirm security controls are active
- [ ] Update deployment documentation

### Short-term (2-24 hours)
- [ ] Review detailed monitoring data
- [ ] Analyze performance trends
- [ ] Verify backup systems
- [ ] Communicate success to stakeholders

### Medium-term (1-7 days)
- [ ] Conduct post-deployment review
- [ ] Update runbook based on lessons learned
- [ ] Plan performance optimizations
- [ ] Document any issues and resolutions

## Success Criteria

### Deployment Success
- ✅ All services deployed without errors
- ✅ Security validation passes with no critical issues
- ✅ Performance metrics meet SLA requirements
- ✅ User acceptance testing successful

### Operational Success
- ✅ 99.9%+ uptime maintained
- ✅ Response times < 100ms (P95)
- ✅ Error rates < 1%
- ✅ Security incidents = 0

### Business Success
- ✅ User satisfaction maintained
- ✅ Trading functionality operational
- ✅ Admin functionality accessible
- ✅ Financial data integrity preserved

## Appendices

### A. File Structure
```
deployment/
├── README-Production-Deployment.md
├── DEPLOYMENT-RUNBOOK.md (this file)
├── environments/
│   ├── production.env
│   ├── staging.env
│   └── development.env
├── security/
│   ├── hardening/
│   │   ├── production-security-config.yaml
│   │   └── container-security.dockerfile
│   ├── validation/
│   │   └── security-test-suite.sh
├── monitoring/
│   ├── production-dashboard.json
│   └── alerts/
│       ├── security-alerts.yaml
│       └── performance-alerts.yaml
└── scripts/
    ├── deploy-production.sh
    ├── rollback-production.sh
    └── validate-deployment.sh
```

### B. Environment Variables Reference
See `deployment/environments/production.env` for complete list.

### C. Monitoring Metrics
- Security: threats_detected, brute_force_attempts, blocked_ips
- Performance: request_latencies, error_rates, database_query_time

### D. Alert Thresholds
- Critical: Security threats > 50/min, Response time > 500ms
- High: Error rate > 5%, Database query time > 50ms
- Medium: Cache hit rate < 90%, Memory usage > 80%

---

**Document Version**: 1.0.0  
**Last Updated**: 2024-01-01  
**Next Review**: 2024-02-01  
**Owner**: Production Engineering Team  
**Approved By**: Security Team, Engineering Leadership