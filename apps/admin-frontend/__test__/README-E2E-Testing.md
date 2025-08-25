# Comprehensive Admin E2E Testing Guide

This guide covers the complete end-to-end testing setup for all admin modules with automatic permission assignment.

## 📋 Overview

The E2E testing suite provides comprehensive coverage for all admin modules. You can specify a test user via environment variable, and use the new admin management system to assign permissions.

## 🏗️ Test Structure

### Test Files
- `all-modules-comprehensive.spec.ts` - Main comprehensive test covering all admin modules
- `admin.spec.ts` - Existing core admin functionality tests
- `complete-admin-coverage.spec.ts` - Existing comprehensive coverage tests

### Scripts
- `run-comprehensive-e2e.sh` - Complete test runner (updated to use new admin system)

## 🎯 Test Coverage

### Modules Tested
✅ **Dashboard** - Main admin dashboard  
✅ **User Management** - Users list, create, permissions, roles  
✅ **Permission Management** - Permission profiles, assignment  
✅ **IAM Management** - Identity & access management, admin roles  
✅ **Analytics** - Analytics dashboard and reporting  
✅ **Billing** - Billing management and analytics  
✅ **System Administration** - Settings, database, developer portal  
✅ **Stock Ranking** - Stock ranking package management  
✅ **Access Control** - Access denied, unauthorized pages  

### Test Categories
- 🔐 **Authentication & Authorization** - Login flow, session persistence
- 🚀 **Module Access** - All admin pages accessibility
- ⚡ **Performance** - Page load times, rapid navigation
- 📱 **Responsive Design** - Mobile compatibility
- 🔄 **Error Handling** - Network errors, permission errors
- 🎯 **Deep Testing** - Module-specific functionality

## 🚀 Quick Start

### Prerequisites
1. Backend server running on port 8080
2. Admin frontend running on port 3001
3. Database accessible (PostgreSQL)
4. Test user created and assigned admin permissions

### Step 1: Create and Assign Admin User
```bash
# Set your test user email
export TEST_EMAIL="your-test-user@example.com"

# Promote to full admin
./scripts/promote-admin.sh $TEST_EMAIL
```

### Step 2: Run E2E Tests
```bash
cd apps/admin-frontend/__test__/scripts
TEST_EMAIL="your-test-user@example.com" ./run-comprehensive-e2e.sh
```

### Alternative: Manual Test Execution
```bash
# After assigning permissions, run tests directly
cd apps/admin-frontend
npx playwright test all-modules-comprehensive.spec.ts
```

## 🔧 Configuration

### Environment Variables
```bash
# Backend URL (default: http://localhost:8080)
export BACKEND_URL="http://localhost:8080"

# Database URL (default: postgresql://localhost/epsx)
export DATABASE_URL="postgresql://localhost/epsx"

# Test user (set via environment variable)
TEST_EMAIL="your-test-user@example.com"
```

### Required Permissions
The test user needs these admin modules:
- `system_admin` - Full system administration
- `user_management` - User management capabilities
- `permission_management` - Permission assignment
- `iam_management` - Identity & access management
- `analytics` - Analytics and reporting
- `billing_management` - Billing operations
- `stock_ranking_management` - Stock ranking features
- `database_management` - Database operations
- `developer_portal` - Developer tools
- `module_management` - Module administration

## 📊 Test Execution

### Running Specific Test Suites
```bash
# Complete admin module access test
npx playwright test all-modules-comprehensive.spec.ts

# Core admin functionality
npx playwright test admin.spec.ts

# Full coverage tests
npx playwright test complete-admin-coverage.spec.ts

# Run all E2E tests
npx playwright test
```

### Running with Different Browsers
```bash
# Chrome (default)
npx playwright test --project=admin-core

# Firefox
npx playwright test --project=admin-firefox

# Safari/WebKit
npx playwright test --project=admin-webkit

# Mobile Chrome
npx playwright test --project=admin-mobile-chrome
```

