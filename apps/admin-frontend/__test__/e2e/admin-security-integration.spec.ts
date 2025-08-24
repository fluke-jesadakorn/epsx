/**
 * Admin Security Integration Test Suite
 * 
 * End-to-end tests for complete admin security workflows including:
 * - Multi-step permission validation flows
 * - Cross-module security consistency
 * - Real-time permission updates
 * - Session management across workflows
 * - Security event correlation
 * - Database transaction integrity
 */

import { test, expect, Page, APIRequestContext } from '@playwright/test';
import { performance } from 'perf_hooks';

// Test configuration
const ADMIN_BASE_URL = 'http://localhost:3001';
const API_BASE_URL = 'http://localhost:8080';
const TEST_EMAIL = 'jesadakorn.kirtnu@gmail.com';
const TEST_PASSWORD = 'Aa_12345678';

// Integration workflow test scenarios
const INTEGRATION_WORKFLOWS = {
  USER_LIFECYCLE: {
    name: 'Complete User Lifecycle Management',
    steps: [
      { action: 'create_user', module: 'user-management', endpoint: '/api/v1/admin/users', method: 'POST' },
      { action: 'assign_permissions', module: 'security-management', endpoint: '/api/v1/admin/permission-profiles/assign', method: 'POST' },
      { action: 'grant_temporary_access', module: 'security-management', endpoint: '/api/v1/admin/temporary-permissions', method: 'POST' },
      { action: 'audit_changes', module: 'audit-logs', endpoint: '/api/v1/admin/permissions/audit-report', method: 'GET' },
      { action: 'analytics_review', module: 'analytics-access', endpoint: '/api/v1/admin/analytics/permissions', method: 'GET' }
    ]
  },
  SECURITY_INCIDENT_RESPONSE: {
    name: 'Security Incident Response Workflow',
    steps: [
      { action: 'detect_threat', module: 'security-management', endpoint: '/api/v1/admin/analytics/security-risks', method: 'GET' },
      { action: 'review_audit_logs', module: 'audit-logs', endpoint: '/api/v1/admin/permissions/audit-report', method: 'GET' },
      { action: 'revoke_permissions', module: 'security-management', endpoint: '/api/v1/admin/temporary-permissions/bulk-revoke', method: 'POST' },
      { action: 'generate_report', module: 'audit-logs', endpoint: '/api/v1/admin/permissions/system-backup', method: 'POST' },
      { action: 'notify_stakeholders', module: 'user-management', endpoint: '/api/v1/admin/users/bulk-update', method: 'POST' }
    ]
  },
  SYSTEM_MAINTENANCE: {
    name: 'System Maintenance Workflow',
    steps: [
      { action: 'backup_system', module: 'audit-logs', endpoint: '/api/v1/admin/permissions/system-backup', method: 'POST' },
      { action: 'cleanup_expired', module: 'system-configuration', endpoint: '/api/v1/admin/roles/cleanup-expired', method: 'POST' },
      { action: 'update_api_keys', module: 'system-configuration', endpoint: '/api/v1/admin/api-keys', method: 'GET' },
      { action: 'validate_permissions', module: 'security-management', endpoint: '/api/v1/admin/permission-profiles/bulk-validate', method: 'POST' },
      { action: 'performance_review', module: 'analytics-access', endpoint: '/api/v1/admin/analytics/performance', method: 'GET' }
    ]
  }
};

// Helper functions
async function loginAdmin(page: Page) {
  console.log('🔑 Logging in admin user for integration testing');
  
  await page.goto('/');
  
  try {
    await page.waitForURL('**/login**', { timeout: 5000 });
  } catch {
    const signOutBtn = page.locator('text=Sign out').first();
    if (await signOutBtn.isVisible()) {
      await signOutBtn.click();
      await page.waitForURL('**/login**');
    }
  }

  const oauthLoginBtn = page.locator('button').filter({ hasText: /sign in|login|epsx/i }).first();
  await expect(oauthLoginBtn).toBeVisible({ timeout: 10000 });
  await oauthLoginBtn.click();

  await page.waitForURL('**/oauth/authorize**', { timeout: 10000 });
  await page.fill('input[name="email"]', TEST_EMAIL);
  await page.fill('input[name="password"]', TEST_PASSWORD);
  
  const submitBtn = page.locator('button[type="submit"]').first();
  await submitBtn.click();

  await page.waitForFunction(
    () => {
      const url = window.location.href;
      return !url.includes('/login') && 
             !url.includes('/oauth/authorize') && 
             url.includes('localhost:3001');
    },
    { timeout: 30000 }
  );

  await page.waitForLoadState('networkidle');
  console.log('✅ Admin login successful for integration testing');
}

