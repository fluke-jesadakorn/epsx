'use client';

import { useState, useEffect, createContext, useContext } from 'react';
import { useRouter } from 'next/navigation';

interface Platform {
  id: string;
  code: string;
  name: string;
  description: string;
  baseUrl: string;
  isActive: boolean;
}

interface User {
  id: string;
  email: string;
  name: string;
  role: string;
  platforms: string[];
  primaryPlatform: string;
  platformContext?: string;
  permissions: string[];
}

interface CrossPlatformAuthState {
  user: User | null;
  currentPlatform: Platform | null;
  availablePlatforms: Platform[];
  isLoading: boolean;
  error: string | null;
}

interface CrossPlatformAuthContextType extends CrossPlatformAuthState {
  switchPlatform: (platformCode: string) => Promise<void>;
  hasPermission: (permission: string) => boolean;
  hasPlatformAccess: (platformCode: string) => boolean;
  refreshUserData: () => Promise<void>;
  logout: () => Promise<void>;
}

const CrossPlatformAuthContext = createContext<CrossPlatformAuthContextType | null>(null);

// Default platforms configuration
const DEFAULT_PLATFORMS: Platform[] = [
  {
    id: '1',
    code: 'epsx',
    name: 'EPSX Platform',
    description: 'Main analytics and trading platform',
    baseUrl: 'https://epsx.io',
    isActive: true,
  },
  {
    id: '2',
    code: 'epsx-pay',
    name: 'EPSX Pay',
    description: 'Cryptocurrency payment and DeFi platform',
    baseUrl: 'https://pay.epsx.io',
    isActive: true,
  },
  {
    id: '3',
    code: 'epsx-token',
    name: 'EPSX Token',
    description: 'Governance and treasury management platform',
    baseUrl: 'https://token.epsx.io',
    isActive: true,
  },
];

export function useCrossPlatformAuth(): CrossPlatformAuthContextType {
  const context = useContext(CrossPlatformAuthContext);
  if (!context) {
    throw new Error('useCrossPlatformAuth must be used within a CrossPlatformAuthProvider');
  }
  return context;
}

