'use client';

import React, { createContext, useContext } from 'react';
import { useAdminAuth } from './ctx';

interface ModuleAuthContextType {
  hasModuleAccess: (module: string) => boolean;
  canPerformAction: (module: string, action: string) => boolean;
  getAccessLevel: (module: string) => string;
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
      moduleAccess: {},
      loading: false,
    };
  }
  return context;
}