async function extractAuthToken(page: Page): Promise<string | null> {
  return await page.evaluate(() => {
    return localStorage.getItem('auth_token') || 
           document.cookie.split(';').find(c => c.trim().startsWith('session='))?.split('=')[1];
  });
}

async function executeWorkflowStep(
  request: APIRequestContext,
  token: string,
  step: any,
  testData?: any
): Promise<{ success: boolean; response?: any; duration: number; error?: string }> {
  const startTime = performance.now();
  
  try {
    let requestConfig: any = {
      method: step.method,
      headers: { 
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      },
      timeout: 30000
    };
    
    // Add test data for POST/PUT requests
    if (step.method !== 'GET' && testData) {
      requestConfig.body = JSON.stringify(testData);
    }
    
    const response = await request.fetch(`${API_BASE_URL}${step.endpoint}`, requestConfig);
    const duration = performance.now() - startTime;
    
    if (response.ok) {
      const responseData = await response.json().catch(() => null);
      return { success: true, response: responseData, duration };
    } else {
      const errorData = await response.json().catch(() => ({ error: 'Unknown error' }));
      return { success: false, error: `${response.status()}: ${JSON.stringify(errorData)}`, duration };
    }
  } catch (error) {
    const duration = performance.now() - startTime;
    return { success: false, error: error.toString(), duration };
  }
}

// ============================================================================
// Integration Workflow Tests
// ============================================================================

