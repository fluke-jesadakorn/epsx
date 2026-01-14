import { defineConfig, devices } from '@playwright/test';

// Simplified environment for testing
const CI = process.env.CI === 'true';

// Constants to avoid duplication
const TEST_ID = 'data-testid';
const AUTH_FLOWS_TEST = '**/auth-flows.spec.ts';
const NAV_LAYOUT_TEST = '**/navigation-layout.spec.ts';
const WALLET_MGMT_TEST = '**/wallet-management.spec.ts';

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
    // Capture trace for all tests (on = always, retain-on-failure = only on fail)
    trace: 'on',
    // Capture screenshot for all tests (on = always, only-on-failure = only on fail)
    screenshot: 'on',
    // Capture video for all tests (on = always, retain-on-failure = only on fail)
    video: 'on',
    actionTimeout: 15000,
    navigationTimeout: 30000,
  },
  projects: [
    // Web3 Authentication E2E Tests (Mock-based)
    {
      name: 'web3-auth',
      testMatch: '**/web3-auth-e2e.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: TEST_ID,
      },
    },

    // Core Authentication and Authorization Tests
    {
      name: 'auth-flows',
      testMatch: AUTH_FLOWS_TEST,
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: TEST_ID,
      },
    },

    // Wallet Management Tests
    {
      name: 'wallet-management',
      testMatch: WALLET_MGMT_TEST,
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: TEST_ID,
      },
    },

    // Group Management Tests
    {
      name: 'group-management',
      testMatch: '**/group-management.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: TEST_ID,
      },
    },

    // Permission Management Tests
    {
      name: 'permission-management',
      testMatch: '**/permission-management-comprehensive.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: TEST_ID,
      },
    },

    // Navigation and Layout Tests
    {
      name: 'navigation-layout',
      testMatch: NAV_LAYOUT_TEST,
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: TEST_ID,
      },
    },

    // API Documentation Tests
    {
      name: 'api-documentation',
      testMatch: '**/api-documentation.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: TEST_ID,
      },
    },

    // Analytics and Monitoring Tests
    {
      name: 'analytics-monitoring',
      testMatch: '**/analytics-monitoring.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: TEST_ID,
      },
    },

    // Subscription and Plan Management Tests
    {
      name: 'subscription-plan-management',
      testMatch: '**/subscription-plan-management.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: TEST_ID,
      },
    },

    // Notification System Tests - Admin Complete Coverage
    {
      name: 'notifications-admin',
      testMatch: '**/notifications-admin-complete.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: TEST_ID,
      },
    },

    // Stock Ranking and Policies Tests
    {
      name: 'stock-ranking-policies',
      testMatch: '**/stock-ranking-policies.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: TEST_ID,
      },
    },

    // Profile and Settings Tests
    {
      name: 'profile-settings',
      testMatch: '**/profile-settings.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: TEST_ID,
      },
    },

    // Error Handling and Edge Cases Tests
    {
      name: 'error-edge-cases',
      testMatch: '**/error-edge-cases.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: TEST_ID,
      },
    },

    // Legacy Tests
    {
      name: 'admin-login',
      testMatch: '**/admin-login-test.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        testIdAttribute: TEST_ID,
      },
    },

    // Cross-browser Testing - Firefox
    {
      name: 'firefox',
      testMatch: [
        AUTH_FLOWS_TEST,
        WALLET_MGMT_TEST,
        NAV_LAYOUT_TEST,
      ],
      use: { ...devices['Desktop Firefox'] },
    },

    // Cross-browser Testing - Safari
    {
      name: 'webkit',
      testMatch: [
        AUTH_FLOWS_TEST,
        WALLET_MGMT_TEST,
        NAV_LAYOUT_TEST,
      ],
      use: { ...devices['Desktop Safari'] },
    },

    // Mobile Testing - Chrome
    {
      name: 'mobile-chrome',
      testMatch: [
        NAV_LAYOUT_TEST,
        AUTH_FLOWS_TEST,
      ],
      use: { ...devices['Pixel 5'] },
    },

    // Mobile Testing - Safari
    {
      name: 'mobile-safari',
      testMatch: [
        NAV_LAYOUT_TEST,
        AUTH_FLOWS_TEST,
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
