'use client';

// Enhanced Module Authentication Context
// Integrates with Casbin authorization system and multi-provider authentication

import React, { createContext, useContext, useEffect, useState } from 'react';
import { useAdminAuth, useEnhancedAdminAuth } from './ctx';

/**
 * Module access levels based on Casbin permissions
 */
type ModuleAccessLevel = 'none' | 'read' | 'write' | 'admin' | 'full';

/**
 * Module permissions structure
 */
interface ModulePermissions {
  read: boolean;
  write: boolean;
  delete: boolean;
  admin: boolean;
}

/**
 * Enhanced module auth context type with Casbin integration
 */
interface EnhancedModuleAuthContextType {
  // Legacy compatibility
  hasModuleAccess: (module: string) => boolean;
  canPerformAction: (module: string, action: string) => boolean;
  getAccessLevel: (module: string) => string;
  hasFeatureAccess: (module: string, feature: string) => boolean;
  getQuotaStatus: (module: string) => { rate_limit_per_minute: number; daily_limit?: number } | null;
  moduleAccess: Record<string, any>;
  loading: boolean;

  // Enhanced Casbin-based methods
  checkPermission: (resource: string, action: string) => Promise<boolean>;
  getModulePermissions: (module: string) => Promise<ModulePermissions>;
  getUserPermissions: () => string[];
  getEffectiveRole: () => string;
  
  // Admin-specific capabilities
  isAdminLevel: () => boolean;
  canManageUsers: () => boolean;
  canAccessAnalytics: () => boolean;
  canManageBilling: () => boolean;
  canManageSettings: () => boolean;
  
  // Real-time permission updates
  refreshPermissions: () => Promise<void>;
  onPermissionChanged: (callback: (permissions: string[]) => void) => void;
}

const EnhancedModuleAuthContext = createContext<EnhancedModuleAuthContextType | null>(null);

/**
 * Enhanced module auth provider with Casbin integration
 */
