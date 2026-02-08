/**
 * Global Setup for Comprehensive E2E Tests
 * Initializes test environment, validates services, and prepares test data
 */

import { test as setup, expect, chromium } from '@playwright/test';
import { initializeTestUsers } from '../fixtures/user-fixtures';
import { URL, URLContext, Service } from '@/shared/utils/url-resolver';

const BASE_URL = URL.get(Service.FRONTEND, URLContext.CLIENT);
const API_URL = URL.get(Service.BACKEND, URLContext.CLIENT);
const ADMIN_URL = URL.get(Service.ADMIN, URLContext.CLIENT);

setup('🔧 Global Setup - Service Health Check', async ({ page }) => {
  console.log('🚀 Starting comprehensive global setup...');
  
  // Initialize test users with JWT tokens
  initializeTestUsers();
  console.log('👥 Test users initialized with JWT tokens');
  
  // Check if backend is running
  await checkBackendHealth(page);
  
  // Check if frontend is accessible
  await checkFrontendHealth(page);
  
  // Check if admin frontend is accessible
  await checkAdminHealth(page);
  
  // Verify OIDC endpoint is responsive
  await checkOIDCHealth(page);
  
  // Validate critical API endpoints
  await validateCriticalEndpoints(page);
  
  // Check for JavaScript errors in base application
  await checkJavaScriptHealth(page);
  
  // Verify responsive design basics
  await checkResponsiveBasics(page);
  
  // Setup test environment variables
  await setupTestEnvironment();
  
  console.log('🏁 Comprehensive global setup completed successfully');
});

/**
 * Check backend service health
 */
async function checkBackendHealth(page: any): Promise<void> {
  try {
    const response = await page.request.get(`${API_URL}/health`);
    if (response.status() === 200) {
      console.log('✅ Backend service is running');
      
      // Check backend version and capabilities
      try {
        const healthData = await response.json();
        console.log(`📊 Backend version: ${healthData.version || 'unknown'}`);
        console.log(`🗄️  Database: ${healthData.database ? 'connected' : 'disconnected'}`);
        console.log(`🔴 Redis: ${healthData.redis ? 'connected' : 'disconnected'}`);
      } catch (e) {
        console.log('✅ Backend health endpoint accessible (no JSON response)');
      }
    } else {
      console.warn('⚠️ Backend health check returned:', response.status());
    }
  } catch (error) {
    console.warn('⚠️ Backend service may not be running:', error);
  }
}

/**
 * Check frontend service health
 */
async function checkFrontendHealth(page: any): Promise<void> {
  try {
    await page.goto(BASE_URL, { timeout: 10000 });
    console.log('✅ Frontend service is accessible');
    
    // Verify page loads without critical errors
    await page.waitForSelector('body', { timeout: 5000 });
    
    // Check if main navigation is present
    const hasNav = await page.locator('nav, [data-testid*="nav"]').count() > 0;
    if (hasNav) {
      console.log('🧭 Navigation elements detected');
    }
    
    // Check if the app is in development mode
    const isDev = await page.evaluate(() => {
      return window.location.hostname === 'localhost';
    });
    
    if (isDev) {
      console.log('🛠️  Running in development mode');
    }
    
  } catch (error) {
    console.error('❌ Frontend service is not accessible:', error);
    throw error;
  }
}

/**
 * Check admin frontend health
 */
async function checkAdminHealth(page: any): Promise<void> {
  try {
    await page.goto(ADMIN_URL, { timeout: 10000 });
    console.log('✅ Admin frontend service is accessible');
  } catch (error) {
    console.warn('⚠️ Admin frontend service may not be accessible:', error);
    console.warn('   Some admin-related tests may be skipped');
  }
}

/**
 * Check OIDC endpoint health
 */
async function checkOIDCHealth(page: any): Promise<void> {
  try {
    const oidcResponse = await page.request.get(
      `${API_URL}/oauth/authorize?client_id=epsx-frontend&response_type=code&scope=openid&redirect_uri=${BASE_URL}/callback&state=test`
    );
    
    if (oidcResponse.status() === 200 || oidcResponse.status() === 302) {
      console.log('✅ OIDC authorization endpoint is responsive');
    } else {
      console.warn('⚠️ OIDC endpoint returned:', oidcResponse.status());
    }
  } catch (error) {
    console.warn('⚠️ OIDC endpoint may not be accessible:', error);
    console.warn('   Authentication tests may use mock mode');
  }
}