export function CrossPlatformAuthProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<CrossPlatformAuthState>({
    user: null,
    currentPlatform: null,
    availablePlatforms: DEFAULT_PLATFORMS,
    isLoading: true,
    error: null,
  });
  const router = useRouter();

  // Initialize auth state
  useEffect(() => {
    initializeAuth();
  }, []);

  const initializeAuth = async () => {
    try {
      setState(prev => ({ ...prev, isLoading: true, error: null }));
      
      // Get user data from session/token
      const response = await fetch('/api/auth/session');
      if (!response.ok) {
        throw new Error('Failed to get session');
      }
      
      const userData = await response.json();
      
      if (userData.user) {
        const user: User = {
          id: userData.user.id,
          email: userData.user.email,
          name: userData.user.name,
          role: userData.user.role,
          platforms: userData.user.platforms || ['epsx'],
          primaryPlatform: userData.user.primaryPlatform || 'epsx',
          platformContext: userData.user.platformContext,
          permissions: userData.user.permissions || [],
        };
        
        // Get current platform
        const currentPlatform = DEFAULT_PLATFORMS.find(p => 
          p.code === (user.platformContext || user.primaryPlatform)
        ) || DEFAULT_PLATFORMS[0];
        
        setState(prev => ({
          ...prev,
          user,
          currentPlatform,
          isLoading: false,
        }));
      } else {
        setState(prev => ({
          ...prev,
          user: null,
          currentPlatform: null,
          isLoading: false,
        }));
      }
    } catch (error) {
      console.error('Auth initialization error:', error);
      setState(prev => ({
        ...prev,
        error: error instanceof Error ? error.message : 'Authentication error',
        isLoading: false,
      }));
    }
  };

  const switchPlatform = async (platformCode: string): Promise<void> => {
    try {
      if (!state.user) {
        throw new Error('User not authenticated');
      }

      // Check if user has access to the platform
      if (!state.user.platforms.includes(platformCode)) {
        throw new Error(`Access denied to platform: ${platformCode}`);
      }

      const targetPlatform = state.availablePlatforms.find(p => p.code === platformCode);
      if (!targetPlatform) {
        throw new Error(`Platform not found: ${platformCode}`);
      }

      setState(prev => ({ ...prev, isLoading: true, error: null }));

      // Update platform context on server
      const response = await fetch('/api/auth/switch-platform', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ platformCode }),
      });

      if (!response.ok) {
        throw new Error('Failed to switch platform');
      }

      // Update local state
      setState(prev => ({
        ...prev,
        currentPlatform: targetPlatform,
        user: prev.user ? {
          ...prev.user,
          platformContext: platformCode,
        } : null,
        isLoading: false,
      }));

      // Navigate to platform-specific routes if needed
      if (platformCode !== 'epsx') {
        // Could redirect to platform-specific admin interface
        // router.push(`/${platformCode}/admin`);
      }

    } catch (error) {
      console.error('Platform switch error:', error);
      setState(prev => ({
        ...prev,
        error: error instanceof Error ? error.message : 'Platform switch failed',
        isLoading: false,
      }));
      throw error;
    }
  };

  const hasPermission = (permission: string): boolean => {
    if (!state.user) return false;
    
    // Check exact permission
    if (state.user.permissions.includes(permission)) {
      return true;
    }
    
    // Check wildcard permissions
    for (const userPerm of state.user.permissions) {
      if (userPerm.endsWith('*')) {
        const prefix = userPerm.slice(0, -1);
        if (permission.startsWith(prefix)) {
          return true;
        }
      }
    }
    
    // Admin has all permissions
    if (state.user.role === 'admin') {
      return true;
    }
    
    return false;
  };

  const hasPlatformAccess = (platformCode: string): boolean => {
    if (!state.user) return false;
    return state.user.platforms.includes(platformCode);
  };

  const refreshUserData = async (): Promise<void> => {
    await initializeAuth();
  };

  const logout = async (): Promise<void> => {
    try {
      await fetch('/api/auth/logout', { method: 'POST' });
      setState({
        user: null,
        currentPlatform: null,
        availablePlatforms: DEFAULT_PLATFORMS,
        isLoading: false,
        error: null,
      });
      router.push('/login');
    } catch (error) {
      console.error('Logout error:', error);
    }
  };

  const contextValue: CrossPlatformAuthContextType = {
    ...state,
    switchPlatform,
    hasPermission,
    hasPlatformAccess,
    refreshUserData,
    logout,
  };

  return (
    <CrossPlatformAuthContext.Provider value={contextValue}>
      {children}
    </CrossPlatformAuthContext.Provider>
  );
}

// Hook for platform-specific permissions
export function usePlatformPermissions(platformCode: string) {
  const { user, hasPermission } = useCrossPlatformAuth();
  
  const hasPlatformPermission = (resource: string, action: string): boolean => {
    const permission = `${platformCode}:${resource}:${action}`;
    return hasPermission(permission);
  };
  
  const getPlatformPermissions = (): string[] => {
    if (!user) return [];
    return user.permissions.filter(perm => perm.startsWith(`${platformCode}:`));
  };
  
  return {
    hasPlatformPermission,
    getPlatformPermissions,
    platformPermissions: getPlatformPermissions(),
  };
}

// Hook for managing platform context
export function usePlatformContext() {
  const { currentPlatform, switchPlatform, availablePlatforms, user } = useCrossPlatformAuth();
  
  const switchToPlatform = async (platformCode: string) => {
    await switchPlatform(platformCode);
  };
  
  const getAccessiblePlatforms = (): Platform[] => {
    if (!user) return [];
    return availablePlatforms.filter(platform => 
      user.platforms.includes(platform.code)
    );
  };
  
  return {
    currentPlatform,
    switchToPlatform,
    accessiblePlatforms: getAccessiblePlatforms(),
    allPlatforms: availablePlatforms,
  };
}