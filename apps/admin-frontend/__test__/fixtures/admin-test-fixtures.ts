/**
 * Admin Test Fixtures - Simplified Role System
 * 
 * Test data fixtures for admin testing with unified role system:
 * - Simple role hierarchy: admin > user > guest
 * - Session and authentication test data
 * - Mock API responses
 * - Test environment configuration
 */

import { getBackendUrl } from '@/shared/utils/url-resolver';

// Define Role enum for test fixtures
export enum Role {
  Admin = 'admin',
  User = 'user', 
  Guest = 'guest'
}

// ============================================================================
// Simple Role Definitions
// ============================================================================

export const TEST_ROLES = {
  ADMIN: 'admin' as const,
  USER: 'user' as const,
  GUEST: 'guest' as const
};

// ============================================================================
// Test User Fixtures
// ============================================================================

export interface TestUser {
  id: string;
  email: string;
  name: string;
  role: Role;
  features: string[];
  sessionToken?: string;
  description: string;
}

export const TEST_USERS: Record<string, TestUser> = {
  ADMIN: {
    id: 'admin-001',
    email: 'admin@epsx.test',
    name: 'Administrator',
    role: Role.Admin,
    features: [
      'advanced-analytics',
      'bulk-operations',
      'system-backup',
      'security-monitoring',
      'performance-analytics'
    ],
    description: 'Full system access with admin privileges'
  },
  
  USER_MANAGER: {
    id: 'user-mgr-002',
    email: 'user.manager@epsx.test',
    name: 'User Manager',
    role: Role.User,
    features: [
      'user-creation',
      'user-editing',
      'user-analytics',
      'bulk-user-operations'
    ],
    description: 'User with management capabilities'
  },
  
  SECURITY_MANAGER: {
    id: 'sec-mgr-003',
    email: 'security.manager@epsx.test',
    name: 'Security Manager',
    role: Role.Admin,
    features: [
      'permission-management',
      'security-monitoring',
      'audit-reporting',
      'threat-analysis'
    ],
    description: 'Admin with security focus'
  },
  
  ANALYST: {
    id: 'analyst-004',
    email: 'analyst@epsx.test',
    name: 'Data Analyst',
    role: Role.User,
    features: [
      'analytics-dashboard',
      'report-generation',
      'data-export'
    ],
    description: 'User with analytics access'
  },
  
  SYSTEM_ADMIN: {
    id: 'sys-admin-005',
    email: 'system.admin@epsx.test',
    name: 'System Administrator',
    role: Role.Admin,
    features: [
      'system-configuration',
      'api-key-management',
      'maintenance-mode'
    ],
    description: 'Admin with system configuration access'
  },
  
  RESTRICTED_USER: {
    id: 'restricted-006',
    email: 'restricted@epsx.test',
    name: 'Restricted user',
    role: Role.Guest,
    features: [],
    description: 'Guest user for testing limited access'
  },
  
  MULTI_ACCESS_ADMIN: {
    id: 'multi-admin-007',
    email: 'multi.admin@epsx.test',
    name: 'Multi-Access Admin',
    role: Role.Admin,
    features: [
      'user-management',
      'analytics-dashboard',
      'audit-reporting',
      'cross-system-operations'
    ],
    description: 'Admin with comprehensive access for integration testing'
  },
  
  TEMP_ACCESS_USER: {
    id: 'temp-user-008',
    email: 'temporary@epsx.test',
    name: 'Temporary Access user',
    role: Role.User,
    features: ['user-management'],
    description: 'User for temporary permission testing'
  }
};

// ============================================================================
// Role Profile Fixtures - Simplified
// ============================================================================

