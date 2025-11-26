/**
 * UNIFIED AUTH ADAPTER
 * 
 * Provides a bridge between platform-specific auth hooks and the unified permission guard.
 * This adapter allows the UnifiedPermissionGuard to work with both admin-frontend and 
 * frontend auth systems without directly importing platform-specific code.
 * 
 * Usage:
 * 1. Each platform calls registerAuthHook() on app initialization
 * 2. UnifiedPermissionGuard uses getAuthHook() to access platform-specific auth
 */

export type AuthLevel = 'ANONYMOUS' | 'AUTHENTICATED' | 'PROGRESSIVE' | 'FULL';

export interface UnifiedAuthInterface {
  user: any;
  level?: AuthLevel;
  permissions: string[];
  hasPermission: (permission: string) => boolean;
  hasAnyPermission: (permissions: string[]) => boolean;
  hasAllPermissions: (permissions: string[]) => boolean;
  canAccess?: (level: AuthLevel) => boolean;
}

// Platform-specific auth hook registry
const authHookRegistry = new Map<string, () => UnifiedAuthInterface>();

/**
 * Register a platform-specific auth hook
 * This should be called during app initialization
 */
export function registerAuthHook(platform: 'admin' | 'frontend', authHook: () => UnifiedAuthInterface): void {
  authHookRegistry.set(platform, authHook);
}

/**
 * Get the auth hook for a specific platform
 */
export function getAuthHook(platform: 'admin' | 'frontend'): UnifiedAuthInterface {
  const authHook = authHookRegistry.get(platform);
  
  if (authHook) {
    return authHook();
  }
  
  // Fallback for server-side rendering or missing hooks
  return {
    user: null,
    permissions: [],
    hasPermission: () => false,
    hasAnyPermission: () => false,
    hasAllPermissions: () => false
  };
}

/**
 * Check if an auth hook is registered for a platform
 */
export function hasAuthHook(platform: 'admin' | 'frontend'): boolean {
  return authHookRegistry.has(platform);
}

/**
 * Clear all registered auth hooks (useful for testing)
 */
export function clearAuthHooks(): void {
  authHookRegistry.clear();
}

// ============================================================================
// ADAPTER FACTORIES FOR COMMON AUTH HOOK PATTERNS
// ============================================================================

/**
 * Create an adapter for admin-frontend auth hooks
 * Converts admin-specific auth hook interface to unified interface
 */
export function createAdminAuthAdapter(adminAuthHook: () => {
  user: any;
  level?: AuthLevel;
  adminPermissions?: string[];
  hasPermission: (permission: string) => boolean;
  hasAnyPermission: (permissions: string[]) => boolean;
  hasAllPermissions?: (permissions: string[]) => boolean;
  canAccess?: (level: AuthLevel) => boolean;
}): () => UnifiedAuthInterface {
  return () => {
    const auth = adminAuthHook();
    return {
      user: auth.user,
      level: auth.level,
      permissions: auth.adminPermissions || [],
      hasPermission: auth.hasPermission,
      hasAnyPermission: auth.hasAnyPermission,
      hasAllPermissions: auth.hasAllPermissions || ((perms) => perms.every(p => auth.hasPermission(p))),
      canAccess: auth.canAccess
    };
  };
}

/**
 * Create an adapter for frontend auth hooks
 * Converts frontend-specific auth hook interface to unified interface
 */
export function createFrontendAuthAdapter(frontendAuthHook: () => {
  user: any;
  permissions?: string[];
  hasPermission: (permission: string) => boolean;
  hasAnyPermission: (permissions: string[]) => boolean;
  hasAllPermissions?: (permissions: string[]) => boolean;
}): () => UnifiedAuthInterface {
  return () => {
    const auth = frontendAuthHook();
    return {
      user: auth.user,
      permissions: auth.permissions || [],
      hasPermission: auth.hasPermission,
      hasAnyPermission: auth.hasAnyPermission,
      hasAllPermissions: auth.hasAllPermissions || ((perms) => perms.every(p => auth.hasPermission(p)))
    };
  };
}

// ============================================================================
// CONTEXT-BASED AUTH HOOK PROVIDERS
// ============================================================================

/**
 * Create a context-based auth provider that can be used across platforms
 * This provides a consistent interface for auth state management
 */
export interface AuthContextValue extends UnifiedAuthInterface {
  platform: 'admin' | 'frontend';
  isLoading: boolean;
  error?: string;
}

// Simple context-based auth state (can be extended with React Context if needed)
let contextualAuth: AuthContextValue | null = null;

export function setContextualAuth(auth: AuthContextValue): void {
  contextualAuth = auth;
}

export function getContextualAuth(): AuthContextValue | null {
  return contextualAuth;
}

export function clearContextualAuth(): void {
  contextualAuth = null;
}

// ============================================================================
// DEVELOPMENT AND DEBUGGING UTILITIES
// ============================================================================

/**
 * Debug utility to inspect registered auth hooks
 */
export function debugAuthHooks(): void {
  console.log('Registered auth hooks:', Array.from(authHookRegistry.keys()));
  
  for (const [platform, hook] of authHookRegistry) {
    try {
      const auth = hook();
      console.log(`${platform} auth:`, {
        hasUser: !!auth.user,
        permissionCount: auth.permissions.length,
        level: auth.level,
        canAccess: !!auth.canAccess
      });
    } catch (error) {
      console.error(`Error accessing ${platform} auth:`, error);
    }
  }
}

/**
 * Mock auth hook for testing
 */
export function createMockAuthHook(options: {
  user?: any;
  permissions?: string[];
  level?: AuthLevel;
}): () => UnifiedAuthInterface {
  return () => ({
    user: options.user || { id: 'test-user' },
    permissions: options.permissions || [],
    level: options.level || 'AUTHENTICATED',
    hasPermission: (permission: string) => options.permissions?.includes(permission) || false,
    hasAnyPermission: (permissions: string[]) => permissions.some(p => options.permissions?.includes(p) || false),
    hasAllPermissions: (permissions: string[]) => permissions.every(p => options.permissions?.includes(p) || false),
    canAccess: (level: AuthLevel) => {
      const levels = ['ANONYMOUS', 'AUTHENTICATED', 'PROGRESSIVE', 'FULL'];
      const currentLevel = levels.indexOf(options.level || 'AUTHENTICATED');
      const requiredLevel = levels.indexOf(level);
      return currentLevel >= requiredLevel;
    }
  });
}