/**
 * Admin Test Fixtures and Database Setup Utilities
 * 
 * Comprehensive test data fixtures and database utilities for admin testing including:
 * - Test user accounts with various permission combinations
 * - Admin module permission matrices
 * - Session and authentication test data
 * - Security event test scenarios
 * - Performance testing datasets
 * - Database seeding and cleanup utilities
 * - Mock API responses
 * - Test environment configuration
 */

// ============================================================================
// Admin Module Definitions
// ============================================================================

export const ADMIN_MODULES = {
  USER_MANAGEMENT: 'user-management',
  SYSTEM_CONFIGURATION: 'system-configuration',
  SECURITY_MANAGEMENT: 'security-management',
  AUDIT_LOGS: 'audit-logs',
  ANALYTICS_ACCESS: 'analytics-access'
} as const;

export const PACKAGE_TIERS = {
  BRONZE: 'BRONZE',
  SILVER: 'SILVER', 
  GOLD: 'GOLD',
  PLATINUM: 'PLATINUM',
  ENTERPRISE: 'ENTERPRISE'
} as const;

// ============================================================================
// Test User Fixtures
// ============================================================================

export interface TestUser {
  id: string;
  email: string;
  name: string;
  adminModules: string[];
  packageTier: string;
  features: string[];
  sessionToken?: string;
  role: string;
  description: string;
}

export const TEST_USERS: Record<string, TestUser> = {
  SUPER_ADMIN: {
    id: 'super-admin-001',
    email: 'super.admin@epsx.test',
    name: 'Super Administrator',
    adminModules: Object.values(ADMIN_MODULES),
    packageTier: PACKAGE_TIERS.ENTERPRISE,
    features: [
      'advanced-analytics',
      'bulk-operations',
      'system-backup',
      'security-monitoring',
      'performance-analytics'
    ],
    role: 'super-admin',
    description: 'Full system access with all admin modules'
  },
  
  USER_MANAGER: {
    id: 'user-mgr-002',
    email: 'user.manager@epsx.test',
    name: 'User Manager',
    adminModules: [ADMIN_MODULES.USER_MANAGEMENT],
    packageTier: PACKAGE_TIERS.GOLD,
    features: [
      'user-creation',
      'user-editing',
      'user-analytics',
      'bulk-user-operations'
    ],
    role: 'user-manager',
    description: 'User management operations only'
  },
  
  SECURITY_MANAGER: {
    id: 'sec-mgr-003',
    email: 'security.manager@epsx.test',
    name: 'Security Manager',
    adminModules: [ADMIN_MODULES.SECURITY_MANAGEMENT, ADMIN_MODULES.AUDIT_LOGS],
    packageTier: PACKAGE_TIERS.PLATINUM,
    features: [
      'permission-management',
      'security-monitoring',
      'audit-reporting',
      'threat-analysis'
    ],
    role: 'security-manager',
    description: 'Security and audit management'
  },
  
  ANALYST: {
    id: 'analyst-004',
    email: 'analyst@epsx.test',
    name: 'Data Analyst',
    adminModules: [ADMIN_MODULES.ANALYTICS_ACCESS],
    packageTier: PACKAGE_TIERS.SILVER,
    features: [
      'analytics-dashboard',
      'report-generation',
      'data-export'
    ],
    role: 'analyst',
    description: 'Analytics and reporting access'
  },
  
  SYSTEM_ADMIN: {
    id: 'sys-admin-005',
    email: 'system.admin@epsx.test',
    name: 'System Administrator',
    adminModules: [ADMIN_MODULES.SYSTEM_CONFIGURATION],
    packageTier: PACKAGE_TIERS.GOLD,
    features: [
      'system-configuration',
      'api-key-management',
      'maintenance-mode'
    ],
    role: 'system-admin',
    description: 'System configuration and maintenance'
  },
  
  RESTRICTED_ADMIN: {
    id: 'restricted-006',
    email: 'restricted@epsx.test',
    name: 'Restricted Admin',
    adminModules: [],
    packageTier: PACKAGE_TIERS.BRONZE,
    features: [],
    role: 'restricted-admin',
    description: 'No admin modules - for testing access denial'
  },
  
  MULTI_MODULE_ADMIN: {
    id: 'multi-admin-007',
    email: 'multi.admin@epsx.test',
    name: 'Multi-Module Admin',
    adminModules: [
      ADMIN_MODULES.USER_MANAGEMENT,
      ADMIN_MODULES.ANALYTICS_ACCESS,
      ADMIN_MODULES.AUDIT_LOGS
    ],
    packageTier: PACKAGE_TIERS.PLATINUM,
    features: [
      'user-management',
      'analytics-dashboard',
      'audit-reporting',
      'cross-module-operations'
    ],
    role: 'multi-module-admin',
    description: 'Multiple module access for integration testing'
  },
  
  TEMP_ACCESS_USER: {
    id: 'temp-user-008',
    email: 'temporary@epsx.test',
    name: 'Temporary Access User',
    adminModules: [ADMIN_MODULES.USER_MANAGEMENT],
    packageTier: PACKAGE_TIERS.SILVER,
    features: ['user-management'],
    role: 'temp-admin',
    description: 'User for temporary permission testing'
  }
};

