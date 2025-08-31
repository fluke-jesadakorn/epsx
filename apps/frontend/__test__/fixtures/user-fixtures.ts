/**
 * Comprehensive User Fixtures for Permission-Based Testing
 * Contains test users for all permission tiers with structured permissions
 * Format: "platform:resource:action" (e.g., "epsx:rankings:view:25")
 */

export interface TestUser {
  id: string;
  email: string;
  name: string;
  role: string;
  package_tier: string; // @deprecated - kept for backward compatibility, derive from permissions
  permissions: string[]; // NEW: Structured permission format
  firebase_uid?: string;
  subscription_status: 'active' | 'expired' | 'trial' | 'cancelled';
  subscription_expires_at?: string;
  features: string[]; // @deprecated - derive from permissions
  rate_limits: {
    per_minute: number;
    per_hour: number;
  };
  created_at: string;
  last_login?: string;
  jwt_token?: string; // Will be populated during test setup
}

export interface TierFeatureMap {
  [tier: string]: {
    features: string[];
    routes: string[];
    restrictions: string[];
    api_endpoints: string[];
  };
}

// Package tier feature mapping
export const TIER_FEATURES: TierFeatureMap = {
  FREE: {
    features: ['basic-trading', 'portfolio-view', 'basic-notifications'],
    routes: ['/', '/trading', '/portfolio'],
    restrictions: ['advanced-analytics', 'portfolio-tools', 'research-reports', 'api-access'],
    api_endpoints: ['/api/portfolio/basic', '/api/trading/basic']
  },
  BRONZE: {
    features: ['basic-trading', 'portfolio-view', 'enhanced-notifications', 'portfolio-history'],
    routes: ['/', '/trading', '/portfolio', '/premium', '/advanced-analytics'],
    restrictions: ['portfolio-tools', 'research-reports', 'api-access', 'institutional-features'],
    api_endpoints: ['/api/portfolio/basic', '/api/trading/basic', '/api/analytics/basic']
  },
  SILVER: {
    features: ['basic-trading', 'portfolio-view', 'enhanced-notifications', 'portfolio-history', 'advanced-trading', 'advanced-analytics'],
    routes: ['/', '/trading', '/portfolio', '/premium', '/advanced-analytics', '/professional', '/alerts'],
    restrictions: ['portfolio-tools', 'research-reports', 'api-access', 'institutional-features'],
    api_endpoints: ['/api/portfolio/basic', '/api/trading/basic', '/api/analytics/basic', '/api/trading/advanced']
  },
  GOLD: {
    features: ['basic-trading', 'portfolio-view', 'enhanced-notifications', 'portfolio-history', 'advanced-trading', 'advanced-analytics', 'portfolio-tools', 'premium-analytics', 'advanced-order-types'],
    routes: ['/', '/trading', '/portfolio', '/premium', '/advanced-analytics', '/professional', '/alerts', '/vip', '/priority-support'],
    restrictions: ['research-reports', 'api-access', 'institutional-features'],
    api_endpoints: ['/api/portfolio/basic', '/api/trading/basic', '/api/analytics/basic', '/api/trading/advanced', '/api/portfolio/tools']
  },
  PLATINUM: {
    features: ['basic-trading', 'portfolio-view', 'enhanced-notifications', 'portfolio-history', 'advanced-trading', 'advanced-analytics', 'portfolio-tools', 'premium-analytics', 'advanced-order-types', 'research-reports', 'priority-support'],
    routes: ['/', '/trading', '/portfolio', '/premium', '/advanced-analytics', '/professional', '/alerts', '/vip', '/priority-support', '/elite', '/custom-dashboards', '/reports'],
    restrictions: ['api-access', 'institutional-features'],
    api_endpoints: ['/api/portfolio/basic', '/api/trading/basic', '/api/analytics/basic', '/api/trading/advanced', '/api/portfolio/tools', '/api/research/reports']
  },
  ENTERPRISE: {
    features: ['basic-trading', 'portfolio-view', 'enhanced-notifications', 'portfolio-history', 'advanced-trading', 'advanced-analytics', 'portfolio-tools', 'premium-analytics', 'advanced-order-types', 'research-reports', 'priority-support', 'api-access', 'institutional-features', 'bulk-operations'],
    routes: ['/', '/trading', '/portfolio', '/premium', '/advanced-analytics', '/professional', '/alerts', '/vip', '/priority-support', '/elite', '/custom-dashboards', '/reports', '/enterprise', '/api-access'],
    restrictions: [],
    api_endpoints: ['/api/portfolio/basic', '/api/trading/basic', '/api/analytics/basic', '/api/trading/advanced', '/api/portfolio/tools', '/api/research/reports', '/api/enterprise/bulk', '/api/institutional/access']
  }
};