export interface RoleProfile {
  id: string;
  name: string;
  description: string;
  role: Role;
  features: string[];
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

export const ROLE_PROFILES: Record<string, RoleProfile> = {
  GUEST_BASIC: {
    id: 'guest-basic-001',
    name: 'Guest Basic Profile',
    description: 'Basic read-only access',
    role: Role.Guest,
    features: ['view-profile', 'view-analytics'],
    isActive: true,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-01T00:00:00Z'
  },
  
  USER_STANDARD: {
    id: 'user-standard-002',
    name: 'User Standard Profile',
    description: 'Standard user with full features',
    role: Role.User,
    features: ['view-analytics', 'export-data', 'manage-profile'],
    isActive: true,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-01T00:00:00Z'
  },
  
  ADMIN_FULL: {
    id: 'admin-full-003',
    name: 'Full Admin Profile',
    description: 'Complete administrative access',
    role: Role.Admin,
    features: ['*'],
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
  VALID_ADMIN: {
    sessionId: 'sess-admin-001',
    userId: TEST_USERS['ADMIN']?.id || 'admin-001',
    token: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhZG1pbi0wMDEiLCJpYXQiOjE3MDAwMDAwMDAsImV4cCI6MTcwMDAwMzYwMH0.test_token_admin',
    expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    createdAt: new Date().toISOString(),
    ipAddress: '192.168.1.100',
    userAgent: 'Mozilla/5.0 (Test Browser)',
    isActive: true
  },
  
  VALID_USER_MANAGER: {
    sessionId: 'sess-user-mgr-002',
    userId: TEST_USERS['USER_MANAGER']?.id || 'user-mgr-002',
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
    userId: TEST_USERS['ADMIN']?.id || 'admin-001',
    endpoint: '/api/admin/auth/login',
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
    userId: TEST_USERS['RESTRICTED_USER']?.id || 'restricted-003',
    endpoint: '/api/admin/admin-modules',
    method: 'GET',
    statusCode: 403,
    ipAddress: '192.168.1.200',
    userAgent: 'Mozilla/5.0 (Test Browser)',
    timestamp: new Date().toISOString(),
    details: { requiredRole: Role.Admin }
  },
  
  SQL_INJECTION_ATTEMPT: {
    id: 'evt-sqli-003',
    eventType: 'SQL_INJECTION_ATTEMPT',
    severity: 'CRITICAL',
    userId: 'unknown',
    endpoint: '/api/admin/users/search',
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
    userId: TEST_USERS['USER_MANAGER']?.id || 'user-mgr-002',
    endpoint: '/api/admin/users',
    method: 'GET',
    statusCode: 429,
    ipAddress: '192.168.1.101',
    userAgent: 'Mozilla/5.0 (Test Browser)',
    timestamp: new Date().toISOString(),
    details: { userRole: Role.User, requestCount: 1001 }
  },
  
  PRIVILEGE_ESCALATION: {
    id: 'evt-priv-005',
    eventType: 'PRIVILEGE_ESCALATION_ATTEMPT',
    severity: 'CRITICAL',
    userId: TEST_USERS['USER_MANAGER']?.id || 'user-mgr-002',
    endpoint: '/api/admin/admin-modules/assign',
    method: 'POST',
    statusCode: 403,
    ipAddress: '192.168.1.101',
    userAgent: 'Mozilla/5.0 (Test Browser)',
    timestamp: new Date().toISOString(),
    details: { attemptedRole: Role.Admin }
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
    
    ROLE_PROFILES: {
      status: 200,
      data: {
        profiles: Object.values(ROLE_PROFILES),
        total: Object.values(ROLE_PROFILES).length
      }
    },
    
    ROLES: {
      status: 200,
      data: {
        roles: Object.values(Role).map(role => ({
          value: role,
          label: role.charAt(0).toUpperCase() + role.slice(1),
          description: `${role.charAt(0).toUpperCase() + role.slice(1)} role`,
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
        usersByRole: {
          [Role.Guest]: 60,
          [Role.User]: 75,
          [Role.Admin]: 15
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
      requiredRole: Role.Admin
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
    role: index % 10 === 0 ? Role.Admin : index % 3 === 0 ? Role.User : Role.Guest,
    features: ['basic-access'],
    createdAt: new Date(Date.now() - Math.random() * 365 * 24 * 60 * 60 * 1000).toISOString()
  })),
  
  BULK_ROLE_UPDATES: Array.from({ length: 100 }, (_, index) => ({
    userId: `bulk-user-${index}`,
    role: Object.values(Role)[index % Object.values(Role).length],
    operation: index % 2 === 0 ? 'assign' : 'revoke'
  })),
  
  CONCURRENT_REQUESTS: Array.from({ length: 50 }, (_, index) => ({
    endpoint: '/api/admin/users',
    method: 'GET',
    userId: `concurrent-user-${index}`,
    timestamp: new Date(Date.now() + index * 100).toISOString()
  }))
};

// ============================================================================
// Database Utilities
// ============================================================================

/**
 *
 */
export class TestDatabaseUtilities {
  private static instance: TestDatabaseUtilities;
  
  /**
   *
   */
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
    
    // This would interact with your actual database
    // Implementation depends on your database client
    for (const [_key, _user] of Object.entries(TEST_USERS)) {
      // await this.createUser(_user);
    }
    
  }
  
  /**
   * Seed database with role profiles
   */
  async seedRoleProfiles(): Promise<void> {
    
    for (const [_key, _profile] of Object.entries(ROLE_PROFILES)) {
      // await this.createRoleProfile(profile);
    }
    
  }
  
  /**
   * Create test sessions
   */
  async seedTestSessions(): Promise<void> {
    
    for (const [_key, session] of Object.entries(TEST_SESSIONS)) {
      if (session.isActive) {
        // await this.createSession(session);
      }
    }
    
  }
  
  /**
   * Clean up all test data
   */
  async cleanupTestData(): Promise<void> {
    
    // Clean up in reverse order to handle dependencies
    await this.cleanupSessions();
    await this.cleanupRoleProfiles();
    await this.cleanupUsers();
    await this.cleanupSecurityEvents();
    
  }
  
  private async cleanupUsers(): Promise<void> {
    // Implementation: Delete test users from database
  }
  
  private async cleanupRoleProfiles(): Promise<void> {
    // Implementation: Delete test role profiles
  }
  
  private async cleanupSessions(): Promise<void> {
    // Implementation: Delete test sessions
  }
  
  private async cleanupSecurityEvents(): Promise<void> {
    // Implementation: Delete test security events
  }
  
  /**
   * Verify database integrity
   */
  async verifyDatabaseIntegrity(): Promise<boolean> {
    
    try {
      // Check foreign key constraints
      await this.checkForeignKeyConstraints();
      
      // Check data consistency
      await this.checkDataConsistency();
      
      // Check indexes
      await this.checkIndexes();
      
      return true;
    } catch (_error) {
       
      console.error('❌ Database integrity check failed:', _error);
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
    
    // Seed large dataset for performance testing
    // Implementation: Batch insert performance test users
    
  }
  
  /**
   * Get database statistics
   */
  async getDatabaseStats(): Promise<any> {
    
    return {
      totalUsers: 0,
      totalSessions: 0,
      totalRoleProfiles: 0,
      totalSecurityEvents: 0,
      // Implementation: Query actual counts from database
    };
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
    host: (process.env['TEST_DB_HOST']) || 'localhost',
    port: parseInt((process.env['TEST_DB_PORT']) || '5432'),
    name: (process.env['TEST_DB_NAME']) || 'epsx_test',
    user: (process.env['TEST_DB_USER']) || 'test_user',
    password: process.env['TEST_DB_PASSWORD'] || 'test_password'
  },
  api: {
    baseUrl: process.env['TEST_API_BASE_URL'] || getBackendUrl('server'),
    timeout: parseInt(process.env['TEST_API_TIMEOUT'] || '30000')
  },
  auth: {
    // Admin user must be promoted via database script: ./scripts/promote-admin.sh jesadakorn.kirtnu@gmail.com
    testEmail: 'jesadakorn.kirtnu@gmail.com',
    testPassword: 'Aa_12345678'
  },
  performance: {
    maxResponseTime: parseInt(process.env['TEST_MAX_RESPONSE_TIME'] || '1000'),
    maxConcurrentUsers: parseInt(process.env['TEST_MAX_CONCURRENT_USERS'] || '100'),
    testDuration: parseInt(process.env['TEST_DURATION'] || '30000')
  }
};

// ============================================================================
// Mock API Client
// ============================================================================

/**
 *
 */
export class MockAPIClient {
  private _baseUrl: string;

  /**
   *
   */
  get baseUrl(): string {
    return this._baseUrl;
  }
  
  /**
   *
   * @param baseUrl
   */
  constructor(baseUrl: string = getBackendUrl('server')) {
    this._baseUrl = baseUrl;
  }
  
  /**
   * Mock successful API responses
   * @param _endpoint
   * @param _response
   */
  mockSuccessResponse(_endpoint: string, _response: any): void {
    // Implementation: Set up mock response
  }
  
  /**
   * Mock error API responses
   * @param _endpoint
   * @param _status
   * @param _error
   */
  mockErrorResponse(_endpoint: string, _status: number, _error: any): void {
    // Implementation: Set up mock error response
  }
  
  /**
   * Mock rate limited responses
   * @param _endpoint
   * @param _retryAfter
   */
  mockRateLimitedResponse(_endpoint: string, _retryAfter = 60): void {
    this.mockErrorResponse(_endpoint, 429, API_RESPONSE_FIXTURES.ERROR.RATE_LIMITED);
  }
  
  /**
   * Clear all mocks
   */
  clearMocks(): void {
    // Implementation: Clear all mock responses
  }
}

// ============================================================================
// Test Utilities
// ============================================================================

/**
 *
 */
export class TestUtilities {
  /**
   * Generate random test data
   */
  static generateRandomUser(): TestUser {
    const id = `random-${Date.now()}`;
    const roles = Object.values(Role).filter((role): role is Role => role !== undefined);
    const randomRole = roles[Math.floor(Math.random() * roles.length)] || Role.User;

    return {
      id,
      email: `random.${id}@epsx.test`,
      name: `Random User ${id}`,
      role: randomRole,
      features: ['basic-access'],
      description: 'Randomly generated test user'
    };
  }
  
  /**
   * Wait for specified duration
   * @param ms
   */
  static async wait(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
  
  /**
   * Generate performance test metrics
   * @param requestCount
   * @param duration
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
   * @param response
   * @param expectedKeys
   */
  static validateResponseStructure(response: any, expectedKeys: string[]): boolean {
    return expectedKeys.every(key => key in response);
  }
}

// ============================================================================
// Export All Fixtures and Utilities
// ============================================================================

export {
  TestDatabaseUtilities as DatabaseUtilities
};

export default {
  TEST_ROLES,
  TEST_USERS,
  ROLE_PROFILES,
  TEST_SESSIONS,
  SECURITY_EVENT_FIXTURES,
  API_RESPONSE_FIXTURES,
  PERFORMANCE_TEST_DATA,
  TEST_ENVIRONMENT_CONFIG,
  TestDatabaseUtilities
};