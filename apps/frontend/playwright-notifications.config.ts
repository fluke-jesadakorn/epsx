/**
 * Playwright Configuration for User Notification Experience E2E Tests
 * 
 * Frontend-focused configuration for user notification testing
 * with optimized settings for user experience validation
 */
import { defineConfig, devices } from '@playwright/test';

// Test environment configuration
const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';
const FRONTEND_URL = process.env.FRONTEND_URL || 'http://localhost:3000';
const ADMIN_FRONTEND_URL = process.env.ADMIN_FRONTEND_URL || 'http://localhost:3001';
const HEADLESS = process.env.HEADLESS !== 'false';
const TIMEOUT = parseInt(process.env.TIMEOUT || '60000');

export default defineConfig({
  // Test directory
  testDir: './__test__/e2e',
  
  // Test patterns - focus on user notification experience tests
  testMatch: [
    '**/notifications-user-experience.spec.ts',
    '**/notifications-*.spec.ts'
  ],
  
  // Global timeout settings
  timeout: TIMEOUT,
  expect: {
    timeout: 10000,
  },
  
  // Test execution settings
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 1,
  workers: process.env.CI ? 1 : 2, // Limited workers to avoid notification conflicts
  
  // Test output configuration
  outputDir: '__test__/test-results/',
  
  // Reporter configuration
  reporter: [
    ['html', { 
      outputFolder: '__test__/playwright-report',
      open: process.env.CI ? 'never' : 'on-failure'
    }],
    ['json', { outputFile: '__test__/test-results/results.json' }],
    ['junit', { outputFile: '__test__/test-results/results.xml' }],
    ['line'],
    ...(process.env.CI ? [['github'] as const] : [])
  ],
  
  // Browser and device configuration
  use: {
    // Base URL for frontend tests
    baseURL: FRONTEND_URL,
    
    // Browser settings
    headless: HEADLESS,
    viewport: { width: 1280, height: 720 },
    
    // Test artifacts
    screenshot: 'only-on-failure',
    video: 'retain-on-failure', 
    trace: 'retain-on-failure',
    
    // Browser launch options optimized for notifications
    launchOptions: {
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
    
    // Context settings for user notification tests
    permissions: ['notifications'],
    
    // Ignore HTTPS errors for local development
    ignoreHTTPSErrors: true,
    
    // Extended timeout for user interactions
    actionTimeout: 15000,
    navigationTimeout: 30000,
  },
  
  // Test projects for different user scenarios
  projects: [
    // Core user notification experience
    {
      name: 'user-notification-core',
      use: {
        ...devices['Desktop Chrome'],
        permissions: ['notifications'],
        userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36 EPSX-UserNotificationTests/1.0'
      },
    },
    
    // Mobile user notification experience
    {
      name: 'user-notification-mobile',
      use: {
        ...devices['iPhone 13'],
        permissions: ['notifications'],
      },
    },
    
    // Tablet user notification experience  
    {
      name: 'user-notification-tablet',
      use: {
        ...devices['iPad Pro'],
        permissions: ['notifications'],
      },
    },
    
    // User notification performance tests
    {
      name: 'user-notification-performance',
      use: {
        ...devices['Desktop Chrome'],
        permissions: ['notifications'],
        video: 'off',
        screenshot: 'off',
      },
      testMatch: '**/notifications-*performance*.spec.ts'
    },
    
    // Accessibility testing for notifications
    {
      name: 'user-notification-a11y',
      use: {
        ...devices['Desktop Chrome'],
        permissions: ['notifications'],
        // Enable accessibility tree debugging
        launchOptions: {
          args: [
            '--enable-accessibility-object-model',
            '--enable-automation',
            '--disable-web-security',
          ]
        }
      },
      testMatch: '**/notifications-*accessibility*.spec.ts'
    },
    
    // Cross-browser user notification testing
    {
      name: 'user-notification-firefox',
      use: {
        ...devices['Desktop Firefox'],
        permissions: ['notifications'],
      },
    },
    
    {
      name: 'user-notification-safari',
      use: {
        ...devices['Desktop Safari'],
        permissions: ['notifications'],
      },
    },
  ],
  
  // Web server configuration
  webServer: [
    // Ensure all services are running for user tests
    {
      command: 'echo "Backend should be running at ' + BACKEND_URL + '"',
      url: BACKEND_URL + '/health',
      timeout: 120000,
      reuseExistingServer: !process.env.CI,
    },
    {
      command: 'echo "Frontend should be running at ' + FRONTEND_URL + '"',
      url: FRONTEND_URL,
      timeout: 120000,
      reuseExistingServer: !process.env.CI,
    },
    {
      command: 'echo "Admin frontend should be running at ' + ADMIN_FRONTEND_URL + '"',
      url: ADMIN_FRONTEND_URL,
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
    TEST_USER_EMAIL: process.env.TEST_USER_EMAIL || 'testuser@epsx.io',
    TEST_USER_PASSWORD: process.env.TEST_USER_PASSWORD || 'testuser123',
    ADMIN_EMAIL: process.env.ADMIN_EMAIL || 'admin@epsx.io',
    ADMIN_PASSWORD: process.env.ADMIN_PASSWORD || 'admin123',
    // Database configuration
    DATABASE_URL: process.env.DATABASE_URL || 'postgresql://epsx_user:epsx_password@localhost:5432/epsx_db',
    // Test configuration
    NOTIFICATION_TEST_MODE: 'true',
    USER_FOCUSED_TESTS: 'true',
    HEADLESS: HEADLESS.toString(),
    TIMEOUT: TIMEOUT.toString(),
  },
});