// Test users for each package tier
export const TEST_USERS: Record<string, TestUser> = {
  FREE_USER: {
    id: 'user_free_001',
    email: 'free.user@epsx.io',
    name: 'Free User',
    role: 'user',
    package_tier: 'FREE', // @deprecated - for backward compatibility
    permissions: [
      'epsx:rankings:view:3',
      'epsx:trading:basic',
      'epsx:portfolio:view',
      'epsx:notifications:basic'
    ],
    firebase_uid: 'firebase_free_001',
    subscription_status: 'active',
    features: TIER_FEATURES.FREE.features, // @deprecated
    rate_limits: { per_minute: 10, per_hour: 100 },
    created_at: '2024-01-01T00:00:00Z',
    last_login: '2024-08-22T10:00:00Z'
  },
  
  BRONZE_USER: {
    id: 'user_bronze_001',
    email: 'bronze.user@epsx.io',
    name: 'Bronze User',
    role: 'user',
    package_tier: 'BRONZE', // @deprecated
    permissions: [
      'epsx:rankings:view:5',
      'epsx:trading:basic',
      'epsx:portfolio:view',
      'epsx:portfolio:history',
      'epsx:notifications:enhanced',
      'epsx:analytics:basic'
    ],
    firebase_uid: 'firebase_bronze_001',
    subscription_status: 'active',
    subscription_expires_at: '2025-01-01T00:00:00Z',
    features: TIER_FEATURES.BRONZE.features, // @deprecated
    rate_limits: { per_minute: 30, per_hour: 500 },
    created_at: '2024-02-01T00:00:00Z',
    last_login: '2024-08-22T10:15:00Z'
  },
  
  SILVER_USER: {
    id: 'user_silver_001',
    email: 'silver.user@epsx.io',
    name: 'Silver User',
    role: 'user',
    package_tier: 'SILVER', // @deprecated
    permissions: [
      'epsx:rankings:view:25',
      'epsx:trading:basic',
      'epsx:trading:advanced',
      'epsx:portfolio:view',
      'epsx:portfolio:history',
      'epsx:notifications:enhanced',
      'epsx:analytics:basic',
      'epsx:analytics:advanced',
      'epsx:alerts:email'
    ],
    firebase_uid: 'firebase_silver_001',
    subscription_status: 'active',
    subscription_expires_at: '2025-03-01T00:00:00Z',
    features: TIER_FEATURES.SILVER.features, // @deprecated
    rate_limits: { per_minute: 60, per_hour: 1500 },
    created_at: '2024-03-01T00:00:00Z',
    last_login: '2024-08-22T10:30:00Z'
  },
  
  GOLD_USER: {
    id: 'user_gold_001',
    email: 'gold.user@epsx.io',
    name: 'Gold User',
    role: 'premium',
    package_tier: 'GOLD', // @deprecated
    permissions: [
      'epsx:rankings:view:50',
      'epsx:trading:basic',
      'epsx:trading:advanced',
      'epsx:trading:premium',
      'epsx:portfolio:view',
      'epsx:portfolio:history',
      'epsx:portfolio:tools',
      'epsx:notifications:enhanced',
      'epsx:analytics:basic',
      'epsx:analytics:advanced',
      'epsx:analytics:premium',
      'epsx:alerts:email',
      'epsx:support:priority'
    ],
    firebase_uid: 'firebase_gold_001',
    subscription_status: 'active',
    subscription_expires_at: '2025-06-01T00:00:00Z',
    features: TIER_FEATURES.GOLD.features, // @deprecated
    rate_limits: { per_minute: 120, per_hour: 5000 },
    created_at: '2024-04-01T00:00:00Z',
    last_login: '2024-08-22T10:45:00Z'
  },
  
  PLATINUM_USER: {
    id: 'user_platinum_001',
    email: 'platinum.user@epsx.io',
    name: 'Platinum User',
    role: 'premium',
    package_tier: 'PLATINUM', // @deprecated
    permissions: [
      'epsx:rankings:view:100',
      'epsx:trading:basic',
      'epsx:trading:advanced',
      'epsx:trading:premium',
      'epsx:portfolio:view',
      'epsx:portfolio:history',
      'epsx:portfolio:tools',
      'epsx:notifications:enhanced',
      'epsx:analytics:basic',
      'epsx:analytics:advanced',
      'epsx:analytics:premium',
      'epsx:alerts:email',
      'epsx:support:priority',
      'epsx:research:reports',
      'epsx:dashboards:custom'
    ],
    firebase_uid: 'firebase_platinum_001',
    subscription_status: 'active',
    subscription_expires_at: '2025-12-01T00:00:00Z',
    features: TIER_FEATURES.PLATINUM.features, // @deprecated
    rate_limits: { per_minute: 300, per_hour: 15000 },
    created_at: '2024-05-01T00:00:00Z',
    last_login: '2024-08-22T11:00:00Z'
  },
  
  ENTERPRISE_USER: {
    id: 'user_enterprise_001',
    email: 'enterprise.user@epsx.io',
    name: 'Enterprise User',
    role: 'enterprise',
    package_tier: 'ENTERPRISE', // @deprecated
    permissions: [
      'epsx:rankings:view:unlimited',
      'epsx:*:*',
      'epsx-pay:*:*',
      'epsx-token:*:*',
      'admin:*:*'
    ], // Enterprise users get unlimited access across all platforms
    firebase_uid: 'firebase_enterprise_001',
    subscription_status: 'active',
    subscription_expires_at: '2026-01-01T00:00:00Z',
    features: TIER_FEATURES.ENTERPRISE.features, // @deprecated
    rate_limits: { per_minute: 1000, per_hour: 50000 },
    created_at: '2024-06-01T00:00:00Z',
    last_login: '2024-08-22T11:15:00Z'
  },
  
  // Additional test cases
  EXPIRED_USER: {
    id: 'user_expired_001',
    email: 'expired.user@epsx.io',
    name: 'Expired Subscription User',
    role: 'user',
    package_tier: 'FREE', // @deprecated - Downgraded from GOLD
    permissions: [
      'epsx:rankings:view:3',
      'epsx:trading:basic',
      'epsx:portfolio:view',
      'epsx:notifications:basic'
    ], // Downgraded permissions after expiry
    firebase_uid: 'firebase_expired_001',
    subscription_status: 'expired',
    subscription_expires_at: '2024-06-01T00:00:00Z',
    features: TIER_FEATURES.FREE.features, // @deprecated
    rate_limits: { per_minute: 10, per_hour: 100 },
    created_at: '2024-01-01T00:00:00Z',
    last_login: '2024-08-22T09:00:00Z'
  },
  
  TRIAL_USER: {
    id: 'user_trial_001',
    email: 'trial.user@epsx.io',
    name: 'Trial User',
    role: 'user',
    package_tier: 'GOLD', // @deprecated - On trial
    permissions: [
      'epsx:rankings:view:50',
      'epsx:trading:basic',
      'epsx:trading:advanced',
      'epsx:trading:premium',
      'epsx:portfolio:view',
      'epsx:portfolio:history',
      'epsx:portfolio:tools',
      'epsx:notifications:enhanced',
      'epsx:analytics:basic',
      'epsx:analytics:advanced',
      'epsx:analytics:premium'
    ], // Full Gold permissions during trial
    firebase_uid: 'firebase_trial_001',
    subscription_status: 'trial',
    subscription_expires_at: '2024-09-22T00:00:00Z', // 30 days from now
    features: TIER_FEATURES.GOLD.features, // @deprecated
    rate_limits: { per_minute: 120, per_hour: 5000 },
    created_at: '2024-08-22T00:00:00Z',
    last_login: '2024-08-22T11:30:00Z'
  },
  
  CANCELLED_USER: {
    id: 'user_cancelled_001',
    email: 'cancelled.user@epsx.io',
    name: 'Cancelled Subscription User',
    role: 'user',
    package_tier: 'SILVER', // @deprecated - Still active until expiry
    permissions: [
      'epsx:rankings:view:25',
      'epsx:trading:basic',
      'epsx:trading:advanced',
      'epsx:portfolio:view',
      'epsx:portfolio:history',
      'epsx:notifications:enhanced',
      'epsx:analytics:basic',
      'epsx:analytics:advanced'
    ], // Silver permissions until expiry
    firebase_uid: 'firebase_cancelled_001',
    subscription_status: 'cancelled',
    subscription_expires_at: '2024-09-30T00:00:00Z',
    features: TIER_FEATURES.SILVER.features, // @deprecated
    rate_limits: { per_minute: 60, per_hour: 1500 },
    created_at: '2024-03-01T00:00:00Z',
    last_login: '2024-08-22T09:30:00Z'
  }
};

