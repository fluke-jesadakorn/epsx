import { test as base, expect, Page, BrowserContext } from '@playwright/test';
import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';

// Extend Playwright test to include coverage collection
export const test = base.extend<{
  context: BrowserContext;
  page: Page;
}>({
  context: async ({ browser }, use) => {
    // Create context with coverage enabled
    const context = await browser.newContext({
      // Enable coverage collection
      recordVideo: { dir: 'test-results/videos' },
      recordHar: { path: 'test-results/network.har' },
    });

    // Add script to collect coverage on all pages
    await context.addInitScript(() => {
      // Initialize coverage collection
      (window as any).__coverage__ = {};
      
      // Hook into module loading to track execution
      const originalRequire = (window as any).require;
      if (originalRequire) {
        (window as any).require = function(id: string) {
          const module = originalRequire.apply(this, arguments);
          if (id.includes('web3') || id.includes('wallet') || id.includes('auth')) {
            // Track module execution
            if (!(window as any).__coverage__[id]) {
              (window as any).__coverage__[id] = {
                path: id,
                statementMap: {},
                fnMap: {},
                branchMap: {},
                s: {},
                f: {},
                b: {}
              };
            }
          }
          return module;
        };
      }

      // Track function calls
      const originalFunction = Function.prototype.apply;
      Function.prototype.apply = function(thisArg, args) {
        const name = this.name;
        if (name && (name.includes('web3') || name.includes('wallet') || name.includes('auth'))) {
          if (!(window as any).__coverage__['functions']) {
            (window as any).__coverage__['functions'] = {};
          }
          (window as any).__coverage__['functions'][name] = 
            ((window as any).__coverage__['functions'][name] || 0) + 1;
        }
        return originalFunction.call(this, thisArg, args);
      };

      // Track Web3 method calls
      const originalFetch = window.fetch;
      window.fetch = async function(input, init) {
        const url = typeof input === 'string' ? input : input.url;
        if (url.includes('/api/auth/web3') || url.includes('/api/auth/session')) {
          if (!(window as any).__coverage__['api_calls']) {
            (window as any).__coverage__['api_calls'] = {};
          }
          (window as any).__coverage__['api_calls'][url] = 
            ((window as any).__coverage__['api_calls'][url] || 0) + 1;
        }
        return originalFetch.apply(this, arguments);
      };

      // Track wallet interactions
      if ((window as any).ethereum) {
        const originalRequest = (window as any).ethereum.request;
        (window as any).ethereum.request = async function(args: any) {
          if (!(window as any).__coverage__['wallet_methods']) {
            (window as any).__coverage__['wallet_methods'] = {};
          }
          (window as any).__coverage__['wallet_methods'][args.method] = 
            ((window as any).__coverage__['wallet_methods'][args.method] || 0) + 1;
          return originalRequest.apply(this, arguments);
        };
      }
    });

    await use(context);

    // Collect coverage data after test
    const pages = context.pages();
    for (const page of pages) {
      try {
        const coverage = await page.evaluate(() => (window as any).__coverage__);
        if (coverage) {
          await saveCoverageData(coverage, `coverage-${Date.now()}.json`);
        }
      } catch (error) {
        console.warn('Failed to collect coverage:', error);
      }
    }

    await context.close();
  },

  page: async ({ context }, use) => {
    const page = await context.newPage();
    
    // Set up console logging for debugging
    page.on('console', msg => {
      if (msg.type() === 'error') {
        console.error('Browser console error:', msg.text());
      }
    });

    // Track page navigation for coverage
    page.on('framenavigated', (frame) => {
      if (frame === page.mainFrame()) {
        console.log('Navigated to:', frame.url());
      }
    });

    await use(page);
  },
});

// Helper function to save coverage data
async function saveCoverageData(coverage: any, filename: string) {
  const coverageDir = path.join(process.cwd(), 'coverage', 'e2e');
  
  if (!fs.existsSync(coverageDir)) {
    fs.mkdirSync(coverageDir, { recursive: true });
  }

  const filePath = path.join(coverageDir, filename);
  fs.writeFileSync(filePath, JSON.stringify(coverage, null, 2));
  console.log(`Coverage data saved to: ${filePath}`);
}

// Coverage utilities
export class CoverageTracker {
  private static instance: CoverageTracker;
  private coverageData: Map<string, any> = new Map();

  static getInstance(): CoverageTracker {
    if (!CoverageTracker.instance) {
      CoverageTracker.instance = new CoverageTracker();
    }
    return CoverageTracker.instance;
  }

  addCoverage(testName: string, data: any) {
    this.coverageData.set(testName, data);
  }

