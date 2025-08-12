'use client';

// Enhanced OIDC Authentication Context
// Sophisticated state management with real-time sync and adaptive security

import React, { 
  createContext, useContext, useState, useEffect, useCallback, 
  ReactNode, useRef, useMemo 
} from 'react';
import { useRouter } from 'next/navigation';
import { User } from 'oidc-client-ts';
import { 
  getOIDCClient, 
  type AuthenticationState, 
  type HealthMetrics,
  type AuthenticationMethod,
  type AuthenticationRisk
} from '@/lib/auth/oidc-client-wrapper';
import { getTenantDetectionService, type TenantInfo } from '@/lib/auth/tenant-detection-service';
import { getCurrentOIDCUser, logoutOIDC } from '@/app/actions/oidc-auth';

interface EnhancedUser extends User {
  tenant?: TenantInfo;
  permissions?: string[];
  subscriptionTier?: string;
  lastActivity?: number;
  riskLevel?: AuthenticationRisk;
}

interface AuthContextState extends AuthenticationState {
  user: EnhancedUser | null;
  tenant: TenantInfo | null;
  permissions: string[];
  subscriptionTier: string | null;
  riskLevel: AuthenticationRisk;
  sessionTimeRemaining: number;
  multiFactorEnabled: boolean;
  biometricEnabled: boolean;
  deviceTrusted: boolean;
}

interface AuthContextMethods {
  // Authentication methods
  signIn: (email?: string, method?: AuthenticationMethod) => Promise<User | void>;
  signInWithPopup: (email?: string) => Promise<User>;
  signInSilent: () => Promise<User | null>;
  signOut: () => Promise<void>;
  refreshSession: () => Promise<void>;
  
  // Session management
  extendSession: () => Promise<void>;
  checkSessionHealth: () => Promise<boolean>;
  validateSession: () => Promise<boolean>;
  
  // Tenant management
  switchTenant: (tenant: TenantInfo) => Promise<void>;
  detectTenant: (email: string) => Promise<TenantInfo | null>;
  
  // Security operations
  enableMFA: () => Promise<void>;
  disableMFA: () => Promise<void>;
  enableBiometric: () => Promise<void>;
  disableBiometric: () => Promise<void>;
  trustDevice: () => Promise<void>;
  
  // Permission checks
  hasPermission: (permission: string) => boolean;
  hasRole: (role: string) => boolean;
  canAccess: (resource: string, action?: string) => boolean;
  
  // Subscription checks
  hasFeature: (feature: string) => boolean;
  canUseFeature: (feature: string) => boolean;
  getFeatureLimit: (feature: string) => number | null;
}

interface AuthConfig {
  enableAutoRefresh?: boolean;
  enableSessionMonitoring?: boolean;
  enableRiskAssessment?: boolean;
  enableBiometric?: boolean;
  sessionTimeoutWarning?: number; // minutes before timeout
  maxInactivityTime?: number; // minutes
  tenantDetectionEnabled?: boolean;
  adaptiveAuthEnabled?: boolean;
}

interface AuthProviderProps {
  children: ReactNode;
  config?: AuthConfig;
}

type AuthContextType = AuthContextState & AuthContextMethods & {
  healthMetrics: HealthMetrics;
  config: Required<AuthConfig>;
};

const defaultConfig: Required<AuthConfig> = {
  enableAutoRefresh: true,
  enableSessionMonitoring: true,
  enableRiskAssessment: true,
  enableBiometric: true,
  sessionTimeoutWarning: 5, // 5 minutes
  maxInactivityTime: 30, // 30 minutes
  tenantDetectionEnabled: true,
  adaptiveAuthEnabled: true
};

const AuthContext = createContext<AuthContextType | null>(null);

