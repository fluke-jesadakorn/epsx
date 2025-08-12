'use client';

// Admin OIDC Authentication Context
// Simplified authentication context for admin frontend using OIDC only

import React, { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';

/**
 * Admin IAM profiles that have access to admin frontend
 */
const ADMIN_IAM_PROFILES = [
  'super_admin',
  'admin-full-004',
  'moderator-standard-003',
  'admin',
  'moderator'
];

/**
 * Unified user profile for OIDC authentication
 */
export interface AdminUserProfile {
  // Core identifiers
  id: string;
  email: string | null;
  
  // Display information
  displayName: string | null;
  photoURL: string | null;
  
  // Verification status
  emailVerified: boolean;
  
  // Provider information (OIDC only)
  provider: 'oidc';
  providerId: string;
  providerUserId: string; // OIDC sub
  
  // Application-specific data
  role: string;
  permissions: string[];
  subscriptionTier?: string;
  
  // Admin-specific metadata
  adminAccessLevel?: string;
  lastAdminAction?: string;
  adminSessionStarted?: string;
  
  // Metadata
  createdAt?: string;
  lastSignInAt?: string;
}

/**
 * JWT token from backend
 */
export interface AdminJWT {
  access_token: string;
  token_type: string;
  expires_at: string;
  expires_in: number;
  session_id: string;
  jti: string;
  refresh_token?: string;
}

/**
 * Admin OIDC authentication state
 */
export interface AdminOIDCAuthState {
  // Authentication status
  isAuthenticated: boolean;
  isLoading: boolean;
  isInitialized: boolean;
  
  // User information
  user: AdminUserProfile | null;
  
  // Tokens
  accessToken: string | null;
  jwt: AdminJWT | null;
  
  // Error handling
  error: string | null;
  
  // Authentication methods
  loginWithCredentials: (email: string, password: string) => Promise<void>;
  loginWithOIDC: () => Promise<void>;
  logout: () => Promise<void>;
  
  // Token management
  refreshToken: () => Promise<boolean>;
  getValidToken: () => Promise<string | null>;
  
  // Admin utilities
  clearError: () => void;
  isAdminUser: () => boolean;
  getAdminAccessLevel: () => string;
  logAdminAction: (action: string, details?: any) => void;
}

const AdminOIDCAuthContext = createContext<AdminOIDCAuthState | undefined>(undefined);

/**
 * Admin OIDC authentication provider props
 */
interface AdminOIDCAuthProviderProps {
  children: ReactNode;
  backendUrl?: string;
  adminTokenConfig?: {
    refreshThresholdMinutes?: number;
    sessionTimeoutMinutes?: number;
    maxRetryAttempts?: number;
    enableAuditLogging?: boolean;
  };
}

/**
 * Admin OIDC authentication provider component
 */
export function AdminOIDCAuthProvider({
  children,
  backendUrl,
  adminTokenConfig = {
    refreshThresholdMinutes: 2,
    sessionTimeoutMinutes: 60,
    maxRetryAttempts: 2,
    enableAuditLogging: true,
  }
}: AdminOIDCAuthProviderProps) {
  // State management
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isInitialized, setIsInitialized] = useState(false);
  const [user, setUser] = useState<AdminUserProfile | null>(null);
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const [jwt, setJWT] = useState<AdminJWT | null>(null);
  const [error, setError] = useState<string | null>(null);

  /**
   * Admin action logging
   */
  const logAdminAction = useCallback((action: string, details?: any) => {
    if (adminTokenConfig.enableAuditLogging) {
      const logEntry = {
        timestamp: new Date().toISOString(),
        userId: user?.id,
        email: user?.email,
        role: user?.role,
        provider: 'oidc',
        action,
        details,
        sessionId: jwt?.session_id,
        userAgent: typeof window !== 'undefined' ? navigator.userAgent : 'Server',
      };
      
      console.log('🔐 Admin Action:', logEntry);
      
      // TODO: Send to backend audit service
    }
  }, [user, jwt, adminTokenConfig.enableAuditLogging]);

  /**
   * Check if current user has admin access
   */
  const isAdminUser = useCallback((): boolean => {
    return user ? ADMIN_IAM_PROFILES.includes(user.role) : false;
  }, [user]);

  /**
   * Get admin access level
   */
  const getAdminAccessLevel = useCallback((): string => {
    if (!user) return 'none';
    
    switch (user.role) {
      case 'super_admin':
        return 'super';
      case 'admin-full-004':
      case 'admin':
        return 'full';
      case 'moderator-standard-003':
      case 'moderator':
        return 'standard';
      default:
        return 'none';
    }
  }, [user]);

  /**
   * Clear any authentication errors
   */
  const clearError = useCallback(() => {
    setError(null);
  }, []);

  /**
   * Direct backend login with email and password
   */
  const loginWithCredentials = useCallback(async (email: string, password: string) => {
    setIsLoading(true);
    setError(null);
    
    logAdminAction('admin_credentials_login_attempt', { email });

    try {
      // Direct backend authentication
      const apiUrl = backendUrl || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
      const response = await fetch(`${apiUrl}/api/v1/auth/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          type: 'credentials',
          email,
          password,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: 'Authentication failed' }));
        throw new Error(errorData.error || errorData.message || 'Authentication failed');
      }

      const authResult = await response.json();
      
      // Validate admin access
      if (!ADMIN_IAM_PROFILES.includes(authResult.role)) {
        throw new Error('Access denied: Admin privileges required');
      }

      // Create JWT token object
      const jwtToken: AdminJWT = {
        access_token: authResult.access_token,
        token_type: authResult.token_type || 'Bearer',
        expires_at: authResult.expires_at,
        expires_in: Math.floor((new Date(authResult.expires_at).getTime() - Date.now()) / 1000),
        session_id: authResult.access_token,
        jti: authResult.access_token,
      };

      // Create admin user profile
      const adminProfile: AdminUserProfile = {
        id: authResult.user_id,
        email: authResult.email,
        displayName: authResult.email?.split('@')[0] || null,
        photoURL: null,
        emailVerified: true,
        provider: 'oidc',
        providerId: 'backend',
        providerUserId: authResult.user_id,
        role: authResult.role,
        permissions: authResult.permissions || [],
        subscriptionTier: authResult.subscription_tier,
        
        // Admin-specific fields
        adminAccessLevel: ADMIN_IAM_PROFILES.includes(authResult.role) ? 
          (authResult.role === 'super_admin' ? 'super' : 
           authResult.role === 'admin-full-004' || authResult.role === 'admin' ? 'full' : 'standard') : 'none',
        adminSessionStarted: new Date().toISOString(),
        
        createdAt: new Date().toISOString(),
        lastSignInAt: new Date().toISOString(),
      };

      // Update state
      setUser(adminProfile);
      setAccessToken(jwtToken.access_token);
      setJWT(jwtToken);
      setIsAuthenticated(true);
      setError(null);
      
      console.log('Admin OIDC credential sign-in successful:', adminProfile.email);
      logAdminAction('admin_credentials_login_success', { 
        email: adminProfile.email,
        role: adminProfile.role,
        accessLevel: adminProfile.adminAccessLevel
      });
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Admin authentication failed';
      setError(errorMessage);
      console.error('Admin credential sign-in error:', error);
      
      logAdminAction('admin_credentials_login_failed', { 
        email,
        error: errorMessage,
      });
      
    } finally {
      setIsLoading(false);
    }
  }, [logAdminAction, backendUrl]);

  /**
   * Admin login with OIDC (redirect to backend)
   */
  const loginWithOIDC = useCallback(async () => {
    setError(null);
    logAdminAction('admin_oidc_login_attempt');

    try {
      const oidcUrl = `${backendUrl || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'}/api/auth/oidc/authorize?admin=true`;
      window.location.href = oidcUrl;
      
      logAdminAction('admin_oidc_redirect', { oidcUrl });
      
    } catch (error) {
      setError('Admin OIDC sign-in failed');
      console.error('Admin OIDC sign-in error:', error);
      
      logAdminAction('admin_oidc_login_failed', { error: error instanceof Error ? error.message : 'Unknown error' });
    }
  }, [backendUrl, logAdminAction]);

  /**
   * Sign out current admin user
   */
  const logout = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    const logoutDetails = {
      userId: user?.id,
      email: user?.email,
      role: user?.role,
      provider: 'oidc',
      sessionDuration: user?.adminSessionStarted ? 
        Date.now() - new Date(user.adminSessionStarted).getTime() : null
    };

    logAdminAction('admin_logout_attempt', logoutDetails);

    try {
      // Clear local state
      setIsAuthenticated(false);
      setUser(null);
      setAccessToken(null);
      setJWT(null);
      
      console.log('Admin logout successful');
      logAdminAction('admin_logout_success', logoutDetails);
      
    } catch (error) {
      console.error('Admin logout error:', error);
      setError('Logout failed');
      
      logAdminAction('admin_logout_failed', { 
        ...logoutDetails,
        error: error instanceof Error ? error.message : 'Unknown error'
      });
      
      // Clear state anyway for security
      setIsAuthenticated(false);
      setUser(null);
      setAccessToken(null);
      setJWT(null);
    } finally {
      setIsLoading(false);
    }
  }, [user, logAdminAction]);

  /**
   * Refresh authentication token
   */
  const refreshToken = useCallback(async (): Promise<boolean> => {
    if (!jwt) {
      return false;
    }

    logAdminAction('admin_token_refresh_attempt', { 
      tokenExpiry: jwt.expires_at 
    });

    try {
      // TODO: Implement token refresh with backend
      console.warn('Token refresh not implemented yet');
      return false;
      
    } catch (error) {
      console.error('Admin token refresh failed:', error);
      setError('Token refresh failed');
      
      logAdminAction('admin_token_refresh_error', { 
        error: error instanceof Error ? error.message : 'Unknown error'
      });
      
      return false;
    }
  }, [jwt, logAdminAction]);

  /**
   * Get valid access token (refresh if needed)
   */
  const getValidToken = useCallback(async (): Promise<string | null> => {
    if (!jwt) {
      return null;
    }

    // Check if token is still valid
    const now = new Date();
    const expiresAt = new Date(jwt.expires_at);
    const minutesUntilExpiry = (expiresAt.getTime() - now.getTime()) / (1000 * 60);

    if (minutesUntilExpiry <= adminTokenConfig.refreshThresholdMinutes!) {
      // Token needs refresh
      const refreshed = await refreshToken();
      if (!refreshed) {
        return null;
      }
    }

    return jwt.access_token;
  }, [jwt, refreshToken, adminTokenConfig.refreshThresholdMinutes]);

  // Initialize authentication
  useEffect(() => {
    const initializeAuth = async () => {
      try {
        // Check for existing session in localStorage/cookies
        const storedAuth = localStorage.getItem('admin_auth');
        if (storedAuth) {
          const authData = JSON.parse(storedAuth);
          // TODO: Validate stored auth data and restore session
        }
        
        setIsInitialized(true);
        logAdminAction('admin_auth_initialized', { provider: 'oidc' });
        
      } catch (error) {
        console.error('Failed to initialize admin authentication:', error);
        setError('Admin authentication initialization failed');
        
        logAdminAction('admin_auth_init_failed', { error: error instanceof Error ? error.message : 'Unknown error' });
      } finally {
        setIsLoading(false);
      }
    };

    initializeAuth();
  }, [logAdminAction]);

  // Context value
  const value: AdminOIDCAuthState = {
    // State
    isAuthenticated,
    isLoading,
    isInitialized,
    user,
    accessToken,
    jwt,
    error,
    
    // Methods
    loginWithCredentials,
    loginWithOIDC,
    logout,
    refreshToken,
    getValidToken,
    clearError,
    
    // Admin-specific methods
    isAdminUser,
    getAdminAccessLevel,
    logAdminAction,
  };

  return (
    <AdminOIDCAuthContext.Provider value={value}>
      {children}
    </AdminOIDCAuthContext.Provider>
  );
}

/**
 * Hook to use admin OIDC authentication context
 */
export function useAdminOIDCAuth(): AdminOIDCAuthState {
  const context = useContext(AdminOIDCAuthContext);
  
  if (context === undefined) {
    throw new Error('useAdminOIDCAuth must be used within AdminOIDCAuthProvider');
  }
  
  return context;
}