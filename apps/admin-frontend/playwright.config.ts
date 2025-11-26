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
    // Core Authentication and Authorization Tests
    {
      name: 'auth-flows',
      testMatch: '**/auth-flows.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
    },

    // Wallet Management Tests
    {
      name: 'wallet-management',
      testMatch: '**/wallet-management.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
    },

    // Group Management Tests
    {
      name: 'group-management',
      testMatch: '**/group-management.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
    },

    // Permission Management Tests
    {
      name: 'permission-management',
      testMatch: '**/permission-management-comprehensive.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
    },

    // Navigation and Layout Tests
    {
      name: 'navigation-layout',
      testMatch: '**/navigation-layout.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
    },

    // API Documentation Tests
    {
      name: 'api-documentation',
      testMatch: '**/api-documentation.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
    },

    // Analytics and Monitoring Tests
    {
      name: 'analytics-monitoring',
      testMatch: '**/analytics-monitoring.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
    },

    // Subscription and Plan Management Tests
    {
      name: 'subscription-plan-management',
      testMatch: '**/subscription-plan-management.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
    },

    // Notification System Tests - Admin Complete Coverage
    {
      name: 'notifications-admin',
      testMatch: '**/notifications-admin-complete.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
    },

    // Stock Ranking and Policies Tests
    {
      name: 'stock-ranking-policies',
      testMatch: '**/stock-ranking-policies.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
    },

    // Profile and Settings Tests
    {
      name: 'profile-settings',
      testMatch: '**/profile-settings.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
    },

    // Error Handling and Edge Cases Tests
    {
      name: 'error-edge-cases',
      testMatch: '**/error-edge-cases.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
    },

    // Legacy Tests
    {
      name: 'admin-login',
      testMatch: '**/admin-login-test.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: 'data-testid',
      },
    },

    // Cross-browser Testing - Firefox
    {
      name: 'firefox',
      testMatch: [
        '**/auth-flows.spec.ts',
        '**/wallet-management.spec.ts',
        '**/navigation-layout.spec.ts',
      ],
      use: { ...devices['Desktop Firefox'] },
    },

    // Cross-browser Testing - Safari
    {
      name: 'webkit',
      testMatch: [
        '**/auth-flows.spec.ts',
        '**/wallet-management.spec.ts',
        '**/navigation-layout.spec.ts',
      ],
      use: { ...devices['Desktop Safari'] },
    },

    // Mobile Testing - Chrome
    {
      name: 'mobile-chrome',
      testMatch: [
        '**/navigation-layout.spec.ts',
        '**/auth-flows.spec.ts',
      ],
      use: { ...devices['Pixel 5'] },
    },

    // Mobile Testing - Safari
    {
      name: 'mobile-safari',
      testMatch: [
        '**/navigation-layout.spec.ts',
        '**/auth-flows.spec.ts',
      ],
      use: { ...devices['iPhone 12'] },
    },
  ],
  // webServer: {
  //   command: 'npm run dev',
  //   url: 'http://localhost:3001',
  //   reuseExistingServer: !env.CI,
  // },
});