### Debugging Tests
```bash
# Run in headed mode (visible browser)
npx playwright test --headed

# Debug mode with browser inspector
npx playwright test --debug

# Run specific test
npx playwright test --grep "should access ALL admin modules"
```

## 🔍 Test Reports

### View Reports
```bash
# HTML report (recommended)
npx playwright show-report

# JSON report location
cat test-results/results.json

# JUnit report location
cat test-results/results.xml
```

### Test Artifacts
- **Screenshots** - Captured on test failure
- **Videos** - Recorded for failed tests
- **Traces** - Full interaction traces for debugging

## 🚨 Troubleshooting

### Common Issues

#### 1. Permission Denied Errors
```bash
# Re-run permission assignment
cd apps/admin-frontend/__test__/scripts
./assign-permissions.sh

# Verify user exists in database
psql $DATABASE_URL -c "SELECT email, role, admin_modules FROM users WHERE email = 'jesadakorn.kirtnu@gmail.com';"
```

#### 2. Backend Not Running
```bash
# Start backend
cd apps/backend
cargo run
```

#### 3. Frontend Not Running
```bash
# Start admin frontend
cd apps/admin-frontend
npm run dev
```

#### 4. OAuth Authentication Issues
- Check backend OAuth configuration
- Verify test user credentials
- Ensure backend can connect to OAuth provider

#### 5. Database Connection Issues
```bash
# Test database connection
psql $DATABASE_URL -c "SELECT 1;"

# Check user table structure
psql $DATABASE_URL -c "\\d users"
```

### Test Failure Analysis

#### High-Level Failures
- Check service availability (backend, frontend)
- Verify user permissions
- Review OAuth configuration

#### Module-Specific Failures
- Check module implementation status
- Verify route configurations
- Review component error boundaries

#### Performance Failures
- Check network conditions
- Review page optimization
- Consider timeout adjustments

## 🎯 Test Customization

### Adding New Modules
Edit `all-modules-comprehensive.spec.ts`:
```typescript
const ALL_ADMIN_MODULES = [
  // Add new module
  { path: '/new-module', name: 'New Module', module: 'new_module_management' }
];
```

### Updating Permissions
Edit required permissions in `assign-test-permissions.ts`:
```typescript
const REQUIRED_PERMISSIONS = [
  // Add new permissions
  'new_module:read',
  'new_module:write'
];
```

### Custom Test User
Update test credentials in test files:
```typescript
const TEST_EMAIL = 'your-test-user@example.com';
const TEST_PASSWORD = 'your-password';
```

## 📈 Performance Metrics

### Expected Load Times
- Dashboard: < 3 seconds
- User pages: < 5 seconds
- Analytics: < 8 seconds (data-heavy)
- Settings: < 4 seconds

### Success Rate Targets
- Module access: > 95%
- Authentication: 100%
- Session persistence: 100%
- Mobile compatibility: > 90%

## 🔄 CI/CD Integration

### GitHub Actions
```yaml
- name: Run Admin E2E Tests
  run: |
    cd apps/admin-frontend
    npm ci
    npm run build
    ./__test__/scripts/run-comprehensive-e2e.sh
```

### Local Pre-commit
```bash
# Add to .git/hooks/pre-commit
#!/bin/bash
cd apps/admin-frontend
npm run test:e2e:quick
```

## 📝 Best Practices

### Test Maintenance
1. Update tests when adding new admin features
2. Keep permission assignments current
3. Review test timeouts for performance
4. Monitor success rates over time

### Development Workflow
1. Run quick tests during development
2. Full test suite before major releases
3. Permission verification after user model changes
4. Cross-browser testing for critical updates

### Security Considerations
1. Use test-specific user accounts
2. Limit test permissions to necessary scope
3. Clean up test data after runs
4. Secure test environment access

## 🎉 Success Metrics

A successful test run should show:
- ✅ 95%+ module access success rate
- ✅ All authentication flows working
- ✅ No critical functionality errors
- ✅ Acceptable performance metrics
- ✅ Cross-browser compatibility

For issues or questions, check the test output logs and follow the troubleshooting guide above.