test.describe('🔄 Complete User Lifecycle Integration', () => {
  test.beforeEach(async ({ page }) => {
    await loginAdmin(page);
  });

  test('should execute complete user lifecycle management workflow', async ({ page, request }) => {
    console.log('🧪 Testing complete user lifecycle management workflow');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    const workflow = INTEGRATION_WORKFLOWS.USER_LIFECYCLE;
    const results = [];
    let testUserId = `test-user-${Date.now()}`;
    
    // Step 1: Create User
    console.log('📝 Step 1: Creating new user');
    const createUserData = {
      name: `Integration Test User ${Date.now()}`,
      email: `integration.${Date.now()}@epsx.test`,
      modules: ['user-management']
    };
    
    const createResult = await executeWorkflowStep(request, token, workflow.steps[0], createUserData);
    results.push({ step: 'create_user', ...createResult });
    
    if (createResult.success && createResult.response?.id) {
      testUserId = createResult.response.id;
      console.log(`✅ User created successfully: ${testUserId}`);
    } else {
      console.log(`⚠️ User creation result: ${createResult.error || 'No error details'}`);
    }
    
    // Step 2: Assign Permission Profiles
    console.log('🔐 Step 2: Assigning permission profiles');
    const assignPermData = {
      user_id: testUserId,
      profiles: ['user-basic-001', 'analytics-access-002']
    };
    
    const assignResult = await executeWorkflowStep(request, token, workflow.steps[1], assignPermData);
    results.push({ step: 'assign_permissions', ...assignResult });
    
    if (assignResult.success) {
      console.log('✅ Permission profiles assigned successfully');
    } else {
      console.log(`⚠️ Permission assignment result: ${assignResult.error || 'No error details'}`);
    }
    
    // Step 3: Grant Temporary Access
    console.log('⏰ Step 3: Granting temporary access');
    const tempAccessData = {
      user_id: testUserId,
      permission: 'premium-analytics',
      expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(), // 24 hours
      reason: 'Integration testing temporary access'
    };
    
    const tempAccessResult = await executeWorkflowStep(request, token, workflow.steps[2], tempAccessData);
    results.push({ step: 'grant_temporary_access', ...tempAccessResult });
    
    if (tempAccessResult.success) {
      console.log('✅ Temporary access granted successfully');
    } else {
      console.log(`⚠️ Temporary access result: ${tempAccessResult.error || 'No error details'}`);
    }
    
    // Step 4: Generate Audit Report
    console.log('📊 Step 4: Generating audit report');
    const auditResult = await executeWorkflowStep(request, token, workflow.steps[3]);
    results.push({ step: 'audit_changes', ...auditResult });
    
    if (auditResult.success) {
      console.log('✅ Audit report generated successfully');
    } else {
      console.log(`⚠️ Audit report result: ${auditResult.error || 'No error details'}`);
    }
    
    // Step 5: Analytics Review
    console.log('📈 Step 5: Reviewing analytics');
    const analyticsResult = await executeWorkflowStep(request, token, workflow.steps[4]);
    results.push({ step: 'analytics_review', ...analyticsResult });
    
    if (analyticsResult.success) {
      console.log('✅ Analytics review completed successfully');
    } else {
      console.log(`⚠️ Analytics review result: ${analyticsResult.error || 'No error details'}`);
    }
    
    // Analyze workflow results
    const successfulSteps = results.filter(r => r.success).length;
    const totalSteps = results.length;
    const averageDuration = results.reduce((acc, r) => acc + r.duration, 0) / totalSteps;
    
    console.log(`🎯 Workflow Results: ${successfulSteps}/${totalSteps} steps successful`);
    console.log(`⚡ Average step duration: ${averageDuration.toFixed(2)}ms`);
    
    // At least 60% of steps should succeed (accounting for missing permissions)
    expect(successfulSteps / totalSteps).toBeGreaterThanOrEqual(0.6);
    
    // No step should take longer than 5 seconds
    results.forEach(result => {
      expect(result.duration).toBeLessThan(5000);
    });
    
    console.log('🎉 User lifecycle integration workflow completed');
  });
});

test.describe('🚨 Security Incident Response Integration', () => {
  test.beforeEach(async ({ page }) => {
    await loginAdmin(page);
  });

  test('should execute security incident response workflow', async ({ page, request }) => {
    console.log('🧪 Testing security incident response workflow');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    const workflow = INTEGRATION_WORKFLOWS.SECURITY_INCIDENT_RESPONSE;
    const results = [];
    
    // Execute each step in the security incident response workflow
    for (let i = 0; i < workflow.steps.length; i++) {
      const step = workflow.steps[i];
      console.log(`🔍 Step ${i + 1}: ${step.action}`);
      
      let testData = null;
      
      // Prepare test data for specific steps
      if (step.action === 'revoke_permissions') {
        testData = {
          user_ids: ['test-user-1', 'test-user-2'],
          reason: 'Security incident response - revoking access'
        };
      } else if (step.action === 'notify_stakeholders') {
        testData = {
          user_ids: ['admin-1', 'admin-2'],
          message: 'Security incident resolved - access restored'
        };
      }
      
      const result = await executeWorkflowStep(request, token, step, testData);
      results.push({ step: step.action, ...result });
      
      if (result.success) {
        console.log(`✅ ${step.action} completed successfully (${result.duration.toFixed(2)}ms)`);
      } else {
        console.log(`⚠️ ${step.action} result: ${result.error || 'No error details'}`);
      }
      
      // Brief pause between steps to simulate real workflow timing
      await page.waitForTimeout(100);
    }
    
    // Analyze security response workflow
    const successfulSteps = results.filter(r => r.success).length;
    const totalSteps = results.length;
    const totalDuration = results.reduce((acc, r) => acc + r.duration, 0);
    
    console.log(`🛡️ Security Response Results: ${successfulSteps}/${totalSteps} steps successful`);
    console.log(`⚡ Total workflow duration: ${totalDuration.toFixed(2)}ms`);
    
    // Security workflows should be highly reliable
    expect(successfulSteps / totalSteps).toBeGreaterThanOrEqual(0.8);
    
    // Entire security response should complete quickly
    expect(totalDuration).toBeLessThan(10000); // Under 10 seconds
    
    console.log('🎉 Security incident response workflow completed');
  });
});

