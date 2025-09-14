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
  reporter: [
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['junit', { outputFile: 'test-results/results.xml' }],
    ['list']
  ],
  outputDir: 'test-results',
  use: {
    baseURL: process.env.FRONTEND_URL || 'http://localhost:3002',
    trace: 'on-first-retry',
    screenshot: 'on-failure',
    video: 'retain-on-failure',
    actionTimeout: 15000,
    navigationTimeout: 30000,
  },
  projects: [
    // Setup project for authentication
    {
      name: 'setup',
      testMatch: /global\.setup\.ts/,
    },
    
    // Analytics Platform Complete Flow Tests - 100% Coverage
    {
      name: 'analytics-complete',
      testMatch: '**/analytics-auth-complete-flow.spec.ts',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup'],
    },
    
    // Core functionality tests
    {
      name: 'core',
      testMatch: [
        '**/frontend.spec.ts',
        '**/pagination.spec.ts',
        '**/auth-flow.spec.ts',
        '**/comprehensive-auth-test.spec.ts'
      ],
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup'],
    },
    
    // Embedded Permissions System Tests
    {
      name: 'embedded-permissions',
      testMatch: '**/embedded-permissions-comprehensive.spec.ts',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup'],
    },
    
    // Complete coverage tests
    {
      name: 'coverage',
      testMatch: '**/complete-coverage.spec.ts',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup'],
    },
    
    // Enhanced auth flow tests
    {
      name: 'auth-enhanced',
      testMatch: '**/enhanced-auth-flow.spec.ts',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup'],
    },

    // Production deployment check
    {
      name: 'production-check',
      testMatch: '**/production-deployment-check.spec.ts',
      use: { ...devices['Desktop Chrome'] },
    },
    
    // User journey tests
    {
      name: 'journeys',
      testMatch: '**/user-journey-flows.spec.ts',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup'],
    },
    
    // Cross-browser testing
    {
      name: 'firefox',
      testMatch: '**/complete-coverage.spec.ts',
      use: { ...devices['Desktop Firefox'] },
      dependencies: ['setup'],
    },
    
    {
      name: 'webkit',
      testMatch: '**/complete-coverage.spec.ts', 
      use: { ...devices['Desktop Safari'] },
      dependencies: ['setup'],
    },
    
    // Mobile testing
    {
      name: 'mobile-chrome',
      testMatch: [
        '**/complete-coverage.spec.ts',
        '**/enhanced-auth-flow.spec.ts'
      ],
      use: { ...devices['Pixel 5'] },
      dependencies: ['setup'],
    },
    
    {
      name: 'mobile-safari',
      testMatch: [
        '**/complete-coverage.spec.ts',
        '**/enhanced-auth-flow.spec.ts'
      ],
      use: { ...devices['iPhone 12'] },
      dependencies: ['setup'],
    },
  ],
  // webServer: {
  //   command: 'npm run dev',
  //   url: 'http://localhost:3000',
  //   reuseExistingServer: !env.CI,
  // },
});