// Helper functions for test setup
export function getUserByTier(tier: string): TestUser {
  const userKey = Object.keys(TEST_USERS).find(key => 
    TEST_USERS[key].package_tier === tier && TEST_USERS[key].subscription_status === 'active'
  );
  
  if (!userKey) {
    throw new Error(`No test user found for tier: ${tier}`);
  }
  
  return TEST_USERS[userKey];
}

export function getAllActiveTierUsers(): TestUser[] {
  return Object.values(TEST_USERS).filter(user => 
    user.subscription_status === 'active' && 
    !user.email.includes('expired') && 
    !user.email.includes('cancelled')
  );
}

export function getUsersForTierTesting(): TestUser[] {
  return [
    TEST_USERS.FREE_USER,
    TEST_USERS.BRONZE_USER,
    TEST_USERS.SILVER_USER,
    TEST_USERS.GOLD_USER,
    TEST_USERS.PLATINUM_USER,
    TEST_USERS.ENTERPRISE_USER
  ];
}

export function getSpecialCaseUsers(): TestUser[] {
  return [
    TEST_USERS.EXPIRED_USER,
    TEST_USERS.TRIAL_USER,
    TEST_USERS.CANCELLED_USER
  ];
}

// JWT token generation for testing (mock implementation)
export function generateMockJWT(user: TestUser): string {
  // This would typically be generated by your auth service
  // For testing, we'll use a predictable format
  const payload = {
    sub: user.id,
    email: user.email,
    role: user.role,
    package_tier: user.package_tier,
    permissions: user.permissions,
    iat: Math.floor(Date.now() / 1000),
    exp: Math.floor(Date.now() / 1000) + 3600 // 1 hour
  };
  
  // In real implementation, this would be properly signed
  // For testing, we'll use base64 encoding
  return `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.${Buffer.from(JSON.stringify(payload)).toString('base64')}.mock_signature_${user.id}`;
}