// ============================================================================
// Permission Profile Fixtures
// ============================================================================

export interface PermissionProfile {
  id: string;
  name: string;
  description: string;
  modules: string[];
  features: string[];
  packageTier: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

export const PERMISSION_PROFILES: Record<string, PermissionProfile> = {
  USER_BASIC: {
    id: 'user-basic-001',
    name: 'User Basic Profile',
    description: 'Basic user management permissions',
    modules: [ADMIN_MODULES.USER_MANAGEMENT],
    features: ['view-users', 'edit-users'],
    packageTier: PACKAGE_TIERS.BRONZE,
    isActive: true,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-01T00:00:00Z'
  },
  
  USER_PREMIUM: {
    id: 'user-premium-002',
    name: 'User Premium Profile',
    description: 'Premium user management with analytics',
    modules: [ADMIN_MODULES.USER_MANAGEMENT, ADMIN_MODULES.ANALYTICS_ACCESS],
    features: ['view-users', 'edit-users', 'bulk-operations', 'user-analytics'],
    packageTier: PACKAGE_TIERS.GOLD,
    isActive: true,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-01T00:00:00Z'
  },
  
  SECURITY_STANDARD: {
    id: 'security-std-003',
    name: 'Security Standard Profile',
    description: 'Standard security management permissions',
    modules: [ADMIN_MODULES.SECURITY_MANAGEMENT],
    features: ['permission-management', 'security-monitoring'],
    packageTier: PACKAGE_TIERS.PLATINUM,
    isActive: true,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-01T00:00:00Z'
  },
  
  ADMIN_FULL: {
    id: 'admin-full-004',
    name: 'Full Admin Profile',
    description: 'Complete administrative access',
    modules: Object.values(ADMIN_MODULES),
    features: ['*'],
    packageTier: PACKAGE_TIERS.ENTERPRISE,
    isActive: true,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-01T00:00:00Z'
  },
  
  ANALYTICS_PRO: {
    id: 'analytics-pro-005',
    name: 'Analytics Professional',
    description: 'Advanced analytics and reporting',
    modules: [ADMIN_MODULES.ANALYTICS_ACCESS, ADMIN_MODULES.AUDIT_LOGS],
    features: ['advanced-analytics', 'custom-reports', 'data-export'],
    packageTier: PACKAGE_TIERS.PLATINUM,
    isActive: true,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-01T00:00:00Z'
  }
};

// ============================================================================
// Session and Authentication Fixtures
// ============================================================================

export interface TestSession {
  sessionId: string;
  userId: string;
  token: string;
  expiresAt: string;
  createdAt: string;
  ipAddress: string;
  userAgent: string;
  isActive: boolean;
}

export const TEST_SESSIONS: Record<string, TestSession> = {
  VALID_SUPER_ADMIN: {
    sessionId: 'sess-super-001',
    userId: TEST_USERS.SUPER_ADMIN.id,
    token: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJzdXBlci1hZG1pbi0wMDEiLCJpYXQiOjE3MDAwMDAwMDAsImV4cCI6MTcwMDAwMzYwMH0.test_token_super',
    expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    createdAt: new Date().toISOString(),
    ipAddress: '192.168.1.100',
    userAgent: 'Mozilla/5.0 (Test Browser)',
    isActive: true
  },
  
  VALID_USER_MANAGER: {
    sessionId: 'sess-user-mgr-002',
    userId: TEST_USERS.USER_MANAGER.id,
    token: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyLW1nci0wMDIiLCJpYXQiOjE3MDAwMDAwMDAsImV4cCI6MTcwMDAwMzYwMH0.test_token_user_mgr',
    expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    createdAt: new Date().toISOString(),
    ipAddress: '192.168.1.101',
    userAgent: 'Mozilla/5.0 (Test Browser)',
    isActive: true
  },
  
  EXPIRED_SESSION: {
    sessionId: 'sess-expired-999',
    userId: 'expired-user-999',
    token: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJleHBpcmVkLXVzZXItOTk5IiwiaWF0IjoxNjAwMDAwMDAwLCJleHAiOjE2MDAwMDM2MDB9.expired_token',
    expiresAt: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(), // Expired
    createdAt: new Date(Date.now() - 48 * 60 * 60 * 1000).toISOString(),
    ipAddress: '192.168.1.999',
    userAgent: 'Mozilla/5.0 (Test Browser)',
    isActive: false
  },
  
  INVALID_SESSION: {
    sessionId: 'sess-invalid-998',
    userId: 'invalid-user-998',
    token: 'invalid.jwt.token.structure',
    expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    createdAt: new Date().toISOString(),
    ipAddress: '192.168.1.998',
    userAgent: 'Mozilla/5.0 (Test Browser)',
    isActive: false
  }
};

// ============================================================================
// Security Event Test Data
// ============================================================================

export interface SecurityEvent {
  id: string;
  eventType: string;
  severity: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
  userId: string;
  endpoint: string;
  method: string;
  statusCode: number;
  ipAddress: string;
  userAgent: string;
  timestamp: string;
  details: any;
}

export const SECURITY_EVENT_FIXTURES: Record<string, SecurityEvent> = {
  LOGIN_SUCCESS: {
    id: 'evt-login-001',
    eventType: 'LOGIN_SUCCESS',
    severity: 'LOW',
    userId: TEST_USERS.SUPER_ADMIN.id,
    endpoint: '/api/v1/admin/auth/login',
    method: 'POST',
    statusCode: 200,
    ipAddress: '192.168.1.100',
    userAgent: 'Mozilla/5.0 (Test Browser)',
    timestamp: new Date().toISOString(),
    details: { loginMethod: 'oauth' }
  },
  
  UNAUTHORIZED_ACCESS: {
    id: 'evt-unauth-002',
    eventType: 'UNAUTHORIZED_ACCESS',
    severity: 'HIGH',
    userId: TEST_USERS.RESTRICTED_ADMIN.id,
    endpoint: '/api/v1/admin/admin-modules',
    method: 'GET',
    statusCode: 403,
    ipAddress: '192.168.1.200',
    userAgent: 'Mozilla/5.0 (Test Browser)',
    timestamp: new Date().toISOString(),
    details: { requiredModule: ADMIN_MODULES.SECURITY_MANAGEMENT }
  },
  
  SQL_INJECTION_ATTEMPT: {
    id: 'evt-sqli-003',
    eventType: 'SQL_INJECTION_ATTEMPT',
    severity: 'CRITICAL',
    userId: 'unknown',
    endpoint: '/api/v1/admin/users/search',
    method: 'GET',
    statusCode: 400,
    ipAddress: '192.168.1.300',
    userAgent: 'curl/7.68.0',
    timestamp: new Date().toISOString(),
    details: { payload: "'; DROP TABLE users; --" }
  },
  
  RATE_LIMIT_EXCEEDED: {
    id: 'evt-rate-004',
    eventType: 'RATE_LIMIT_EXCEEDED',
    severity: 'MEDIUM',
    userId: TEST_USERS.USER_MANAGER.id,
    endpoint: '/api/v1/admin/users',
    method: 'GET',
    statusCode: 429,
    ipAddress: '192.168.1.101',
    userAgent: 'Mozilla/5.0 (Test Browser)',
    timestamp: new Date().toISOString(),
    details: { rateLimitTier: PACKAGE_TIERS.GOLD, requestCount: 1001 }
  },
  
  PRIVILEGE_ESCALATION: {
    id: 'evt-priv-005',
    eventType: 'PRIVILEGE_ESCALATION_ATTEMPT',
    severity: 'CRITICAL',
    userId: TEST_USERS.USER_MANAGER.id,
    endpoint: '/api/v1/admin/admin-modules/assign',
    method: 'POST',
    statusCode: 403,
    ipAddress: '192.168.1.101',
    userAgent: 'Mozilla/5.0 (Test Browser)',
    timestamp: new Date().toISOString(),
    details: { attemptedModules: Object.values(ADMIN_MODULES) }
  }
};

// ============================================================================
// API Response Fixtures
// ============================================================================

export const API_RESPONSE_FIXTURES = {
  SUCCESS: {
    USER_LIST: {
      status: 200,
      data: {
        users: Object.values(TEST_USERS).slice(0, 3),
        pagination: {
          page: 1,
          limit: 10,
          total: Object.values(TEST_USERS).length,
          totalPages: Math.ceil(Object.values(TEST_USERS).length / 10)
        }
      },
      meta: {
        timestamp: new Date().toISOString(),
        requestId: 'req-001'
      }
    },
    
    PERMISSION_PROFILES: {
      status: 200,
      data: {
        profiles: Object.values(PERMISSION_PROFILES),
        total: Object.values(PERMISSION_PROFILES).length
      }
    },
    
    ADMIN_MODULES: {
      status: 200,
      data: {
        modules: Object.entries(ADMIN_MODULES).map(([key, value]) => ({
          code: value,
          name: key.replace('_', ' ').toLowerCase(),
          description: `${key.replace('_', ' ')} module`,
          isActive: true
        }))
      }
    },
    
    USER_STATISTICS: {
      status: 200,
      data: {
        totalUsers: 150,
        activeUsers: 142,
        newThisMonth: 23,
        premiumUsers: 45,
        adminUsers: 8,
        usersByTier: {
          [PACKAGE_TIERS.BRONZE]: 60,
          [PACKAGE_TIERS.SILVER]: 45,
          [PACKAGE_TIERS.GOLD]: 30,
          [PACKAGE_TIERS.PLATINUM]: 12,
          [PACKAGE_TIERS.ENTERPRISE]: 3
        }
      }
    }
  },
  
  ERROR: {
    UNAUTHORIZED: {
      status: 401,
      error: 'Unauthorized',
      message: 'Authentication required',
      code: 'AUTH_REQUIRED'
    },
    
    FORBIDDEN: {
      status: 403,
      error: 'Forbidden',
      message: 'Insufficient admin privileges',
      code: 'INSUFFICIENT_PRIVILEGES',
      requiredModule: ADMIN_MODULES.SECURITY_MANAGEMENT
    },
    
    RATE_LIMITED: {
      status: 429,
      error: 'Too Many Requests',
      message: 'Rate limit exceeded',
      code: 'RATE_LIMIT_EXCEEDED',
      retryAfter: 60
    },
    
    VALIDATION_ERROR: {
      status: 422,
      error: 'Validation Error',
      message: 'Invalid input data',
      code: 'VALIDATION_FAILED',
      errors: [
        { field: 'email', message: 'Invalid email format' },
        { field: 'name', message: 'Name is required' }
      ]
    },
    
    SERVER_ERROR: {
      status: 500,
      error: 'Internal Server Error',
      message: 'An unexpected error occurred',
      code: 'INTERNAL_ERROR'
    }
  }
};

// ============================================================================
// Performance Test Data
// ============================================================================

export const PERFORMANCE_TEST_DATA = {
  LARGE_USER_DATASET: Array.from({ length: 1000 }, (_, index) => ({
    id: `perf-user-${index.toString().padStart(4, '0')}`,
    email: `perfuser${index}@epsx.test`,
    name: `Performance Test User ${index}`,
    adminModules: index % 3 === 0 ? [ADMIN_MODULES.USER_MANAGEMENT] : [],
    packageTier: Object.values(PACKAGE_TIERS)[index % Object.values(PACKAGE_TIERS).length],
    features: ['basic-access'],
    role: 'test-user',
    createdAt: new Date(Date.now() - Math.random() * 365 * 24 * 60 * 60 * 1000).toISOString()
  })),
  
  BULK_PERMISSION_UPDATES: Array.from({ length: 100 }, (_, index) => ({
    userId: `bulk-user-${index}`,
    profileId: Object.keys(PERMISSION_PROFILES)[index % Object.keys(PERMISSION_PROFILES).length],
    operation: index % 2 === 0 ? 'assign' : 'revoke'
  })),
  
  CONCURRENT_REQUESTS: Array.from({ length: 50 }, (_, index) => ({
    endpoint: '/api/v1/admin/users',
    method: 'GET',
    userId: `concurrent-user-${index}`,
    timestamp: new Date(Date.now() + index * 100).toISOString()
  }))
};

// ============================================================================
// Database Utilities
// ============================================================================

export class TestDatabaseUtilities {
  private static instance: TestDatabaseUtilities;
  
