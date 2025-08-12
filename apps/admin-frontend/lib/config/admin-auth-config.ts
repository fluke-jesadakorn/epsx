// Admin Authentication Configuration
// OIDC-only admin authentication configuration

/**
 * Admin-specific authentication configuration for OIDC-only
 */
export const ADMIN_AUTH_CONFIG = {
  // Enhanced security settings for admin
  autoRefresh: true,
  refreshThresholdMinutes: 2,      // Shorter refresh for admin security
  maxRetryAttempts: 2,             // Fewer retries for admin
  useSessionStorage: false,        // Admin persists sessions in localStorage
  storagePrefix: 'admin_auth_',    // Admin-specific storage prefix
  clientId: 'admin-client',        // Admin-specific client ID
  
  // Admin-specific security settings
  adminConfig: {
    enableAuditLogging: true,      // Enable comprehensive admin action logging
    sessionTimeoutMinutes: 60,     // 1 hour max session
    requireAdminRole: true,        // Strictly validate admin IAM profiles
    maxIdleMinutes: 30,            // 30 minutes idle timeout
  },
};


/**
 * Admin-specific OIDC configuration
 */
export const ADMIN_OIDC_CONFIG = {
  // Enable OIDC for admin authentication
  enableOIDC: true,
  
  // Admin-specific OIDC settings
  backendUrl: process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080',
  
  // Admin OIDC flow settings
  redirectUri: `${process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3001'}/api/auth/oidc/callback`,
  
  // Enhanced security for admin OIDC
  requireAdminScope: true,
};

/**
 * Admin security policies
 */
export const ADMIN_SECURITY_POLICIES = {
  // Session management
  maxSessionDuration: 60 * 60 * 1000,    // 1 hour in milliseconds
  maxIdleTime: 30 * 60 * 1000,           // 30 minutes in milliseconds
  
  // Token management
  tokenRefreshThreshold: 2 * 60 * 1000,   // 2 minutes in milliseconds
  maxTokenRefreshAttempts: 2,
  
  // Security features
  requireMFA: false,                       // TODO: Implement 2FA
  requireIPValidation: false,              // TODO: Implement IP restrictions
  enableSessionFingerprinting: false,     // TODO: Implement device fingerprinting
  
  // Audit logging
  auditAllActions: true,
  auditLoginAttempts: true,
  auditPermissionChanges: true,
  auditTokenOperations: true,
};

/**
 * Admin environment-specific configuration
 */
export const ADMIN_ENVIRONMENT_CONFIG = {
  development: {
    ...ADMIN_AUTH_CONFIG,
    adminConfig: {
      ...ADMIN_AUTH_CONFIG.adminConfig!,
      sessionTimeoutMinutes: 120,   // Longer sessions for development
      maxIdleMinutes: 60,           // Longer idle time for development
    },
  },
  
  production: {
    ...ADMIN_AUTH_CONFIG,
    adminConfig: {
      ...ADMIN_AUTH_CONFIG.adminConfig!,
      sessionTimeoutMinutes: 60,    // Strict 1 hour sessions in production
      maxIdleMinutes: 15,           // Shorter idle time in production
      requireAdminRole: true,       // Always require admin role in production
    },
  },
  
  test: {
    ...ADMIN_AUTH_CONFIG,
    adminConfig: {
      ...ADMIN_AUTH_CONFIG.adminConfig!,
      enableAuditLogging: false,    // Disable logging in tests
      sessionTimeoutMinutes: 5,     // Very short sessions for testing
      maxIdleMinutes: 2,            // Very short idle time for testing
    },
  },
};

/**
 * Get environment-specific admin configuration
 */
export function getAdminAuthConfig() {
  const env = process.env.NODE_ENV || 'development';
  
  switch (env) {
    case 'production':
      return ADMIN_ENVIRONMENT_CONFIG.production;
    case 'test':
      return ADMIN_ENVIRONMENT_CONFIG.test;
    default:
      return ADMIN_ENVIRONMENT_CONFIG.development;
  }
}

/**
 * Admin IAM profiles configuration
 */
export const ADMIN_IAM_PROFILES = {
  'admin-full-004': {
    name: 'Admin Full Access',
    description: 'Complete administrative access to all systems',
    permissions: ['admin:*'],
    accessLevel: 'full',
    sessionTimeout: 60,    // 1 hour
    maxIdleTime: 30,       // 30 minutes
  },
  
  'moderator-standard-003': {
    name: 'Moderator Standard',
    description: 'Standard moderation capabilities',
    permissions: [
      'users:read', 'users:write',
      'analytics:read',
      'settings:read',
      'moderate:content',
    ],
    accessLevel: 'standard',
    sessionTimeout: 60,    // 1 hour  
    maxIdleTime: 30,       // 30 minutes
  },
} as const;

/**
 * Get IAM profile configuration
 */
export function getIAMProfileConfig(profileId: string) {
  return ADMIN_IAM_PROFILES[profileId as keyof typeof ADMIN_IAM_PROFILES] || null;
}

/**
 * Check if a role is an admin role
 */
export function isAdminRole(role: string): boolean {
  return Object.keys(ADMIN_IAM_PROFILES).includes(role);
}

/**
 * Default admin configuration export
 */
export default {
  auth: getAdminAuthConfig(),
  oidc: ADMIN_OIDC_CONFIG,
  security: ADMIN_SECURITY_POLICIES,
  iam: ADMIN_IAM_PROFILES,
};