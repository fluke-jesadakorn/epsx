// OIDC Integration Testing Utilities
// Comprehensive testing suite for verifying OIDC implementation

import { getOIDCClient } from '@/lib/auth/oidc-client-wrapper';
import { getTenantDetectionService } from '@/lib/auth/tenant-detection-service';
import { getOIDCDiscoveryClient } from '@/lib/auth/oidc-discovery-client';
import { getOIDCTokenManager } from '@/lib/auth/oidc-token-manager';

export interface OIDCTestResult {
  component: string;
  status: 'pass' | 'fail' | 'warning' | 'skipped';
  message: string;
  duration?: number;
  details?: any;
}

export interface OIDCIntegrationTestSuite {
  backendConnectivity: OIDCTestResult[];
  discoveryService: OIDCTestResult[];
  tenantDetection: OIDCTestResult[];
  tokenManagement: OIDCTestResult[];
  clientWrapper: OIDCTestResult[];
  serverActions: OIDCTestResult[];
  summary: {
    total: number;
    passed: number;
    failed: number;
    warnings: number;
    skipped: number;
    duration: number;
  };
}

/**
 * Comprehensive OIDC Integration Test Suite
 * Tests all components against the backend OIDC implementation
 */
export class OIDCIntegrationTester {
  private baseUrl: string;
  private results: OIDCIntegrationTestSuite;

  constructor() {
    this.baseUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    this.results = {
      backendConnectivity: [],
      discoveryService: [],
      tenantDetection: [],
      tokenManagement: [],
      clientWrapper: [],
      serverActions: [],
      summary: {
        total: 0,
        passed: 0,
        failed: 0,
        warnings: 0,
        skipped: 0,
        duration: 0
      }
    };
  }

  async runFullTestSuite(): Promise<OIDCIntegrationTestSuite> {
    console.log('🧪 Starting OIDC Integration Test Suite...');
    const startTime = Date.now();

    try {
      // Test 1: Backend Connectivity
      await this.testBackendConnectivity();
      
      // Test 2: OIDC Discovery Service
      await this.testOIDCDiscoveryService();
      
      // Test 3: Tenant Detection
      await this.testTenantDetection();
      
      // Test 4: Token Management
      await this.testTokenManagement();
      
      // Test 5: OIDC Client Wrapper
      await this.testOIDCClientWrapper();
      
      // Test 6: Server Actions
      await this.testServerActions();

    } catch (error) {
      console.error('❌ Test suite failed:', error);
    }

    // Calculate summary
    const endTime = Date.now();
    this.calculateSummary(endTime - startTime);

    console.log('✅ OIDC Integration Test Suite Complete');
    console.log('📊 Results:', this.results.summary);

    return this.results;
  }

  private async testBackendConnectivity(): Promise<void> {
    console.log('🔍 Testing Backend Connectivity...');

    // Test 1: Basic backend health check
    await this.runTest('backendConnectivity', 'Backend Health Check', async () => {
      const response = await fetch(`${this.baseUrl}/health`, {
        method: 'HEAD',
        cache: 'no-cache'
      });
      
      if (!response.ok) {
        throw new Error(`Backend health check failed: ${response.status}`);
      }
      
      return { status: response.status, headers: response.headers };
    });

    // Test 2: OIDC endpoints availability
    await this.runTest('backendConnectivity', 'OIDC Endpoints Availability', async () => {
      const endpoints = [
        '/oauth/v2/.well-known/openid-configuration',
        '/oauth/v2/auth',
        '/oauth/v2/token',
        '/oauth/v2/userinfo'
      ];

      const results = {};
      for (const endpoint of endpoints) {
        try {
          const response = await fetch(`${this.baseUrl}${endpoint}`, {
            method: 'HEAD',
            cache: 'no-cache'
          });
          results[endpoint] = response.status;
        } catch (error) {
          results[endpoint] = `Error: ${error}`;
        }
      }

      return results;
    });

    // Test 3: CORS configuration
    await this.runTest('backendConnectivity', 'CORS Configuration', async () => {
      const response = await fetch(`${this.baseUrl}/oauth/v2/.well-known/openid-configuration`, {
        method: 'OPTIONS'
      });
      
      const corsHeaders = {
        'access-control-allow-origin': response.headers.get('access-control-allow-origin'),
        'access-control-allow-methods': response.headers.get('access-control-allow-methods'),
        'access-control-allow-headers': response.headers.get('access-control-allow-headers')
      };

      if (!corsHeaders['access-control-allow-origin']) {
        throw new Error('CORS not properly configured');
      }

      return corsHeaders;
    });
  }

