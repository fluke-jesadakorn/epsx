/**
 * Comprehensive Test Configuration
 * Centralized configuration for all E2E tests including timeouts, retries, and environment settings
 */

export interface TestEnvironmentConfig {
  baseUrl: string;
  apiUrl: string;
  adminUrl: string;
  timeout: {
    action: number;
    navigation: number;
    test: number;
    expect: number;
  };
  retries: {
    flaky: number;
    critical: number;
  };
  performance: {
    thresholds: {
      loadTime: Record<string, number>;
      apiResponse: Record<string, number>;
      coreWebVitals: {
        lcp: number;
        fcp: number;
        cls: number;
      };
    };
  };
  tiers: {
    [key: string]: {
      features: string[];
      rateLimits: {
        perMinute: number;
        perHour: number;
      };
      performanceExpectations: {
        maxLoadTime: number;
        maxApiResponse: number;
      };
    };
  };
}

/**
 * Default test configuration
 */
export const TEST_CONFIG: TestEnvironmentConfig = {
  baseUrl: process.env.PLAYWRIGHT_BASE_URL || 'http://localhost:3000',
  apiUrl: process.env.PLAYWRIGHT_API_URL || 'http://localhost:8080',
  adminUrl: process.env.PLAYWRIGHT_ADMIN_URL || 'http://localhost:3001',

  timeout: {
    action: 15000,      // 15 seconds for actions like click, fill
    navigation: 30000,  // 30 seconds for page navigation
    test: 60000,        // 60 seconds for entire test
    expect: 10000       // 10 seconds for expect assertions
  },

  retries: {
    flaky: 2,          // Retry flaky tests 2 times
    critical: 3        // Retry critical tests 3 times
  },

  performance: {
    thresholds: {
      loadTime: {
        FREE: 3000,
        BRONZE: 2500,
        SILVER: 2000,
        GOLD: 1500,
        PLATINUM: 1200,
        ENTERPRISE: 1000
      },
      apiResponse: {
        FREE: 2000,
        BRONZE: 1500,
        SILVER: 1000,
        GOLD: 750,
        PLATINUM: 500,
        ENTERPRISE: 250
      },
      coreWebVitals: {
        lcp: 2500,     // Largest Contentful Paint
        fcp: 1800,     // First Contentful Paint
        cls: 0.1       // Cumulative Layout Shift
      }
    }
  },

  tiers: {
    FREE: {
      features: ['basic-analytics', 'portfolio-view', 'basic-notifications'],
      rateLimits: { perMinute: 10, perHour: 100 },
      performanceExpectations: { maxLoadTime: 3000, maxApiResponse: 2000 }
    },
    BRONZE: {
      features: ['basic-analytics', 'portfolio-view', 'enhanced-notifications', 'portfolio-history'],
      rateLimits: { perMinute: 30, perHour: 500 },
      performanceExpectations: { maxLoadTime: 2500, maxApiResponse: 1500 }
    },
    SILVER: {
      features: ['basic-analytics', 'portfolio-view', 'enhanced-notifications', 'portfolio-history', 'advanced-analytics'],
      rateLimits: { perMinute: 60, perHour: 1500 },
      performanceExpectations: { maxLoadTime: 2000, maxApiResponse: 1000 }
    },
    GOLD: {
      features: ['basic-analytics', 'portfolio-view', 'enhanced-notifications', 'portfolio-history', 'advanced-analytics', 'portfolio-tools', 'priority-support'],
      rateLimits: { perMinute: 120, perHour: 5000 },
      performanceExpectations: { maxLoadTime: 1500, maxApiResponse: 750 }
    },
    PLATINUM: {
      features: ['basic-analytics', 'portfolio-view', 'enhanced-notifications', 'portfolio-history', 'advanced-analytics', 'portfolio-tools', 'priority-support', 'research-reports', 'custom-dashboards'],
      rateLimits: { perMinute: 300, perHour: 15000 },
      performanceExpectations: { maxLoadTime: 1200, maxApiResponse: 500 }
    },
    ENTERPRISE: {
      features: ['all-features', 'api-access', 'institutional-features', 'bulk-operations'],
      rateLimits: { perMinute: 1000, perHour: 50000 },
      performanceExpectations: { maxLoadTime: 1000, maxApiResponse: 250 }
    }
  }
};

/**
 * Test selectors and data attributes
 */