// Populate JWT tokens for all test users
export function initializeTestUsers(): void {
  Object.keys(TEST_USERS).forEach(key => {
    TEST_USERS[key].jwt_token = generateMockJWT(TEST_USERS[key]);
  });
}

// Feature access validation helpers
export function canUserAccessRoute(user: TestUser, route: string): boolean {
  const tierFeatures = TIER_FEATURES[user.package_tier];
  if (!tierFeatures) return false;
  
  // Check if route is explicitly allowed
  if (tierFeatures.routes.includes(route)) return true;
  
  // Check if route matches any allowed patterns
  return tierFeatures.routes.some(allowedRoute => {
    if (allowedRoute.endsWith('*')) {
      return route.startsWith(allowedRoute.slice(0, -1));
    }
    return route === allowedRoute || route.startsWith(allowedRoute + '/');
  });
}

export function canUserAccessFeature(user: TestUser, feature: string): boolean {
  const tierFeatures = TIER_FEATURES[user.package_tier];
  if (!tierFeatures) return false;
  
  return tierFeatures.features.includes(feature);
}

export function isFeatureRestricted(user: TestUser, feature: string): boolean {
  const tierFeatures = TIER_FEATURES[user.package_tier];
  if (!tierFeatures) return true;
  
  return tierFeatures.restrictions.includes(feature);
}

export function canUserAccessApiEndpoint(user: TestUser, endpoint: string): boolean {
  const tierFeatures = TIER_FEATURES[user.package_tier];
  if (!tierFeatures) return false;
  
  return tierFeatures.api_endpoints.some(allowedEndpoint => {
    if (allowedEndpoint.endsWith('*')) {
      return endpoint.startsWith(allowedEndpoint.slice(0, -1));
    }
    return endpoint === allowedEndpoint || endpoint.startsWith(allowedEndpoint + '/');
  });
}

// Rate limiting helpers
export function getUserRateLimit(user: TestUser): { perMinute: number; perHour: number } {
  return user.rate_limits;
}

// Subscription status helpers
export function isUserSubscriptionActive(user: TestUser): boolean {
  if (user.subscription_status === 'active') return true;
  if (user.subscription_status === 'trial') return true;
  if (user.subscription_status === 'cancelled' && user.subscription_expires_at) {
    return new Date(user.subscription_expires_at) > new Date();
  }
  return false;
}

export function getSubscriptionDaysRemaining(user: TestUser): number {
  if (!user.subscription_expires_at) return 0;
  const expiryDate = new Date(user.subscription_expires_at);
  const now = new Date();
  const diffTime = expiryDate.getTime() - now.getTime();
  return Math.max(0, Math.ceil(diffTime / (1000 * 60 * 60 * 24)));
}

