# Admin Middleware Validation Test Suite

Comprehensive end-to-end testing suite for the EPSX admin middleware validation system. This test suite validates the complete admin security architecture including permission-based access control, session management, security event logging, and attack prevention.

## 📋 Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Test Coverage](#test-coverage)
- [Quick Start](#quick-start)
- [Test Suites](#test-suites)
- [Configuration](#configuration)
- [Running Tests](#running-tests)
- [Performance Testing](#performance-testing)
- [Security Testing](#security-testing)
- [Troubleshooting](#troubleshooting)

## 🎯 Overview

The admin middleware validation test suite provides comprehensive testing for:

- **Permission Matrix Testing**: Validates all 5 admin modules (user-management, system-configuration, security-management, audit-logs, analytics-access)
- **Session Security**: JWT validation, session hijacking prevention, elevated session requirements
- **Attack Prevention**: SQL injection, XSS, CSRF, path traversal, and other security attacks  
- **Performance Validation**: Sub-100ms response times, concurrent load handling, cache effectiveness
- **Integration Testing**: End-to-end admin workflows with cross-module validation
- **Database Integrity**: ACID compliance, transaction rollbacks, connection pooling

## 🏗️ Architecture

### Admin Security Architecture

```
Frontend (3001) → Backend API (8080) → Database
     ↓              ↓                    ↓
  E2E Tests    → Middleware Tests  → Integration Tests
```

### Admin Modules Structure

- `user-management`: User CRUD, analytics, search operations
- `system-configuration`: API keys, system maintenance, settings
- `security-management`: Admin modules, permission profiles, security policies
- `audit-logs`: Reports, backups, compliance, audit events
- `analytics-access`: Permission analytics, security analysis, dashboards

### Middleware Stack Order

1. Enhanced CORS → Performance Headers → Request ID → Security Headers
2. CSP → Security Monitoring → Rate Limiting → Security Event Logging
3. Session Validation → Admin Module Protection → Elevated Session Requirements

## 📊 Test Coverage

### 1. Admin Module Permission Tests
- ✅ user-management module access control
- ✅ system-configuration module permissions
- ✅ security-management module validation
- ✅ audit-logs module access
- ✅ analytics-access module permissions
- ✅ Cross-module permission consistency
- ✅ Permission inheritance patterns

### 2. Session Security Tests
- ✅ JWT token authentication
- ✅ Expired token rejection
- ✅ Invalid signature detection
- ✅ Session hijacking prevention
- ✅ Elevated session requirements
- ✅ Session fixation prevention

### 3. Security Event Logging
- ✅ Complete audit trail validation
- ✅ Security event severity classification
- ✅ Real-time security monitoring
- ✅ Threat detection and alerting
- ✅ Audit log completeness

### 4. Rate Limiting
- ✅ Per-user rate limiting validation
- ✅ Tier-based rate limits (BRONZE: 60/min, ENTERPRISE: 10,000/min)
- ✅ Per-endpoint rate limiting
- ✅ Rate limit bypass prevention
- ✅ Rate limit header validation

### 5. Attack Prevention
- ✅ SQL injection detection (90%+ block rate)
- ✅ XSS payload filtering (85%+ sanitization)
- ✅ Path traversal prevention (95%+ block rate)
- ✅ CSRF protection validation
- ✅ Command injection prevention
- ✅ Request forgery detection

### 6. Performance Validation
- ✅ Sub-100ms response time targets
- ✅ Concurrent session handling (10-100 users)
- ✅ Memory stability under load
- ✅ Cache effectiveness measurement
- ✅ Database query performance
- ✅ Connection pool optimization

## 🚀 Quick Start

### Prerequisites

```bash
# Ensure services are running
pnpm dev:all  # Start all services (frontend, admin, backend)

# Or start individually:
pnpm dev:admin    # Admin frontend (port 3001)
pnpm dev:backend  # Backend API (port 8080)
```

### Install Dependencies

```bash
cd apps/admin-frontend
pnpm install
```

### Run All Tests

```bash
# Run complete test suite
pnpm test:e2e:admin-security

# Run specific test categories
pnpm test:e2e:permissions
pnpm test:e2e:security-attacks  
pnpm test:e2e:performance
```

### Quick Validation

```bash
# Run core admin middleware tests (5 minutes)
npx playwright test comprehensive-admin-middleware-validation.spec.ts

# Run security attack simulation (10 minutes)
npx playwright test admin-security-attack-simulation.spec.ts

# Run performance tests (5 minutes)
npx playwright test admin-performance-load-testing.spec.ts
```

## 🧪 Test Suites

### 1. Comprehensive Middleware Validation (`comprehensive-admin-middleware-validation.spec.ts`)

**Focus**: Core middleware functionality and permission matrix testing

**Key Tests**:
- Admin Module Permission Matrix (all 5 modules)
- Session Validation & JWT Authentication  
- Security Event Logging & Audit Trails
- Rate Limiting Validation
- Error Handling & Response Consistency

**Duration**: ~15 minutes
**Success Criteria**: 100% permission validation, <100ms average response time

```bash
npx playwright test comprehensive-admin-middleware-validation.spec.ts --reporter=html
```

### 2. Security Integration Workflows (`admin-security-integration.spec.ts`)

**Focus**: End-to-end admin security workflows

**Key Tests**:
- Complete User Lifecycle Management
- Security Incident Response Workflow  
- System Maintenance Workflow
- Cross-Module Permission Consistency
- Real-time Permission Updates
- Database Transaction Integrity

**Duration**: ~20 minutes
**Success Criteria**: 80%+ workflow success rate, consistent permissions

```bash
npx playwright test admin-security-integration.spec.ts --reporter=html
```

### 3. Security Attack Simulation (`admin-security-attack-simulation.spec.ts`)

**Focus**: Comprehensive penetration testing

**Key Tests**:
- SQL Injection (16 attack vectors)
- XSS Prevention (12 payload types)
- CSRF Protection Validation
- Path Traversal Prevention (12 techniques)
- Session Security Testing
- Authorization Bypass Prevention
- Input Validation & Sanitization

**Duration**: ~25 minutes  
**Success Criteria**: 90%+ attack block rate, proper error handling

```bash
npx playwright test admin-security-attack-simulation.spec.ts --reporter=html
```

### 4. Performance & Load Testing (`admin-performance-load-testing.spec.ts`)

**Focus**: Performance validation and load handling

**Key Tests**:
- Sub-100ms Response Time Validation
- Concurrent Session Handling (10-100 users)
- Memory Usage Stability
- Cache Performance Effectiveness
- Database Query Optimization
- Rate Limiting Performance Impact

**Duration**: ~30 minutes
**Success Criteria**: <100ms response times, stable memory usage

```bash
npx playwright test admin-performance-load-testing.spec.ts --reporter=html
```

## ⚙️ Configuration

### Environment Variables

```bash
# Test Configuration
# Admin user must be promoted via: ./scripts/promote-admin.sh jesadakorn.kirtnu@gmail.com
TEST_ADMIN_PASSWORD=Aa_12345678
TEST_API_BASE_URL=http://localhost:8080
TEST_ADMIN_BASE_URL=http://localhost:3001

# Performance Thresholds  
TEST_MAX_RESPONSE_TIME=1000
TEST_MAX_CONCURRENT_USERS=100
TEST_DURATION=30000

# Database Configuration
TEST_DB_HOST=localhost
TEST_DB_PORT=5432
TEST_DB_NAME=epsx_test
TEST_DB_USER=test_user
TEST_DB_PASSWORD=test_password
```

### Playwright Configuration

The test suite uses `playwright.config.ts` with optimized settings:

- **Parallel Execution**: 2 workers (CI: 1 worker)
- **Timeouts**: 60s test timeout, 15s action timeout
- **Retries**: 2 retries in CI, 0 in local development
- **Browsers**: Chrome (primary), Firefox, Safari (cross-browser validation)
- **Mobile**: Pixel 5, iPhone 12 (responsive testing)

### Test User Matrix

```typescript
// Pre-configured test users with different permission combinations
const TEST_USERS = {
  SUPER_ADMIN: ['user-management', 'system-configuration', 'security-management', 'audit-logs', 'analytics-access'],
  USER_MANAGER: ['user-management'],
  SECURITY_MANAGER: ['security-management', 'audit-logs'],
  ANALYST: ['analytics-access'],
  SYSTEM_ADMIN: ['system-configuration'],
  RESTRICTED_ADMIN: [] // No permissions - for testing access denial
};
```

## 🏃 Running Tests

### Local Development

```bash
# Run all admin security tests
pnpm test:e2e:admin

# Run specific test file
npx playwright test comprehensive-admin-middleware-validation.spec.ts

# Run with UI mode for debugging
npx playwright test --ui

# Run specific test pattern
npx playwright test --grep "SQL injection"
```

### Continuous Integration

```bash
# CI optimized test execution
npx playwright test --reporter=junit --reporter=html

# Parallel execution across test files
npx playwright test --workers=4

# Generate test reports
npx playwright show-report
```

### Test Execution Strategies

**Quick Smoke Tests** (5 minutes):
```bash
# Essential middleware validation
npx playwright test comprehensive-admin-middleware-validation.spec.ts --grep "@smoke"
```

**Security Focus** (15 minutes):
```bash  
# Security-focused testing
npx playwright test admin-security-attack-simulation.spec.ts
npx playwright test admin-security-integration.spec.ts --grep "@security"
```

**Performance Focus** (20 minutes):
```bash
# Performance and load testing
npx playwright test admin-performance-load-testing.spec.ts
npx playwright test comprehensive-admin-middleware-validation.spec.ts --grep "@performance"
```

**Complete Validation** (60 minutes):
```bash
# Full test suite execution
pnpm test:e2e:admin-complete
```

## 📈 Performance Testing

### Response Time Targets

- **Fast Response**: ≤ 100ms (target for core endpoints)
- **Normal Response**: ≤ 500ms (acceptable performance)
- **Slow Response**: ≤ 1000ms (warning threshold)  
- **Timeout**: 5000ms (maximum acceptable)

### Load Testing Scenarios

**Light Load** (10 concurrent users):
- Target: <100ms response time, <5% error rate
- Duration: 10 seconds
- Expected throughput: >10 req/s

**Medium Load** (25 concurrent users):  
- Target: <500ms response time, <10% error rate
- Duration: 15 seconds
- Expected throughput: >15 req/s

**Heavy Load** (50 concurrent users):
- Target: <1000ms response time, <20% error rate  
- Duration: 20 seconds
- Expected throughput: >10 req/s

**Stress Test** (100 concurrent users):
- Target: No server crashes, graceful degradation
- Duration: 30 seconds
- Expected: Rate limiting activation, stable responses

### Performance Metrics Tracked

- Average/Min/Max response times
- 95th/99th percentile response times
- Throughput (requests per second)
- Error rate percentage
- Memory usage patterns
- Cache hit rates
- Database connection pool utilization

## 🛡️ Security Testing

### Attack Vector Coverage

**SQL Injection** (16 vectors):
- Union-based injection
- Boolean-based blind injection  
- Time-based blind injection
- Error-based injection
- Second-order injection
- NoSQL injection variants

**XSS Prevention** (12 payload types):
- Reflected XSS
- DOM-based XSS
- Event handler XSS
- CSS-based XSS
- Filter bypass techniques
- Polyglot payloads

**Path Traversal** (12 techniques):
- Unix path traversal (../../../etc/passwd)
- Windows path traversal (..\\..\\..\\boot.ini)
- URL encoded traversal
- Double encoding
- Null byte injection
- Advanced traversal techniques

### Security Validation Criteria

- **SQL Injection Block Rate**: ≥90%
- **XSS Sanitization Rate**: ≥85%  
- **Path Traversal Block Rate**: ≥95%
- **CSRF Protection**: 100% for state-changing operations
- **Rate Limiting**: Proper enforcement with 429 responses
- **Session Security**: No token manipulation acceptance

### Security Event Logging

All security events are logged with:
- Event ID (UUID)
- User ID
- Endpoint accessed
- HTTP method
- Status code  
- Security level (PUBLIC/STANDARD/ELEVATED/SENSITIVE/CRITICAL)
- Duration
- IP address
- Timestamp
- Additional context

## 🔧 Troubleshooting

### Common Issues

**1. Authentication Failures**
```bash
# Symptom: Login fails or tokens are invalid
# Solution: Verify admin user is promoted in database and OAuth configuration
# Promote user: ./scripts/promote-admin.sh jesadakorn.kirtnu@gmail.com
echo $TEST_ADMIN_PASSWORD

# Reset test session
npx playwright test --grep "login" --debug
```

**2. Database Connection Issues**  
```bash
# Symptom: Database-related test failures
# Solution: Check database connectivity and test data
psql -h localhost -p 5432 -U test_user -d epsx_test -c "SELECT COUNT(*) FROM users;"
```

**3. Performance Test Failures**
```bash
# Symptom: Response times exceed thresholds
# Solution: Check system resources and service health
curl http://localhost:8080/health
curl http://localhost:3001/health
```

**4. Rate Limiting Issues**
```bash
# Symptom: Unexpected rate limiting behavior
# Solution: Clear cache and verify rate limit configuration  
redis-cli FLUSHALL  # If using Redis
```

**5. Permission Validation Failures**
```bash
# Symptom: Permission tests fail unexpectedly
# Solution: Verify test user permissions in database
# Check admin_modules assignments for test users
```

### Debug Mode Execution

```bash
# Run tests with debug output
DEBUG=pw:api npx playwright test comprehensive-admin-middleware-validation.spec.ts

# Run single test with UI debugger
npx playwright test --debug --grep "should validate user-management module permissions"

# Generate trace files for analysis
npx playwright test --trace on
```

### Log Analysis

```bash
# View test execution logs
cat test-results/results.json | jq '.suites[].tests[].results[]'

# Analyze performance metrics
grep "Response Time" test-results/*.log

# Check security event logs
grep "Security event" test-results/*.log
```

### Performance Troubleshooting

```bash
# Check system resources during tests
htop  # Monitor CPU/memory usage
iostat 1  # Monitor I/O performance  
netstat -an | grep :8080  # Check API connections

# Database performance analysis
EXPLAIN ANALYZE SELECT * FROM users WHERE admin_modules @> '["user-management"]';
```

## 📝 Test Reports

### HTML Reports
```bash
# Generate and view HTML report
npx playwright show-report
```

### JSON Reports  
```bash
# Generate JSON report for CI integration
npx playwright test --reporter=json --output-dir=test-results/
```

### Custom Metrics Dashboard
```bash
# Generate performance metrics summary
node scripts/generate-performance-report.js test-results/results.json
```

## 🤝 Contributing

### Adding New Tests

1. **Create test file**: Follow naming pattern `admin-[feature]-[type].spec.ts`
2. **Use test fixtures**: Import from `fixtures/admin-test-fixtures.ts`
3. **Follow patterns**: Use existing test structure and helpers
4. **Add documentation**: Update this README with new test coverage

### Test Development Guidelines

- **Descriptive names**: Use clear, behavior-focused test names
- **Proper cleanup**: Ensure test isolation and cleanup
- **Performance aware**: Consider test execution time
- **Error handling**: Include proper error assertions
- **Documentation**: Add comments for complex test logic

### Example Test Structure

```typescript
test.describe('🔐 New Security Feature', () => {
  test.beforeEach(async ({ page }) => {
    await loginAdmin(page);
  });

  test('should validate new security control', async ({ page, request }) => {
    console.log('🧪 Testing new security control');
    
    // Arrange
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Act
    const result = await executeSecurityTest(request, token);
    
    // Assert
    expect(result.blocked).toBe(true);
    expect(result.status).toBe(403);
    
    console.log('✅ Security control validated');
  });
});
```

## 📚 Additional Resources

- [Playwright Documentation](https://playwright.dev/)
- [Admin Architecture Documentation](../../../ARCHITECTURE.md)
- [Security Requirements](../../../SECURITY.md)
- [Performance Benchmarks](../../../PERFORMANCE.md)

## 📞 Support

For test-related questions or issues:

1. **Check logs**: Review test execution logs and reports
2. **Verify environment**: Ensure all services are running properly  
3. **Database state**: Verify test data integrity
4. **Performance**: Check system resources and response times
5. **Security**: Review security event logs for anomalies

---

**Success Criteria Summary**:
- ✅ 100% admin endpoint coverage
- ✅ All security controls validated and functioning  
- ✅ Performance targets met (sub-100ms response times)
- ✅ Security vulnerabilities prevented (90%+ block rates)
- ✅ Comprehensive audit trail validation
- ✅ Clean test execution with reliable results

*This comprehensive test suite ensures the admin middleware validation system is robust, secure, and performant.*