export const TEST_SELECTORS = {
  // Authentication
  auth: {
    loginForm: '[data-testid="login-form"]',
    emailInput: '[data-testid="email-input"]',
    passwordInput: '[data-testid="password-input"]',
    loginButton: '[data-testid="login-button"]',
    logoutButton: '[data-testid="logout-button"]',
    userProfile: '[data-testid="user-profile"]',
    tierBadge: '[data-testid="tier-badge"]'
  },

  // Navigation
  navigation: {
    mainNav: '[data-testid="main-nav"]',
    mobileNav: '[data-testid="mobile-nav"]',
    sidebar: '[data-testid="sidebar"]',
    breadcrumb: '[data-testid="breadcrumb"]',
    bottomNav: '[data-testid="bottom-nav"]'
  },

  // Market Data
  market: {
    interface: '[data-testid=\"market-interface\"]',
    buyButton: '[data-testid=\"buy-button\"]',
    sellButton: '[data-testid=\"sell-button\"]',
    orderForm: '[data-testid=\"order-form\"]',
    orderHistory: '[data-testid=\"order-history\"]',
    priceChart: '[data-testid=\"price-chart\"]'
  },

  // Portfolio
  portfolio: {
    balance: '[data-testid=\"portfolio-balance\"]',
    positions: '[data-testid=\"positions-table\"]',
    performance: '[data-testid=\"performance-metrics\"]',
    history: '[data-testid=\"portfolio-history\"]',
    optimizer: '[data-testid=\"portfolio-optimizer\"]'
  },

  // Analytics
  analytics: {
    dashboard: '[data-testid=\"analytics-dashboard\"]',
    charts: '[data-testid=\"analytics-charts\"]',
    indicators: '[data-testid=\"technical-indicators\"]',
    reports: '[data-testid=\"analytics-reports\"]'
  },

  // Subscription
  subscription: {
    upgradeButton: '[data-testid="upgrade-button"]',
    tierComparison: '[data-testid="tier-comparison"]',
    paymentForm: '[data-testid="payment-form"]',
    billingHistory: '[data-testid="billing-history"]',
    cancelButton: '[data-testid="cancel-subscription"]'
  },

  // Mobile specific
  mobile: {
    menuButton: '[data-testid="mobile-menu-button"]',
    touchArea: '[data-testid="touch-area"]',
    swipeContainer: '[data-testid="swipe-container"]',
    pullToRefresh: '[data-testid="pull-to-refresh"]'
  },

  // Performance indicators
  performance: {
    loadingSpinner: '[data-testid="loading-spinner"]',
    pageLoaded: '[data-testid="page-loaded"]',
    lazyLoading: '[data-lazy="true"]',
    virtualScroll: '[data-virtual-scroll="true"]'
  },

  // Error states
  errors: {
    errorBoundary: '[data-testid="error-boundary"]',
    errorMessage: '[data-testid="error-message"]',
    retryButton: '[data-testid="retry-button"]',
    fallbackContent: '[data-testid="fallback-content"]'
  }
};

/**
 * Mock API responses for testing
 */
export const MOCK_RESPONSES = {
  auth: {
    validSession: {
      valid: true,
      user: {
        id: 'user123',
        email: 'test@example.com',
        role: 'user',
        package_tier: 'GOLD',
        permissions: ['analytics:basic', 'portfolio:view']
      },
      performance: {
        validation_time_ms: 45,
        cache_hit: true
      }
    },
    invalidSession: {
      valid: false,
      error: 'Session expired'
    }
  },

  portfolio: {
    balance: {
      total: 125000.50,
      available: 120000.00,
      pending: 5000.50,
      currency: 'USD'
    },
    positions: [
      {
        symbol: 'AAPL',
        quantity: 100,
        current_price: 150.00,
        market_value: 15000.00,
        unrealized_pnl: 500.00
      }
    ]
  },

  analytics: {
    basic: {
      daily_pnl: 1250.75,
      portfolio_value: 125000.50,
      best_performer: 'AAPL',
      worst_performer: 'TSLA'
    },
    advanced: {
      risk_metrics: {
        beta: 1.2,
        sharpe_ratio: 1.8,
        max_drawdown: 0.15
      },
      correlations: {
        spy: 0.85,
        qqq: 0.78
      }
    }
  },

  subscription: {
    current: {
      tier: 'GOLD',
      status: 'active',
      expires_at: '2025-01-01T00:00:00Z',
      features: ['portfolio-tools', 'advanced-analytics']
    }
  }
};

/**
 * Test data generators
 */
