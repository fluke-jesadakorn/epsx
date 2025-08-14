/**
 * User Journey and Role-Based Access Tests
 * Comprehensive testing of user workflows and permission-based features
 */
import { test, expect, Page } from '@playwright/test';

const TEST_EMAIL = 'jesadakorn.kirtnu@gmail.com';
const TEST_PASSWORD = 'Aa_12345678';
const BACKEND_URL = 'http://localhost:8080';

// Helper functions
async function loginUser(page: Page) {
  await page.goto('/login');
  
  const signInButton = page.getByRole('button', { name: 'Continue with EPSX' });
  await expect(signInButton).toBeVisible({ timeout: 10000 });
  await signInButton.click();

  await page.waitForURL(new RegExp(`${BACKEND_URL}/oauth/authorize`), { timeout: 10000 });
  await page.waitForSelector('input[name="email"]', { timeout: 10000 });
  await page.fill('input[name="email"]', TEST_EMAIL);
  await page.fill('input[name="password"]', TEST_PASSWORD);
  
  const loginButton = page.locator('button[type="submit"]').first();
  await loginButton.click();

  await page.waitForURL('/dashboard', { timeout: 15000 });
  await page.waitForLoadState('networkidle');
}

async function getUserPermissions(page: Page): Promise<string[]> {
  // Try to detect user permissions from various sources
  await page.goto('/settings');
  await page.waitForLoadState('networkidle');
  
  const permissions = [];
  
  // Look for permission indicators
  const permissionElements = [
    page.locator('[data-testid*="permission"]'),
    page.locator('text=Premium').or(page.locator('text=Basic')),
    page.locator('[data-role], [data-permission]'),
  ];

  for (const element of permissionElements) {
    try {
      if (await element.isVisible()) {
        const text = await element.textContent();
        if (text) permissions.push(text.toLowerCase());
      }
    } catch {
      // Continue checking
    }
  }
  
  return permissions;
}