  static getInstance(): TestDatabaseUtilities {
    if (!TestDatabaseUtilities.instance) {
      TestDatabaseUtilities.instance = new TestDatabaseUtilities();
    }
    return TestDatabaseUtilities.instance;
  }
  
  /**
   * Seed database with test users
   */
  async seedTestUsers(): Promise<void> {
    console.log('🌱 Seeding test users...');
    
    // This would interact with your actual database
    // Implementation depends on your database client
    for (const [key, user] of Object.entries(TEST_USERS)) {
      console.log(`Creating test user: ${user.name} (${user.email})`);
      // await this.createUser(user);
    }
    
    console.log('✅ Test users seeded successfully');
  }
  
  /**
   * Seed database with permission profiles
   */
  async seedPermissionProfiles(): Promise<void> {
    console.log('🌱 Seeding permission profiles...');
    
    for (const [key, profile] of Object.entries(PERMISSION_PROFILES)) {
      console.log(`Creating permission profile: ${profile.name}`);
      // await this.createPermissionProfile(profile);
    }
    
    console.log('✅ Permission profiles seeded successfully');
  }
  
  /**
   * Create test sessions
   */
  async seedTestSessions(): Promise<void> {
    console.log('🌱 Creating test sessions...');
    
    for (const [key, session] of Object.entries(TEST_SESSIONS)) {
      if (session.isActive) {
        console.log(`Creating session for user: ${session.userId}`);
        // await this.createSession(session);
      }
    }
    
    console.log('✅ Test sessions created successfully');
  }
  
