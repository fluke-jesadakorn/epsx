import { defineConfig, devices } from '@playwright/test';

// Simplified environment for testing
const CI = process.env.CI === 'true';

export default defineConfig({
  testDir: './__test__/e2e',
  fullyParallel: true,
  forbidOnly: !!CI,
  retries: CI ? 2 : 0,
  workers: CI ? 1 : 2,
  timeout: 60000,
  expect: {
    timeout: 10000
  },
  outputDir: 'test-results',
  reporter: [
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['junit', { outputFile: 'test-results/results.xml' }],
    ['list']
  ],
  use: {
    baseURL: 'http://localhost:3001',
    trace: 'on-first-retry',
    screenshot: { mode: 'only-on-failure' },
    video: 'retain-on-failure',
    actionTimeout: 15000,
    navigationTimeout: 30000,
  },
  projects: [
    // Setup project for admin authentication
    {
      name: 'admin-setup',
      testMatch: /admin\.setup\.ts/,
    },
    
    // Analytics Command Center Admin Complete Flow Tests - 100% Coverage
    {
      name: 'analytics-admin-complete',
      testMatch: '**/analytics-admin-complete-flow.spec.ts',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['admin-setup'],
    },
    
    // Core admin functionality tests
    {
      name: 'admin-core',
      testMatch: [
        '**/admin.spec.ts',
        '**/auth-comprehensive-test.spec.ts'
      ],
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['admin-setup'],
    },
    
    // Complete admin coverage tests
    {
      name: 'admin-coverage',
      testMatch: '**/complete-admin-coverage.spec.ts',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['admin-setup'],
    },
    
    // Enhanced user management tests (new advanced features)
    {
      name: 'enhanced-user-management',
      testMatch: '**/enhanced-user-management.spec.ts',
      use: { 
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
      dependencies: ['admin-setup'],
    },
    
    // User management module tests
    {
      name: 'user-management',
      testMatch: '**/complete-admin-coverage.spec.ts',
      use: { 
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
      dependencies: ['admin-setup'],
      grep: /@user-management/,
    },
    
    // Permission management tests
    {
      name: 'permission-management', 
      testMatch: '**/complete-admin-coverage.spec.ts',
      use: { 
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
      dependencies: ['admin-setup'],
      grep: /@permission/,
    },
    
    // System administration tests
    {
      name: 'system-admin',
      testMatch: '**/complete-admin-coverage.spec.ts',
      use: { 
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
      dependencies: ['admin-setup'],
      grep: /@system/,
    },
    
    // Cross-browser admin testing
    {
      name: 'admin-firefox',
      testMatch: '**/complete-admin-coverage.spec.ts',
      use: { ...devices['Desktop Firefox'] },
      dependencies: ['admin-setup'],
    },
    
    {
      name: 'admin-webkit',
      testMatch: '**/complete-admin-coverage.spec.ts',
      use: { ...devices['Desktop Safari'] },
      dependencies: ['admin-setup'],
    },
    
    // Mobile admin testing
    {
      name: 'admin-mobile-chrome',
      testMatch: '**/complete-admin-coverage.spec.ts',
      use: { ...devices['Pixel 5'] },
      dependencies: ['admin-setup'],
    },
    
    {
      name: 'admin-mobile-safari',
      testMatch: '**/complete-admin-coverage.spec.ts',
      use: { ...devices['iPhone 12'] },
      dependencies: ['admin-setup'],
    },
    
    // Enhanced user management mobile testing
    {
      name: 'enhanced-user-management-mobile',
      testMatch: '**/enhanced-user-management.spec.ts',
      use: { 
        ...devices['Pixel 5'],
        testIdAttribute: 'data-testid',
      },
      dependencies: ['admin-setup'],
    },
  ],
  // webServer: {
  //   command: 'npm run dev',
  //   url: 'http://localhost:3001',
  //   reuseExistingServer: !env.CI,
  // },
});