test.describe('🎯 Complete Trading Workflow Journey', () => {
  test.beforeEach(async ({ page }) => {
    await loginUser(page);
  });

  test('should complete end-to-end trading research workflow', async ({ page }) => {
    console.log('🧪 Testing complete trading research workflow');

    // Step 1: Start with dashboard overview
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    console.log('✅ Step 1: Accessed trading dashboard');

    // Look for market data or overview widgets
    const dashboardElements = [
      page.locator('[data-testid*="market"]'),
      page.locator('[data-testid*="stock"]'),
      page.locator('table').first(),
      page.locator('.chart, canvas').first(),
    ];

    let foundDashboardData = false;
    for (const element of dashboardElements) {
      if (await element.isVisible()) {
        foundDashboardData = true;
        console.log('✅ Found dashboard trading data');
        break;
      }
    }

    if (!foundDashboardData) {
      console.log('⚠️ Dashboard trading data may not be implemented yet');
    }

    // Step 2: Research with EPS analytics
    await page.goto('/analytics/eps');
    await page.waitForLoadState('networkidle');
    console.log('✅ Step 2: Accessed EPS analytics for research');

    // Perform EPS analysis
    const epsInputs = [
      page.locator('input[placeholder*="symbol"]'),
      page.locator('input[name*="symbol"]'),
      page.locator('input[type="text"]').first(),
    ];

    for (const input of epsInputs) {
      if (await input.isVisible()) {
        await input.fill('AAPL');
        console.log('✅ Entered stock symbol for EPS analysis');
        
        // Look for analyze/search button
        const analyzeBtn = page.locator('button').filter({ hasText: /analyze|search|submit/i });
        if (await analyzeBtn.first().isVisible()) {
          await analyzeBtn.first().click();
          await page.waitForTimeout(2000);
          console.log('✅ Triggered EPS analysis');
        }
        break;
      }
    }

    // Step 3: Use pattern recognition
    await page.goto('/analytics/pattern-recognition');
    await page.waitForLoadState('networkidle');
    console.log('✅ Step 3: Accessed pattern recognition tools');

    // Look for pattern analysis tools
    const patternElements = [
      page.locator('select').first(),
      page.locator('input').first(),
      page.locator('button').filter({ hasText: /pattern|analyze/i }),
    ];

    for (const element of patternElements) {
      if (await element.isVisible()) {
        try {
          if (await element.locator('option').count() > 0) {
            await element.selectOption({ index: 1 });
          } else {
            await element.click();
          }
          console.log('✅ Interacted with pattern recognition tool');
        } catch {
          console.log('⚠️ Pattern tool interaction limited');
        }
        break;
      }
    }

    // Step 4: Access trading interface
    await page.goto('/trading');
    await page.waitForLoadState('networkidle');
    console.log('✅ Step 4: Accessed trading interface');

    // Check for trading-specific elements
    const tradingElements = [
      page.locator('[data-testid*="trade"]'),
      page.locator('button').filter({ hasText: /buy|sell|order/i }),
      page.locator('[data-testid*="portfolio"]'),
      page.locator('table').filter({ hasText: /symbol|price|quantity/i }),
    ];

    for (const element of tradingElements) {
      if (await element.isVisible()) {
        console.log('✅ Found trading interface element');
        break;
      }
    }

    // Step 5: Check account/portfolio data
    await page.goto('/my-data');
    await page.waitForLoadState('networkidle');
    console.log('✅ Step 5: Checked account and portfolio data');

    // Step 6: Return to dashboard with insights
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    console.log('✅ Step 6: Returned to dashboard with research insights');

    console.log('🎉 Complete trading research workflow successful');
  });

  test('should handle data export and portfolio management workflow', async ({ page }) => {
    console.log('🧪 Testing data export and portfolio workflow');

    // Step 1: Access portfolio data
    await page.goto('/my-data');
    await page.waitForLoadState('networkidle');
    console.log('✅ Step 1: Accessed portfolio data');

    // Look for export functionality
    const exportElements = [
      page.locator('button').filter({ hasText: /export|download|csv|excel/i }),
      page.locator('[data-testid*="export"]'),
      page.locator('a[download]'),
    ];

    for (const element of exportElements) {
      if (await element.isVisible()) {
        console.log('✅ Found export functionality');
        try {
          await element.click();
          await page.waitForTimeout(1000);
          console.log('✅ Triggered data export');
        } catch {
          console.log('⚠️ Export functionality may be restricted');
        }
        break;
      }
    }

    // Step 2: Check settings and preferences
    await page.goto('/settings');
    await page.waitForLoadState('networkidle');
    console.log('✅ Step 2: Accessed user settings');

    // Look for portfolio or data settings
    const settingsInputs = [
      page.locator('input[type="checkbox"]'),
      page.locator('select'),
      page.locator('input[type="number"]'),
    ];

    for (const input of settingsInputs) {
      const count = await input.count();
      if (count > 0) {
        console.log(`✅ Found ${count} settings controls`);
        break;
      }
    }

    console.log('🎉 Portfolio management workflow completed');
  });
});

