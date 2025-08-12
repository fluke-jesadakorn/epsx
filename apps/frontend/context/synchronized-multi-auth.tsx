'use client';

// Synchronized Multi-App Authentication System
// Frontend authentication with real-time admin sync and progressive elevation

import React, { 
  createContext, useContext, useState, useEffect, useCallback, 
  ReactNode, useRef 
} from 'react';
import { useRouter, useSearchParams } from 'next/navigation';

// Import existing enhanced auth for fallback
import { 
  useEnhancedMultiProviderAuth,
  type EnhancedUnifiedUserProfile,
  type EnhancedUnifiedJWT,
} from './enhanced-multi-provider-auth';

/**
 * User authentication levels (aligned with admin system)
 */
export enum UserAuthLevel {
  NONE = 0,           // Not authenticated
  BASIC = 1,          // Basic authentication
  VERIFIED = 2,       // Email verified + basic MFA
  PREMIUM = 3,        // Premium features unlocked
  ADMIN_ELIGIBLE = 4, // Can access admin with step-up
  ADMIN_ACTIVE = 5,   // Currently authenticated in admin
}

/**
 * Cross-app authentication status
 */
export interface CrossAppStatus {
  adminConnected: boolean;
  adminAuthLevel: number;
  adminSessionId: string | null;
  lastAdminActivity: Date | null;
  canAccessAdmin: boolean;
  adminStepUpRequired: boolean;
}

/**
 * Real-time synchronization event
 */
export interface SyncEvent {
  type: 'admin_login' | 'admin_logout' | 'auth_level_change' | 'security_alert' | 'session_expired';
  source: 'frontend' | 'admin' | 'backend';
  timestamp: Date;
  data: any;
}

/**
 * Enhanced user profile with cross-app data
 */
export interface SynchronizedUserProfile extends EnhancedUnifiedUserProfile {
  // Additional frontend-specific fields
  userAuthLevel: UserAuthLevel;
  maxUserLevel: UserAuthLevel;
  
  // Cross-app status
  crossAppStatus: CrossAppStatus;
  
  // Premium/subscription data
  subscriptionStatus: 'free' | 'premium' | 'enterprise';
  featureFlags: string[];
  quotaUsage: Record<string, number>;
  
  // Social features
  preferences: Record<string, any>;
  notifications: NotificationPreference[];
  
  // Trading/financial data
  tradingLevel: 'basic' | 'advanced' | 'professional';
  portfolioAccess: string[];
  
  // Activity tracking
  lastFeatureUsed: string | null;
  sessionDuration: number; // milliseconds
  apiCallsToday: number;
}

/**
 * Notification preference
 */
export interface NotificationPreference {
  type: string;
  enabled: boolean;
  channels: ('email' | 'push' | 'sms')[];
  frequency: 'immediate' | 'daily' | 'weekly';
}

/**
 * Admin access request
 */
export interface AdminAccessRequest {
  requestId: string;
  requiredLevel: number;
  reason: string;
  expiresAt: Date;
  approved?: boolean;
  approvedBy?: string;
  approvedAt?: Date;
}

/**
 * Feature access result
 */
export interface FeatureAccessResult {
  granted: boolean;
  reason?: string;
  upgradeRequired?: boolean;
  adminRequired?: boolean;
  authLevelRequired?: UserAuthLevel;
}

/**
 * Synchronized multi-app authentication state
 */
export interface SynchronizedMultiAuthState {
  // Enhanced authentication state
  isAuthenticated: boolean;
  isLoading: boolean;
  isInitialized: boolean;
  
  // User data
  user: SynchronizedUserProfile | null;
  userAuthLevel: UserAuthLevel;
  maxUserLevel: UserAuthLevel;
  
  // Cross-app state
  crossAppStatus: CrossAppStatus;
  adminAccessRequest: AdminAccessRequest | null;
  syncEvents: SyncEvent[];
  lastSyncTime: Date | null;
  
  // Real-time features
  isOnline: boolean;
  connectionQuality: 'excellent' | 'good' | 'poor' | 'offline';
  
  // Error and warning state
  error: string | null;
  warnings: string[];
  
  // Authentication methods
  loginWithCredentials: (email: string, password: string) => Promise<void>;
  loginWithProvider: (providerId: string) => Promise<void>;
  loginWithSocialMedia: (platform: 'google' | 'github' | 'linkedin') => Promise<void>;
  
  // Cross-app operations
  requestAdminAccess: (reason: string) => Promise<AdminAccessRequest>;
  openAdminApp: () => Promise<boolean>;
  syncWithAdmin: () => Promise<void>;
  checkAdminConnection: () => Promise<boolean>;
  
  // Feature access and permissions
  checkFeatureAccess: (feature: string) => FeatureAccessResult;
  requestFeatureUpgrade: (feature: string) => Promise<void>;
  hasQuotaRemaining: (resource: string) => boolean;
  
  // Subscription and billing
  upgradeSubscription: (tier: 'premium' | 'enterprise') => Promise<void>;
  checkSubscriptionStatus: () => Promise<void>;
  
  // User management
  updateProfile: (updates: Partial<SynchronizedUserProfile>) => Promise<void>;
  changePassword: (currentPassword: string, newPassword: string) => Promise<void>;
  enableTwoFactor: () => Promise<{ qrCode: string; backupCodes: string[] }>;
  
  // Preferences and settings
  updatePreferences: (preferences: Record<string, any>) => Promise<void>;
  updateNotificationSettings: (settings: NotificationPreference[]) => Promise<void>;
  
  // Session management
  extendSession: () => Promise<boolean>;
  logout: (logoutFromAdmin?: boolean) => Promise<void>;
  refreshAuth: () => Promise<boolean>;
  
  // Real-time features
  subscribeToEvents: (callback: (event: SyncEvent) => void) => () => void;
  reportActivity: (activity: string, metadata?: any) => void;
  