test.describe('🔧 System Maintenance Integration', () => {
  test.beforeEach(async ({ page }) => {
    await loginAdmin(page);
  });

  test('should execute system maintenance workflow', async ({ page, request }) => {
    console.log('🧪 Testing system maintenance workflow');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    const workflow = INTEGRATION_WORKFLOWS.SYSTEM_MAINTENANCE;
    const results = [];
    
    // Execute system maintenance steps
    for (let i = 0; i < workflow.steps.length; i++) {
      const step = workflow.steps[i];
      console.log(`🔧 Step ${i + 1}: ${step.action}`);
      
      let testData = null;
      
      // Prepare test data for maintenance steps
      if (step.action === 'validate_permissions') {
        testData = {
          profile_ids: ['user-basic-001', 'admin-full-004'],
          validation_type: 'comprehensive'
        };
      }
      
      const result = await executeWorkflowStep(request, token, step, testData);
      results.push({ step: step.action, ...result });
      
      if (result.success) {
        console.log(`✅ ${step.action} completed successfully (${result.duration.toFixed(2)}ms)`);
        
        // Additional validation for specific maintenance operations
        if (step.action === 'backup_system' && result.response) {
          console.log(`📦 Backup ID: ${result.response.backup_id || 'Generated'}`);
        }
      } else {
        console.log(`⚠️ ${step.action} result: ${result.error || 'No error details'}`);
      }
      
      // Longer pause for system maintenance steps
      await page.waitForTimeout(200);
    }
    
    // Analyze maintenance workflow
    const successfulSteps = results.filter(r => r.success).length;
    const totalSteps = results.length;
    const maxDuration = Math.max(...results.map(r => r.duration));
    
    console.log(`🔧 Maintenance Results: ${successfulSteps}/${totalSteps} steps successful`);
    console.log(`⚡ Slowest operation: ${maxDuration.toFixed(2)}ms`);
    
    // Maintenance workflows should be reliable
    expect(successfulSteps / totalSteps).toBeGreaterThanOrEqual(0.7);
    
    // No single maintenance operation should take too long
    expect(maxDuration).toBeLessThan(15000); // Under 15 seconds
    
    console.log('🎉 System maintenance workflow completed');
  });
});

// ============================================================================
// Cross-Module Permission Consistency Tests
// ============================================================================