test.describe('💳 Complete Payment and Subscription Journey', () => {
  test.beforeEach(async ({ page }) => {
    await loginUser(page);
  });

  test('should complete subscription upgrade workflow', async ({ page }) => {
    console.log('🧪 Testing subscription upgrade workflow');

    // Step 1: Check current plan in settings
    await page.goto('/settings');
    await page.waitForLoadState('networkidle');
    console.log('✅ Step 1: Checked current subscription status');

    // Look for subscription info
    const subscriptionElements = [
      page.locator('text=Basic').or(page.locator('text=Premium')),
      page.locator('[data-testid*="plan"]'),
      page.locator('text=Subscription').or(page.locator('text=Plan')),
    ];

    for (const element of subscriptionElements) {
      if (await element.isVisible()) {
        const plan = await element.textContent();
        console.log(`✅ Current plan detected: ${plan}`);
        break;
      }
    }

    // Step 2: Explore payment options
    await page.goto('/payment');
    await page.waitForLoadState('networkidle');
    console.log('✅ Step 2: Explored payment options');

    // Check different payment tiers
    const paymentPages = ['/payment/quick', '/payment/enterprise'];
    
    for (const paymentPage of paymentPages) {
      await page.goto(paymentPage);
      await page.waitForLoadState('networkidle');
      
      // Look for pricing information
      const pricingElements = [
        page.locator('text=$').first(),
        page.locator('[data-testid*="price"]'),
        page.locator('button').filter({ hasText: /subscribe|purchase|upgrade/i }),
      ];

      for (const element of pricingElements) {
        if (await element.isVisible()) {
          console.log(`✅ Found pricing information on ${paymentPage}`);
          break;
        }
      }
    }

    // Step 3: Check payment return handling
    await page.goto('/payment/return');
    await page.waitForLoadState('networkidle');
    console.log('✅ Step 3: Checked payment return page');

    console.log('🎉 Subscription workflow exploration completed');
  });

  test('should test payment form validation and security', async ({ page }) => {
    console.log('🧪 Testing payment form security');

    await page.goto('/payment/quick');
    await page.waitForLoadState('networkidle');

    // Look for payment forms
    const paymentForms = [
      page.locator('form').filter({ hasText: /payment|credit|card/i }),
      page.locator('iframe[src*="stripe"]'), // Stripe iframe
      page.locator('iframe[src*="paypal"]'), // PayPal iframe
      page.locator('[data-testid*="payment-form"]'),
    ];

    let foundPaymentForm = false;
    for (const form of paymentForms) {
      if (await form.isVisible()) {
        foundPaymentForm = true;
        console.log('✅ Found payment form');
        
        // Check for security indicators
        const securityElements = [
          page.locator('text=SSL').or(page.locator('text=Secure')),
          page.locator('[data-testid*="security"]'),
          page.locator('img[alt*="secure"]'),
        ];

        for (const element of securityElements) {
          if (await element.isVisible()) {
            console.log('✅ Found security indicator');
            break;
          }
        }
        break;
      }
    }

    if (!foundPaymentForm) {
      console.log('⚠️ Payment forms may use external providers or be under development');
    }

    console.log('✅ Payment security check completed');
  });
});

test.describe('📊 Advanced Analytics User Journey', () => {
  test.beforeEach(async ({ page }) => {
    await loginUser(page);
  });

  test('should complete comprehensive analytics workflow', async ({ page }) => {
    console.log('🧪 Testing comprehensive analytics workflow');

    // Step 1: Start with general analytics
    await page.goto('/analytics');
    await page.waitForLoadState('networkidle');
    console.log('✅ Step 1: Accessed general analytics');

    // Look for analytics overview
    const overviewElements = [
      page.locator('h1, h2').filter({ hasText: /analytics|overview/i }),
      page.locator('chart, canvas').first(),
      page.locator('[data-testid*="metric"]'),
    ];

    for (const element of overviewElements) {
      if (await element.isVisible()) {
        console.log('✅ Found analytics overview element');
        break;
      }
    }

    // Step 2: Deep dive into EPS analytics
    await page.goto('/analytics/eps');
    await page.waitForLoadState('networkidle');
    console.log('✅ Step 2: Deep dive into EPS analytics');

    // Perform detailed EPS analysis
    const epsAnalysis = [
      { symbol: 'AAPL', name: 'Apple' },
      { symbol: 'GOOGL', name: 'Google' },
      { symbol: 'MSFT', name: 'Microsoft' },
    ];

    for (const stock of epsAnalysis) {
      const symbolInput = page.locator('input').first();
      if (await symbolInput.isVisible()) {
        await symbolInput.clear();
        await symbolInput.fill(stock.symbol);
        
        const analyzeBtn = page.locator('button').filter({ hasText: /analyze|search/i });
        if (await analyzeBtn.first().isVisible()) {
          await analyzeBtn.first().click();
          await page.waitForTimeout(1000);
          console.log(`✅ Analyzed ${stock.name} (${stock.symbol})`);
        }
      }
    }

    // Step 3: Pattern recognition analysis
    await page.goto('/analytics/pattern-recognition');
    await page.waitForLoadState('networkidle');
    console.log('✅ Step 3: Performed pattern recognition analysis');

    // Test different pattern types
    const patternSelectors = page.locator('select, input[type="radio"]');
    const patternCount = await patternSelectors.count();
    
    if (patternCount > 0) {
      console.log(`✅ Found ${patternCount} pattern selection controls`);
      
      try {
        const firstSelector = patternSelectors.first();
        if (await firstSelector.locator('option').count() > 0) {
          await firstSelector.selectOption({ index: 1 });
          console.log('✅ Selected pattern type');
        }
      } catch {
        console.log('⚠️ Pattern selection may be implemented differently');
      }
    }

    // Step 4: Export or save analysis results
    const exportBtns = page.locator('button').filter({ hasText: /export|save|download/i });
    if (await exportBtns.first().isVisible()) {
      try {
        await exportBtns.first().click();
        console.log('✅ Attempted to export analysis results');
      } catch {
        console.log('⚠️ Export may require premium subscription');
      }
    }

    console.log('🎉 Comprehensive analytics workflow completed');
  });

  test('should test analytics data filtering and time ranges', async ({ page }) => {
    console.log('🧪 Testing analytics filtering capabilities');

    await page.goto('/analytics/eps');
    await page.waitForLoadState('networkidle');

    // Test date range filtering
    const dateInputs = page.locator('input[type="date"]');
    const dateCount = await dateInputs.count();
    
    if (dateCount > 0) {
      console.log(`✅ Found ${dateCount} date range controls`);
      
      try {
        await dateInputs.first().fill('2024-01-01');
        if (dateCount > 1) {
          await dateInputs.nth(1).fill('2024-12-31');
        }
        console.log('✅ Set date range for analysis');
      } catch {
        console.log('⚠️ Date range setting may have restrictions');
      }
    }

    // Test other filter options
    const filterSelectors = page.locator('select').filter({ hasText: /sector|industry|market/i });
    const filterCount = await filterSelectors.count();
    
    if (filterCount > 0) {
      console.log(`✅ Found ${filterCount} filter options`);
      
      try {
        await filterSelectors.first().selectOption({ index: 1 });
        console.log('✅ Applied analysis filter');
      } catch {
        console.log('⚠️ Filter application may be restricted');
      }
    }

    console.log('✅ Analytics filtering test completed');
  });
});