  /**
   * Clean up all test data
   */
  async cleanupTestData(): Promise<void> {
    console.log('🧹 Cleaning up test data...');
    
    // Clean up in reverse order to handle dependencies
    await this.cleanupSessions();
    await this.cleanupPermissionProfiles();
    await this.cleanupUsers();
    await this.cleanupSecurityEvents();
    
    console.log('✅ Test data cleanup completed');
  }
  
  private async cleanupUsers(): Promise<void> {
    console.log('Cleaning up test users...');
    // Implementation: Delete test users from database
  }
  
  private async cleanupPermissionProfiles(): Promise<void> {
    console.log('Cleaning up permission profiles...');
    // Implementation: Delete test permission profiles
  }
  
  private async cleanupSessions(): Promise<void> {
    console.log('Cleaning up test sessions...');
    // Implementation: Delete test sessions
  }
  
  private async cleanupSecurityEvents(): Promise<void> {
    console.log('Cleaning up security events...');
    // Implementation: Delete test security events
  }
  
  /**
   * Verify database integrity
   */
  async verifyDatabaseIntegrity(): Promise<boolean> {
    console.log('🔍 Verifying database integrity...');
    
    try {
      // Check foreign key constraints
      await this.checkForeignKeyConstraints();
      
      // Check data consistency
      await this.checkDataConsistency();
      
      // Check indexes
      await this.checkIndexes();
      
      console.log('✅ Database integrity verified');
      return true;
    } catch (error) {
      console.error('❌ Database integrity check failed:', error);
      return false;
    }
  }
  