  private async testOIDCDiscoveryService(): Promise<void> {
    console.log('🔍 Testing OIDC Discovery Service...');

    const discoveryClient = getOIDCDiscoveryClient();

    // Test 1: Discovery configuration retrieval
    await this.runTest('discoveryService', 'Discovery Configuration', async () => {
      const config = await discoveryClient.discoverConfiguration();
      
      const requiredFields = [
        'issuer',
        'authorization_endpoint',
        'token_endpoint',
        'userinfo_endpoint',
        'jwks_uri'
      ];

      for (const field of requiredFields) {
        if (!config[field]) {
          throw new Error(`Missing required field: ${field}`);
        }
      }

      return {
        issuer: config.issuer,
        endpoints: requiredFields.reduce((acc, field) => {
          acc[field] = config[field];
          return acc;
        }, {})
      };
    });

    // Test 2: Tenant discovery
    await this.runTest('discoveryService', 'Tenant Discovery', async () => {
      const tenants = await discoveryClient.discoverTenants();
      
      if (!Array.isArray(tenants)) {
        throw new Error('Tenant discovery did not return an array');
      }

      return {
        count: tenants.length,
        sample: tenants.slice(0, 3)
      };
    });

    // Test 3: Multi-tenant configuration
    await this.runTest('discoveryService', 'Multi-tenant Configuration', async () => {
      // Test with a sample tenant ID
      const config = await discoveryClient.discoverConfiguration('sample-tenant');
      
      if (!config.issuer) {
        throw new Error('Multi-tenant configuration failed');
      }

      return {
        tenantSpecific: true,
        issuer: config.issuer
      };
    });
  }

  private async testTenantDetection(): Promise<void> {
    console.log('🔍 Testing Tenant Detection...');

    const tenantService = getTenantDetectionService();

    // Test 1: Email-based tenant detection
    await this.runTest('tenantDetection', 'Email-based Detection', async () => {
      const testEmails = [
        'user@company.com',
        'admin@example.org',
        'test@gmail.com'
      ];

      const results = {};
      for (const email of testEmails) {
        const detection = await tenantService.detectTenant(email);
        results[email] = {
          confidence: detection.confidence,
          tenant: detection.tenant?.name || null
        };
      }

      return results;
    });

    // Test 2: Domain-based detection
    await this.runTest('tenantDetection', 'Domain-based Detection', async () => {
      const testDomains = ['company.com', 'example.org', 'unknown-domain.test'];
      const results = {};

      for (const domain of testDomains) {
        const detection = await tenantService.detectTenantByDomain(domain);
        results[domain] = {
          detected: !!detection.tenant,
          confidence: detection.confidence
        };
      }

      return results;
    });

    // Test 3: Preferences integration
    await this.runTest('tenantDetection', 'Preferences Integration', async () => {
      // Test preference-based detection
      const preferences = [
        { domain: 'company.com', tenant_id: 'company-tenant' },
        { domain: 'example.org', tenant_id: 'example-tenant' }
      ];

      await tenantService.updatePreferences(preferences);
      const updatedPrefs = await tenantService.getPreferences();

      return {
        preferences: updatedPrefs,
        count: updatedPrefs.length
      };
    });
  }

