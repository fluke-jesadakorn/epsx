'use client';

// Enhanced Multi-Provider Authentication with Tenant Detection
// Integrates with the new backend OIDC system for seamless multi-tenant authentication

import React, { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';
import { User as FirebaseUser } from 'firebase/auth';
import { useRouter, useSearchParams } from 'next/navigation';

// Import Firebase authentication (keeping for fallback)
import { 
  getFirebaseAuth,
  getFirebaseTokenValidator,
  type FirebaseUserProfile,
  type FirebaseAuthResult,
  FirebaseAuthError
} from '@epsx/firebase-analytics';

/**
 * Enhanced unified user profile with tenant information
 */
export interface EnhancedUnifiedUserProfile {
  // Core identifiers
  id: string;
  email: string | null;
  
  // Display information
  displayName: string | null;
  photoURL: string | null;
  
  // Verification status
  emailVerified: boolean;
  
  // Provider information
  provider: 'firebase' | 'oidc' | 'google' | 'microsoft' | 'auth0' | 'enterprise';
  providerId: string;
  providerUserId: string;
  
  // Multi-tenant information
  tenantId: string;
  tenantName?: string;
  providerType: string;
  
  // Application-specific data
  role: string;
  permissions: string[];
  subscriptionTier?: string;
  
  // Security metadata
  sessionId?: string;
  authTime?: number;
  riskScore?: number;
  
  // Metadata
  createdAt?: string;
  lastSignInAt?: string;
}

/**
 * Enhanced JWT token from enhanced backend
 */
export interface EnhancedUnifiedJWT {
  access_token: string;
  token_type: string;
  expires_in: number;
  expires_at: string;
  refresh_token?: string;
  refresh_expires_in?: number;
  refresh_expires_at?: string;
  id_token?: string;
  scope: string;
  session_id: string;
  jti: string;
  custom_ttl_applied: boolean;
  provider_id: string;
  tenant_id: string;
}

/**
 * Tenant detection result
 */
export interface TenantDetectionResult {
  tenantId: string;
  providerId: string;
  providerType: string;
  autoProvision: boolean;
  defaultRole: string;
  requiresStepUp: boolean;
}

/**
 * Cross-app authentication result
 */
export interface CrossAppAuthResult {
  access_token: string;
  token_type: string;
  expires_in: number;
  expires_at: string;
  scope: string;
  global_session_id: string;
  app_session_id: string;
  step_up_completed: boolean;
}

/**
 * Enhanced multi-provider authentication state
 */
export interface EnhancedMultiProviderAuthState {
  // Authentication status
  isAuthenticated: boolean;
  isLoading: boolean;
  isInitialized: boolean;
  
  // User information
  user: EnhancedUnifiedUserProfile | null;
  firebaseUser: FirebaseUser | null;
  
  // Tokens and session
  accessToken: string | null;
  unifiedJWT: EnhancedUnifiedJWT | null;
  globalSessionId: string | null;
  
  // Provider and tenant information
  currentProvider: string | null;
  currentTenant: string | null;
  availableProviders: string[];
  
  // Error handling
  error: string | null;
  
  // Authentication methods
  loginWithGoogle: () => Promise<void>;
  loginWithMicrosoft: () => Promise<void>;
  loginWithCredentials: (email: string, password: string) => Promise<void>;
  loginWithTenantHint: (email: string, tenantId?: string) => Promise<void>;
  loginWithProvider: (providerId: string, tenantId?: string) => Promise<void>;
  
  // Tenant operations
  detectTenant: (email: string) => Promise<TenantDetectionResult | null>;
  switchTenant: (tenantId: string) => Promise<void>;
  
  // Cross-app authentication
  authenticateForApp: (targetApp: string, scopes?: string[]) => Promise<CrossAppAuthResult>;
  
  // Account management
  createAccount: (email: string, password: string) => Promise<void>;
  logout: (logoutFromAllApps?: boolean) => Promise<void>;
  
  // Token management
  refreshToken: () => Promise<boolean>;
  getValidToken: () => Promise<string | null>;
  
  // Utilities
  clearError: () => void;
  hasPermission: (permission: string) => boolean;
  hasRole: (role: string) => boolean;
}

const EnhancedMultiProviderAuthContext = createContext<EnhancedMultiProviderAuthState | undefined>(undefined);

/**
 * Enhanced authentication provider props
 */
interface EnhancedMultiProviderAuthProviderProps {
  children: ReactNode;
  backendUrl?: string;
  enableFirebaseFallback?: boolean;
  enableTenantDetection?: boolean;
  enableCrossAppAuth?: boolean;
  appId?: string;
}

/**
 * Enhanced multi-provider authentication provider component
 */
export function EnhancedMultiProviderAuthProvider({
  children,
  backendUrl,
  enableFirebaseFallback = true,
  enableTenantDetection = true,
  enableCrossAppAuth = true,
  appId = 'frontend'
}: EnhancedMultiProviderAuthProviderProps) {
  const router = useRouter();
  const searchParams = useSearchParams();
  
  // State management
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isInitialized, setIsInitialized] = useState(false);
  const [user, setUser] = useState<EnhancedUnifiedUserProfile | null>(null);
  const [firebaseUser, setFirebaseUser] = useState<FirebaseUser | null>(null);
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const [unifiedJWT, setUnifiedJWT] = useState<EnhancedUnifiedJWT | null>(null);
  const [globalSessionId, setGlobalSessionId] = useState<string | null>(null);
  const [currentProvider, setCurrentProvider] = useState<string | null>(null);
  const [currentTenant, setCurrentTenant] = useState<string | null>(null);
  const [availableProviders, setAvailableProviders] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  // Firebase authentication manager (fallback)
  const firebaseAuth = enableFirebaseFallback ? getFirebaseAuth() : null;
  const tokenValidator = enableFirebaseFallback ? getFirebaseTokenValidator() : null;
  
  const baseApiUrl = backendUrl || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
  
  /**
   * Clear any authentication errors
   */
  const clearError = useCallback(() => {
    setError(null);
  }, []);
  
  /**
   * Check if user has specific permission
   */
  const hasPermission = useCallback((permission: string): boolean => {
    return user?.permissions.includes(permission) || false;
  }, [user]);
  
  /**
   * Check if user has specific role
   */
  const hasRole = useCallback((role: string): boolean => {
    return user?.role === role;
  }, [user]);
  
  /**
   * Detect tenant based on email domain
   */
  const detectTenant = useCallback(async (email: string): Promise<TenantDetectionResult | null> => {
    if (!enableTenantDetection) {
      return null;
    }
    
    try {
      const response = await fetch(`${baseApiUrl}/api/v1/auth/detect-tenant`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email }),
      });
      
      if (response.ok) {
        const result = await response.json();
        return {
          tenantId: result.tenant_id,
          providerId: result.provider_id,
          providerType: result.provider_type,
          autoProvision: result.auto_provision,
          defaultRole: result.default_role,
          requiresStepUp: result.requires_step_up || false,
        };
      }
    } catch (error) {
      console.warn('Tenant detection failed:', error);
    }
    
    return null;
  }, [enableTenantDetection, baseApiUrl]);
  
  /**
   * Initiate enhanced OIDC authorization flow
   */
  const initiateOIDCFlow = useCallback(async (
    email?: string,
    tenantHint?: string,
    providerHint?: string
  ) => {
    try {
      const params = new URLSearchParams({
        client_id: appId,
        response_type: 'code',
        redirect_uri: `${window.location.origin}/api/auth/callback`,
        scope: 'openid profile email',
        state: crypto.randomUUID(),
      });
      
      if (email) {
        params.append('email_hint', email);
      }
      
      if (tenantHint) {
        params.append('tenant_hint', tenantHint);
      }
      
      if (providerHint) {
        params.append('provider_hint', providerHint);
      }
      
      // Generate PKCE challenge
      const codeVerifier = generateCodeVerifier();
      const codeChallenge = await generateCodeChallenge(codeVerifier);
      
      params.append('code_challenge', codeChallenge);
      params.append('code_challenge_method', 'S256');
      
      // Store PKCE verifier in session storage
      sessionStorage.setItem('pkce_code_verifier', codeVerifier);
      sessionStorage.setItem('auth_state', params.get('state')!);
      
      const authUrl = `${baseApiUrl}/oauth/v2/authorize?${params.toString()}`;
      window.location.href = authUrl;
      
    } catch (error) {
      console.error('Failed to initiate OIDC flow:', error);
      setError('Authentication initialization failed');
    }
  }, [baseApiUrl, appId]);
  
  /**
   * Handle OAuth callback
   */
  const handleOAuthCallback = useCallback(async () => {
    const code = searchParams.get('code');
    const state = searchParams.get('state');
    const storedState = sessionStorage.getItem('auth_state');
    const codeVerifier = sessionStorage.getItem('pkce_code_verifier');
    
    if (!code || !state || !storedState || !codeVerifier || state !== storedState) {
      setError('Invalid OAuth callback parameters');
      return;
    }
    
    try {
      const tokenResponse = await fetch(`${baseApiUrl}/oauth/v2/token`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: new URLSearchParams({
          grant_type: 'authorization_code',
          client_id: appId,
          code,
          redirect_uri: `${window.location.origin}/api/auth/callback`,
          code_verifier: codeVerifier,
        }),
      });
      
      if (!tokenResponse.ok) {
        throw new Error('Token exchange failed');
      }
      
      const tokenData: EnhancedUnifiedJWT = await tokenResponse.json();
      
      // Get user information
      const userinfoResponse = await fetch(`${baseApiUrl}/oauth/v2/userinfo`, {
        headers: {
          'Authorization': `Bearer ${tokenData.access_token}`,
        },
      });
      
      if (!userinfoResponse.ok) {
        throw new Error('Failed to get user information');
      }
      
      const userInfo = await userinfoResponse.json();
      
      // Create enhanced user profile
      const enhancedUser: EnhancedUnifiedUserProfile = {
        id: userInfo.sub,
        email: userInfo.email,
        displayName: userInfo.name,
        photoURL: userInfo.picture,
        emailVerified: userInfo.email_verified || false,
        provider: mapProviderType(userInfo.provider_type || 'oidc'),
        providerId: userInfo.provider_id || 'unknown',
        providerUserId: userInfo.sub,
        tenantId: userInfo.tenant_id || 'default',
        tenantName: userInfo.tenant_name,
        providerType: userInfo.provider_type || 'oidc',
        role: userInfo.role || 'user',
        permissions: userInfo.permissions || [],
        subscriptionTier: userInfo.subscription_tier,
        sessionId: userInfo.session_id,
        authTime: userInfo.auth_time,
        riskScore: userInfo.risk_score,
        createdAt: new Date().toISOString(),
        lastSignInAt: new Date().toISOString(),
      };
      
      // Update state
      setUser(enhancedUser);
      setAccessToken(tokenData.access_token);
      setUnifiedJWT(tokenData);
      setGlobalSessionId(tokenData.session_id);
      setCurrentProvider(tokenData.provider_id);
      setCurrentTenant(tokenData.tenant_id);
      setIsAuthenticated(true);
      setError(null);
      
      // Clean up
      sessionStorage.removeItem('pkce_code_verifier');
      sessionStorage.removeItem('auth_state');
      
      console.log('Enhanced OAuth authentication successful');
      
    } catch (error) {
      console.error('OAuth callback error:', error);
      setError('Authentication failed');
      setIsAuthenticated(false);
    } finally {
      setIsLoading(false);
    }
  }, [searchParams, baseApiUrl, appId]);
  
  /**
   * Login with tenant hint (email-based tenant detection)
   */
  const loginWithTenantHint = useCallback(async (email: string, tenantId?: string) => {
    setIsLoading(true);
    setError(null);
    
    try {
      let resolvedTenantId = tenantId;
      
      if (!resolvedTenantId && enableTenantDetection) {
        const tenantResult = await detectTenant(email);
        if (tenantResult) {
          resolvedTenantId = tenantResult.tenantId;
        }
      }
      
      await initiateOIDCFlow(email, resolvedTenantId);
      
    } catch (error) {
      setError('Failed to initiate login');
      setIsLoading(false);
    }
  }, [enableTenantDetection, detectTenant, initiateOIDCFlow]);
  
  /**
   * Login with specific provider
   */
  const loginWithProvider = useCallback(async (providerId: string, tenantId?: string) => {
    setIsLoading(true);
    setError(null);
    
    try {
      await initiateOIDCFlow(undefined, tenantId, providerId);
    } catch (error) {
      setError('Failed to initiate provider login');
      setIsLoading(false);
    }
  }, [initiateOIDCFlow]);
  
  /**
   * Login with Google (enhanced)
   */
  const loginWithGoogle = useCallback(async () => {
    await loginWithProvider('google');
  }, [loginWithProvider]);
  
  /**
   * Login with Microsoft (enhanced)
   */
  const loginWithMicrosoft = useCallback(async () => {
    await loginWithProvider('microsoft');
  }, [loginWithProvider]);
  
  /**
   * Login with email and password (enhanced)
   */
  const loginWithCredentials = useCallback(async (email: string, password: string) => {
    setIsLoading(true);
    setError(null);
    
    try {
      // Try enhanced backend first
      const response = await fetch(`${baseApiUrl}/api/v1/auth/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          email,
          password,
          app_type: 'frontend',
          app_id: appId,
        }),
      });
      
      if (response.ok) {
        const authData = await response.json();
        // Handle enhanced auth response
        console.log('Enhanced credential authentication successful');
      } else if (enableFirebaseFallback && firebaseAuth) {
        // Fall back to Firebase
        await firebaseAuth.signInWithCredentials(email, password);
      } else {
        throw new Error('Credential authentication failed');
      }
    } catch (error) {
      const errorMessage = error instanceof FirebaseAuthError 
        ? error.getUserFriendlyMessage()
        : 'Email/password sign-in failed';
      setError(errorMessage);
      console.error('Credential sign-in error:', error);
      setIsLoading(false);
    }
  }, [baseApiUrl, appId, enableFirebaseFallback, firebaseAuth]);
  
  /**
   * Cross-app authentication
   */
  const authenticateForApp = useCallback(async (
    targetApp: string,
    scopes: string[] = ['openid', 'profile', 'email']
  ): Promise<CrossAppAuthResult> => {
    if (!enableCrossAppAuth || !globalSessionId) {
      throw new Error('Cross-app authentication not available');
    }
    
    try {
      // Generate federation token
      const federationResponse = await fetch(`${baseApiUrl}/api/v1/federation/generate-token`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${accessToken}`,
        },
        body: JSON.stringify({
          source_app: appId,
          target_apps: [targetApp],
          global_session_id: globalSessionId,
        }),
      });
      
      if (!federationResponse.ok) {
        throw new Error('Failed to generate federation token');
      }
      
      const { federation_token } = await federationResponse.json();
      
      // Authenticate with target app
      const authResponse = await fetch(`${baseApiUrl}/api/v1/federation/authenticate`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          source_app_id: appId,
          target_app_id: targetApp,
          federation_token,
          requested_scopes: scopes,
        }),
      });
      
      if (!authResponse.ok) {
        throw new Error('Cross-app authentication failed');
      }
      
      const result: CrossAppAuthResult = await authResponse.json();
      
      console.log('Cross-app authentication successful:', targetApp);
      return result;
      
    } catch (error) {
      console.error('Cross-app authentication error:', error);
      throw error;
    }
  }, [enableCrossAppAuth, globalSessionId, baseApiUrl, appId, accessToken]);
  
  /**
   * Switch tenant (if supported)
   */
  const switchTenant = useCallback(async (tenantId: string) => {
    if (!user) {
      throw new Error('No authenticated user');
    }
    
    try {
      // Initiate tenant switch (would require re-authentication)
      await loginWithTenantHint(user.email!, tenantId);
    } catch (error) {
      console.error('Tenant switch error:', error);
      throw error;
    }
  }, [user, loginWithTenantHint]);
  
  /**
   * Create new account
   */
  const createAccount = useCallback(async (email: string, password: string) => {
    setIsLoading(true);
    setError(null);
    
    try {
      // Try enhanced backend first
      const response = await fetch(`${baseApiUrl}/api/v1/auth/register`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          email,
          password,
          app_id: appId,
        }),
      });
      
      if (response.ok) {
        console.log('Enhanced account creation successful');
      } else if (enableFirebaseFallback && firebaseAuth) {
        await firebaseAuth.createAccount(email, password);
      } else {
        throw new Error('Account creation failed');
      }
    } catch (error) {
      const errorMessage = error instanceof FirebaseAuthError 
        ? error.getUserFriendlyMessage()
        : 'Account creation failed';
      setError(errorMessage);
      setIsLoading(false);
    }
  }, [baseApiUrl, appId, enableFirebaseFallback, firebaseAuth]);
  
  /**
   * Sign out current user
   */
  const logout = useCallback(async (logoutFromAllApps: boolean = false) => {
    setIsLoading(true);
    setError(null);
    
    try {
      if (logoutFromAllApps && globalSessionId) {
        // Terminate federated session
        await fetch(`${baseApiUrl}/api/v1/federation/terminate-session`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${accessToken}`,
          },
          body: JSON.stringify({
            global_session_id: globalSessionId,
          }),
        });
      }
      
      // Firebase logout if applicable
      if (currentProvider === 'firebase' && firebaseAuth) {
        await firebaseAuth.signOut();
      }
      
      // Clear local state
      setIsAuthenticated(false);
      setUser(null);
      setFirebaseUser(null);
      setAccessToken(null);
      setUnifiedJWT(null);
      setGlobalSessionId(null);
      setCurrentProvider(null);
      setCurrentTenant(null);
      
      console.log('Logout successful');
    } catch (error) {
      console.error('Logout error:', error);
      setError('Logout failed');
    } finally {
      setIsLoading(false);
    }
  }, [globalSessionId, baseApiUrl, accessToken, currentProvider, firebaseAuth]);
  
  /**
   * Refresh authentication token
   */
  const refreshToken = useCallback(async (): Promise<boolean> => {
    if (!unifiedJWT?.refresh_token) {
      return false;
    }
    
    try {
      const response = await fetch(`${baseApiUrl}/oauth/v2/token`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: new URLSearchParams({
          grant_type: 'refresh_token',
          client_id: appId,
          refresh_token: unifiedJWT.refresh_token,
        }),
      });
      
      if (response.ok) {
        const newToken: EnhancedUnifiedJWT = await response.json();
        setUnifiedJWT(newToken);
        setAccessToken(newToken.access_token);
        return true;
      }
    } catch (error) {
      console.error('Token refresh failed:', error);
    }
    
    return false;
  }, [unifiedJWT, baseApiUrl, appId]);
  
  /**
   * Get valid access token (refresh if needed)
   */
  const getValidToken = useCallback(async (): Promise<string | null> => {
    if (!unifiedJWT) {
      return null;
    }
    
    const now = new Date();
    const expiresAt = new Date(unifiedJWT.expires_at);
    const minutesUntilExpiry = (expiresAt.getTime() - now.getTime()) / (1000 * 60);
    
    if (minutesUntilExpiry <= 2) {
      // Token needs refresh
      const refreshed = await refreshToken();
      if (!refreshed) {
        return null;
      }
    }
    
    return unifiedJWT.access_token;
  }, [unifiedJWT, refreshToken]);
  
  // Initialize authentication
  useEffect(() => {
    const initializeAuth = async () => {
      try {
        // Check for OAuth callback
        if (searchParams.has('code')) {
          await handleOAuthCallback();
          return;
        }
        
        // Check for existing session
        const storedToken = localStorage.getItem('enhanced_auth_token');
        if (storedToken) {
          try {
            const tokenData: EnhancedUnifiedJWT = JSON.parse(storedToken);
            const now = new Date();
            const expiresAt = new Date(tokenData.expires_at);
            
            if (expiresAt > now) {
              // Token is still valid, restore session
              setUnifiedJWT(tokenData);
              setAccessToken(tokenData.access_token);
              setGlobalSessionId(tokenData.session_id);
              setCurrentProvider(tokenData.provider_id);
              setCurrentTenant(tokenData.tenant_id);
              // TODO: Restore user info from token or fetch from backend
            }
          } catch (e) {
            localStorage.removeItem('enhanced_auth_token');
          }
        }
        
        // Firebase fallback initialization
        if (enableFirebaseFallback && firebaseAuth) {
          // Set up Firebase auth state listener as fallback
          const unsubscribe = firebaseAuth.onAuthStateChanged((firebaseUser) => {
            setFirebaseUser(firebaseUser);
            if (!isAuthenticated && firebaseUser) {
              // Handle Firebase fallback authentication
              console.log('Firebase fallback authentication detected');
            }
          });
          
          return () => unsubscribe();
        }
        
        setIsInitialized(true);
      } catch (error) {
        console.error('Failed to initialize enhanced authentication:', error);
        setError('Authentication initialization failed');
      } finally {
        setIsLoading(false);
      }
    };
    
    initializeAuth();
  }, [searchParams, handleOAuthCallback, enableFirebaseFallback, firebaseAuth, isAuthenticated]);
  
  // Persist token to localStorage
  useEffect(() => {
    if (unifiedJWT) {
      localStorage.setItem('enhanced_auth_token', JSON.stringify(unifiedJWT));
    } else {
      localStorage.removeItem('enhanced_auth_token');
    }
  }, [unifiedJWT]);
  
  // Context value
  const value: EnhancedMultiProviderAuthState = {
    // State
    isAuthenticated,
    isLoading,
    isInitialized,
    user,
    firebaseUser,
    accessToken,
    unifiedJWT,
    globalSessionId,
    currentProvider,
    currentTenant,
    availableProviders,
    error,
    
    // Methods
    loginWithGoogle,
    loginWithMicrosoft,
    loginWithCredentials,
    loginWithTenantHint,
    loginWithProvider,
    detectTenant,
    switchTenant,
    authenticateForApp,
    createAccount,
    logout,
    refreshToken,
    getValidToken,
    clearError,
    hasPermission,
    hasRole,
  };
  
  return (
    <EnhancedMultiProviderAuthContext.Provider value={value}>
      {children}
    </EnhancedMultiProviderAuthContext.Provider>
  );
}

/**
 * Hook to use enhanced multi-provider authentication context
 */
export function useEnhancedMultiProviderAuth(): EnhancedMultiProviderAuthState {
  const context = useContext(EnhancedMultiProviderAuthContext);
  
  if (context === undefined) {
    throw new Error('useEnhancedMultiProviderAuth must be used within EnhancedMultiProviderAuthProvider');
  }
  
  return context;
}

/**
 * Hook to get current user with enhanced type safety
 */
export function useEnhancedCurrentUser(): EnhancedUnifiedUserProfile | null {
  const { user, isAuthenticated } = useEnhancedMultiProviderAuth();
  return isAuthenticated ? user : null;
}

/**
 * Hook to get tenant information
 */
export function useTenantInfo(): {
  tenantId: string | null;
  tenantName: string | null;
  canSwitchTenant: boolean;
  switchTenant: (tenantId: string) => Promise<void>;
} {
  const { user, currentTenant, switchTenant } = useEnhancedMultiProviderAuth();
  
  return {
    tenantId: currentTenant,
    tenantName: user?.tenantName || null,
    canSwitchTenant: true, // TODO: Check permissions
    switchTenant,
  };
}

/**
 * Hook for cross-app authentication
 */
export function useCrossAppAuth(): {
  authenticateForApp: (targetApp: string, scopes?: string[]) => Promise<CrossAppAuthResult>;
  globalSessionId: string | null;
} {
  const { authenticateForApp, globalSessionId } = useEnhancedMultiProviderAuth();
  
  return {
    authenticateForApp,
    globalSessionId,
  };
}

// Helper functions

function generateCodeVerifier(): string {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return base64URLEncode(array);
}

async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const digest = await crypto.subtle.digest('SHA-256', data);
  return base64URLEncode(new Uint8Array(digest));
}

function base64URLEncode(array: Uint8Array): string {
  return btoa(String.fromCharCode(...array))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

function mapProviderType(providerType: string): EnhancedUnifiedUserProfile['provider'] {
  switch (providerType.toLowerCase()) {
    case 'google':
      return 'google';
    case 'microsoft':
      return 'microsoft';
    case 'auth0':
      return 'auth0';
    case 'firebase':
      return 'firebase';
    case 'enterprise':
      return 'enterprise';
    default:
      return 'oidc';
  }
}

export default EnhancedMultiProviderAuthProvider;