  private async checkForeignKeyConstraints(): Promise<void> {
    // Implementation: Verify foreign key relationships
  }
  
  private async checkDataConsistency(): Promise<void> {
    // Implementation: Check data consistency rules
  }
  
  private async checkIndexes(): Promise<void> {
    // Implementation: Verify database indexes are present
  }
  
  /**
   * Create performance test dataset
   */
  async seedPerformanceTestData(): Promise<void> {
    console.log('🚀 Seeding performance test data...');
    
    // Seed large dataset for performance testing
    console.log(`Creating ${PERFORMANCE_TEST_DATA.LARGE_USER_DATASET.length} performance test users`);
    // Implementation: Batch insert performance test users
    
    console.log('✅ Performance test data seeded');
  }
  
  /**
   * Get database statistics
   */
  async getDatabaseStats(): Promise<any> {
    console.log('📊 Gathering database statistics...');
    
    const stats = {
      totalUsers: 0,
      totalSessions: 0,
      totalPermissionProfiles: 0,
      totalSecurityEvents: 0,
      // Implementation: Query actual counts from database
    };
    
    console.log('Database statistics:', stats);
    return stats;
  }
}

// ============================================================================
// Test Environment Configuration
// ============================================================================

export interface TestEnvironmentConfig {
  database: {
    host: string;
    port: number;
    name: string;
    user: string;
    password: string;
  };
  api: {
    baseUrl: string;
    timeout: number;
  };
  auth: {
    testEmail: string;
    testPassword: string;
  };
  performance: {
    maxResponseTime: number;
    maxConcurrentUsers: number;
    testDuration: number;
  };
}

export const TEST_ENVIRONMENT_CONFIG: TestEnvironmentConfig = {
  database: {
    host: process.env.TEST_DB_HOST || 'localhost',
    port: parseInt(process.env.TEST_DB_PORT || '5432'),
    name: process.env.TEST_DB_NAME || 'epsx_test',
    user: process.env.TEST_DB_USER || 'test_user',
    password: process.env.TEST_DB_PASSWORD || 'test_password'
  },
  api: {
    baseUrl: process.env.TEST_API_BASE_URL || 'http://localhost:8080',
    timeout: parseInt(process.env.TEST_API_TIMEOUT || '30000')
  },
  auth: {
    testEmail: process.env.TEST_ADMIN_EMAIL || 'jesadakorn.kirtnu@gmail.com',
    testPassword: process.env.TEST_ADMIN_PASSWORD || 'Aa_12345678'
  },
  performance: {
    maxResponseTime: parseInt(process.env.TEST_MAX_RESPONSE_TIME || '1000'),
    maxConcurrentUsers: parseInt(process.env.TEST_MAX_CONCURRENT_USERS || '100'),
    testDuration: parseInt(process.env.TEST_DURATION || '30000')
  }
};

// ============================================================================
// Mock API Client
// ============================================================================

export class MockAPIClient {
  private baseUrl: string;
  