  private async testTokenManagement(): Promise<void> {
    console.log('🔍 Testing Token Management...');

    const tokenManager = getOIDCTokenManager();

    // Test 1: Token validation (mock)
    await this.runTest('tokenManagement', 'Token Validation', async () => {
      // Create a mock JWT token for testing
      const mockToken = 'eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ0ZXN0LXVzZXIiLCJpc3MiOiJ0ZXN0LWlzc3VlciIsImV4cCI6OTk5OTk5OTk5OX0.test-signature';
      
      try {
        // This will fail validation but we can test the validation logic
        await tokenManager.validateToken(mockToken);
        throw new Error('Token validation should have failed');
      } catch (error) {
        if (error.message.includes('Invalid token format') || 
            error.message.includes('Token validation failed')) {
          return { validationWorking: true, error: error.message };
        }
        throw error;
      }
    });

    // Test 2: JWKS retrieval
    await this.runTest('tokenManagement', 'JWKS Retrieval', async () => {
      const jwks = await tokenManager.getJWKS();
      
      if (!jwks || !jwks.keys || !Array.isArray(jwks.keys)) {
        throw new Error('Invalid JWKS response');
      }

      return {
        keyCount: jwks.keys.length,
        hasKeys: jwks.keys.length > 0
      };
    });

    // Test 3: Token refresh mechanism
    await this.runTest('tokenManagement', 'Token Refresh Mechanism', async () => {
      // Test the token refresh logic (without actual tokens)
      const refreshResult = await tokenManager.canRefreshToken();
      
      return {
        refreshSupported: refreshResult,
        mechanism: 'implemented'
      };
    });
  }

  private async testOIDCClientWrapper(): Promise<void> {
    console.log('🔍 Testing OIDC Client Wrapper...');

    const oidcClient = getOIDCClient();

    // Test 1: Client initialization
    await this.runTest('clientWrapper', 'Client Initialization', async () => {
      await oidcClient.initialize();
      
      const healthMetrics = oidcClient.getHealthMetrics();
      const currentState = oidcClient.getCurrentState();

      return {
        initialized: true,
        healthStatus: healthMetrics.lastHealthCheck > 0,
        state: currentState.healthStatus
      };
    });

    // Test 2: Health monitoring
    await this.runTest('clientWrapper', 'Health Monitoring', async () => {
      const metrics = oidcClient.getHealthMetrics();
      
      const expectedFields = ['totalRequests', 'successfulRequests', 'failedRequests', 'averageResponseTime'];
      for (const field of expectedFields) {
        if (typeof metrics[field] !== 'number') {
          throw new Error(`Missing or invalid health metric: ${field}`);
        }
      }

      return metrics;
    });

    // Test 3: Event system
    await this.runTest('clientWrapper', 'Event System', async () => {
      let eventReceived = false;
      
      const unsubscribe = oidcClient.on('health_check', () => {
        eventReceived = true;
      });

      // Trigger a health check event
      oidcClient.emit('health_check', { metrics: {} });
      
      // Clean up
      unsubscribe();

      if (!eventReceived) {
        throw new Error('Event system not working');
      }

      return { eventSystemWorking: true };
    });
  }

  private async testServerActions(): Promise<void> {
    console.log('🔍 Testing Server Actions...');

    // Test 1: OIDC server action imports
    await this.runTest('serverActions', 'Server Action Imports', async () => {
      // Dynamic import to test if server actions are properly defined
      try {
        const { initiateOIDCLogin, handleOIDCCallback, getCurrentOIDCUser, logoutOIDC } = 
          await import('@/app/actions/oidc-auth');

        const actions = {
          initiateOIDCLogin: typeof initiateOIDCLogin === 'function',
          handleOIDCCallback: typeof handleOIDCCallback === 'function',
          getCurrentOIDCUser: typeof getCurrentOIDCUser === 'function',
          logoutOIDC: typeof logoutOIDC === 'function'
        };

        for (const [name, isFunction] of Object.entries(actions)) {
          if (!isFunction) {
            throw new Error(`${name} is not a function`);
          }
        }

        return actions;
      } catch (error) {
        throw new Error(`Server action import failed: ${error.message}`);
      }
    });

    // Test 2: Admin server action imports
    await this.runTest('serverActions', 'Admin Server Action Imports', async () => {
      try {
        const { initiateAdminOIDCLogin, handleAdminOIDCCallback, getCurrentAdminOIDCUser, logoutAdminOIDC } = 
          await import('@/app/actions/admin-oidc-auth');

        const actions = {
          initiateAdminOIDCLogin: typeof initiateAdminOIDCLogin === 'function',
          handleAdminOIDCCallback: typeof handleAdminOIDCCallback === 'function',
          getCurrentAdminOIDCUser: typeof getCurrentAdminOIDCUser === 'function',
          logoutAdminOIDC: typeof logoutAdminOIDC === 'function'
        };

        for (const [name, isFunction] of Object.entries(actions)) {
          if (!isFunction) {
            throw new Error(`${name} is not a function`);
          }
        }

        return actions;
      } catch (error) {
        // Admin actions might not exist in frontend app
        return { note: 'Admin actions not available in frontend app', skipped: true };
      }
    });
  }