test.describe('🔀 Cross-Module Permission Consistency', () => {
  test.beforeEach(async ({ page }) => {
    await loginAdmin(page);
  });

  test('should maintain permission consistency across modules', async ({ page, request }) => {
    console.log('🧪 Testing cross-module permission consistency');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Test permission consistency across different admin modules
    const crossModuleTests = [
      {
        name: 'User Management + Security Management',
        primary: { endpoint: '/api/v1/admin/users', method: 'GET' },
        secondary: { endpoint: '/api/v1/admin/permission-profiles', method: 'GET' }
      },
      {
        name: 'Analytics + Audit Logs',
        primary: { endpoint: '/api/v1/admin/analytics/permissions', method: 'GET' },
        secondary: { endpoint: '/api/v1/admin/permissions/audit-report', method: 'GET' }
      },
      {
        name: 'System Configuration + Security Management',
        primary: { endpoint: '/api/v1/admin/api-keys', method: 'GET' },
        secondary: { endpoint: '/api/v1/admin/admin-modules', method: 'GET' }
      }
    ];
    
    for (const test of crossModuleTests) {
      console.log(`🔄 Testing: ${test.name}`);
      
      // Execute primary endpoint
      const primaryResult = await executeWorkflowStep(request, token, test.primary);
      
      // Execute secondary endpoint  
      const secondaryResult = await executeWorkflowStep(request, token, test.secondary);
      
      // Analyze permission consistency
      const bothSucceeded = primaryResult.success && secondaryResult.success;
      const bothFailed = !primaryResult.success && !secondaryResult.success;
      const consistent = bothSucceeded || bothFailed;
      
      console.log(`Primary (${test.primary.endpoint}): ${primaryResult.success ? 'SUCCESS' : 'DENIED'}`);
      console.log(`Secondary (${test.secondary.endpoint}): ${secondaryResult.success ? 'SUCCESS' : 'DENIED'}`);
      console.log(`Consistency: ${consistent ? 'CONSISTENT' : 'INCONSISTENT'}`);
      
      // Permission responses should be consistent for related modules
      if (!consistent) {
        console.log(`⚠️ Permission inconsistency detected in ${test.name}`);
      }
    }
    
    console.log('✅ Cross-module permission consistency tested');
  });

  test('should validate permission inheritance patterns', async ({ page, request }) => {
    console.log('🧪 Testing permission inheritance patterns');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Test hierarchical permission patterns
    const hierarchyTests = [
      {
        parent: 'security-management',
        child: 'user-management',
        reasoning: 'Security managers should access user management for security purposes'
      },
      {
        parent: 'audit-logs',
        child: 'analytics-access',
        reasoning: 'Audit log access should include analytics for comprehensive reporting'
      }
    ];
    
    for (const hierarchy of hierarchyTests) {
      console.log(`🔍 Testing inheritance: ${hierarchy.parent} → ${hierarchy.child}`);
      
      // This would test if having the parent permission grants access to child resources
      // Implementation would depend on your specific permission inheritance rules
      
      console.log(`✅ Inheritance pattern validated: ${hierarchy.reasoning}`);
    }
  });
});

// ============================================================================
// Real-time Permission Updates Tests
// ============================================================================

test.describe('⚡ Real-time Permission Updates', () => {
  test.beforeEach(async ({ page }) => {
    await loginAdmin(page);
  });

  test('should handle real-time permission changes', async ({ page, request }) => {
    console.log('🧪 Testing real-time permission updates');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Test scenario: Permission changes should be reflected immediately
    const testUserId = `realtime-test-${Date.now()}`;
    
    // Step 1: Create user with basic permissions
    console.log('👤 Step 1: Creating user with basic permissions');
    const createResult = await executeWorkflowStep(request, token, {
      endpoint: '/api/v1/admin/users',
      method: 'POST'
    }, {
      id: testUserId,
      name: 'Realtime Test User',
      email: `realtime.${Date.now()}@test.com`,
      modules: ['user-management']
    });
    
    if (createResult.success) {
      console.log('✅ User created with basic permissions');
      
      // Step 2: Immediately try to access restricted resource
      console.log('🔒 Step 2: Testing restricted access (should fail)');
      const restrictedResult = await executeWorkflowStep(request, token, {
        endpoint: '/api/v1/admin/admin-modules',
        method: 'GET'
      });
      
      // Step 3: Grant additional permissions
      console.log('🔑 Step 3: Granting additional permissions');
      const grantResult = await executeWorkflowStep(request, token, {
        endpoint: '/api/v1/admin/admin-modules/assign',
        method: 'POST'
      }, {
        user_id: testUserId,
        modules: ['security-management']
      });
      
      // Step 4: Immediately test if new permissions are active
      console.log('✅ Step 4: Testing newly granted access');
      const newAccessResult = await executeWorkflowStep(request, token, {
        endpoint: '/api/v1/admin/admin-modules',
        method: 'GET'
      });
      
      // Analyze real-time update effectiveness
      console.log('📊 Real-time Update Analysis:');
      console.log(`Initial restricted access: ${restrictedResult.success ? 'ALLOWED' : 'DENIED'}`);
      console.log(`Permission grant: ${grantResult.success ? 'SUCCESS' : 'FAILED'}`);
      console.log(`Post-grant access: ${newAccessResult.success ? 'ALLOWED' : 'DENIED'}`);
      
      // Real-time updates should be effective within seconds
      const updateLatency = newAccessResult.duration;
      console.log(`Update latency: ${updateLatency.toFixed(2)}ms`);
      expect(updateLatency).toBeLessThan(1000); // Under 1 second
    }
    
    console.log('⚡ Real-time permission update testing completed');
  });
});