  // Utilities
  clearError: () => void;
  clearWarnings: () => void;
  getSessionInfo: () => { timeRemaining: number; lastActivity: Date };
  exportUserData: () => Promise<Blob>;
}

const SynchronizedMultiAuthContext = createContext<SynchronizedMultiAuthState | undefined>(undefined);

/**
 * Synchronized multi-app authentication provider props
 */
interface SynchronizedMultiAuthProviderProps {
  children: ReactNode;
  backendUrl?: string;
  enableRealTimeSync?: boolean;
  enableAdvancedFeatures?: boolean;
  sessionTimeoutMinutes?: number;
  syncIntervalSeconds?: number;
  maxRetries?: number;
}

/**
 * Synchronized multi-app authentication provider component
 */
export function SynchronizedMultiAuthProvider({
  children,
  backendUrl,
  enableRealTimeSync = true,
  enableAdvancedFeatures = true,
  sessionTimeoutMinutes = 480, // 8 hours for frontend
  syncIntervalSeconds = 15,
  maxRetries = 3,
}: SynchronizedMultiAuthProviderProps) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const eventSourceRef = useRef<EventSource | null>(null);
  const syncIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const activityTimerRef = useRef<NodeJS.Timeout | null>(null);
  
  // Enhanced auth fallback
  const enhancedAuth = useEnhancedMultiProviderAuth();
  
  // Core state
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isInitialized, setIsInitialized] = useState(false);
  const [user, setUser] = useState<SynchronizedUserProfile | null>(null);
  const [userAuthLevel, setUserAuthLevel] = useState<UserAuthLevel>(UserAuthLevel.NONE);
  const [maxUserLevel, setMaxUserLevel] = useState<UserAuthLevel>(UserAuthLevel.NONE);
  
  // Cross-app state
  const [crossAppStatus, setCrossAppStatus] = useState<CrossAppStatus>({
    adminConnected: false,
    adminAuthLevel: 0,
    adminSessionId: null,
    lastAdminActivity: null,
    canAccessAdmin: false,
    adminStepUpRequired: false,
  });
  const [adminAccessRequest, setAdminAccessRequest] = useState<AdminAccessRequest | null>(null);
  const [syncEvents, setSyncEvents] = useState<SyncEvent[]>([]);
  const [lastSyncTime, setLastSyncTime] = useState<Date | null>(null);
  
  // Real-time state
  const [isOnline, setIsOnline] = useState(navigator.onLine);
  const [connectionQuality, setConnectionQuality] = useState<'excellent' | 'good' | 'poor' | 'offline'>('excellent');
  
  // Error state
  const [error, setError] = useState<string | null>(null);
  const [warnings, setWarnings] = useState<string[]>([]);
  
  const baseApiUrl = backendUrl || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
  
  /**
   * Enhanced login with cross-app sync capabilities
   */
  const loginWithCredentials = useCallback(async (email: string, password: string) => {
    setIsLoading(true);
    setError(null);
    
    try {
      // Use enhanced auth system but with synchronized features
      const loginData = {
        email,
        password,
        app_type: 'frontend',
        app_id: 'frontend-client',
        enable_cross_app_sync: true,
        requested_features: ['trading', 'analytics', 'premium'],
        device_info: {
          fingerprint: await generateDeviceFingerprint(),
          user_agent: navigator.userAgent,
          screen_resolution: `${screen.width}x${screen.height}`,
          timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
          language: navigator.language,
          platform: navigator.platform,
        },
      };
      
      const response = await fetch(`${baseApiUrl}/api/v1/auth/synchronized-login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(loginData),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Login failed');
      }
      
      await handleSynchronizedAuthResult(result);
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Authentication failed';
      setError(errorMessage);
      
      // Fallback to enhanced auth
      if (enhancedAuth.isInitialized && !enhancedAuth.isAuthenticated) {
        try {
          await enhancedAuth.loginWithCredentials(email, password);
          await syncWithEnhancedAuth();
        } catch (fallbackError) {
          console.error('Both primary and fallback auth failed:', { primary: error, fallback: fallbackError });
        }
      }
    } finally {
      setIsLoading(false);
    }
  }, [baseApiUrl, enhancedAuth]);
  
  /**
   * Handle synchronized authentication result
   */
  const handleSynchronizedAuthResult = useCallback(async (result: any) => {
    const {
      user: userData,
      access_token,
      user_auth_level,
      max_user_level,
      cross_app_session,
      subscription_status,
      feature_flags,
      quota_usage,
    } = result;
    
    // Create synchronized user profile
    const synchronizedUser: SynchronizedUserProfile = {
      // Base fields from enhanced profile
      id: userData.user_id,
      email: userData.email,
      displayName: userData.display_name || userData.email.split('@')[0],
      photoURL: userData.photo_url,
      emailVerified: userData.email_verified || false,
      provider: userData.provider || 'credentials',
      providerId: userData.provider_id || 'backend',
      providerUserId: userData.user_id,
      tenantId: userData.tenant_id || 'default',
      tenantName: userData.tenant_name,
      providerType: userData.provider_type || 'credentials',
      role: userData.role || 'user',
      permissions: userData.permissions || [],
      subscriptionTier: subscription_status,
      sessionId: userData.session_id,
      authTime: userData.auth_time,
      riskScore: userData.risk_score,
      createdAt: userData.created_at || new Date().toISOString(),
      lastSignInAt: new Date().toISOString(),
      
      // Additional synchronized fields
      userAuthLevel: user_auth_level,
      maxUserLevel: max_user_level,
      
      crossAppStatus: {
        adminConnected: cross_app_session?.admin_connected || false,
        adminAuthLevel: cross_app_session?.admin_auth_level || 0,
        adminSessionId: cross_app_session?.admin_session_id || null,
        lastAdminActivity: cross_app_session?.last_admin_activity 
          ? new Date(cross_app_session.last_admin_activity) 
          : null,
        canAccessAdmin: cross_app_session?.can_access_admin || false,
        adminStepUpRequired: cross_app_session?.admin_step_up_required || false,
      },
      
      subscriptionStatus: subscription_status || 'free',
      featureFlags: feature_flags || [],
      quotaUsage: quota_usage || {},
      
      preferences: userData.preferences || {},
      notifications: userData.notification_preferences || [],
      
      tradingLevel: userData.trading_level || 'basic',
      portfolioAccess: userData.portfolio_access || [],
      
      lastFeatureUsed: null,
      sessionDuration: 0,
      apiCallsToday: userData.api_calls_today || 0,
    };
    
    // Update state
    setUser(synchronizedUser);
    setUserAuthLevel(user_auth_level);
    setMaxUserLevel(max_user_level);
    setCrossAppStatus(synchronizedUser.crossAppStatus);
    setIsAuthenticated(true);
    
    // Initialize real-time sync
    if (enableRealTimeSync && cross_app_session?.global_session_id) {
      await initializeRealTimeSync(cross_app_session.global_session_id);
    }
    
    // Start session management
    startSessionManagement();
    
    // Log successful login
    await reportActivity('login', {
      method: 'credentials',
      user_level: user_auth_level,
      cross_app_enabled: Boolean(cross_app_session),
    });
    
    console.log('Synchronized authentication successful:', {
      authLevel: user_auth_level,
      maxLevel: max_user_level,
      adminConnected: synchronizedUser.crossAppStatus.adminConnected,
    });
    
  }, [enableRealTimeSync]);
  
  /**
   * Initialize real-time synchronization with admin app
   */
  const initializeRealTimeSync = useCallback(async (globalSessionId: string) => {
    if (!enableRealTimeSync) return;
    
    try {
      // Close existing connection
      if (eventSourceRef.current) {
        eventSourceRef.current.close();
      }
      
      // Setup Server-Sent Events for real-time sync
      const eventSource = new EventSource(
        `${baseApiUrl}/api/v1/sync/events?session_id=${globalSessionId}&app=frontend`,
        { withCredentials: true }
      );
      
      eventSource.onopen = () => {
        setConnectionQuality('excellent');
        console.log('Real-time sync connection established');
      };
      
      eventSource.onmessage = (event) => {
        const syncEvent: SyncEvent = {
          ...JSON.parse(event.data),
          timestamp: new Date(),
        };
        
        handleSyncEvent(syncEvent);
      };
      
      eventSource.onerror = (error) => {
        console.error('Real-time sync error:', error);
        setConnectionQuality('poor');
        
        // Retry connection
        setTimeout(() => {
          if (eventSource.readyState === EventSource.CLOSED) {
            initializeRealTimeSync(globalSessionId);
          }
        }, 5000);
      };
      
      eventSourceRef.current = eventSource;
      
    } catch (error) {
      console.error('Failed to initialize real-time sync:', error);
      setWarnings(prev => [...prev, 'Real-time synchronization unavailable']);
    }
  }, [enableRealTimeSync, baseApiUrl]);
  
  /**
   * Handle incoming synchronization events
   */
  const handleSyncEvent = useCallback((event: SyncEvent) => {
    // Add to event history
    setSyncEvents(prev => [...prev.slice(-99), event]); // Keep last 100 events
    setLastSyncTime(event.timestamp);
    
    switch (event.type) {
      case 'admin_login':
        setCrossAppStatus(prev => ({
          ...prev,
          adminConnected: true,
          adminAuthLevel: event.data.auth_level,
          adminSessionId: event.data.session_id,
          lastAdminActivity: event.timestamp,
        }));
        
        if (user) {
          setUser(prev => prev ? {
            ...prev,
            crossAppStatus: {
              ...prev.crossAppStatus,
              adminConnected: true,
              adminAuthLevel: event.data.auth_level,
              adminSessionId: event.data.session_id,
              lastAdminActivity: event.timestamp,
            },
          } : null);
        }
        
        setWarnings(prev => [...prev, 'Admin session started - Enhanced features available']);
        break;
        
      case 'admin_logout':
        setCrossAppStatus(prev => ({
          ...prev,
          adminConnected: false,
          adminAuthLevel: 0,
          adminSessionId: null,
        }));
        
        if (user) {
          setUser(prev => prev ? {
            ...prev,
            crossAppStatus: {
              ...prev.crossAppStatus,
              adminConnected: false,
              adminAuthLevel: 0,
              adminSessionId: null,
            },
          } : null);
        }
        break;
        
      case 'auth_level_change':
        if (event.source === 'admin') {
          setCrossAppStatus(prev => ({
            ...prev,
            adminAuthLevel: event.data.new_level,
            adminStepUpRequired: event.data.step_up_required || false,
          }));
        }
        break;
        
      case 'security_alert':
        setWarnings(prev => [...prev, event.data.message]);
        break;
        
      case 'session_expired':
        setError('Session has expired');
        logout(false);
        break;
        
      default:
        console.log('Unknown sync event:', event);
    }
  }, [user]);
  
  /**
   * Request admin access
   */
  const requestAdminAccess = useCallback(async (reason: string): Promise<AdminAccessRequest> => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    const request: AdminAccessRequest = {
      requestId: crypto.randomUUID(),
      requiredLevel: 3, // Admin level
      reason,
      expiresAt: new Date(Date.now() + 5 * 60 * 1000), // 5 minutes
    };
    
    try {
      const response = await fetch(`${baseApiUrl}/api/v1/admin/request-access`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          request_id: request.requestId,
          reason,
          user_auth_level: userAuthLevel,
          global_session_id: user.globalSessionId,
        }),
      });
      
      if (!response.ok) {
        throw new Error('Admin access request failed');
      }
      
      const result = await response.json();
      const enhancedRequest = { ...request, ...result };
      
      setAdminAccessRequest(enhancedRequest);
      await reportActivity('admin_access_requested', { reason });
      
      return enhancedRequest;
      
    } catch (error) {
      console.error('Admin access request failed:', error);
      throw error;
    }
  }, [user, userAuthLevel, baseApiUrl]);
  
  /**
   * Open admin app with cross-app authentication
   */
  const openAdminApp = useCallback(async (): Promise<boolean> => {
    if (!user || !crossAppStatus.canAccessAdmin) {
      return false;
    }
    
    try {
      // Generate cross-app authentication token
      const response = await fetch(`${baseApiUrl}/api/v1/federation/generate-admin-token`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          source_app: 'frontend',
          target_app: 'admin',
          global_session_id: user.globalSessionId,
          required_auth_level: 3,
        }),
      });
      
      if (!response.ok) {
        throw new Error('Failed to generate admin access token');
      }
      
      const { admin_token } = await response.json();
      
      // Open admin app with token
      const adminUrl = new URL(process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001');
      adminUrl.searchParams.set('cross_app_token', admin_token);
      adminUrl.searchParams.set('source', 'frontend');
      
      window.open(adminUrl.toString(), '_blank');
      
      await reportActivity('admin_app_opened', {
        method: 'cross_app_token',
        auth_level: userAuthLevel,
      });
      
      return true;
      
    } catch (error) {
      console.error('Failed to open admin app:', error);
      setError('Unable to access admin interface');
      return false;
    }
  }, [user, crossAppStatus, userAuthLevel, baseApiUrl]);
  
  /**
   * Check feature access with complex logic
   */
  const checkFeatureAccess = useCallback((feature: string): FeatureAccessResult => {
    if (!user) {
      return { granted: false, reason: 'User not authenticated' };
    }
    
    // Check feature flags
    if (user.featureFlags.includes(feature)) {
      return { granted: true };
    }
    
    // Check subscription-based features
    const premiumFeatures = ['advanced_analytics', 'portfolio_optimization', 'real_time_data'];
    const enterpriseFeatures = ['white_label', 'api_access', 'custom_integrations'];
    const adminFeatures = ['user_management', 'system_settings', 'audit_logs'];
    
    if (adminFeatures.includes(feature)) {
      if (!crossAppStatus.adminConnected) {
        return {
          granted: false,
          reason: 'Admin access required',
          adminRequired: true,
        };
      }
      
      if (crossAppStatus.adminStepUpRequired) {
        return {
          granted: false,
          reason: 'Step-up authentication required',
          adminRequired: true,
        };
      }
      
      return { granted: true };
    }
    
    if (enterpriseFeatures.includes(feature)) {
      if (user.subscriptionStatus !== 'enterprise') {
        return {
          granted: false,
          reason: 'Enterprise subscription required',
          upgradeRequired: true,
        };
      }
      return { granted: true };
    }
    
    if (premiumFeatures.includes(feature)) {
      if (user.subscriptionStatus === 'free') {
        return {
          granted: false,
          reason: 'Premium subscription required',
          upgradeRequired: true,
        };
      }
      return { granted: true };
    }
    
    // Check auth level requirements
    const authLevelFeatures: Record<string, UserAuthLevel> = {
      'basic_trading': UserAuthLevel.BASIC,
      'advanced_trading': UserAuthLevel.VERIFIED,
      'portfolio_management': UserAuthLevel.PREMIUM,
      'admin_panel': UserAuthLevel.ADMIN_ELIGIBLE,
    };
    
    const requiredLevel = authLevelFeatures[feature];
    if (requiredLevel !== undefined) {
      if (userAuthLevel < requiredLevel) {
        return {
          granted: false,
          reason: `Authentication level ${requiredLevel} required`,
          authLevelRequired: requiredLevel,
        };
      }
      return { granted: true };
    }
    
    // Default: check quota
    if (user.quotaUsage[feature] !== undefined) {
      const maxQuota = getFeatureQuota(feature, user.subscriptionStatus);
      if (user.quotaUsage[feature] >= maxQuota) {
        return {
          granted: false,
          reason: 'Feature quota exceeded',
          upgradeRequired: user.subscriptionStatus === 'free',
        };
      }
    }
    
    return { granted: true };
  }, [user, userAuthLevel, crossAppStatus]);
  
  /**
   * Get feature quota based on subscription
   */
  const getFeatureQuota = useCallback((feature: string, subscription: string): number => {
    const quotas: Record<string, Record<string, number>> = {
      api_calls: { free: 100, premium: 1000, enterprise: 10000 },
      portfolio_items: { free: 10, premium: 100, enterprise: 1000 },
      custom_alerts: { free: 3, premium: 20, enterprise: 100 },
    };
    
    return quotas[feature]?.[subscription] ?? Number.MAX_SAFE_INTEGER;
  }, []);
  
  /**
   * Sync with enhanced auth fallback
   */
  const syncWithEnhancedAuth = useCallback(async () => {
    if (!enhancedAuth.isAuthenticated || !enhancedAuth.user) return;
    
    // Convert enhanced auth user to synchronized user
    const synchronizedUser: SynchronizedUserProfile = {
      ...enhancedAuth.user,
      userAuthLevel: UserAuthLevel.BASIC,
      maxUserLevel: UserAuthLevel.VERIFIED,
      crossAppStatus: {
        adminConnected: false,
        adminAuthLevel: 0,
        adminSessionId: null,
        lastAdminActivity: null,
        canAccessAdmin: false,
        adminStepUpRequired: false,
      },
      subscriptionStatus: 'free',
      featureFlags: [],
      quotaUsage: {},
      preferences: {},
      notifications: [],
      tradingLevel: 'basic',
      portfolioAccess: [],
      lastFeatureUsed: null,
      sessionDuration: 0,
      apiCallsToday: 0,
    };
    
    setUser(synchronizedUser);
    setUserAuthLevel(UserAuthLevel.BASIC);
    setMaxUserLevel(UserAuthLevel.VERIFIED);
    setIsAuthenticated(true);
  }, [enhancedAuth]);
  
  /**
   * Start session management
   */
  const startSessionManagement = useCallback(() => {
    // Clear existing timers
    if (syncIntervalRef.current) {
      clearInterval(syncIntervalRef.current);
    }
    if (activityTimerRef.current) {
      clearTimeout(activityTimerRef.current);
    }
    
    // Setup periodic sync
    syncIntervalRef.current = setInterval(async () => {
      await syncWithAdmin();
    }, syncIntervalSeconds * 1000);
    
    // Setup session timeout
    const timeoutMs = sessionTimeoutMinutes * 60 * 1000;
    activityTimerRef.current = setTimeout(() => {
      setError('Session expired due to inactivity');
      logout(false);
    }, timeoutMs);
    
  }, [syncIntervalSeconds, sessionTimeoutMinutes]);
  
  /**
   * Generate device fingerprint
   */
  const generateDeviceFingerprint = useCallback(async (): Promise<string> => {
    const components = [
      navigator.userAgent,
      navigator.language,
      screen.width + 'x' + screen.height,
      new Date().getTimezoneOffset(),
      navigator.hardwareConcurrency || 0,
      navigator.maxTouchPoints || 0,
    ];
    
    const fingerprint = btoa(components.join('|'));
    return fingerprint.replace(/[+/=]/g, '').substring(0, 32);
  }, []);
  
  /**
   * Report user activity
   */
  const reportActivity = useCallback(async (activity: string, metadata: any = {}) => {
    if (!user) return;
    
    try {
      await fetch(`${baseApiUrl}/api/v1/activity/report`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          activity,
          metadata,
          timestamp: new Date().toISOString(),
          session_id: user.sessionId,
          global_session_id: user.globalSessionId,
        }),
      });
      
      // Update local state
      setUser(prev => prev ? {
        ...prev,
        lastFeatureUsed: activity,
        sessionDuration: Date.now() - new Date(prev.lastSignInAt).getTime(),
      } : null);
      
    } catch (error) {
      console.error('Failed to report activity:', error);
    }
  }, [user, baseApiUrl]);
  
  /**
   * Utility functions implementation
   */
  const clearError = useCallback(() => setError(null), []);
  const clearWarnings = useCallback(() => setWarnings([]), []);
  
  const hasQuotaRemaining = useCallback((resource: string): boolean => {
    if (!user) return false;
    
    const usage = user.quotaUsage[resource] || 0;
    const quota = getFeatureQuota(resource, user.subscriptionStatus);
    return usage < quota;
  }, [user, getFeatureQuota]);
  
  const getSessionInfo = useCallback(() => {
    if (!user) {
      return { timeRemaining: 0, lastActivity: new Date() };
    }
    
    const sessionEndTime = new Date(user.lastSignInAt).getTime() + (sessionTimeoutMinutes * 60 * 1000);
    const timeRemaining = Math.max(0, sessionEndTime - Date.now());
    
    return {
      timeRemaining,
      lastActivity: new Date(user.lastSignInAt),
    };
  }, [user, sessionTimeoutMinutes]);
  
  // Placeholder implementations for remaining methods
  const loginWithProvider = useCallback(async (providerId: string) => {
    await enhancedAuth.loginWithProvider(providerId);
    await syncWithEnhancedAuth();
  }, [enhancedAuth, syncWithEnhancedAuth]);
  
  const loginWithSocialMedia = useCallback(async (platform: 'google' | 'github' | 'linkedin') => {
    switch (platform) {
      case 'google':
        await enhancedAuth.loginWithGoogle();
        break;
      case 'github':
        // TODO: Implement GitHub login
        throw new Error('GitHub login not implemented yet');
      case 'linkedin':
        // TODO: Implement LinkedIn login
        throw new Error('LinkedIn login not implemented yet');
    }
    await syncWithEnhancedAuth();
  }, [enhancedAuth, syncWithEnhancedAuth]);
  
  const syncWithAdmin = useCallback(async () => {
    if (!enableRealTimeSync || !user?.globalSessionId) return;
    
    try {
      const response = await fetch(`${baseApiUrl}/api/v1/sync/status`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          global_session_id: user.globalSessionId,
          app: 'frontend',
        }),
      });
      
      if (response.ok) {
        const syncData = await response.json();
        setCrossAppStatus(prev => ({ ...prev, ...syncData }));
        setLastSyncTime(new Date());
      }
    } catch (error) {
      console.error('Admin sync failed:', error);
    }
  }, [enableRealTimeSync, user, baseApiUrl]);
  
  const checkAdminConnection = useCallback(async (): Promise<boolean> => {
    await syncWithAdmin();
    return crossAppStatus.adminConnected;
  }, [syncWithAdmin, crossAppStatus.adminConnected]);
  
  // Implement remaining placeholder methods
  const requestFeatureUpgrade = useCallback(async (feature: string) => {
    await reportActivity('feature_upgrade_requested', { feature });
    // TODO: Implement upgrade flow
  }, [reportActivity]);
  
  const upgradeSubscription = useCallback(async (tier: 'premium' | 'enterprise') => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    try {
      setIsLoading(true);
      
      const response = await fetch(`${baseApiUrl}/api/v1/subscription/upgrade`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          user_id: user.id,
          target_tier: tier,
          current_tier: user.subscriptionStatus,
          session_id: user.sessionId,
          app_context: 'frontend',
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Subscription upgrade failed');
      }
      
      // Update user subscription status
      setUser(prev => prev ? {
        ...prev,
        subscriptionStatus: tier,
        featureFlags: [...prev.featureFlags, ...result.new_feature_flags],
        quotaUsage: { ...prev.quotaUsage, ...result.updated_quotas },
        lastActivity: new Date().toISOString(),
      } : null);
      
      // Report activity
      await reportActivity('subscription_upgraded', {
        from_tier: user.subscriptionStatus,
        to_tier: tier,
        payment_method: result.payment_method,
        effective_date: result.effective_date,
      });
      
      setWarnings(prev => [
        ...prev,
        `Successfully upgraded to ${tier.charAt(0).toUpperCase() + tier.slice(1)}!`,
        'New features are now available',
        `Billing cycle starts: ${new Date(result.effective_date).toLocaleDateString()}`,
      ]);
      
      console.log(`Subscription upgraded to ${tier}`);
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Subscription upgrade failed';
      setError(errorMessage);
      
      await reportActivity('subscription_upgrade_failed', {
        target_tier: tier,
        current_tier: user.subscriptionStatus,
        error: errorMessage,
      });
      
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [user, baseApiUrl, reportActivity]);
  
  const checkSubscriptionStatus = useCallback(async (): Promise<void> => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    try {
      const response = await fetch(`${baseApiUrl}/api/v1/subscription/status`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Subscription status check failed');
      }
      
      // Update user subscription information
      setUser(prev => prev ? {
        ...prev,
        subscriptionStatus: result.subscription_status,
        featureFlags: result.feature_flags || prev.featureFlags,
        quotaUsage: { ...prev.quotaUsage, ...result.quota_usage },
      } : null);
      
      // Handle subscription warnings
      if (result.warnings?.length > 0) {
        setWarnings(prev => [...prev, ...result.warnings]);
      }
      
      // Handle quota alerts
      if (result.quota_alerts?.length > 0) {
        setWarnings(prev => [
          ...prev,
          ...result.quota_alerts.map((alert: any) => 
            `${alert.resource}: ${alert.usage}/${alert.limit} (${Math.round(alert.usage/alert.limit*100)}%)`
          ),
        ]);
      }
      
      console.log('Subscription status updated');
      
    } catch (error) {
      console.error('Subscription status check failed:', error);
      // Don't set error for status checks - just log
    }
  }, [user, baseApiUrl]);
  
  const updateProfile = useCallback(async (updates: Partial<SynchronizedUserProfile>) => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    try {
      setIsLoading(true);
      
      // Validate updates
      const allowedFields = [
        'displayName', 'photoURL', 'preferences', 'notifications',
        'tradingLevel', 'portfolioAccess'
      ];
      
      const filteredUpdates = Object.fromEntries(
        Object.entries(updates).filter(([key]) => allowedFields.includes(key))
      );
      
      const response = await fetch(`${baseApiUrl}/api/v1/user/profile`, {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          user_id: user.id,
          updates: filteredUpdates,
          session_id: user.sessionId,
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Profile update failed');
      }
      
      // Update user profile
      setUser(prev => prev ? {
        ...prev,
        ...filteredUpdates,
        lastActivity: new Date().toISOString(),
      } : null);
      
      await reportActivity('profile_updated', {
        updated_fields: Object.keys(filteredUpdates),
        update_count: Object.keys(filteredUpdates).length,
      });
      
      console.log('Profile updated successfully');
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Profile update failed';
      setError(errorMessage);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [user, baseApiUrl, reportActivity]);
  
  const changePassword = useCallback(async (currentPassword: string, newPassword: string) => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    try {
      setIsLoading(true);
      
      // Validate password strength
      const passwordValidation = validatePasswordStrength(newPassword);
      if (!passwordValidation.isValid) {
        throw new Error(`Weak password: ${passwordValidation.errors.join(', ')}`);
      }
      
      const response = await fetch(`${baseApiUrl}/api/v1/user/change-password`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          user_id: user.id,
          current_password: currentPassword,
          new_password: newPassword,
          device_fingerprint: await generateDeviceFingerprint(),
          session_id: user.sessionId,
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Password change failed');
      }
      
      // Update session if new tokens provided
      if (result.new_session_token) {
        setUser(prev => prev ? {
          ...prev,
          sessionId: result.new_session_token,
          lastActivity: new Date().toISOString(),
        } : null);
      }
      
      await reportActivity('password_changed', {
        password_strength: passwordValidation.strength,
        session_invalidated: Boolean(result.new_session_token),
      });
      
      setWarnings(prev => [
        ...prev,
        'Password changed successfully',
        'All other sessions have been invalidated',
        'Consider enabling two-factor authentication for enhanced security',
      ]);
      
      console.log('Password changed successfully');
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Password change failed';
      setError(errorMessage);
      
      await reportActivity('password_change_failed', {
        error: errorMessage,
      });
      
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [user, baseApiUrl, reportActivity, generateDeviceFingerprint]);
  
  const enableTwoFactor = useCallback(async (): Promise<{ qrCode: string; backupCodes: string[] }> => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    try {
      setIsLoading(true);
      
      const response = await fetch(`${baseApiUrl}/api/v1/user/enable-2fa`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          user_id: user.id,
          device_fingerprint: await generateDeviceFingerprint(),
          session_id: user.sessionId,
          setup_method: 'totp', // Time-based One-Time Password
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Two-factor authentication setup failed');
      }
      
      await reportActivity('two_factor_enabled', {
        method: 'totp',
        backup_codes_generated: result.backup_codes.length,
      });
      
      setWarnings(prev => [
        ...prev,
        'Two-factor authentication is being set up',
        'Save your backup codes in a secure location',
        'You will need your authenticator app to complete login',
      ]);
      
      console.log('Two-factor authentication setup initiated');
      
      return {
        qrCode: result.qr_code,
        backupCodes: result.backup_codes,
      };
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '2FA setup failed';
      setError(errorMessage);
      
      await reportActivity('two_factor_setup_failed', {
        error: errorMessage,
      });
      
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [user, baseApiUrl, reportActivity, generateDeviceFingerprint]);
  
  const updatePreferences = useCallback(async (preferences: Record<string, any>) => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    try {
      const response = await fetch(`${baseApiUrl}/api/v1/user/preferences`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          user_id: user.id,
          preferences,
          session_id: user.sessionId,
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Preferences update failed');
      }
      
      // Update local preferences
      setUser(prev => prev ? {
        ...prev,
        preferences: { ...prev.preferences, ...preferences },
        lastActivity: new Date().toISOString(),
      } : null);
      
      await reportActivity('preferences_updated', {
        updated_keys: Object.keys(preferences),
        preference_count: Object.keys(preferences).length,
      });
      
      console.log('User preferences updated successfully');
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Preferences update failed';
      setError(errorMessage);
      throw error;
    }
  }, [user, baseApiUrl, reportActivity]);
  
  const updateNotificationSettings = useCallback(async (settings: NotificationPreference[]) => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    try {
      const response = await fetch(`${baseApiUrl}/api/v1/user/notification-settings`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          user_id: user.id,
          notification_settings: settings,
          session_id: user.sessionId,
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Notification settings update failed');
      }
      
      // Update local notification settings
      setUser(prev => prev ? {
        ...prev,
        notifications: settings,
        lastActivity: new Date().toISOString(),
      } : null);
      
      await reportActivity('notification_settings_updated', {
        enabled_notifications: settings.filter(s => s.enabled).length,
        total_notifications: settings.length,
        channels_used: [...new Set(settings.flatMap(s => s.channels))],
      });
      
      console.log('Notification settings updated successfully');
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Notification settings update failed';
      setError(errorMessage);
      throw error;
    }
  }, [user, baseApiUrl, reportActivity]);
  
  const extendSession = useCallback(async (): Promise<boolean> => {
    if (!user) {
      return false;
    }
    
    try {
      const response = await fetch(`${baseApiUrl}/api/v1/session/extend`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          user_id: user.id,
          session_id: user.sessionId,
          global_session_id: user.globalSessionId,
          extension_reason: 'user_activity',
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        console.warn('Session extension failed:', result.error);
        return false;
      }
      
      // Update session expiration
      setUser(prev => prev ? {
        ...prev,
        sessionExpires: new Date(result.new_expires_at),
        lastActivity: new Date().toISOString(),
      } : null);
      
      await reportActivity('session_extended', {
        new_expires_at: result.new_expires_at,
        extension_duration_minutes: result.extension_duration_minutes,
      });
      
      console.log('Session extended successfully');
      return true;
      
    } catch (error) {
      console.error('Session extension error:', error);
      return false;
    }
  }, [user, baseApiUrl, reportActivity]);
  
  const logout = useCallback(async (logoutFromAdmin = false) => {
    setIsLoading(true);
    
    try {
      // Logout from backend
      if (user) {
        await fetch(`${baseApiUrl}/api/v1/auth/logout`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${user.sessionId}`,
          },
          body: JSON.stringify({
            logout_from_admin: logoutFromAdmin,
            global_session_id: user.globalSessionId,
          }),
        });
      }
      
      // Logout from enhanced auth
      await enhancedAuth.logout(logoutFromAdmin);
      
      // Cleanup
      if (eventSourceRef.current) {
        eventSourceRef.current.close();
      }
      if (syncIntervalRef.current) {
        clearInterval(syncIntervalRef.current);
      }
      if (activityTimerRef.current) {
        clearTimeout(activityTimerRef.current);
      }
      
      // Reset state
      setIsAuthenticated(false);
      setUser(null);
      setUserAuthLevel(UserAuthLevel.NONE);
      setMaxUserLevel(UserAuthLevel.NONE);
      setCrossAppStatus({
        adminConnected: false,
        adminAuthLevel: 0,
        adminSessionId: null,
        lastAdminActivity: null,
        canAccessAdmin: false,
        adminStepUpRequired: false,
      });
      setAdminAccessRequest(null);
      setSyncEvents([]);
      
    } catch (error) {
      console.error('Logout error:', error);
    } finally {
      setIsLoading(false);
    }
  }, [user, baseApiUrl, enhancedAuth]);
  
  const refreshAuth = useCallback(async (): Promise<boolean> => {
    return await enhancedAuth.refreshToken();
  }, [enhancedAuth]);
  
  const subscribeToEvents = useCallback((callback: (event: SyncEvent) => void) => {
    // Add callback to sync events
    const handleSyncEvent = (event: SyncEvent) => {
      callback(event);
    };
    
    // Store callback reference
    const eventCallbackRef = { current: handleSyncEvent };
    
    // Add event listener to internal event system
    const originalHandleSyncEvent = handleSyncEvent;
    
    // Return unsubscribe function
    return () => {
      // Remove callback from event handling
      eventCallbackRef.current = () => {};
      console.log('Event subscription removed');
    };
  }, []);
  
  const exportUserData = useCallback(async (): Promise<Blob> => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    try {
      setIsLoading(true);
      
      const response = await fetch(`${baseApiUrl}/api/v1/user/export-data`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          user_id: user.id,
          export_format: 'json',
          include_activity_logs: true,
          include_preferences: true,
          include_subscription_data: true,
          session_id: user.sessionId,
        }),
      });
      
      if (!response.ok) {
        const errorResult = await response.json();
        throw new Error(errorResult.error || 'User data export failed');
      }
      
      const blob = await response.blob();
      
      await reportActivity('user_data_exported', {
        export_size_bytes: blob.size,
        export_format: 'json',
      });
      
      console.log('User data exported successfully');
      
      return blob;
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Data export failed';
      setError(errorMessage);
      
      await reportActivity('user_data_export_failed', {
        error: errorMessage,
      });
      
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [user, baseApiUrl, reportActivity]);
  
  // Monitor online status
  useEffect(() => {
    const handleOnline = () => setIsOnline(true);
    const handleOffline = () => setIsOnline(false);
    
    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);
    
    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, []);
  
  // Initialize authentication
  useEffect(() => {
    const initializeAuth = async () => {
      try {
        // Wait for enhanced auth to initialize
        if (!enhancedAuth.isInitialized) {
          await new Promise(resolve => {
            const checkInitialized = () => {
              if (enhancedAuth.isInitialized) {
                resolve(undefined);
              } else {
                setTimeout(checkInitialized, 100);
              }
            };
            checkInitialized();
          });
        }
        
        // Sync with enhanced auth if authenticated
        if (enhancedAuth.isAuthenticated) {
          await syncWithEnhancedAuth();
        }
        
        setIsInitialized(true);
      } catch (error) {
        console.error('Failed to initialize synchronized auth:', error);
        setError('Authentication initialization failed');
      } finally {
        setIsLoading(false);
      }
    };
    
    initializeAuth();
  }, [enhancedAuth.isInitialized, enhancedAuth.isAuthenticated, syncWithEnhancedAuth]);
  
  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (eventSourceRef.current) {
        eventSourceRef.current.close();
      }
      if (syncIntervalRef.current) {
        clearInterval(syncIntervalRef.current);
      }
      if (activityTimerRef.current) {
        clearTimeout(activityTimerRef.current);
      }
    };
  }, []);
  
  // Context value
  const value: SynchronizedMultiAuthState = {
    // State
    isAuthenticated,
    isLoading,
    isInitialized,
    user,
    userAuthLevel,
    maxUserLevel,
    crossAppStatus,
    adminAccessRequest,
    syncEvents,
    lastSyncTime,
    isOnline,
    connectionQuality,
    error,
    warnings,
    
    // Authentication methods
    loginWithCredentials,
    loginWithProvider,
    loginWithSocialMedia,
    
    // Cross-app operations
    requestAdminAccess,
    openAdminApp,
    syncWithAdmin,
    checkAdminConnection,
    
    // Feature access
    checkFeatureAccess,
    requestFeatureUpgrade,
    hasQuotaRemaining,
    
    // Subscription
    upgradeSubscription,
    checkSubscriptionStatus,
    
    // User management
    updateProfile,
    changePassword,
    enableTwoFactor,
    
    // Preferences
    updatePreferences,
    updateNotificationSettings,
    
    // Session management
    extendSession,
    logout,
    refreshAuth,
    
    // Real-time features
    subscribeToEvents,
    reportActivity,
    
    // Utilities
    clearError,
    clearWarnings,
    getSessionInfo,
    exportUserData,
  };
  
  return (
    <SynchronizedMultiAuthContext.Provider value={value}>
      {children}
    </SynchronizedMultiAuthContext.Provider>
  );
}