// ============================================================================
// NEW: Permission-Based Testing Helpers
// ============================================================================

/**
 * Extract ranking limit from user permissions for testing
 */
export function getUserRankingLimit(user: TestUser): number {
  for (const perm of user.permissions) {
    if (perm.startsWith('epsx:rankings:view:')) {
      const limit = perm.split(':')[3];
      if (limit === 'unlimited') return -1;
      const parsed = parseInt(limit, 10);
      if (!isNaN(parsed)) return parsed;
    }
  }
  return 5; // Default fallback
}

/**
 * Check if user has specific permission for testing
 */
export function userHasPermission(user: TestUser, permission: string): boolean {
  // Check for exact match
  if (user.permissions.includes(permission)) return true;
  
  // Check for wildcard matches
  for (const userPerm of user.permissions) {
    if (userPerm.endsWith(':*:*') || userPerm.endsWith(':*')) {
      const userParts = userPerm.split(':');
      const requiredParts = permission.split(':');
      
      // Admin wildcard
      if (userPerm === 'admin:*:*') return true;
      
      // Platform wildcard (epsx:*:*)
      if (userParts.length === 3 && userParts[1] === '*' && userParts[2] === '*') {
        if (requiredParts[0] === userParts[0]) return true;
      }
      
      // Resource wildcard (epsx:trading:*)
      if (userParts.length === 3 && userParts[2] === '*') {
        if (requiredParts[0] === userParts[0] && requiredParts[1] === userParts[1]) {
          return true;
        }
      }
    }
  }
  
  return false;
}

/**
 * Check if user can view specific ranking position for testing
 */
export function canUserViewRanking(user: TestUser, position: number): boolean {
  const limit = getUserRankingLimit(user);
  if (limit === -1) return true; // Unlimited
  return position <= limit;
}

/**
 * Derive tier from permissions for testing UI compatibility
 */
export function deriveTierFromUserPermissions(user: TestUser): string {
  const limit = getUserRankingLimit(user);
  switch (limit) {
    case 3: return 'FREE';
    case 5: return 'BRONZE';
    case 25: return 'SILVER';
    case 50: return 'GOLD';
    case 100: return 'PLATINUM';
    case -1: return 'ENTERPRISE';
    default:
      // Handle custom limits
      if (limit <= 10) return 'BRONZE';
      if (limit <= 30) return 'SILVER';
      if (limit <= 75) return 'GOLD';
      if (limit <= 150) return 'PLATINUM';
      return 'ENTERPRISE';
  }
}

/**
 * Get users by permission criteria for testing
 */
export function getUsersByPermission(permission: string): TestUser[] {
  return Object.values(TEST_USERS).filter(user => 
    userHasPermission(user, permission)
  );
}

/**
 * Get users by ranking limit for testing
 */
export function getUsersByRankingLimit(limit: number): TestUser[] {
  return Object.values(TEST_USERS).filter(user => 
    getUserRankingLimit(user) === limit
  );
}

/**
 * Get permission-based test users (recommended over tier-based)
 */
export function getPermissionTestUsers(): TestUser[] {
  return [
    TEST_USERS.FREE_USER,    // epsx:rankings:view:3
    TEST_USERS.BRONZE_USER,  // epsx:rankings:view:5
    TEST_USERS.SILVER_USER,  // epsx:rankings:view:25
    TEST_USERS.GOLD_USER,    // epsx:rankings:view:50
    TEST_USERS.PLATINUM_USER, // epsx:rankings:view:100
    TEST_USERS.ENTERPRISE_USER // epsx:rankings:view:unlimited
  ];
}

/**
 * Create custom test user with specific permissions
 */
export function createTestUserWithPermissions(
  permissions: string[], 
  overrides: Partial<TestUser> = {}
): TestUser {
  const derivedTier = deriveTierFromUserPermissions({ permissions } as TestUser);
  const rankingLimit = getUserRankingLimit({ permissions } as TestUser);
  
  return {
    id: `test_user_${Date.now()}`,
    email: `test.user.${Date.now()}@epsx.io`,
    name: 'Test User',
    role: 'user',
    package_tier: derivedTier, // @deprecated but derived for compatibility
    permissions,
    firebase_uid: `firebase_${Date.now()}`,
    subscription_status: 'active',
    features: [], // @deprecated
    rate_limits: { per_minute: 60, per_hour: 1000 },
    created_at: new Date().toISOString(),
    ...overrides
  };
}