export function EnhancedOIDCAuthProvider({ children, config = {} }: AuthProviderProps) {
  const router = useRouter();
  const oidcClient = getOIDCClient();
  const tenantService = getTenantDetectionService();
  const finalConfig = { ...defaultConfig, ...config };
  
  // Refs for intervals and timeouts
  const refreshTimerRef = useRef<NodeJS.Timeout>();
  const sessionMonitorRef = useRef<NodeJS.Timeout>();
  const healthCheckRef = useRef<NodeJS.Timeout>();
  const inactivityTimerRef = useRef<NodeJS.Timeout>();

  // Core authentication state
  const [authState, setAuthState] = useState<AuthContextState>({
    isLoading: true,
    isAuthenticated: false,
    user: null,
    error: null,
    tenant: null,
    healthStatus: 'healthy',
    lastActivity: Date.now(),
    sessionTimeoutWarning: false,
    permissions: [],
    subscriptionTier: null,
    riskLevel: 'low',
    sessionTimeRemaining: 0,
    multiFactorEnabled: false,
    biometricEnabled: false,
    deviceTrusted: false
  });

  const [healthMetrics, setHealthMetrics] = useState<HealthMetrics>({
    totalRequests: 0,
    successfulRequests: 0,
    failedRequests: 0,
    averageResponseTime: 0,
    lastHealthCheck: Date.now(),
    uptime: Date.now()
  });

  // Initialize authentication system
  useEffect(() => {
    initializeAuth();
    
    return () => {
      cleanup();
    };
  }, []);

  // Session monitoring
  useEffect(() => {
    if (finalConfig.enableSessionMonitoring) {
      startSessionMonitoring();
    }
    
    return () => {
      if (sessionMonitorRef.current) {
        clearInterval(sessionMonitorRef.current);
      }
    };
  }, [finalConfig.enableSessionMonitoring, authState.isAuthenticated]);

  // Auto-refresh token
  useEffect(() => {
    if (finalConfig.enableAutoRefresh && authState.isAuthenticated) {
      startTokenRefresh();
    }
    
    return () => {
      if (refreshTimerRef.current) {
        clearInterval(refreshTimerRef.current);
      }
    };
  }, [finalConfig.enableAutoRefresh, authState.isAuthenticated]);

  // Health monitoring
  useEffect(() => {
    startHealthMonitoring();
    
    return () => {
      if (healthCheckRef.current) {
        clearInterval(healthCheckRef.current);
      }
    };
  }, []);

  // Initialize authentication system
  const initializeAuth = async () => {
    try {
      console.log('🚀 Initializing enhanced OIDC auth context...');
      
      // Initialize OIDC client
      await oidcClient.initialize();
      
      // Check for existing session
      const existingUser = await getCurrentOIDCUser();
      
      if (existingUser) {
        const enhancedUser = await enhanceUserData(existingUser);
        setAuthState(prev => ({
          ...prev,
          isLoading: false,
          isAuthenticated: true,
          user: enhancedUser,
          permissions: enhancedUser.permissions || [],
          subscriptionTier: enhancedUser.subscriptionTier || null,
          deviceTrusted: checkDeviceTrust(),
          lastActivity: Date.now()
        }));
        
        console.log('✅ Existing session restored:', enhancedUser.profile?.email);
      } else {
        setAuthState(prev => ({ ...prev, isLoading: false }));
      }
      
      // Set up OIDC client event listeners
      setupEventListeners();
      
    } catch (error) {
      console.error('❌ Auth initialization failed:', error);
      setAuthState(prev => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Initialization failed'
      }));
    }
  };

  // Set up OIDC client event listeners
  const setupEventListeners = () => {
    oidcClient.on('user_loaded', async (data: { user: User }) => {
      const enhancedUser = await enhanceUserData(data.user);
      setAuthState(prev => ({
        ...prev,
        isAuthenticated: true,
        user: enhancedUser,
        permissions: enhancedUser.permissions || [],
        subscriptionTier: enhancedUser.subscriptionTier || null,
        lastActivity: Date.now()
      }));
    });

    oidcClient.on('user_unloaded', () => {
      setAuthState(prev => ({
        ...prev,
        isAuthenticated: false,
        user: null,
        permissions: [],
        subscriptionTier: null
      }));
    });

    oidcClient.on('silent_renew_error', (data: { error: Error }) => {
      console.warn('🔇 Silent renew failed:', data.error);
      setAuthState(prev => ({
        ...prev,
        error: 'Session refresh failed',
        healthStatus: 'degraded'
      }));
    });

    oidcClient.on('session_timeout_warning', (data: { remainingTime: number }) => {
      setAuthState(prev => ({
        ...prev,
        sessionTimeoutWarning: true,
        sessionTimeRemaining: data.remainingTime
      }));
    });

    oidcClient.on('health_check', (data: { metrics: HealthMetrics }) => {
      setHealthMetrics(data.metrics);
    });

    oidcClient.on('state_changed', (data: { state: AuthenticationState }) => {
      setAuthState(prev => ({
        ...prev,
        ...data.state
      }));
    });
  };

  // Enhance user data with additional information
  const enhanceUserData = async (user: User): Promise<EnhancedUser> => {
    try {
      const enhancedUser: EnhancedUser = {
        ...user,
        permissions: user.profile.permissions || [],
        subscriptionTier: user.profile.subscription_tier,
        lastActivity: Date.now(),
        riskLevel: assessUserRisk(user)
      };

      // Detect tenant if enabled
      if (finalConfig.tenantDetectionEnabled && user.profile.email) {
        try {
          const detection = await tenantService.detectTenant(user.profile.email);
          if (detection.tenant) {
            enhancedUser.tenant = detection.tenant;
          }
        } catch (error) {
          console.debug('Tenant detection failed:', error);
        }
      }

      return enhancedUser;
      
    } catch (error) {
      console.error('User enhancement failed:', error);
      return user;
    }
  };

  // Assess user risk level
  const assessUserRisk = (user: User): AuthenticationRisk => {
    if (!finalConfig.enableRiskAssessment) return 'low';
    
    let riskScore = 0;
    
    // Check for admin privileges
    if (user.profile.role?.includes('admin')) riskScore += 20;
    
    // Check device trust
    if (!checkDeviceTrust()) riskScore += 15;
    
    // Check session age
    const sessionAge = Date.now() - (user.profile.iat * 1000);
    if (sessionAge > 4 * 60 * 60 * 1000) riskScore += 10; // Over 4 hours
    
    // Check for unusual access patterns
    if (isUnusualAccessPattern()) riskScore += 25;
    
    if (riskScore >= 50) return 'critical';
    if (riskScore >= 35) return 'high';
    if (riskScore >= 20) return 'medium';
    return 'low';
  };

  // Check device trust status
  const checkDeviceTrust = (): boolean => {
    try {
      const trustedDevices = JSON.parse(localStorage.getItem('trusted_devices') || '[]');
      const deviceId = generateDeviceId();
      return trustedDevices.includes(deviceId);
    } catch {
      return false;
    }
  };

  // Check for unusual access patterns
  const isUnusualAccessPattern = (): boolean => {
    // Simplified implementation - would use more sophisticated analysis in production
    const hour = new Date().getHours();
    return hour < 6 || hour > 22; // Outside business hours
  };

  // Generate device ID
  const generateDeviceId = (): string => {
    return btoa([
      navigator.userAgent,
      screen.width + 'x' + screen.height,
      new Date().getTimezoneOffset()
    ].join('|')).substring(0, 32);
  };

  // Start session monitoring
  const startSessionMonitoring = () => {
    sessionMonitorRef.current = setInterval(() => {
      if (!authState.isAuthenticated) return;
      
      const inactiveTime = Date.now() - authState.lastActivity;
      const warningThreshold = finalConfig.sessionTimeoutWarning * 60 * 1000;
      const timeoutThreshold = finalConfig.maxInactivityTime * 60 * 1000;
      
      if (inactiveTime > timeoutThreshold) {
        console.log('🔒 Session timeout due to inactivity');
        signOut();
      } else if (inactiveTime > warningThreshold && !authState.sessionTimeoutWarning) {
        setAuthState(prev => ({
          ...prev,
          sessionTimeoutWarning: true,
          sessionTimeRemaining: timeoutThreshold - inactiveTime
        }));
      }
    }, 30000); // Check every 30 seconds
  };

  // Start token refresh monitoring
  const startTokenRefresh = () => {
    refreshTimerRef.current = setInterval(async () => {
      if (!authState.isAuthenticated || !authState.user) return;
      
      try {
        // Check if token needs refresh (within 5 minutes of expiry)
        const expiryTime = authState.user.expires_at * 1000;
        const now = Date.now();
        const timeToExpiry = expiryTime - now;
        
        if (timeToExpiry < 5 * 60 * 1000) { // 5 minutes
          console.log('🔄 Auto-refreshing token...');
          await refreshSession();
        }
      } catch (error) {
        console.error('❌ Auto-refresh failed:', error);
        setAuthState(prev => ({
          ...prev,
          error: 'Session refresh failed',
          healthStatus: 'degraded'
        }));
      }
    }, 60000); // Check every minute
  };

  // Start health monitoring
  const startHealthMonitoring = () => {
    healthCheckRef.current = setInterval(() => {
      const clientMetrics = oidcClient.getHealthMetrics();
      setHealthMetrics(clientMetrics);
      
      // Update health status based on metrics
      const successRate = clientMetrics.totalRequests > 0 ? 
        clientMetrics.successfulRequests / clientMetrics.totalRequests : 1;
      
      let healthStatus: 'healthy' | 'degraded' | 'unhealthy' = 'healthy';
      if (successRate < 0.5) healthStatus = 'unhealthy';
      else if (successRate < 0.9) healthStatus = 'degraded';
      
      setAuthState(prev => ({ ...prev, healthStatus }));
    }, 30000); // Every 30 seconds
  };

  // Update last activity
  const updateActivity = useCallback(() => {
    setAuthState(prev => ({ ...prev, lastActivity: Date.now() }));
    
    // Clear timeout warning if user is active
    if (authState.sessionTimeoutWarning) {
      setAuthState(prev => ({ ...prev, sessionTimeoutWarning: false }));
    }
  }, [authState.sessionTimeoutWarning]);

  // Authentication methods
  const signIn = useCallback(async (email?: string, method: AuthenticationMethod = 'redirect') => {
    try {
      updateActivity();
      
      if (method === 'popup') {
        return await oidcClient.signInPopup(email);
      } else if (method === 'silent') {
        return await oidcClient.signInSilent();
      } else {
        await oidcClient.signInRedirect(email);
      }
    } catch (error) {
      setAuthState(prev => ({
        ...prev,
        error: error instanceof Error ? error.message : 'Sign in failed'
      }));
      throw error;
    }
  }, [updateActivity]);

  const signInWithPopup = useCallback(async (email?: string) => {
    return oidcClient.signInPopup(email);
  }, []);

  const signInSilent = useCallback(async () => {
    return oidcClient.signInSilent();
  }, []);

  const signOut = useCallback(async () => {
    try {
      await logoutOIDC();
      setAuthState({
        isLoading: false,
        isAuthenticated: false,
        user: null,
        error: null,
        tenant: null,
        healthStatus: 'healthy',
        lastActivity: Date.now(),
        sessionTimeoutWarning: false,
        permissions: [],
        subscriptionTier: null,
        riskLevel: 'low',
        sessionTimeRemaining: 0,
        multiFactorEnabled: false,
        biometricEnabled: false,
        deviceTrusted: false
      });
    } catch (error) {
      console.error('Sign out error:', error);
      setAuthState(prev => ({
        ...prev,
        error: error instanceof Error ? error.message : 'Sign out failed'
      }));
    }
  }, []);

  const refreshSession = useCallback(async () => {
    try {
      const user = await getCurrentOIDCUser();
      if (user) {
        const enhancedUser = await enhanceUserData(user);
        setAuthState(prev => ({
          ...prev,
          user: enhancedUser,
          permissions: enhancedUser.permissions || [],
          subscriptionTier: enhancedUser.subscriptionTier || null,
          lastActivity: Date.now()
        }));
      }
    } catch (error) {
      console.error('Session refresh failed:', error);
      throw error;
    }
  }, []);

  // Permission and access control methods
  const hasPermission = useCallback((permission: string): boolean => {
    return authState.permissions.includes(permission) || 
           authState.permissions.includes('*') ||
           authState.user?.profile.role === 'super_admin';
  }, [authState.permissions, authState.user]);

  const hasRole = useCallback((role: string): boolean => {
    return authState.user?.profile.role === role;
  }, [authState.user]);

  const canAccess = useCallback((resource: string, action?: string): boolean => {
    if (!authState.isAuthenticated) return false;
    
    const permission = action ? `${resource}:${action}` : resource;
    return hasPermission(permission);
  }, [authState.isAuthenticated, hasPermission]);

  const hasFeature = useCallback((feature: string): boolean => {
    if (!authState.subscriptionTier) return false;
    
    // Feature mapping based on subscription tier
    const featureMap: Record<string, string[]> = {
      'user-basic-001': ['basic_analytics', 'basic_reports'],
      'user-premium-002': ['basic_analytics', 'basic_reports', 'advanced_analytics', 'custom_alerts'],
      'admin-full-004': ['*'] // All features
    };
    
    const availableFeatures = featureMap[authState.subscriptionTier] || [];
    return availableFeatures.includes('*') || availableFeatures.includes(feature);
  }, [authState.subscriptionTier]);

  const canUseFeature = useCallback((feature: string): boolean => {
    return hasFeature(feature) && authState.healthStatus !== 'unhealthy';
  }, [hasFeature, authState.healthStatus]);

  const getFeatureLimit = useCallback((feature: string): number | null => {
    if (!hasFeature(feature)) return 0;
    
    // Feature limits based on subscription tier
    const limits: Record<string, Record<string, number>> = {
      'user-basic-001': { api_calls: 1000, reports: 5 },
      'user-premium-002': { api_calls: 10000, reports: 50 }
    };
    
    return limits[authState.subscriptionTier || '']?.[feature] || null;
  }, [hasFeature, authState.subscriptionTier]);

  // Additional methods (simplified implementations)
  const extendSession = useCallback(async () => {
    updateActivity();
    await refreshSession();
  }, [updateActivity, refreshSession]);

  const checkSessionHealth = useCallback(async (): Promise<boolean> => {
    return authState.healthStatus === 'healthy';
  }, [authState.healthStatus]);

  const validateSession = useCallback(async (): Promise<boolean> => {
    try {
      const user = await getCurrentOIDCUser();
      return !!user;
    } catch {
      return false;
    }
  }, []);

  const switchTenant = useCallback(async (tenant: TenantInfo) => {
    await oidcClient.switchTenant(tenant);
    setAuthState(prev => ({ ...prev, tenant }));
  }, []);

  const detectTenant = useCallback(async (email: string): Promise<TenantInfo | null> => {
    const detection = await tenantService.detectTenant(email);
    return detection.tenant;
  }, []);

  // Placeholder implementations for security features
  const enableMFA = useCallback(async () => {
    setAuthState(prev => ({ ...prev, multiFactorEnabled: true }));
  }, []);

  const disableMFA = useCallback(async () => {
    setAuthState(prev => ({ ...prev, multiFactorEnabled: false }));
  }, []);

  const enableBiometric = useCallback(async () => {
    if (finalConfig.enableBiometric && 'PublicKeyCredential' in window) {
      setAuthState(prev => ({ ...prev, biometricEnabled: true }));
    }
  }, [finalConfig.enableBiometric]);

  const disableBiometric = useCallback(async () => {
    setAuthState(prev => ({ ...prev, biometricEnabled: false }));
  }, []);

  const trustDevice = useCallback(async () => {
    try {
      const deviceId = generateDeviceId();
      const trustedDevices = JSON.parse(localStorage.getItem('trusted_devices') || '[]');
      if (!trustedDevices.includes(deviceId)) {
        trustedDevices.push(deviceId);
        localStorage.setItem('trusted_devices', JSON.stringify(trustedDevices));
      }
      setAuthState(prev => ({ ...prev, deviceTrusted: true }));
    } catch (error) {
      console.error('Failed to trust device:', error);
    }
  }, []);

  // Cleanup function
  const cleanup = () => {
    [refreshTimerRef, sessionMonitorRef, healthCheckRef, inactivityTimerRef].forEach(ref => {
      if (ref.current) clearInterval(ref.current);
    });
  };

  // Context value
  const contextValue: AuthContextType = useMemo(() => ({
    // State
    ...authState,
    healthMetrics,
    config: finalConfig,

    // Authentication methods
    signIn,
    signInWithPopup,
    signInSilent,
    signOut,
    refreshSession,

    // Session management
    extendSession,
    checkSessionHealth,
    validateSession,

    // Tenant management
    switchTenant,
    detectTenant,

    // Security operations
    enableMFA,
    disableMFA,
    enableBiometric,
    disableBiometric,
    trustDevice,

    // Permission checks
    hasPermission,
    hasRole,
    canAccess,

    // Subscription checks
    hasFeature,
    canUseFeature,
    getFeatureLimit
  }), [
    authState,
    healthMetrics,
    finalConfig,
    signIn,
    signInWithPopup,
    signInSilent,
    signOut,
    refreshSession,
    extendSession,
    checkSessionHealth,
    validateSession,
    switchTenant,
    detectTenant,
    enableMFA,
    disableMFA,
    enableBiometric,
    disableBiometric,
    trustDevice,
    hasPermission,
    hasRole,
    canAccess,
    hasFeature,
    canUseFeature,
    getFeatureLimit
  ]);

  return (
    <AuthContext.Provider value={contextValue}>
      {children}
    </AuthContext.Provider>
  );
}

// Hook to use auth context
export function useAuth(): AuthContextType {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an EnhancedOIDCAuthProvider');
  }
  return context;
}

// Hook for permission checks
export function usePermissions() {
  const { hasPermission, hasRole, canAccess, permissions } = useAuth();
  return { hasPermission, hasRole, canAccess, permissions };
}

// Hook for subscription features
export function useFeatures() {
  const { hasFeature, canUseFeature, getFeatureLimit, subscriptionTier } = useAuth();
  return { hasFeature, canUseFeature, getFeatureLimit, subscriptionTier };
}

// Hook for auth state
export function useAuthState() {
  const { isLoading, isAuthenticated, user, error, healthStatus } = useAuth();
  return { isLoading, isAuthenticated, user, error, healthStatus };
}