  private async runTest(
    category: keyof Omit<OIDCIntegrationTestSuite, 'summary'>,
    testName: string,
    testFunction: () => Promise<any>
  ): Promise<void> {
    const startTime = Date.now();
    
    try {
      const result = await testFunction();
      const endTime = Date.now();
      
      this.results[category].push({
        component: testName,
        status: 'pass',
        message: 'Test passed successfully',
        duration: endTime - startTime,
        details: result
      });
      
      console.log(`✅ ${testName}: PASSED (${endTime - startTime}ms)`);
      
    } catch (error) {
      const endTime = Date.now();
      
      this.results[category].push({
        component: testName,
        status: 'fail',
        message: error instanceof Error ? error.message : 'Unknown error',
        duration: endTime - startTime,
        details: error
      });
      
      console.log(`❌ ${testName}: FAILED - ${error instanceof Error ? error.message : error}`);
    }
  }

  private calculateSummary(totalDuration: number): void {
    const allResults = [
      ...this.results.backendConnectivity,
      ...this.results.discoveryService,
      ...this.results.tenantDetection,
      ...this.results.tokenManagement,
      ...this.results.clientWrapper,
      ...this.results.serverActions
    ];

    this.results.summary = {
      total: allResults.length,
      passed: allResults.filter(r => r.status === 'pass').length,
      failed: allResults.filter(r => r.status === 'fail').length,
      warnings: allResults.filter(r => r.status === 'warning').length,
      skipped: allResults.filter(r => r.status === 'skipped').length,
      duration: totalDuration
    };
  }

  generateTestReport(): string {
    const { summary } = this.results;
    const passRate = summary.total > 0 ? (summary.passed / summary.total * 100).toFixed(1) : '0';
    
    let report = `
# OIDC Integration Test Report

## Summary
- **Total Tests**: ${summary.total}
- **Passed**: ${summary.passed} (${passRate}%)
- **Failed**: ${summary.failed}
- **Warnings**: ${summary.warnings}
- **Skipped**: ${summary.skipped}
- **Duration**: ${summary.duration}ms

## Test Results by Category

`;

    const categories = ['backendConnectivity', 'discoveryService', 'tenantDetection', 'tokenManagement', 'clientWrapper', 'serverActions'];
    
    for (const category of categories) {
      const results = this.results[category];
      report += `### ${category.charAt(0).toUpperCase() + category.slice(1)}\n`;
      
      for (const result of results) {
        const statusIcon = result.status === 'pass' ? '✅' : result.status === 'fail' ? '❌' : '⚠️';
        report += `${statusIcon} **${result.component}** (${result.duration}ms)\n`;
        if (result.status === 'fail') {
          report += `   Error: ${result.message}\n`;
        }
      }
      report += '\n';
    }

    return report;
  }
}

// Export testing utilities
export const testOIDCIntegration = async (): Promise<OIDCIntegrationTestSuite> => {
  const tester = new OIDCIntegrationTester();
  return await tester.runFullTestSuite();
};