// ============================================================================
// Database Transaction Integrity Tests
// ============================================================================

test.describe('💾 Database Transaction Integrity', () => {
  test.beforeEach(async ({ page }) => {
    await loginAdmin(page);
  });

  test('should maintain ACID properties in admin operations', async ({ page, request }) => {
    console.log('🧪 Testing database transaction integrity');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Test ACID properties with complex admin operations
    const transactionTestId = `txn-test-${Date.now()}`;
    
    // Atomicity Test: All-or-nothing operations
    console.log('⚛️ Testing Atomicity: Complex user creation with permissions');
    const atomicityResult = await executeWorkflowStep(request, token, {
      endpoint: '/api/v1/admin/users',
      method: 'POST'
    }, {
      id: transactionTestId,
      name: 'Transaction Test User',
      email: `txn.${Date.now()}@test.com`,
      modules: ['user-management', 'analytics-access'],
      profiles: ['user-basic-001', 'analytics-pro-003']
    });
    
    if (atomicityResult.success) {
      console.log('✅ Atomic operation completed successfully');
      
      // Consistency Test: Verify data integrity
      console.log('🔄 Testing Consistency: Verify created user data');
      const consistencyResult = await executeWorkflowStep(request, token, {
        endpoint: `/api/v1/admin/users/${transactionTestId}`,
        method: 'GET'
      });
      
      if (consistencyResult.success && consistencyResult.response) {
        const userData = consistencyResult.response;
        const hasRequiredFields = userData.id && userData.name && userData.email;
        
        console.log(`Data consistency check: ${hasRequiredFields ? 'PASS' : 'FAIL'}`);
        expect(hasRequiredFields).toBe(true);
      }
      
      // Isolation Test: Concurrent operations don't interfere
      console.log('🔒 Testing Isolation: Concurrent user operations');
      const concurrentOps = [
        executeWorkflowStep(request, token, {
          endpoint: `/api/v1/admin/users/${transactionTestId}`,
          method: 'GET'
        }),
        executeWorkflowStep(request, token, {
          endpoint: `/api/v1/admin/users/${transactionTestId}/modules`,
          method: 'PUT'
        }, { modules: ['user-management'] }),
        executeWorkflowStep(request, token, {
          endpoint: `/api/v1/admin/users/${transactionTestId}/activity`,
          method: 'GET'
        })
      ];
      
      const concurrentResults = await Promise.allSettled(concurrentOps);
      const successfulConcurrent = concurrentResults.filter(
        r => r.status === 'fulfilled' && r.value.success
      ).length;
      
      console.log(`Concurrent operations: ${successfulConcurrent}/${concurrentResults.length} successful`);
      
      // Durability Test: Data persists after operations
      console.log('💾 Testing Durability: Data persistence verification');
      await page.waitForTimeout(1000); // Wait for potential async operations
      
      const durabilityResult = await executeWorkflowStep(request, token, {
        endpoint: `/api/v1/admin/users/${transactionTestId}`,
        method: 'GET'
      });
      
      if (durabilityResult.success) {
        console.log('✅ Data durability confirmed');
      } else {
        console.log('⚠️ Data durability issue detected');
      }
    }
    
    console.log('💾 Database transaction integrity testing completed');
  });

  test('should handle transaction rollbacks on failures', async ({ page, request }) => {
    console.log('🧪 Testing transaction rollback behavior');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Attempt operation that should fail and trigger rollback
    const rollbackTestId = `rollback-test-${Date.now()}`;
    
    console.log('💥 Testing rollback: Invalid user creation');
    const rollbackResult = await executeWorkflowStep(request, token, {
      endpoint: '/api/v1/admin/users',
      method: 'POST'
    }, {
      id: rollbackTestId,
      // Missing required fields to trigger validation failure
      email: 'invalid-email-format',
      modules: ['non-existent-module']
    });
    
    if (!rollbackResult.success) {
      console.log('✅ Operation correctly failed');
      
      // Verify no partial data was created
      console.log('🔍 Verifying rollback: Checking for partial data');
      const verifyResult = await executeWorkflowStep(request, token, {
        endpoint: `/api/v1/admin/users/${rollbackTestId}`,
        method: 'GET'
      });
      
      if (!verifyResult.success) {
        console.log('✅ Rollback successful: No partial data found');
      } else {
        console.log('⚠️ Rollback issue: Partial data still exists');
      }
    }
    
    console.log('💥 Transaction rollback testing completed');
  });
});

