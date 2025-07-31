import { useCallback, useState } from 'react';
import type { AuthenticatedUser } from '@epsx/types';

export interface AuthCredentials {
  email: string;
  password: string;
}

export interface RegisterCredentials extends AuthCredentials {
  displayName?: string;
  confirmPassword?: string;
}

export interface AuthActionOptions {
  /** Show loading states */
  showLoading?: boolean;
  /** Auto refresh permissions after login */
  autoRefreshPermissions?: boolean;
  /** Redirect URL after successful action */
  redirectUrl?: string;
}

export interface AuthActionsContext {
  login: (credentials: AuthCredentials) => Promise<AuthenticatedUser>;
  logout: () => Promise<void>;
  register?: (credentials: RegisterCredentials) => Promise<AuthenticatedUser>;
  refreshPermissions?: () => Promise<void>;
  updateProfile?: (updates: Partial<AuthenticatedUser>) => Promise<AuthenticatedUser>;
}

/**
 * Unified authentication actions hook
 * Provides consistent action handling with error management and loading states
 */
export function useAuthActions(
  context: AuthActionsContext,
  options: AuthActionOptions = {}
) {
  const {
    showLoading = true,
    autoRefreshPermissions = false,
    redirectUrl,
  } = options;

  // Loading states
  const [isLoggingIn, setIsLoggingIn] = useState(false);
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const [isRegistering, setIsRegistering] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isUpdatingProfile, setIsUpdatingProfile] = useState(false);

  // Error states
  const [loginError, setLoginError] = useState<string | null>(null);
  const [logoutError, setLogoutError] = useState<string | null>(null);
  const [registerError, setRegisterError] = useState<string | null>(null);
  const [refreshError, setRefreshError] = useState<string | null>(null);
  const [profileError, setProfileError] = useState<string | null>(null);

  // Clear all errors
  const clearErrors = useCallback(() => {
    setLoginError(null);
    setLogoutError(null);
    setRegisterError(null);
    setRefreshError(null);
    setProfileError(null);
  }, []);

  // Login action with error handling
  const login = useCallback(async (credentials: AuthCredentials) => {
    if (showLoading) setIsLoggingIn(true);
    setLoginError(null);

    try {
      const user = await context.login(credentials);
      
      // Auto-refresh permissions if enabled
      if (autoRefreshPermissions && context.refreshPermissions) {
        try {
          await context.refreshPermissions();
        } catch (refreshErr) {
          console.warn('Failed to refresh permissions after login:', refreshErr);
          // Don't fail the login for permission refresh errors
        }
      }

      // Handle redirect
      if (redirectUrl && typeof window !== 'undefined') {
        window.location.href = redirectUrl;
      }

      return user;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Login failed';
      setLoginError(errorMessage);
      throw error;
    } finally {
      if (showLoading) setIsLoggingIn(false);
    }
  }, [context.login, context.refreshPermissions, showLoading, autoRefreshPermissions, redirectUrl]);

  // Logout action with error handling
  const logout = useCallback(async () => {
    if (showLoading) setIsLoggingOut(true);
    setLogoutError(null);

    try {
      await context.logout();
      
      // Clear all errors on successful logout
      clearErrors();

      // Handle redirect
      if (redirectUrl && typeof window !== 'undefined') {
        window.location.href = redirectUrl;
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Logout failed';
      setLogoutError(errorMessage);
      throw error;
    } finally {
      if (showLoading) setIsLoggingOut(false);
    }
  }, [context.logout, showLoading, redirectUrl, clearErrors]);

  // Register action with error handling
  const register = useCallback(async (credentials: RegisterCredentials) => {
    if (!context.register) {
      throw new Error('Registration not supported in this context');
    }

    if (showLoading) setIsRegistering(true);
    setRegisterError(null);

    try {
      // Validate passwords match if confirmPassword is provided
      if (credentials.confirmPassword && credentials.password !== credentials.confirmPassword) {
        throw new Error('Passwords do not match');
      }

      const user = await context.register(credentials);
      
      // Auto-refresh permissions if enabled
      if (autoRefreshPermissions && context.refreshPermissions) {
        try {
          await context.refreshPermissions();
        } catch (refreshErr) {
          console.warn('Failed to refresh permissions after registration:', refreshErr);
        }
      }

      // Handle redirect
      if (redirectUrl && typeof window !== 'undefined') {
        window.location.href = redirectUrl;
      }

      return user;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Registration failed';
      setRegisterError(errorMessage);
      throw error;
    } finally {
      if (showLoading) setIsRegistering(false);
    }
  }, [context.register, context.refreshPermissions, showLoading, autoRefreshPermissions, redirectUrl]);

  // Refresh permissions action
  const refreshPermissions = useCallback(async () => {
    if (!context.refreshPermissions) {
      throw new Error('Permission refresh not supported in this context');
    }

    if (showLoading) setIsRefreshing(true);
    setRefreshError(null);

    try {
      await context.refreshPermissions();
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Permission refresh failed';
      setRefreshError(errorMessage);
      throw error;
    } finally {
      if (showLoading) setIsRefreshing(false);
    }
  }, [context.refreshPermissions, showLoading]);

  // Update profile action
  const updateProfile = useCallback(async (updates: Partial<AuthenticatedUser>) => {
    if (!context.updateProfile) {
      throw new Error('Profile update not supported in this context');
    }

    if (showLoading) setIsUpdatingProfile(true);
    setProfileError(null);

    try {
      const updatedUser = await context.updateProfile(updates);
      return updatedUser;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Profile update failed';
      setProfileError(errorMessage);
      throw error;
    } finally {
      if (showLoading) setIsUpdatingProfile(false);
    }
  }, [context.updateProfile, showLoading]);

  // Convenience methods
  const signIn = useCallback((credentials: AuthCredentials) => {
    return login(credentials);
  }, [login]);

  const signOut = useCallback(() => {
    return logout();
  }, [logout]);

  // Utility methods
  const isAnyActionLoading = showLoading && (
    isLoggingIn || isLoggingOut || isRegistering || isRefreshing || isUpdatingProfile
  );

  const hasAnyError = !!(
    loginError || logoutError || registerError || refreshError || profileError
  );

  const getAllErrors = useCallback(() => {
    const errors: string[] = [];
    if (loginError) errors.push(`Login: ${loginError}`);
    if (logoutError) errors.push(`Logout: ${logoutError}`);
    if (registerError) errors.push(`Register: ${registerError}`);
    if (refreshError) errors.push(`Refresh: ${refreshError}`);
    if (profileError) errors.push(`Profile: ${profileError}`);
    return errors;
  }, [loginError, logoutError, registerError, refreshError, profileError]);

  return {
    // Core actions
    login,
    logout,
    register: context.register ? register : undefined,
    refreshPermissions: context.refreshPermissions ? refreshPermissions : undefined,
    updateProfile: context.updateProfile ? updateProfile : undefined,
    
    // Aliases
    signIn,
    signOut,
    
    // Loading states
    isLoggingIn: showLoading ? isLoggingIn : false,
    isLoggingOut: showLoading ? isLoggingOut : false,
    isRegistering: showLoading ? isRegistering : false,
    isRefreshing: showLoading ? isRefreshing : false,
    isUpdatingProfile: showLoading ? isUpdatingProfile : false,
    isAnyActionLoading,
    
    // Error states
    loginError,
    logoutError,
    registerError,
    refreshError,
    profileError,
    hasAnyError,
    
    // Utility methods
    clearErrors,
    getAllErrors,
  };
}