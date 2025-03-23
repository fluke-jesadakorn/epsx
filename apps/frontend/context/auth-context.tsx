"use client";

import React from "react";
import { authService } from "@/services/auth.service";
import { UserRole } from "@/types/auth/roles";

import type { TokenFeature, Permission } from '@/types/auth/features';

interface AuthState {
  isLoggedIn: boolean;
  userEmail: string | null;
  role: UserRole;
  tokenBalance: number;
  features: TokenFeature[];
  permissions: Permission[];
  isAdmin: boolean;
}

interface AuthContextValue extends AuthState {
  logout: () => Promise<void>;
  checkStatus: () => Promise<void>;
  hasFeature: (feature: TokenFeature) => boolean;
  hasPermission: (permission: Permission) => boolean;
}

const initialState: AuthState = {
  isLoggedIn: false,
  userEmail: null,
  role: UserRole.GUEST,
  tokenBalance: 0,
  features: [],
  permissions: [],
  isAdmin: false,
};

const AuthContext = React.createContext<AuthContextValue>({
  ...initialState,
  logout: async () => {},
  checkStatus: async () => {},
  hasFeature: () => false,
  hasPermission: () => false,
});

export const useAuth = (): AuthContextValue =>
  React.useContext(AuthContext);

export const AuthProvider = React.memo(({ children }: { children: React.ReactNode }) => {
  const [state, setState] = React.useState<AuthState>(initialState);
  
  const checkStatus = React.useCallback(async () => {
    const status = await authService.checkAuthStatus();
    setState(status);
  }, []);

  const logout = async () => {
    await authService.logout();
    setState(initialState);
  };

  const hasFeature = React.useCallback((feature: TokenFeature): boolean => {
    return state.features.includes(feature);
  }, [state.features]);

  const hasPermission = React.useCallback((permission: Permission): boolean => {
    return state.permissions.includes(permission);
  }, [state.permissions]);

  // Check status on mount and set up periodic checks
  React.useEffect(() => {
    checkStatus();
    
    // Check status every minute to keep auth state in sync
    const interval = setInterval(checkStatus, 60000);
    return () => clearInterval(interval);
  }, [checkStatus]);

  const value = React.useMemo(() => ({
    ...state,
    logout,
    checkStatus,
    hasFeature,
    hasPermission
  }), [state, checkStatus, hasFeature, hasPermission]);

  return (
    <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
  );
});

AuthProvider.displayName = "AuthProvider";