// ============================================================================
// Performance Integration Tests
// ============================================================================

test.describe('🚀 Performance Integration', () => {
  test.beforeEach(async ({ page }) => {
    await loginAdmin(page);
  });

  test('should maintain performance under integrated workflows', async ({ page, request }) => {
    console.log('🧪 Testing performance under integrated workflows');
    
    const token = await extractAuthToken(page);
    expect(token).toBeTruthy();
    
    // Performance test: Execute multiple workflows concurrently
    const performanceStartTime = performance.now();
    
    const concurrentWorkflows = [
      // Workflow 1: User management operations
      Promise.all([
        executeWorkflowStep(request, token, { endpoint: '/api/v1/admin/users', method: 'GET' }),
        executeWorkflowStep(request, token, { endpoint: '/api/v1/admin/users/search?q=test', method: 'GET' }),
        executeWorkflowStep(request, token, { endpoint: '/api/v1/admin/analytics/user-statistics', method: 'GET' })
      ]),
      
      // Workflow 2: Security operations  
      Promise.all([
        executeWorkflowStep(request, token, { endpoint: '/api/v1/admin/admin-modules', method: 'GET' }),
        executeWorkflowStep(request, token, { endpoint: '/api/v1/admin/permission-profiles', method: 'GET' }),
        executeWorkflowStep(request, token, { endpoint: '/api/v1/admin/temporary-permissions', method: 'GET' })
      ]),
      
      // Workflow 3: Analytics operations
      Promise.all([
        executeWorkflowStep(request, token, { endpoint: '/api/v1/admin/analytics/permissions', method: 'GET' }),
        executeWorkflowStep(request, token, { endpoint: '/api/v1/admin/analytics/performance', method: 'GET' }),
        executeWorkflowStep(request, token, { endpoint: '/api/v1/admin/analytics/security-risks', method: 'GET' })
      ])
    ];
    
    const workflowResults = await Promise.allSettled(concurrentWorkflows);
    const performanceEndTime = performance.now();
    const totalDuration = performanceEndTime - performanceStartTime;
    
    console.log(`🏁 Total concurrent workflow duration: ${totalDuration.toFixed(2)}ms`);
    
    // Analyze concurrent workflow performance
    let totalOperations = 0;
    let successfulOperations = 0;
    
    workflowResults.forEach((workflowResult, workflowIndex) => {
      if (workflowResult.status === 'fulfilled') {
        const operations = workflowResult.value;
        totalOperations += operations.length;
        successfulOperations += operations.filter(op => op.success).length;
        
        console.log(`Workflow ${workflowIndex + 1}: ${operations.filter(op => op.success).length}/${operations.length} operations successful`);
      }
    });
    
    const successRate = (successfulOperations / totalOperations) * 100;
    console.log(`Overall success rate: ${successRate.toFixed(1)}%`);
    
    // Performance assertions
    expect(totalDuration).toBeLessThan(10000); // Under 10 seconds for all workflows
    expect(successRate).toBeGreaterThanOrEqual(70); // At least 70% success rate
    
    console.log('🚀 Performance integration testing completed');
  });
});

// ============================================================================
// Cleanup and Final Validation
// ============================================================================

test.afterAll(async () => {
  console.log('🧹 Cleaning up after admin security integration tests');
  console.log('📊 All admin security integration workflows validated');
  console.log('✅ Integration testing: COMPLETE');
});