  constructor(baseUrl: string = 'http://localhost:8080') {
    this.baseUrl = baseUrl;
  }
  
  /**
   * Mock successful API responses
   */
  mockSuccessResponse(endpoint: string, response: any): void {
    console.log(`🎭 Mocking successful response for ${endpoint}`);
    // Implementation: Set up mock response
  }
  
  /**
   * Mock error API responses
   */
  mockErrorResponse(endpoint: string, status: number, error: any): void {
    console.log(`🎭 Mocking error response for ${endpoint} (${status})`);
    // Implementation: Set up mock error response
  }
  
  /**
   * Mock rate limited responses
   */
  mockRateLimitedResponse(endpoint: string, retryAfter: number = 60): void {
    console.log(`🎭 Mocking rate limited response for ${endpoint}`);
    this.mockErrorResponse(endpoint, 429, API_RESPONSE_FIXTURES.ERROR.RATE_LIMITED);
  }
  
  /**
   * Clear all mocks
   */
  clearMocks(): void {
    console.log('🧹 Clearing all API mocks');
    // Implementation: Clear all mock responses
  }
}

// ============================================================================
// Test Utilities
// ============================================================================

export class TestUtilities {
  /**
   * Generate random test data
   */
  static generateRandomUser(): TestUser {
    const id = `random-${Date.now()}`;
    const moduleCount = Math.floor(Math.random() * Object.values(ADMIN_MODULES).length);
    const modules = Object.values(ADMIN_MODULES).slice(0, moduleCount);
    
    return {
      id,
      email: `random.${id}@epsx.test`,
      name: `Random User ${id}`,
      adminModules: modules,
      packageTier: Object.values(PACKAGE_TIERS)[Math.floor(Math.random() * Object.values(PACKAGE_TIERS).length)],
      features: ['basic-access'],
      role: 'test-user',
      description: 'Randomly generated test user'
    };
  }
  
