/**
 * Playwright Configuration for Notification System E2E Tests
 * 
 * Specialized configuration for comprehensive notification testing
 * with optimized settings for reliability and performance
 */
import { defineConfig, devices } from '@playwright/test';
import path from 'path';

// Test environment configuration
const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';
const FRONTEND_URL = process.env.FRONTEND_URL || 'http://localhost:3000';
const ADMIN_FRONTEND_URL = process.env.ADMIN_FRONTEND_URL || 'http://localhost:3001';
const HEADLESS = process.env.HEADLESS !== 'false';
const TIMEOUT = parseInt(process.env.TIMEOUT || '60000');

export default defineConfig({
  // Test directory
  testDir: './__test__',
  
  // Test patterns - focus on notification tests
  testMatch: [
    '**/notifications-comprehensive.spec.ts',
    '**/notifications-*.spec.ts'
  ],
  
  // Global timeout settings
  timeout: TIMEOUT,
  expect: {
    timeout: 10000, // Assertion timeout
  },
  
  // Test execution settings
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 1,
  workers: process.env.CI ? 1 : 2, // Reduced workers for notification tests to prevent conflicts
  
  // Global test setup (removed - not needed for basic notification tests)
  
  // Test output configuration
  outputDir: 'test-results/',
  
  // Reporter configuration
  reporter: [
    ['html', { 
      outputFolder: 'playwright-report',
      open: process.env.CI ? 'never' : 'on-failure'
    }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['junit', { outputFile: 'test-results/results.xml' }],
    ['line'],
    ...(process.env.CI ? [['github'] as const] : [])
  ],
  
  // Browser and device configuration
  use: {
    // Base URL for tests
    baseURL: ADMIN_FRONTEND_URL,
    
    // Browser settings
    headless: HEADLESS,
    viewport: { width: 1280, height: 720 },
    
    // Test artifacts
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    trace: 'retain-on-failure',
    
    // Network settings optimized for notification tests
    launchOptions: {
      // Enable notifications permission by default
      args: [
        '--disable-web-security',
        '--disable-features=VizDisplayCompositor',
        '--allow-running-insecure-content',
        '--disable-backgrounding-occluded-windows',
        '--disable-renderer-backgrounding',
        '--disable-background-networking',
        '--enable-features=NetworkService,NetworkServiceLogging',
        '--use-fake-ui-for-media-stream',
        '--use-fake-device-for-media-stream',
      ]
    },
    
    // Context settings for notification tests
    permissions: ['notifications'],
    
    // Ignore HTTPS errors for local development
    ignoreHTTPSErrors: true,
    
    // Extended timeout for notification-related operations
    actionTimeout: 15000,
    navigationTimeout: 30000,
  },
  
  // Test projects for different scenarios
  projects: [
    // Main notification tests
    {
      name: 'notification-core',
      use: {
        ...devices['Desktop Chrome'],
        // Enable notifications
        permissions: ['notifications'],
        // Custom user agent for testing
        userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36 EPSX-NotificationTests/1.0'
      },
    },
    
    // Mobile notification tests
    {
      name: 'notification-mobile',
      use: {
        ...devices['iPhone 13'],
        permissions: ['notifications'],
      },
      testMatch: '**/notifications-*mobile*.spec.ts'
    },
    
    // Notification performance tests
    {
      name: 'notification-performance',
      use: {
        ...devices['Desktop Chrome'],
        permissions: ['notifications'],
        // Specific settings for performance testing
        video: 'off', // Disable video recording for performance tests
        screenshot: 'off',
      },
      testMatch: '**/notifications-*performance*.spec.ts'
    },
    
    // Cross-browser notification testing
    {
      name: 'notification-firefox',
      use: {
        ...devices['Desktop Firefox'],
        permissions: ['notifications'],
      },
      testMatch: '**/notifications-comprehensive.spec.ts'
    },
    
    {
      name: 'notification-safari',
      use: {
        ...devices['Desktop Safari'],
        permissions: ['notifications'],
      },
      testMatch: '**/notifications-comprehensive.spec.ts'
    },
  ],
  
  // Web server configuration
  webServer: [
    // Ensure backend is running
    {
      command: 'echo "Backend should be running at ' + BACKEND_URL + '"',
      url: BACKEND_URL + '/health',
      timeout: 120000,
      reuseExistingServer: !process.env.CI,
    },
    // Ensure admin frontend is running
    {
      command: 'echo "Admin frontend should be running at ' + ADMIN_FRONTEND_URL + '"',
      url: ADMIN_FRONTEND_URL,
      timeout: 120000,
      reuseExistingServer: !process.env.CI,
    },
    // Ensure user frontend is running
    {
      command: 'echo "User frontend should be running at ' + FRONTEND_URL + '"',
      url: FRONTEND_URL,
      timeout: 120000,
      reuseExistingServer: !process.env.CI,
    },
  ],
  
  // Test environment variables
  env: {
    BACKEND_URL,
    FRONTEND_URL,
    ADMIN_FRONTEND_URL,
    // Test user configuration
    ADMIN_EMAIL: process.env.ADMIN_EMAIL || 'info@epsx.io',
    ADMIN_PASSWORD: process.env.ADMIN_PASSWORD || 'P@ssword',
    TEST_USER_EMAIL: process.env.TEST_USER_EMAIL || 'testuser@epsx.io', 
    TEST_USER_PASSWORD: process.env.TEST_USER_PASSWORD || 'testuser123',
    // Database configuration
    DATABASE_URL: process.env.DATABASE_URL || 'postgresql://epsx_user:epsx_password@localhost:5432/epsx_db',
    // Test configuration
    NOTIFICATION_TEST_MODE: 'true',
    HEADLESS: HEADLESS.toString(),
    TIMEOUT: TIMEOUT.toString(),
  },
});