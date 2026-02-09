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
  permissions: string[]; // Structured permission format
  wallet_address: string;
  subscription_status: 'active' | 'expired' | 'trial' | 'cancelled';
  subscription_expires_at?: string;
  rate_limits: {
    per_minute: number;
    per_hour: number;
  };
  created_at: string;
  last_login?: string;
  auth_token?: string; // Web3 authentication token
}

// Permission-based route access mapping (for test validation)
export const PERMISSION_ROUTE_MAP: Record<string, string[]> = {
  'epsx:rankings:view': ['/'],
  'epsx:analytics:basic': ['/analytics', '/premium', '/advanced-analytics'],
  'epsx:portfolio:view': ['/portfolio'],
  'epsx:analytics:advanced': ['/professional'],
  'epsx:alerts:email': ['/alerts'],
  'epsx:portfolio:tools': ['/vip'],
  'epsx:support:priority': ['/priority-support'],
  'epsx:research:reports': ['/elite', '/reports'],
  'epsx:dashboards:custom': ['/custom-dashboards'],
  'epsx:*:*': ['/enterprise'],
  'admin:*:*': ['/api-access']
};

// Permission-based API endpoint mapping (for test validation)
export const PERMISSION_API_MAP: Record<string, string[]> = {
  'epsx:portfolio:view': ['/api/portfolio/basic'],
  'epsx:analytics:basic': ['/api/analytics/basic'],
  'epsx:analytics:advanced': ['/api/analytics/advanced'],
  'epsx:portfolio:tools': ['/api/portfolio/tools'],
  'epsx:research:reports': ['/api/research/reports'],
  'epsx:*:*': ['/api/enterprise/bulk'],
  'admin:*:*': ['/api/institutional/access']
};

