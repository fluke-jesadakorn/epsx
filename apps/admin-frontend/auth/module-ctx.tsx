'use client';
import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { useAdminAuth } from './ctx';
import { AdminService } from '@/services/adminService';
import { adminLogger } from '@/lib/logger';

// Types for module-based permissions
interface ModuleAccess {
  assignment_id: string;
  module_id: string;
  module_name: string;
  display_name: string;
  access_level: 'bronze' | 'silver' | 'gold' | 'platinum' | 'enterprise';
  status: string;
  expires_at?: string;
  assigned_at: string;
  quotas: {
    api_calls?: number;
    rate_limit_per_minute: number;
    daily_limit?: number;
    monthly_limit?: number;
    custom_limits: Record<string, number>;
  };
  restrictions: {
    ip_restrictions: string[];
    time_restrictions?: string;
    feature_restrictions: Record<string, boolean>;
    endpoint_restrictions: string[];
  };
}

interface ModuleAuthCtx {
  moduleAccess: ModuleAccess[];
  loading: boolean;
  error: string | null;
  // Permission checking methods
  hasModuleAccess: (moduleName: string) => boolean;
  getAccessLevel: (moduleName: string) => string | null;
  canPerformAction: (moduleName: string, action: string) => boolean;
  hasFeatureAccess: (moduleName: string, feature: string) => boolean;
  getQuotaStatus: (moduleName: string) => {
    api_calls?: number;
    daily_limit?: number;
    monthly_limit?: number;
    rate_limit_per_minute: number;
  } | null;
  // Refresh module access
  refreshModuleAccess: () => Promise<void>;
}

const ModuleAuthContext = createContext<ModuleAuthCtx | null>(null);