/**
 * Validate critical API endpoints
 */
async function validateCriticalEndpoints(page: any): Promise<void> {
  const criticalEndpoints = [
    { path: '/api/auth/validate-session', method: 'POST', expectStatus: [401, 422] },
    { path: '/api/portfolio/balance', method: 'GET', expectStatus: [401, 403] },
    { path: '/api/analytics/basic', method: 'GET', expectStatus: [401, 403] }
  ];
  
  console.log('🔍 Validating critical API endpoints...');
  
  for (const endpoint of criticalEndpoints) {
    try {
      const response = await page.request.fetch(`${API_URL}${endpoint.path}`, {
        method: endpoint.method
      });
      
      if (endpoint.expectStatus.includes(response.status())) {
        console.log(`✅ ${endpoint.method} ${endpoint.path}: ${response.status()} (expected)`);
      } else {
        console.warn(`⚠️ ${endpoint.method} ${endpoint.path}: ${response.status()} (unexpected)`);
      }
    } catch (error) {
      console.warn(`⚠️ ${endpoint.method} ${endpoint.path}: Error -`, error.message);
    }
  }
}

/**
 * Check for JavaScript errors in base application
 */
async function checkJavaScriptHealth(page: any): Promise<void> {
  const jsErrors: string[] = [];
  const consoleWarnings: string[] = [];
  
  page.on('pageerror', (error: Error) => {
    jsErrors.push(error.message);
  });
  
  page.on('console', (msg: any) => {
    if (msg.type() === 'warning') {
      consoleWarnings.push(msg.text());
    }
  });
  
  await page.goto(BASE_URL);
  await page.waitForTimeout(2000); // Allow time for any startup errors
  
  if (jsErrors.length === 0) {
    console.log('✅ No critical JavaScript errors detected');
  } else {
    console.warn('⚠️ JavaScript errors detected:');
    jsErrors.slice(0, 3).forEach(error => console.warn(`   - ${error}`));
    if (jsErrors.length > 3) {
      console.warn(`   ... and ${jsErrors.length - 3} more errors`);
    }
  }
  
  if (consoleWarnings.length > 0 && consoleWarnings.length <= 5) {
    console.log(`💭 ${consoleWarnings.length} console warnings detected (acceptable)`);
  } else if (consoleWarnings.length > 5) {
    console.warn(`⚠️ ${consoleWarnings.length} console warnings detected (review recommended)`);
  }
}

/**
 * Check responsive design basics
 */
async function checkResponsiveBasics(page: any): Promise<void> {
  const viewports = [
    { name: 'mobile', width: 375, height: 667 },
    { name: 'tablet', width: 768, height: 1024 },
    { name: 'desktop', width: 1200, height: 800 }
  ];
  
  for (const viewport of viewports) {
    await page.setViewportSize({ width: viewport.width, height: viewport.height });
    await page.goto(BASE_URL);
    await page.waitForTimeout(1000);
    
    // Check if content overflows horizontally
    const hasOverflow = await page.evaluate(() => {
      return document.body.scrollWidth > document.body.clientWidth;
    });
    
    if (!hasOverflow) {
      console.log(`✅ ${viewport.name} (${viewport.width}x${viewport.height}): No horizontal overflow`);
    } else {
      console.warn(`⚠️ ${viewport.name} (${viewport.width}x${viewport.height}): Horizontal overflow detected`);
    }
  }
  
  // Reset to default viewport
  await page.setViewportSize({ width: 1200, height: 800 });
}

/**
 * Setup test environment variables and constants
 */
async function setupTestEnvironment(): Promise<void> {
  // Define test environment constants
  const testEnv = {
    BASE_URL,
    API_URL,
    ADMIN_URL,
    TEST_MODE: true,
    TIMESTAMP: Date.now(),
    VERSION: '1.0.0'
  };
  
  // Log environment configuration
  console.log('🌍 Test environment configured:');
  console.log(`   Frontend: ${testEnv.BASE_URL}`);
  console.log(`   Backend: ${testEnv.API_URL}`);
  console.log(`   Admin: ${testEnv.ADMIN_URL}`);
  console.log(`   Test ID: ${testEnv.TIMESTAMP}`);
  
  // Set global test markers
  global.TEST_CONFIG = testEnv;
}

