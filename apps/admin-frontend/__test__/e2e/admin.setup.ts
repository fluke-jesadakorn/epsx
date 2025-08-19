/**
 * Admin setup for Playwright E2E tests
 * Ensures admin backend services are available and configures test environment
 */

import { test as setup, expect } from '@playwright/test';

const ADMIN_URL = process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001';
const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

setup('🔧 Admin Setup - Service Health Check', async ({ page }) => {
  console.log('🚀 Starting admin setup...');
  
  // Check if backend is running
  try {
    const response = await page.request.get(`${API_URL}/health`);
    if (response.status() === 200) {
      console.log('✅ Backend service is running for admin');
    } else {
      console.warn('⚠️ Backend health check returned:', response.status());
    }
  } catch (error) {
    console.warn('⚠️ Backend service may not be running:', error);
  }

  // Check if admin frontend is accessible
  try {
    await page.goto(ADMIN_URL, { timeout: 10000 });
    console.log('✅ Admin frontend service is accessible');
  } catch (error) {
    console.error('❌ Admin frontend service is not accessible:', error);
    throw error;
  }

  // Verify admin OIDC endpoint is responsive
  try {
    const adminOidcResponse = await page.request.get(`${API_URL}/oauth/authorize?client_id=epsx-admin&response_type=code&scope=openid%20profile%20email%20admin&redirect_uri=http://localhost:3001/callback&state=test`);
    if (adminOidcResponse.status() === 200) {
      console.log('✅ Admin OIDC authorization endpoint is responsive');
    } else {
      console.warn('⚠️ Admin OIDC endpoint returned:', adminOidcResponse.status());
    }
  } catch (error) {
    console.warn('⚠️ Admin OIDC endpoint may not be accessible:', error);
  }

  console.log('🏁 Admin setup completed');
});

export { };