export function EnhancedModuleAuthProvider({ children }: { children: React.ReactNode }) {
  const { user, loading } = useAdminAuth();
  const { getAccessToken } = useEnhancedAdminAuth();
  
  const [userPermissions, setUserPermissions] = useState<string[]>([]);
  const [isLoadingPermissions, setIsLoadingPermissions] = useState(false);
  const [permissionCallbacks, setPermissionCallbacks] = useState<((permissions: string[]) => void)[]>([]);

  /**
   * Get effective user role for Casbin
   */
  const getEffectiveRole = (): string => {
    if (!user) return 'none';
    return user.role;
  };

  /**
   * Check if user has admin-level access
   */
  const isAdminLevel = (): boolean => {
    const role = getEffectiveRole();
    return ['admin-full-004', 'moderator-standard-003'].includes(role);
  };

  /**
   * Fetch user permissions from backend via Casbin
   */
  const fetchUserPermissions = async (): Promise<string[]> => {
    if (!user) return [];

    try {
      const token = await getAccessToken();
      if (!token) return [];

      // Call backend Casbin API to get user permissions
      const response = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'}/api/auth/permissions`, {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
          'X-Client-Type': 'admin-frontend',
        },
      });

      if (response.ok) {
        const result = await response.json();
        return result.permissions || [];
      } else {
        console.warn('Failed to fetch user permissions:', response.status);
        return [];
      }
    } catch (error) {
      console.error('Error fetching user permissions:', error);
      return [];
    }
  };

  /**
   * Check specific permission via Casbin backend
   */
  const checkPermission = async (resource: string, action: string): Promise<boolean> => {
    if (!user) return false;

    try {
      const token = await getAccessToken();
      if (!token) return false;

      const response = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'}/api/auth/check-permission`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
          'X-Client-Type': 'admin-frontend',
        },
        body: JSON.stringify({
          resource,
          action,
        }),
      });

      if (response.ok) {
        const result = await response.json();
        return result.allowed || false;
      } else {
        console.warn('Permission check failed:', response.status);
        return false;
      }
    } catch (error) {
      console.error('Error checking permission:', error);
      return false;
    }
  };

  /**
   * Get module-specific permissions
   */
  const getModulePermissions = async (module: string): Promise<ModulePermissions> => {
    const [read, write, deleteAccess, admin] = await Promise.all([
      checkPermission(module, 'read'),
      checkPermission(module, 'write'),
      checkPermission(module, 'delete'),
      checkPermission(module, 'admin'),
    ]);

    return { read, write, delete: deleteAccess, admin };
  };

  /**
   * Admin-specific permission methods
   */
  const canManageUsers = (): boolean => {
    return userPermissions.some(p => 
      p.includes('users:write') || 
      p.includes('users:admin') || 
      p.includes('admin:*')
    );
  };

  const canAccessAnalytics = (): boolean => {
    return userPermissions.some(p => 
      p.includes('analytics:read') || 
      p.includes('premium-data:read') ||
      p.includes('admin:*')
    );
  };

  const canManageBilling = (): boolean => {
    return userPermissions.some(p => 
      p.includes('billing:write') || 
      p.includes('admin:*')
    );
  };

  const canManageSettings = (): boolean => {
    return userPermissions.some(p => 
      p.includes('settings:write') || 
      p.includes('admin:*')
    );
  };

  /**
   * Legacy compatibility methods
   */
  const hasModuleAccess = (module: string): boolean => {
    if (!user || !isAdminLevel()) return false;
    
    // Check if user has any permission for this module
    return userPermissions.some(permission => 
      permission.startsWith(`${module}:`) || permission === 'admin:*'
    );
  };

  const canPerformAction = (module: string, action: string): boolean => {
    if (!user || !isAdminLevel()) return false;
    
    // Check specific permission or admin wildcard
    return userPermissions.some(permission => 
      permission === `${action}:${module}` || 
      permission === `${module}:${action}` ||
      permission === 'admin:*'
    );
  };

  const getAccessLevel = (module: string): string => {
    if (!user || !isAdminLevel()) return 'none';
    
    // Check permissions in order of precedence
    if (userPermissions.some(p => p === 'admin:*' || p === `${module}:admin`)) return 'full';
    if (userPermissions.some(p => p === `${module}:write`)) return 'admin';
    if (userPermissions.some(p => p === `${module}:read`)) return 'read';
    
    return 'none';
  };

  const hasFeatureAccess = (module: string, feature: string): boolean => {
    return canPerformAction(module, feature);
  };

  const getQuotaStatus = (module: string) => {
    if (!hasModuleAccess(module)) return null;
    
    // Enhanced quota based on admin level
    const role = getEffectiveRole();
    switch (role) {
      case 'admin-full-004':
        return { rate_limit_per_minute: 200, daily_limit: 10000 };
      case 'moderator-standard-003':
        return { rate_limit_per_minute: 100, daily_limit: 5000 };
      default:
        return { rate_limit_per_minute: 60, daily_limit: 1000 };
    }
  };

  /**
   * Refresh permissions from backend
   */
  const refreshPermissions = async (): Promise<void> => {
    if (!user || isLoadingPermissions) return;

    setIsLoadingPermissions(true);
    try {
      const permissions = await fetchUserPermissions();
      setUserPermissions(permissions);
      
      // Notify callbacks of permission changes
      permissionCallbacks.forEach(callback => callback(permissions));
      
      console.log('🔐 Admin permissions refreshed:', permissions);
    } finally {
      setIsLoadingPermissions(false);
    }
  };

  /**
   * Register callback for permission changes
   */
  const onPermissionChanged = (callback: (permissions: string[]) => void) => {
    setPermissionCallbacks(prev => [...prev, callback]);
    
    // Return cleanup function
    return () => {
      setPermissionCallbacks(prev => prev.filter(cb => cb !== callback));
    };
  };

  /**
   * Load permissions when user changes
   */
  useEffect(() => {
    if (user && !loading) {
      refreshPermissions();
    } else {
      setUserPermissions([]);
    }
  }, [user, loading]);

  // Module access mapping for legacy compatibility
  const moduleAccess = {
    admin: hasModuleAccess('admin'),
    users: hasModuleAccess('users'),
    analytics: hasModuleAccess('analytics'),
    billing: hasModuleAccess('billing'),
    settings: hasModuleAccess('settings'),
  };

  const contextValue: EnhancedModuleAuthContextType = {
    // Legacy compatibility
    hasModuleAccess,
    canPerformAction,
    getAccessLevel,
    hasFeatureAccess,
    getQuotaStatus,
    moduleAccess,
    loading: loading || isLoadingPermissions,

    // Enhanced Casbin-based methods
    checkPermission,
    getModulePermissions,
    getUserPermissions: () => userPermissions,
    getEffectiveRole,
    
    // Admin-specific capabilities
    isAdminLevel,
    canManageUsers,
    canAccessAnalytics,
    canManageBilling,
    canManageSettings,
    
    // Real-time permission updates
    refreshPermissions,
    onPermissionChanged,
  };

  return (
    <EnhancedModuleAuthContext.Provider value={contextValue}>
      {children}
    </EnhancedModuleAuthContext.Provider>
  );
}

/**
 * Enhanced useModuleAuth hook with backward compatibility
 */