export function ModuleAuthProvider({ children }: { children: ReactNode }) {
  const { user, loading: authLoading, init } = useAdminAuth();
  const [moduleAccess, setModuleAccess] = useState<ModuleAccess[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load user's module assignments
  const loadModuleAccess = async (userId: string) => {
    if (!userId) return;
    
    try {
      setLoading(true);
      setError(null);
      
      const response = await AdminService.getUserModuleAssignments(userId);
      
      if (response.success) {
        setModuleAccess(response.data.assignments || []);
        adminLogger.info('Module access loaded', { 
          userId, 
          moduleCount: response.data.assignments?.length || 0 
        });
      } else {
        setError('Failed to load module access');
        adminLogger.error('Failed to load module access', { userId });
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMessage);
      adminLogger.error('Error loading module access', { userId, error: errorMessage });
    } finally {
      setLoading(false);
    }
  };

  // Load module access when user is available
  useEffect(() => {
    if (user?.uid && init && !authLoading) {
      loadModuleAccess(user.uid);
    } else if (!user && init) {
      // Clear module access when user logs out
      setModuleAccess([]);
      setError(null);
    }
  }, [user?.uid, init, authLoading]);

  // Permission checking methods
  const hasModuleAccess = (moduleName: string): boolean => {
    return moduleAccess.some(access => 
      access.module_name === moduleName && 
      access.status === 'active' &&
      (!access.expires_at || new Date(access.expires_at) > new Date())
    );
  };

  const getAccessLevel = (moduleName: string): string | null => {
    const access = moduleAccess.find(access => 
      access.module_name === moduleName && 
      access.status === 'active' &&
      (!access.expires_at || new Date(access.expires_at) > new Date())
    );
    return access?.access_level || null;
  };

  const canPerformAction = (moduleName: string, action: string): boolean => {
    const access = moduleAccess.find(access => 
      access.module_name === moduleName && 
      access.status === 'active' &&
      (!access.expires_at || new Date(access.expires_at) > new Date())
    );
    
    if (!access) return false;

    // Check if action is restricted
    const isActionRestricted = access.restrictions.feature_restrictions[action] === false;
    if (isActionRestricted) return false;

    // Access level hierarchy: bronze < silver < gold < platinum < enterprise
    const levelHierarchy = ['bronze', 'silver', 'gold', 'platinum', 'enterprise'];
    const userLevelIndex = levelHierarchy.indexOf(access.access_level);
    
    // Define action requirements (this would typically come from backend configuration)
    const actionRequirements: Record<string, number> = {
      'view_rankings': 0, // bronze+
      'ai_insights': 1,   // silver+
      'custom_algorithms': 2, // gold+
      'bulk_operations': 3,   // platinum+
      'enterprise_features': 4, // enterprise
    };
    
    const requiredLevel = actionRequirements[action];
    return requiredLevel !== undefined ? userLevelIndex >= requiredLevel : true;
  };

  const hasFeatureAccess = (moduleName: string, feature: string): boolean => {
    const access = moduleAccess.find(access => 
      access.module_name === moduleName && 
      access.status === 'active' &&
      (!access.expires_at || new Date(access.expires_at) > new Date())
    );
    
    if (!access) return false;
    
    // Check explicit feature restrictions
    const featureRestriction = access.restrictions.feature_restrictions[feature];
    if (featureRestriction === false) return false;
    if (featureRestriction === true) return true;
    
    // Default to access level-based permissions
    return canPerformAction(moduleName, feature);
  };

  const getQuotaStatus = (moduleName: string) => {
    const access = moduleAccess.find(access => 
      access.module_name === moduleName && 
      access.status === 'active' &&
      (!access.expires_at || new Date(access.expires_at) > new Date())
    );
    
    return access?.quotas || null;
  };

  const refreshModuleAccess = async () => {
    if (user?.uid) {
      await loadModuleAccess(user.uid);
    }
  };

  const contextValue: ModuleAuthCtx = {
    moduleAccess,
    loading: loading || authLoading,
    error,
    hasModuleAccess,
    getAccessLevel,
    canPerformAction,
    hasFeatureAccess,
    getQuotaStatus,
    refreshModuleAccess,
  };

  return (
    <ModuleAuthContext.Provider value={contextValue}>
      {children}
    </ModuleAuthContext.Provider>
  );
}

export const useModuleAuth = () => {
  const ctx = useContext(ModuleAuthContext);
  if (!ctx) {
    throw new Error('useModuleAuth must be used within ModuleAuthProvider');
  }
  return ctx;
};

// Convenience hook that combines both auth contexts
export const useUserAccess = () => {
  const adminAuth = useAdminAuth();
  const moduleAuth = useModuleAuth();
  
  return {
    ...adminAuth,
    ...moduleAuth,
    // Combined loading state
    loading: adminAuth.loading || moduleAuth.loading,
    // Combined error state
    error: adminAuth.error || moduleAuth.error,
  };
};

// Higher-order component for module-based route protection
export function withModuleAccess(
  Component: React.ComponentType,
  requiredModule: string,
  requiredAction?: string
) {
  return function ModuleProtectedComponent(props: any) {
    const { hasModuleAccess, canPerformAction, loading } = useModuleAuth();
    const { user } = useAdminAuth();
    
    if (loading) {
      return (
        <div className="flex items-center justify-center p-8">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          <span className="ml-2 text-gray-600">Loading...</span>
        </div>
      );
    }
    
    if (!user) {
      return (
        <div className="flex items-center justify-center p-8">
          <div className="text-center">
            <h2 className="text-xl font-semibold text-gray-900 mb-2">Authentication Required</h2>
            <p className="text-gray-600">Please log in to access this feature.</p>
          </div>
        </div>
      );
    }
    
    if (!hasModuleAccess(requiredModule)) {
      return (
        <div className="flex items-center justify-center p-8">
          <div className="text-center">
            <h2 className="text-xl font-semibold text-gray-900 mb-2">Access Denied</h2>
            <p className="text-gray-600">
              You don't have access to the {requiredModule} module.
            </p>
            <p className="text-sm text-gray-500 mt-2">
              Contact your administrator to request access.
            </p>
          </div>
        </div>
      );
    }
    
    if (requiredAction && !canPerformAction(requiredModule, requiredAction)) {
      return (
        <div className="flex items-center justify-center p-8">
          <div className="text-center">
            <h2 className="text-xl font-semibold text-gray-900 mb-2">Insufficient Permissions</h2>
            <p className="text-gray-600">
              Your access level doesn't allow this action in the {requiredModule} module.
            </p>
            <p className="text-sm text-gray-500 mt-2">
              Contact your administrator to upgrade your access.
            </p>
          </div>
        </div>
      );
    }
    
    return <Component {...props} />;
  };
}

// Component for displaying user's module access status
export function ModuleAccessStatus() {
  const { moduleAccess, loading, error } = useModuleAuth();
  
  if (loading) {
    return (
      <div className="animate-pulse bg-gray-200 h-4 w-32 rounded"></div>
    );
  }
  
  if (error) {
    return (
      <div className="text-red-600 text-sm">
        Error loading module access
      </div>
    );
  }
  
  const activeModules = moduleAccess.filter(access => 
    access.status === 'active' &&
    (!access.expires_at || new Date(access.expires_at) > new Date())
  );
  
  return (
    <div className="text-sm text-gray-600">
      {activeModules.length} active module{activeModules.length !== 1 ? 's' : ''}
    </div>
  );
}