// Test users for each package tier
export const TEST_USERS: Record<string, TestUser> = {
  FREE_USER: {
    id: 'user_free_001',
    email: 'free.user@epsx.io',
    name: 'Free user',
    role: 'user',
    permissions: [
      'epsx:rankings:view:3',
      'epsx:analytics:basic',
      'epsx:portfolio:view',
      'epsx:notifications:basic'
    ],
    wallet_address: '0x1234567890123456789012345678901234567890',
    subscription_status: 'active',
    rate_limits: { per_minute: 10, per_hour: 100 },
    created_at: '2024-01-01T00:00:00Z',
    last_login: '2024-08-22T10:00:00Z'
  },

  BRONZE_USER: {
    id: 'user_bronze_001',
    email: 'bronze.user@epsx.io',
    name: 'Bronze user',
    role: 'user',
    permissions: [
      'epsx:rankings:view:5',
      'epsx:analytics:basic',
      'epsx:portfolio:view',
      'epsx:portfolio:history',
      'epsx:notifications:enhanced',
      'epsx:analytics:basic'
    ],
    wallet_address: '0x2345678901234567890123456789012345678901',
    subscription_status: 'active',
    subscription_expires_at: '2025-01-01T00:00:00Z',
    rate_limits: { per_minute: 30, per_hour: 500 },
    created_at: '2024-02-01T00:00:00Z',
    last_login: '2024-08-22T10:15:00Z'
  },

  SILVER_USER: {
    id: 'user_silver_001',
    email: 'silver.user@epsx.io',
    name: 'Silver user',
    role: 'user',
    permissions: [
      'epsx:rankings:view:25',
      'epsx:analytics:basic',
      'epsx:analytics:advanced',
      'epsx:portfolio:view',
      'epsx:portfolio:history',
      'epsx:notifications:enhanced',
      'epsx:analytics:basic',
      'epsx:analytics:advanced',
      'epsx:alerts:email'
    ],
    wallet_address: '0x3456789012345678901234567890123456789012',
    subscription_status: 'active',
    subscription_expires_at: '2025-03-01T00:00:00Z',
    rate_limits: { per_minute: 60, per_hour: 1500 },
    created_at: '2024-03-01T00:00:00Z',
    last_login: '2024-08-22T10:30:00Z'
  },

  GOLD_USER: {
    id: 'user_gold_001',
    email: 'gold.user@epsx.io',
    name: 'Gold user',
    role: 'premium',
    permissions: [
      'epsx:rankings:view:50',
      'epsx:analytics:basic',
      'epsx:analytics:advanced',
      'epsx:analytics:premium',
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
    wallet_address: '0x4567890123456789012345678901234567890123',
    subscription_status: 'active',
    subscription_expires_at: '2025-06-01T00:00:00Z',
    rate_limits: { per_minute: 120, per_hour: 5000 },
    created_at: '2024-04-01T00:00:00Z',
    last_login: '2024-08-22T10:45:00Z'
  },

  PLATINUM_USER: {
    id: 'user_platinum_001',
    email: 'platinum.user@epsx.io',
    name: 'Platinum user',
    role: 'premium',
    permissions: [
      'epsx:rankings:view:100',
      'epsx:analytics:basic',
      'epsx:analytics:advanced',
      'epsx:analytics:premium',
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
    wallet_address: '0x5678901234567890123456789012345678901234',
    subscription_status: 'active',
    subscription_expires_at: '2025-12-01T00:00:00Z',
    rate_limits: { per_minute: 300, per_hour: 15000 },
    created_at: '2024-05-01T00:00:00Z',
    last_login: '2024-08-22T11:00:00Z'
  },

  ENTERPRISE_USER: {
    id: 'user_enterprise_001',
    email: 'enterprise.user@epsx.io',
    name: 'Enterprise user',
    role: 'enterprise',
    permissions: [
      'epsx:rankings:view:unlimited',
      'epsx:*:*',
      'epsx-pay:*:*',
      'epsx-token:*:*',
      'admin:*:*'
    ], // Enterprise users get unlimited access across all platforms
    wallet_address: '0x6789012345678901234567890123456789012345',
    subscription_status: 'active',
    subscription_expires_at: '2026-01-01T00:00:00Z',
    rate_limits: { per_minute: 1000, per_hour: 50000 },
    created_at: '2024-06-01T00:00:00Z',
    last_login: '2024-08-22T11:15:00Z'
  },

  // Additional test cases
  EXPIRED_USER: {
    id: 'user_expired_001',
    email: 'expired.user@epsx.io',
    name: 'Expired Subscription user',
    role: 'user',
    permissions: [
      'epsx:rankings:view:3',
      'epsx:analytics:basic',
      'epsx:portfolio:view',
      'epsx:notifications:basic'
    ], // Downgraded permissions after expiry
    wallet_address: '0x7890123456789012345678901234567890123456',
    subscription_status: 'expired',
    subscription_expires_at: '2024-06-01T00:00:00Z',
    rate_limits: { per_minute: 10, per_hour: 100 },
    created_at: '2024-01-01T00:00:00Z',
    last_login: '2024-08-22T09:00:00Z'
  },

  TRIAL_USER: {
    id: 'user_trial_001',
    email: 'trial.user@epsx.io',
    name: 'Trial user',
    role: 'user',
    permissions: [
      'epsx:rankings:view:50',
      'epsx:analytics:basic',
      'epsx:analytics:advanced',
      'epsx:analytics:premium',
      'epsx:portfolio:view',
      'epsx:portfolio:history',
      'epsx:portfolio:tools',
      'epsx:notifications:enhanced',
      'epsx:analytics:basic',
      'epsx:analytics:advanced',
      'epsx:analytics:premium'
    ], // Full Gold permissions during trial
    wallet_address: '0x8901234567890123456789012345678901234567',
    subscription_status: 'trial',
    subscription_expires_at: '2024-09-22T00:00:00Z', // 30 days from now
    rate_limits: { per_minute: 120, per_hour: 5000 },
    created_at: '2024-08-22T00:00:00Z',
    last_login: '2024-08-22T11:30:00Z'
  },

  CANCELLED_USER: {
    id: 'user_cancelled_001',
    email: 'cancelled.user@epsx.io',
    name: 'Cancelled Subscription user',
    role: 'user',
    permissions: [
      'epsx:rankings:view:25',
      'epsx:analytics:basic',
      'epsx:analytics:advanced',
      'epsx:portfolio:view',
      'epsx:portfolio:history',
      'epsx:notifications:enhanced',
      'epsx:analytics:basic',
      'epsx:analytics:advanced'
    ], // Silver permissions until expiry
    wallet_address: '0x9012345678901234567890123456789012345678',
    subscription_status: 'cancelled',
    subscription_expires_at: '2024-09-30T00:00:00Z',
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

// Permission-based access validation helpers
export function canUserAccessRoute(user: TestUser, route: string): boolean {
  // Check if any user permission grants access to this route
  for (const permission of user.permissions) {
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    const allowedRoutes = PERMISSION_ROUTE_MAP[permission] || [];
    if (allowedRoutes.includes(route)) {return true;}

    // Check for wildcard permission matches
    if (permission.endsWith(':*:*') || permission.endsWith(':*')) {
      const basePermission = permission.replace(/:.*$/, '');
      for (const [perm, routes] of Object.entries(PERMISSION_ROUTE_MAP)) {
        if (perm.startsWith(basePermission) && routes.includes(route)) {
          return true;
        }
      }
    }
  }
  return false;
}

export function canUserAccessFeature(user: TestUser, feature: string): boolean {
  // Check if user has permission that grants this feature
  return userHasPermission(user, feature);
}

export function isFeatureRestricted(user: TestUser, feature: string): boolean {
  return !canUserAccessFeature(user, feature);
}

export function canUserAccessApiEndpoint(user: TestUser, endpoint: string): boolean {
  // Check if any user permission grants access to this API endpoint
  for (const permission of user.permissions) {
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    const allowedEndpoints = PERMISSION_API_MAP[permission] || [];
    if (allowedEndpoints.some(allowed => endpoint.startsWith(allowed))) {
      return true;
    }

    // Check for wildcard permission matches
    if (permission.endsWith(':*:*') || permission.endsWith(':*')) {
      const basePermission = permission.replace(/:.*$/, '');
      for (const [perm, endpoints] of Object.entries(PERMISSION_API_MAP)) {
        if (perm.startsWith(basePermission)) {
          if (endpoints.some(allowed => endpoint.startsWith(allowed))) {
            return true;
          }
        }
      }
    }
  }
  return false;
}

// Rate limiting helpers
export function getUserRateLimit(user: TestUser): { perMinute: number; perHour: number } {
  return user.rate_limits;
}

// Subscription status helpers
export function isUserSubscriptionActive(user: TestUser): boolean {
  if (user.subscription_status === 'active') {return true;}
  if (user.subscription_status === 'trial') {return true;}
  if (user.subscription_status === 'cancelled' && user.subscription_expires_at) {
    return new Date(user.subscription_expires_at) > new Date();
  }
  return false;
}

export function getSubscriptionDaysRemaining(user: TestUser): number {
  if (!user.subscription_expires_at) {return 0;}
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
      if (limit === 'unlimited') {return -1;}
      const parsed = parseInt(limit, 10);
      if (!isNaN(parsed)) {return parsed;}
    }
  }
  return 5; // Default fallback
}

/**
 * Check if user has specific permission for testing
 */
export function userHasPermission(user: TestUser, permission: string): boolean {
  // Check for exact match
  if (user.permissions.includes(permission)) {return true;}

  // Check for wildcard matches
  for (const userPerm of user.permissions) {
    if (userPerm.endsWith(':*:*') || userPerm.endsWith(':*')) {
      const userParts = userPerm.split(':');
      const requiredParts = permission.split(':');

      // Admin wildcard
      if (userPerm === 'admin:*:*') {return true;}

      // Platform wildcard (epsx:*:*)
      if (userParts.length === 3 && userParts[1] === '*' && userParts[2] === '*') {
        if (requiredParts[0] === userParts[0]) {return true;}
      }

      // Resource wildcard (epsx:analytics:*)
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
  if (limit === -1) {return true;} // Unlimited
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
      if (limit <= 10) {return 'BRONZE';}
      if (limit <= 30) {return 'SILVER';}
      if (limit <= 75) {return 'GOLD';}
      if (limit <= 150) {return 'PLATINUM';}
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
    name: 'Test user',
    role: 'user',
    permissions,
    wallet_address: `0xa${Date.now()}234567890123456789012345678901234567890`,
    subscription_status: 'active',
    rate_limits: { per_minute: 60, per_hour: 1000 },
    created_at: new Date().toISOString(),
    ...overrides
  };
}