  getCoverageReport(): any {
    const report = {
      totalTests: this.coverageData.size,
      coverage: {
        functions: {},
        apiCalls: {},
        walletMethods: {},
        components: {}
      }
    };

    // Aggregate coverage data
    for (const [testName, data] of this.coverageData) {
      if (data.functions) {
        Object.assign(report.coverage.functions, data.functions);
      }
      if (data.api_calls) {
        Object.assign(report.coverage.apiCalls, data.api_calls);
      }
      if (data.wallet_methods) {
        Object.assign(report.coverage.walletMethods, data.wallet_methods);
      }
    }

    return report;
  }

  generateReport(): string {
    const report = this.getCoverageReport();
    const functionsCount = Object.keys(report.coverage.functions).length;
    const apiCallsCount = Object.keys(report.coverage.apiCalls).length;
    const walletMethodsCount = Object.keys(report.coverage.walletMethods).length;

    return `
# E2E Test Coverage Report

## Summary
- Total Tests: ${report.totalTests}
- Functions Covered: ${functionsCount}
- API Endpoints Covered: ${apiCallsCount}
- Wallet Methods Covered: ${walletMethodsCount}

## Function Coverage
${Object.entries(report.coverage.functions)
  .map(([name, count]) => `- ${name}: ${count} calls`)
  .join('\n')}

## API Endpoint Coverage
${Object.entries(report.coverage.apiCalls)
  .map(([url, count]) => `- ${url}: ${count} calls`)
  .join('\n')}

## Wallet Method Coverage
${Object.entries(report.coverage.walletMethods)
  .map(([method, count]) => `- ${method}: ${count} calls`)
  .join('\n')}
    `;
  }

  saveFinalReport() {
    const report = this.generateReport();
    const reportPath = path.join(process.cwd(), 'coverage', 'e2e-coverage-report.md');
    
    const coverageDir = path.dirname(reportPath);
    if (!fs.existsSync(coverageDir)) {
      fs.mkdirSync(coverageDir, { recursive: true });
    }

    fs.writeFileSync(reportPath, report);
    console.log(`E2E Coverage report saved to: ${reportPath}`);
  }
}

// Test helpers for comprehensive coverage
export const WalletTestHelpers = {
  // Test all supported wallet types
  async testAllWalletTypes(page: Page, callback: (walletType: string, address: string) => Promise<void>) {
    const walletTypes = [
      { type: 'metamask', address: '0x1234567890123456789012345678901234567890' },
      { type: 'walletconnect', address: '0x742d35Cc6634C0532925a3b8D369D7763F3c45c6' },
      { type: 'coinbase', address: '0xabcdef0123456789012345678901234567890123' }
    ];

    for (const { type, address } of walletTypes) {
      await callback(type, address);
    }
  },

  // Test all permission types
  async testAllPermissionTypes(page: Page, callback: (permissionType: string, permissions: any[]) => Promise<void>) {
    const permissionTypes = [
      {
        type: 'nft',
        permissions: [
          { permission: 'nft:holder:access', source: 'nft' },
          { permission: 'premium:features:access', source: 'nft' }
        ]
      },
      {
        type: 'token',
        permissions: [
          { permission: 'token:holder:access', source: 'token' },
          { permission: 'advanced:trading:access', source: 'token' }
        ]
      },
      {
        type: 'dao',
        permissions: [
          { permission: 'dao:member:access', source: 'dao' },
          { permission: 'governance:voting:access', source: 'dao' }
        ]
      },
      {
        type: 'enterprise',
        permissions: [
          { permission: 'enterprise:full:access', source: 'manual' },
          { permission: 'api:unlimited:access', source: 'manual' }
        ]
      }
    ];

    for (const { type, permissions } of permissionTypes) {
      await callback(type, permissions);
    }
  },

  // Test all error scenarios
  async testAllErrorScenarios(page: Page, callback: (scenario: string, config: any) => Promise<void>) {
    const errorScenarios = [
      { scenario: 'connection_rejected', config: { rejectConnection: true } },
      { scenario: 'signing_rejected', config: { rejectSigning: true } },
      { scenario: 'invalid_signature', config: { invalidSignature: true } },
      { scenario: 'network_error', config: { networkError: true } },
      { scenario: 'api_unavailable', config: { apiError: 503 } },
      { scenario: 'session_expired', config: { sessionExpired: true } }
    ];

    for (const { scenario, config } of errorScenarios) {
      await callback(scenario, config);
    }
  },

  // Test all responsive breakpoints
  async testAllViewports(page: Page, callback: (viewport: string, size: { width: number; height: number }) => Promise<void>) {
    const viewports = [
      { viewport: 'mobile', size: { width: 375, height: 667 } },
      { viewport: 'tablet', size: { width: 768, height: 1024 } },
      { viewport: 'desktop', size: { width: 1920, height: 1080 } },
      { viewport: 'mobile_landscape', size: { width: 667, height: 375 } }
    ];

    for (const { viewport, size } of viewports) {
      await page.setViewportSize(size);
      await callback(viewport, size);
    }
  }
};

export { expect };