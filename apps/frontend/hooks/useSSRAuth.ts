import { useAuth } from '@/context/auth-context';
import { useEffect, useState } from 'react';

/**
 * Enhanced auth hook that works with SSR-hydrated state
 */
export function useSSRAuth() {
  const auth = useAuth();
  const [isHydrated, setIsHydrated] = useState(false);

  useEffect(() => {
    setIsHydrated(true);
  }, []);

  return {
    ...auth,
    // Indicates if the client has fully hydrated with server state
    isHydrated,
    // True when we have either server-side or client-side auth data
    isReady: auth.isInitialized && isHydrated,
  };
}

/**
 * Hook for checking permissions that works with SSR
 */
export function useSSRPermission(permission: string) {
  const { hasPermission, isInitialized } = useAuth();
  const [isHydrated, setIsHydrated] = useState(false);

  useEffect(() => {
    setIsHydrated(true);
  }, []);

  return {
    hasPermission: hasPermission(permission),
    isChecking: !isInitialized || !isHydrated,
    isReady: isInitialized && isHydrated,
  };
}

/**
 * Hook for role-based rendering that works with SSR
 */
export function useSSRRole(requiredPermission: string) {
  const { user, permissions, isInitialized } = useAuth();
  const [isHydrated, setIsHydrated] = useState(false);

  useEffect(() => {
    setIsHydrated(true);
  }, []);

  const hasRequiredPermission = permissions.includes(requiredPermission) ||
    permissions.some(permission => {
      if (permission.endsWith('.*')) {
        const prefix = permission.slice(0, -2);
        return requiredPermission.startsWith(prefix + '.');
      }
      return false;
    });

  return {
    user,
    hasRole: hasRequiredPermission,
    isAuthenticated: !!user,
    isLoading: !isInitialized || !isHydrated,
    isReady: isInitialized && isHydrated,
  };
}

/**
 * Hook for managing auth state transitions (login/logout)
 */
export function useSSRAuthActions() {
  const { login, loginWithFirebaseToken, logout, register } = useAuth();
  const [isProcessing, setIsProcessing] = useState(false);

  const safeLogin = async (email: string, password: string) => {
    setIsProcessing(true);
    try {
      const result = await login(email, password);
      return result;
    } finally {
      setIsProcessing(false);
    }
  };

  const safeFirebaseLogin = async (token: string) => {
    setIsProcessing(true);
    try {
      const result = await loginWithFirebaseToken(token);
      return result;
    } finally {
      setIsProcessing(false);
    }
  };

  const safeLogout = async () => {
    setIsProcessing(true);
    try {
      await logout();
    } finally {
      setIsProcessing(false);
    }
  };

  const safeRegister = async (email: string, password: string, displayName?: string) => {
    setIsProcessing(true);
    try {
      await register(email, password, displayName);
    } finally {
      setIsProcessing(false);
    }
  };

  return {
    login: safeLogin,
    loginWithFirebaseToken: safeFirebaseLogin,
    logout: safeLogout,
    register: safeRegister,
    isProcessing,
  };
}