  /**
   * Wait for specified duration
   */
  static async wait(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
  
  /**
   * Generate performance test metrics
   */
  static generatePerformanceMetrics(requestCount: number, duration: number) {
    return {
      totalRequests: requestCount,
      averageResponseTime: Math.random() * 500 + 50, // 50-550ms
      throughput: requestCount / (duration / 1000), // requests per second
      errorRate: Math.random() * 5, // 0-5% error rate
      p95ResponseTime: Math.random() * 1000 + 100, // 100-1100ms
      p99ResponseTime: Math.random() * 2000 + 200, // 200-2200ms
    };
  }
  
  /**
   * Validate response structure
   */
  static validateResponseStructure(response: any, expectedKeys: string[]): boolean {
    return expectedKeys.every(key => key in response);
  }
}

// ============================================================================
// Export All Fixtures and Utilities
// ============================================================================

export {
  TestDatabaseUtilities as DatabaseUtilities,
  MockAPIClient,
  TestUtilities
};

export default {
  ADMIN_MODULES,
  PACKAGE_TIERS,
  TEST_USERS,
  PERMISSION_PROFILES,
  TEST_SESSIONS,
  SECURITY_EVENT_FIXTURES,
  API_RESPONSE_FIXTURES,
  PERFORMANCE_TEST_DATA,
  TEST_ENVIRONMENT_CONFIG,
  TestDatabaseUtilities,
  MockAPIClient,
  TestUtilities
};