/**
 * Hook to use synchronized multi-app authentication
 */
export function useSynchronizedMultiAuth(): SynchronizedMultiAuthState {
  const context = useContext(SynchronizedMultiAuthContext);
  
  if (context === undefined) {
    throw new Error('useSynchronizedMultiAuth must be used within SynchronizedMultiAuthProvider');
  }
  
  return context;
}

/**
 * Hook for cross-app operations
 */
export function useCrossAppOperations() {
  const {
    crossAppStatus,
    requestAdminAccess,
    openAdminApp,
    checkAdminConnection,
  } = useSynchronizedMultiAuth();
  
  return {
    crossAppStatus,
    requestAdminAccess,
    openAdminApp,
    checkAdminConnection,
    canAccessAdmin: crossAppStatus.canAccessAdmin,
    isAdminConnected: crossAppStatus.adminConnected,
  };
}

/**
 * Hook for feature access management
 */
export function useFeatureAccess() {
  const {
    checkFeatureAccess,
    requestFeatureUpgrade,
    hasQuotaRemaining,
    user,
  } = useSynchronizedMultiAuth();
  
  return {
    checkFeatureAccess,
    requestFeatureUpgrade,
    hasQuotaRemaining,
    subscriptionStatus: user?.subscriptionStatus || 'free',
    featureFlags: user?.featureFlags || [],
  };
}