export function useModuleAuth() {
  const context = useContext(EnhancedModuleAuthContext);
  if (!context) {
    // Return fallback for backward compatibility
    console.warn('useModuleAuth: Context not found, returning fallback');
    return {
      hasModuleAccess: () => false,
      canPerformAction: () => false,
      getAccessLevel: () => 'none',
      hasFeatureAccess: () => false,
      getQuotaStatus: () => null,
      moduleAccess: {},
      loading: false,
    };
  }
  
  // Return legacy interface for backward compatibility
  return {
    hasModuleAccess: context.hasModuleAccess,
    canPerformAction: context.canPerformAction,
    getAccessLevel: context.getAccessLevel,
    hasFeatureAccess: context.hasFeatureAccess,
    getQuotaStatus: context.getQuotaStatus,
    moduleAccess: context.moduleAccess,
    loading: context.loading,
  };
}

/**
 * Enhanced useEnhancedModuleAuth hook with full Casbin capabilities
 */
export function useEnhancedModuleAuth(): EnhancedModuleAuthContextType {
  const context = useContext(EnhancedModuleAuthContext);
  if (!context) {
    throw new Error('useEnhancedModuleAuth must be used within EnhancedModuleAuthProvider');
  }
  return context;
}

/**
 * Hook for admin-specific permissions
 */
export function useAdminPermissions() {
  const context = useContext(EnhancedModuleAuthContext);
  if (!context) {
    throw new Error('useAdminPermissions must be used within EnhancedModuleAuthProvider');
  }
  
  return {
    isAdminLevel: context.isAdminLevel,
    canManageUsers: context.canManageUsers,
    canAccessAnalytics: context.canAccessAnalytics,
    canManageBilling: context.canManageBilling,
    canManageSettings: context.canManageSettings,
    permissions: context.getUserPermissions(),
    role: context.getEffectiveRole(),
    checkPermission: context.checkPermission,
    refreshPermissions: context.refreshPermissions,
  };
}

/**
 * Component to display current module access status
 */
export function ModuleAccessStatus() {
  const { getAccessLevel, getEffectiveRole } = useEnhancedModuleAuth();
  const { user } = useAdminAuth();
  
  if (!user) {
    return (
      <div className="px-2 py-1 bg-red-100 text-red-800 rounded text-xs font-medium">
        No Access
      </div>
    );
  }

  const userLevel = getAccessLevel('admin');
  const role = getEffectiveRole();
  
  const levelColors = {
    'full': 'bg-green-100 text-green-800',
    'admin': 'bg-blue-100 text-blue-800',
    'read': 'bg-yellow-100 text-yellow-800',
    'none': 'bg-red-100 text-red-800'
  };

  return (
    <div className="space-y-1">
      <div className={`px-2 py-1 rounded text-xs font-medium ${levelColors[userLevel as keyof typeof levelColors] || levelColors.none}`}>
        {userLevel.charAt(0).toUpperCase() + userLevel.slice(1)} Access
      </div>
      <div className="px-2 py-1 bg-gray-100 text-gray-800 rounded text-xs">
        {role}
      </div>
    </div>
  );
}

/**
 * HOC for enhanced module access protection with Casbin
 */
export function withEnhancedModuleAccess<T extends object>(
  Component: React.ComponentType<T>,
  requiredResource: string,
  requiredAction?: string
) {
  return function EnhancedModuleProtectedComponent(props: T) {
    const { checkPermission, isAdminLevel } = useEnhancedModuleAuth();
    const [hasAccess, setHasAccess] = useState<boolean | null>(null);
    
    useEffect(() => {
      const checkAccess = async () => {
        if (!isAdminLevel()) {
          setHasAccess(false);
          return;
        }
        
        if (requiredAction) {
          const allowed = await checkPermission(requiredResource, requiredAction);
          setHasAccess(allowed);
        } else {
          // Check if user has any access to the resource
          const readAccess = await checkPermission(requiredResource, 'read');
          setHasAccess(readAccess);
        }
      };
      
      checkAccess();
    }, [requiredResource, requiredAction, checkPermission, isAdminLevel]);
    
    if (hasAccess === null) {
      return (
        <div className="p-4 text-center text-gray-500">
          <div className="animate-pulse">Checking permissions...</div>
        </div>
      );
    }
    
    if (!hasAccess) {
      return (
        <div className="p-4 text-center text-gray-500">
          <div className="mb-2">Access Denied</div>
          <div className="text-sm">
            You don't have permission to {requiredAction || 'access'} {requiredResource}.
          </div>
        </div>
      );
    }
    
    return <Component {...props} />;
  };
}