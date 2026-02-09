import { defineConfig, devices } from '@playwright/test';
import { getFrontendUrl } from '@/shared/utils/url-resolver';

// Simplified environment for testing
const CI = process.env.CI === 'true';

export default defineConfig({
  testDir: './__test__/e2e',
  fullyParallel: true,
  forbidOnly: Boolean(CI),
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
    ['list'],
    ['./scripts/coverage-reporter.js'] // Custom coverage reporter
  ],
  outputDir: 'test-results',
  use: {
    baseURL: getFrontendUrl('client'),
    trace: 'on-first-retry',
    screenshot: 'on',
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

    // Web3 Wallet Comprehensive Testing - 100% Coverage
    {
      name: 'web3-comprehensive',
      testMatch: '**/web3-wallet-comprehensive.spec.ts',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup'],
    },

    // Web3 Complete Coverage Suite
    {
      name: 'web3-complete-coverage',
      testMatch: '**/web3-complete-coverage.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        // Enable coverage collection
        contextOptions: {
          recordVideo: { dir: 'test-results/videos' },
          recordHar: { path: 'test-results/network.har' },
        }
      },
      dependencies: ['setup'],
    },

    // Web3 Authentication Flows
    {
      name: 'web3-auth-flows',
      testMatch: '**/web3-authentication-flows.spec.ts',
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

    // Localhost connection resolution verification
    {
      name: 'localhost-resolution',
      testMatch: '**/localhost-connection-resolution-test.spec.ts',
      use: { ...devices['Desktop Chrome'] },
    },

    // User journey tests
    {
      name: 'journeys',
      testMatch: '**/user-journey-flows.spec.ts',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup'],
    },

    // Notification System Tests - Complete Coverage
    {
      name: 'notifications',
      testMatch: '**/notifications-complete.spec.ts',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup'],
    },

    // Notification Integration Tests - Cross-App
    {
      name: 'notifications-integration',
      testMatch: '**/notifications-integration.spec.ts',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup'],
    },

    // Cross-browser Web3 testing
    {
      name: 'web3-firefox',
      testMatch: [
        '**/web3-complete-coverage.spec.ts',
        '**/web3-wallet-comprehensive.spec.ts'
      ],
      use: { ...devices['Desktop Firefox'] },
      dependencies: ['setup'],
    },

    {
      name: 'web3-webkit',
      testMatch: [
        '**/web3-complete-coverage.spec.ts',
        '**/web3-wallet-comprehensive.spec.ts'
      ],
      use: { ...devices['Desktop Safari'] },
      dependencies: ['setup'],
    },

    // Mobile Web3 testing
    {
      name: 'web3-mobile-chrome',
      testMatch: [
        '**/web3-complete-coverage.spec.ts',
        '**/web3-wallet-comprehensive.spec.ts',
        '**/web3-authentication-flows.spec.ts'
      ],
      use: { ...devices['Pixel 5'] },
      dependencies: ['setup'],
    },

    {
      name: 'web3-mobile-safari',
      testMatch: [
        '**/web3-complete-coverage.spec.ts',
        '**/web3-wallet-comprehensive.spec.ts',
        '**/web3-authentication-flows.spec.ts'
      ],
      use: { ...devices['iPhone 12'] },
      dependencies: ['setup'],
    },

    // Legacy coverage tests
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

    // Plan Upgrade & Analytics Offset Verification
    {
      name: 'plan-upgrade-analytics',
      testMatch: '**/plan-upgrade-analytics.spec.ts',
      use: {
        ...devices['Desktop Chrome'],
        screenshot: 'on',
        actionTimeout: 30000,
      },
    },
  ],
  // webServer: {
  //   command: 'npm run dev',
  //   url: 'http://localhost:3000',
  //   reuseExistingServer: !env.CI,
  // },
});