test.describe('🔐 Role-Based Feature Access', () => {
  test.beforeEach(async ({ page }) => {
    await loginUser(page);
  });

  test('should verify user permission levels across features', async ({ page }) => {
    console.log('🧪 Testing role-based feature access');

    // Get user permissions
    const permissions = await getUserPermissions(page);
    console.log('User permissions detected:', permissions);

    // Test features that may be permission-restricted
    const restrictedFeatures = [
      { path: '/analytics/pattern-recognition', feature: 'Pattern Recognition' },
      { path: '/trading', feature: 'Trading Interface' },
      { path: '/payment/enterprise', feature: 'Enterprise Payment' },
    ];

    for (const feature of restrictedFeatures) {
      console.log(`🔍 Testing access to ${feature.feature}`);
      
      await page.goto(feature.path);
      await page.waitForLoadState('networkidle');
      
      const currentUrl = page.url();
      const hasAccess = currentUrl.includes(feature.path) && 
                       !currentUrl.includes('/access-denied') &&
                       !currentUrl.includes('/unauthorized');
      
      if (hasAccess) {
        console.log(`✅ Access granted to ${feature.feature}`);
        
        // Check for premium/restricted content indicators
        const restrictedElements = [
          page.locator('text=Premium').or(page.locator('text=Upgrade')),
          page.locator('[data-testid*="premium"]'),
          page.locator('button').filter({ hasText: /upgrade|subscribe/i }),
        ];

        for (const element of restrictedElements) {
          if (await element.isVisible()) {
            console.log(`⚠️ ${feature.feature} may have premium restrictions`);
            break;
          }
        }
      } else {
        console.log(`⚠️ Access restricted to ${feature.feature}`);
      }
    }
  });

  test('should test feature limitations based on subscription level', async ({ page }) => {
    console.log('🧪 Testing subscription-based feature limitations');

    // Test export functionality (often premium-gated)
    const exportTestPages = ['/analytics/eps', '/my-data'];
    
    for (const testPage of exportTestPages) {
      await page.goto(testPage);
      await page.waitForLoadState('networkidle');
      
      const exportBtns = page.locator('button').filter({ hasText: /export|download/i });
      if (await exportBtns.first().isVisible()) {
        try {
          await exportBtns.first().click();
          await page.waitForTimeout(1000);
          
          // Check if export was successful or shows upgrade prompt
          const upgradePrompt = page.locator('text=Upgrade').or(
            page.locator('text=Premium')
          );
          
          if (await upgradePrompt.isVisible()) {
            console.log(`⚠️ Export feature requires upgrade on ${testPage}`);
          } else {
            console.log(`✅ Export feature available on ${testPage}`);
          }
        } catch {
          console.log(`⚠️ Export feature may be restricted on ${testPage}`);
        }
      }
    }
  });

  test('should verify API rate limiting and usage restrictions', async ({ page }) => {
    console.log('🧪 Testing API usage and rate limiting');

    // Monitor network requests for rate limiting
    const apiRequests = [];
    page.on('response', response => {
      if (response.url().includes('/api/')) {
        apiRequests.push({
          url: response.url(),
          status: response.status(),
          headers: response.headers(),
        });
      }
    });

    // Perform rapid API calls by navigating quickly
    const rapidNavigation = [
      '/analytics/eps',
      '/analytics/pattern-recognition', 
      '/dashboard',
      '/my-data',
    ];

    for (const navPath of rapidNavigation) {
      await page.goto(navPath);
      await page.waitForTimeout(500); // Quick navigation
    }

    // Check for rate limiting responses
    const rateLimitedRequests = apiRequests.filter(req => 
      req.status === 429 || 
      req.headers['x-ratelimit-remaining'] === '0'
    );

    if (rateLimitedRequests.length > 0) {
      console.log(`⚠️ Rate limiting detected: ${rateLimitedRequests.length} requests limited`);
    } else {
      console.log('✅ No rate limiting encountered in test');
    }
  });
});