export const TEST_DATA_GENERATORS = {
  user: (tier: string) => ({
    id: `user_${tier.toLowerCase()}_${Date.now()}`,
    email: `${tier.toLowerCase()}@test.epsx.io`,
    name: `${tier} Test User`,
    package_tier: tier,
    created_at: new Date().toISOString()
  }),

  transaction: () => ({
    id: `tx_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
    symbol: ['AAPL', 'GOOGL', 'MSFT', 'AMZN'][Math.floor(Math.random() * 4)],
    action: ['BUY', 'SELL'][Math.floor(Math.random() * 2)],
    quantity: Math.floor(Math.random() * 100) + 1,
    price: Math.round((Math.random() * 1000 + 50) * 100) / 100,
    timestamp: new Date().toISOString()
  }),

  portfolioData: (balance: number = 100000) => ({
    balance,
    positions: Array.from({ length: 5 }, (_, i) => ({
      symbol: ['AAPL', 'GOOGL', 'MSFT', 'AMZN', 'TSLA'][i],
      quantity: Math.floor(Math.random() * 100) + 10,
      current_price: Math.round((Math.random() * 500 + 50) * 100) / 100,
      cost_basis: Math.round((Math.random() * 500 + 45) * 100) / 100
    }))
  })
};

/**
 * Environment-specific configurations
 */
export const ENVIRONMENT_CONFIG = {
  development: {
    ...TEST_CONFIG,
    timeout: {
      ...TEST_CONFIG.timeout,
      test: 120000  // Longer timeouts in development
    }
  },

  ci: {
    ...TEST_CONFIG,
    retries: {
      flaky: 3,
      critical: 5  // More retries in CI
    },
    timeout: {
      ...TEST_CONFIG.timeout,
      test: 90000   // Longer timeouts in CI
    }
  },

  production: {
    ...TEST_CONFIG,
    performance: {
      ...TEST_CONFIG.performance,
      thresholds: {
        ...TEST_CONFIG.performance.thresholds,
        // Stricter performance requirements in production
        loadTime: Object.fromEntries(
          Object.entries(TEST_CONFIG.performance.thresholds.loadTime)
            .map(([tier, time]) => [tier, time * 0.8])
        )
      }
    }
  }
};

/**
 * Get configuration for current environment
 */
export function getTestConfig(): TestEnvironmentConfig {
  const env = process.env.NODE_ENV || 'development';
  const isCI = process.env.CI === 'true';

  if (isCI) {
    return ENVIRONMENT_CONFIG.ci;
  }

  return ENVIRONMENT_CONFIG[env as keyof typeof ENVIRONMENT_CONFIG] || TEST_CONFIG;
}

/**
 * Validation helpers
 */
export const VALIDATORS = {
  email: (email: string): boolean => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email),

  currency: (amount: string): boolean => /^\$?[\d,]+\.?\d{0,2}$/.test(amount),

  percentage: (percent: string): boolean => /^-?\d+\.?\d{0,2}%?$/.test(percent),

  tierAccess: (userTier: string, requiredTier: string): boolean => {
    const tierLevels = { FREE: 1, BRONZE: 2, SILVER: 3, GOLD: 4, PLATINUM: 5, ENTERPRISE: 6 };
    return (tierLevels[userTier as keyof typeof tierLevels] || 0) >=
      (tierLevels[requiredTier as keyof typeof tierLevels] || 0);
  },

  performance: (metric: number, threshold: number): boolean => metric <= threshold,

  responseTime: (responseTime: number, tier: string): boolean => {
    const threshold = TEST_CONFIG.performance.thresholds.apiResponse[tier] || 2000;
    return responseTime <= threshold;
  }
};

/**
 * Test categories for organization
 */
export const TEST_CATEGORIES = {
  SMOKE: 'smoke',
  CRITICAL: 'critical',
  REGRESSION: 'regression',
  PERFORMANCE: 'performance',
  SECURITY: 'security',
  ACCESSIBILITY: 'a11y',
  MOBILE: 'mobile',
  CROSS_BROWSER: 'cross-browser',
  TIER_SPECIFIC: 'tier-specific'
};

/**
 * Export all configuration
 */
export default {
  TEST_CONFIG,
  TEST_SELECTORS,
  MOCK_RESPONSES,
  TEST_DATA_GENERATORS,
  ENVIRONMENT_CONFIG,
  VALIDATORS,
  TEST_CATEGORIES,
  getTestConfig
};