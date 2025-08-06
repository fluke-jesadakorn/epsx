'use client';

import React, { createContext, useContext } from 'react';
import { useAdminAuth } from './ctx';

interface ModuleAuthContextType {
  hasModuleAccess: (module: string) => boolean;
  canPerformAction: (module: string, action: string) => boolean;
  getAccessLevel: (module: string) => string;
  hasFeatureAccess: (module: string, feature: string) => boolean;
  getQuotaStatus: (module: string) => { rate_limit_per_minute: number; daily_limit?: number } | null;
  moduleAccess: Record<string, any>;
  loading: boolean;
}

const ModuleAuthContext = createContext<ModuleAuthContextType | null>(null);

export function ModuleAuthProvider({ children }: { children: React.ReactNode }) {
  const { user, loading } = useAdminAuth();

  // Simple permission-based module access using NextAuth user data 
  const hasModuleAccess = (module: string): boolean => {
    if (!user) return false;
    
    // Super admin has access to everything
    if (user.role === 'super_admin') {
      return true;
    }
    
    // Admin has access to most modules
    if (user.role === 'admin') {
      return ['admin', 'users', 'analytics', 'billing', 'settings'].includes(module);
    }
    
    return false;
  };

  const canPerformAction = (module: string, action: string): boolean => {
    if (!user) return false;
    
    // Super admin can perform any action
    if (user.role === 'super_admin') {
      return true;
    }
    
    // Check if user has module access first
    if (!hasModuleAccess(module)) return false;
    
    // Admin can perform most actions
    if (user.role === 'admin') {
      // Restrict some dangerous actions
      const restrictedActions = ['delete_user', 'system_reset', 'backup_restore'];
      return !restrictedActions.includes(action);
    }
    
    return false;
  };

  const getAccessLevel = (module: string): string => {
    if (!user) return 'none';
    
    if (user.role === 'super_admin') {
      return 'full';
    }
    
    if (user.role === 'admin') {
      return hasModuleAccess(module) ? 'admin' : 'none';
    }
    
    return 'none';
  };

  const hasFeatureAccess = (module: string, feature: string): boolean => {
    if (!hasModuleAccess(module)) return false;
    
    // Super admin has access to all features
    if (user?.role === 'super_admin') {
      return true;
    }
    
    // Basic feature access logic - can be expanded
    if (user?.role === 'admin') {
      return true;
    }
    
    return false;
  };

  const getQuotaStatus = (module: string) => {
    if (!hasModuleAccess(module)) return null;
    
    // Mock quota status - replace with real data from backend
    return {
      rate_limit_per_minute: 60,
      daily_limit: 1000
    };
  };

  const moduleAccess = {
    admin: hasModuleAccess('admin'),
    users: hasModuleAccess('users'),
    analytics: hasModuleAccess('analytics'),
    billing: hasModuleAccess('billing'),
    settings: hasModuleAccess('settings'),
  };

  const contextValue: ModuleAuthContextType = {
    hasModuleAccess,
    canPerformAction,
    getAccessLevel,
    hasFeatureAccess,
    getQuotaStatus,
    moduleAccess,
    loading,
  };

  return (
    <ModuleAuthContext.Provider value={contextValue}>
      {children}
    </ModuleAuthContext.Provider>
  );
}

export function useModuleAuth(): ModuleAuthContextType {
  const context = useContext(ModuleAuthContext);
  if (!context) {
    // Return fallback instead of throwing error to prevent app crashes
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
  return context;
}

// Component to display current module access status
export function ModuleAccessStatus() {
  const { moduleAccess: _moduleAccess, getAccessLevel } = useModuleAuth();
  const { user } = useAdminAuth();
  
  if (!user) {
    return (
      <div className="px-2 py-1 bg-red-100 text-red-800 rounded text-xs font-medium">
        No Access
      </div>
    );
  }

  const userLevel = getAccessLevel('admin');
  const levelColors = {
    'full': 'bg-green-100 text-green-800',
    'admin': 'bg-blue-100 text-blue-800',
    'none': 'bg-red-100 text-red-800'
  };

  return (
    <div className={`px-2 py-1 rounded text-xs font-medium ${levelColors[userLevel as keyof typeof levelColors] || levelColors.none}`}>
      {userLevel.charAt(0).toUpperCase() + userLevel.slice(1)} Access
    </div>
  );
}

// HOC for module access protection
export function withModuleAccess<T extends object>(
  Component: React.ComponentType<T>,
  requiredModule: string,
  requiredAction?: string
) {
  return function ModuleProtectedComponent(props: T) {
    const { hasModuleAccess, canPerformAction } = useModuleAuth();
    
    if (!hasModuleAccess(requiredModule)) {
      return (
        <div className="p-4 text-center text-gray-500">
          <div className="mb-2">Access Denied</div>
          <div className="text-sm">You don't have access to the {requiredModule} module.</div>
        </div>
      );
    }
    
    if (requiredAction && !canPerformAction(requiredModule, requiredAction)) {
      return (
        <div className="p-4 text-center text-gray-500">
          <div className="mb-2">Insufficient Permissions</div>
          <div className="text-sm">You can't perform the {requiredAction} action in {requiredModule}.</div>
        </div>
      );
    }
    
    return <Component {...props} />;
  };
}