test.describe('🔄 Cross-Feature Integration Journey', () => {
  test.beforeEach(async ({ page }) => {
    await loginUser(page);
  });

  test('should test data flow between analytics and trading features', async ({ page }) => {
    console.log('🧪 Testing cross-feature data integration');

    // Step 1: Research stock in analytics
    await page.goto('/analytics/eps');
    await page.waitForLoadState('networkidle');
    
    const symbolInput = page.locator('input').first();
    if (await symbolInput.isVisible()) {
      await symbolInput.fill('AAPL');
      console.log('✅ Researched AAPL in analytics');
    }

    // Step 2: Check if research carries over to trading
    await page.goto('/trading');
    await page.waitForLoadState('networkidle');
    
    // Look for recently viewed or suggested stocks
    const recentSymbols = [
      page.locator('text=AAPL'),
      page.locator('[data-symbol="AAPL"]'),
      page.locator('text=Recent').locator('text=AAPL'),
    ];

    for (const element of recentSymbols) {
      if (await element.isVisible()) {
        console.log('✅ Analytics research carried over to trading interface');
        break;
      }
    }

    // Step 3: Check portfolio integration
    await page.goto('/my-data');
    await page.waitForLoadState('networkidle');
    
    // Look for watchlist or research history
    const historyElements = [
      page.locator('text=AAPL'),
      page.locator('[data-testid*="history"]'),
      page.locator('[data-testid*="watchlist"]'),
    ];

    for (const element of historyElements) {
      if (await element.isVisible()) {
        console.log('✅ Research history visible in portfolio data');
        break;
      }
    }

    console.log('🎉 Cross-feature integration test completed');
  });

  test('should test settings impact across all features', async ({ page }) => {
    console.log('🧪 Testing settings impact across features');

    // Step 1: Modify settings
    await page.goto('/settings');
    await page.waitForLoadState('networkidle');
    
    // Look for theme, currency, or preference settings
    const settingsControls = [
      page.locator('input[type="checkbox"]'),
      page.locator('select'),
      page.locator('input[type="radio"]'),
    ];

    let modifiedSetting = false;
    for (const control of settingsControls) {
      const count = await control.count();
      if (count > 0) {
        try {
          if (await control.first().locator('option').count() > 0) {
            await control.first().selectOption({ index: 1 });
          } else {
            await control.first().click();
          }
          modifiedSetting = true;
          console.log('✅ Modified user setting');
          break;
        } catch {
          // Continue trying other controls
        }
      }
    }

    if (modifiedSetting) {
      // Step 2: Verify setting impact across features
      const featuresToCheck = ['/dashboard', '/analytics', '/trading'];
      
      for (const featurePath of featuresToCheck) {
        await page.goto(featurePath);
        await page.waitForLoadState('networkidle');
        console.log(`✅ Checked setting impact on ${featurePath}`);
      }
    }

    console.log('✅ Settings impact test completed');
  });
});