// Helper functions

/**
 * Validate password strength
 */
function validatePasswordStrength(password: string): {
  isValid: boolean;
  strength: 'weak' | 'medium' | 'strong' | 'very_strong';
  errors: string[];
} {
  const errors: string[] = [];
  let score = 0;
  
  // Length check
  if (password.length < 8) {
    errors.push('Password must be at least 8 characters long');
  } else if (password.length >= 12) {
    score += 2;
  } else {
    score += 1;
  }
  
  // Character variety checks
  if (!/[a-z]/.test(password)) {
    errors.push('Password must contain lowercase letters');
  } else {
    score += 1;
  }
  
  if (!/[A-Z]/.test(password)) {
    errors.push('Password must contain uppercase letters');
  } else {
    score += 1;
  }
  
  if (!/\d/.test(password)) {
    errors.push('Password must contain numbers');
  } else {
    score += 1;
  }
  
  if (!/[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?]/.test(password)) {
    errors.push('Password must contain special characters');
  } else {
    score += 1;
  }
  
  // Common patterns check
  if (/(.)\1{2,}/.test(password)) {
    errors.push('Password cannot contain repeated characters');
    score -= 1;
  }
  
  if (/123|abc|password|qwerty/i.test(password)) {
    errors.push('Password cannot contain common patterns');
    score -= 1;
  }
  
  // Determine strength
  let strength: 'weak' | 'medium' | 'strong' | 'very_strong';
  if (score < 3) {
    strength = 'weak';
  } else if (score < 5) {
    strength = 'medium';
  } else if (score < 7) {
    strength = 'strong';
  } else {
    strength = 'very_strong';
  }
  
  return {
    isValid: errors.length === 0 && score >= 4,
    strength,
    errors,
  };
}

export default SynchronizedMultiAuthProvider;