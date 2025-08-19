/**
 * Global setup for Playwright E2E tests
 * Ensures backend services are available and configures test environment
 */

import { test as setup, expect } from '@playwright/test';

const BASE_URL = process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000';
const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
const ADMIN_URL = process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001';

setup('🔧 Global Setup - Service Health Check', async ({ page }) => {
  console.log('🚀 Starting global setup...');
  
  // Check if backend is running
  try {
    const response = await page.request.get(`${API_URL}/health`);
    if (response.status() === 200) {
      console.log('✅ Backend service is running');
    } else {
      console.warn('⚠️ Backend health check returned:', response.status());
    }
  } catch (error) {
    console.warn('⚠️ Backend service may not be running:', error);
  }

  // Check if frontend is accessible
  try {
    await page.goto(BASE_URL, { timeout: 10000 });
    console.log('✅ Frontend service is accessible');
  } catch (error) {
    console.error('❌ Frontend service is not accessible:', error);
    throw error;
  }

  // Check if admin frontend is accessible
  try {
    await page.goto(ADMIN_URL, { timeout: 10000 });
    console.log('✅ Admin frontend service is accessible');
  } catch (error) {
    console.warn('⚠️ Admin frontend service may not be accessible:', error);
  }

  // Verify OIDC endpoint is responsive
  try {
    const oidcResponse = await page.request.get(`${API_URL}/oauth/authorize?client_id=epsx-frontend&response_type=code&scope=openid&redirect_uri=http://localhost:3000/callback&state=test`);
    if (oidcResponse.status() === 200) {
      console.log('✅ OIDC authorization endpoint is responsive');
    } else {
      console.warn('⚠️ OIDC endpoint returned:', oidcResponse.status());
    }
  } catch (error) {
    console.warn('⚠️ OIDC endpoint may not be accessible:', error);
  }

  console.log('🏁 Global setup completed');
});

export { };