'use client';

// Progressive Multi-Level Admin Authentication System
// The hardest possible authentication flow with step-up, MFA, and real-time cross-app sync

import React, { 
  createContext, useContext, useState, useEffect, useCallback, 
  ReactNode, useRef 
} from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { WebAuthnSecurityManager, type BehavioralBiometricsData } from '../lib/webauthn-security';

/**
 * Authentication Levels - Progressive Security Model
 */
export enum AuthLevel {
  NONE = 0,           // Not authenticated
  BASIC = 1,          // Basic authentication (email/password)
  MFA = 2,            // Multi-factor authentication completed
  ADMIN = 3,          // Admin privileges verified
  SUPER_ADMIN = 4,    // Super admin with step-up authentication
  EMERGENCY = 5,      // Emergency admin access (time-limited)
}

/**
 * Security Risk Levels
 */
export enum SecurityRisk {
  LOW = 1,
  MEDIUM = 2,
  HIGH = 3,
  CRITICAL = 4,
}

/**
 * Device Trust Levels
 */
export enum DeviceTrust {
  UNTRUSTED = 1,
  PARTIAL = 2,
  TRUSTED = 3,
  FULLY_TRUSTED = 4,
}

/**
 * Hardware security key information
 */
export interface SecurityKeyInfo {
  credentialId: string;
  keyName: string;
  registeredAt: Date;
  lastUsed: Date | null;
  isActive: boolean;
  keyType: 'usb' | 'nfc' | 'ble' | 'internal';
  authLevel: AuthLevel;
}

/**
 * Biometric authentication status
 */
export interface BiometricStatus {
  enabled: boolean;
  available: boolean;
  verificationLevel: 'none' | 'basic' | 'enhanced';
  lastVerification: Date | null;
  biometricType: 'fingerprint' | 'faceId' | 'voiceId' | 'behavioral' | 'unknown';
  confidenceScore: number;
}

/**
 * Progressive authentication user profile
 */
export interface ProgressiveAdminUser {
  // Core identity
  id: string;
  email: string;
  displayName: string | null;
  photoURL: string | null;
  
  // Authentication state
  authLevel: AuthLevel;
  maxAuthLevel: AuthLevel; // Highest level this user can achieve
  currentRisk: SecurityRisk;
  deviceTrust: DeviceTrust;
  
  // Provider and tenant info
  provider: string;
  providerId: string;
  tenantId: string;
  
  // Admin-specific data
  role: string;
  permissions: string[];
  adminAccessLevel: string;
  
  // Security metadata
  sessionId: string;
  globalSessionId: string;
  lastStepUp: Date | null;
  mfaEnabled: boolean;
  mfaVerified: boolean;
  
  // Device and location
  deviceFingerprint: string;
  lastKnownLocation?: string;
  trustedDevices: string[];
  
  // Advanced security
  securityKeys: SecurityKeyInfo[];
  biometricStatus: BiometricStatus;
  behavioralProfile: BehavioralBiometricsData | null;
  zeroTrustScore: number; // 0-100 trust score
  continuousVerification: boolean;
  
  // Session timing
  authTime: Date;
  lastActivity: Date;
  sessionExpires: Date;
  
  // Emergency access
  emergencyAccessActive: boolean;
  emergencyAccessExpires: Date | null;
  
  // Cross-app data
  frontendSessionId: string | null;
  crossAppSyncEnabled: boolean;
  
  // Audit trail
  loginHistory: LoginAttempt[];
  adminActions: AdminAction[];
}

/**
 * Login attempt record
 */
export interface LoginAttempt {
  timestamp: Date;
  method: string;
  success: boolean;
  ipAddress: string;
  userAgent: string;
  location?: string;
  riskScore: number;
  failureReason?: string;
}

/**
 * Admin action record
 */
export interface AdminAction {
  timestamp: Date;
  action: string;
  resource: string;
  authLevel: AuthLevel;
  ipAddress: string;
  success: boolean;
  details?: any;
}

/**
 * Step-up authentication challenge
 */
export interface StepUpChallenge {
  challengeId: string;
  requiredLevel: AuthLevel;
  currentLevel: AuthLevel;
  methods: string[]; // ['password', 'mfa', 'biometric']
  expiresAt: Date;
  reason: string;
  context: any;
}

/**
 * Cross-app session sync data
 */
export interface CrossAppSession {
  globalSessionId: string;
  frontendActive: boolean;
  adminActive: boolean;
  lastSync: Date;
  syncToken: string;
}

/**
 * Progressive admin authentication state
 */
export interface ProgressiveAdminAuthState {
  // Authentication state
  isAuthenticated: boolean;
  isLoading: boolean;
  isInitialized: boolean;
  
  // User data
  user: ProgressiveAdminUser | null;
  authLevel: AuthLevel;
  maxAuthLevel: AuthLevel;
  
  // Security state
  currentRisk: SecurityRisk;
  deviceTrust: DeviceTrust;
  requiresStepUp: boolean;
  activeChallenge: StepUpChallenge | null;
  
  // Cross-app state
  crossAppSession: CrossAppSession | null;
  frontendConnected: boolean;
  
  // Error handling
  error: string | null;
  securityWarnings: string[];
  
  // Authentication methods
  loginWithCredentials: (email: string, password: string) => Promise<void>;
  loginWithProvider: (providerId: string) => Promise<void>;
  loginFromFrontend: (crossAppToken: string) => Promise<void>;
  
  // Step-up authentication
  initiateStepUp: (targetLevel: AuthLevel, reason: string, context?: any) => Promise<StepUpChallenge>;
  completeStepUp: (challengeId: string, credentials: any) => Promise<boolean>;
  
  // MFA operations
  enableMFA: () => Promise<{ qrCode: string; backupCodes: string[] }>;
  verifyMFA: (code: string) => Promise<boolean>;
  
  // Emergency access
  requestEmergencyAccess: (reason: string, approverContact: string) => Promise<void>;
  
  // Session management
  refreshSession: () => Promise<boolean>;
  elevatePermissions: (requiredPermissions: string[]) => Promise<boolean>;
  logout: (terminateAllSessions?: boolean) => Promise<void>;
  
  // Cross-app operations
  syncWithFrontend: () => Promise<void>;
  authenticateInFrontend: () => Promise<string>; // Returns cross-app token
  
  // Hardware security key operations
  registerSecurityKey: (keyName: string) => Promise<{ success: boolean; keyInfo?: SecurityKeyInfo }>;
  authenticateWithSecurityKey: () => Promise<{ success: boolean; authLevel?: AuthLevel }>;
  removeSecurityKey: (credentialId: string) => Promise<boolean>;
  
  // Biometric operations
  enableBiometricAuth: () => Promise<{ success: boolean; biometricType: string }>;
  verifyBiometric: () => Promise<{ verified: boolean; confidenceScore: number }>;
  
  // Behavioral biometrics
  startBehavioralTracking: () => void;
  stopBehavioralTracking: () => BehavioralBiometricsData | null;
  verifyBehavioralPattern: () => Promise<{ verified: boolean; riskLevel: string }>;
  
  // Zero-trust operations
  performContinuousVerification: () => Promise<number>; // Returns trust score
  triggerTrustEvaluation: () => Promise<{ trustScore: number; recommendations: string[] }>;
  
  // Security operations
  trustCurrentDevice: () => Promise<void>;
  revokeDeviceTrust: (deviceId: string) => Promise<void>;
  reportSuspiciousActivity: (details: any) => Promise<void>;
  
  // Admin utilities
  hasPermission: (permission: string) => boolean;
  hasMinAuthLevel: (level: AuthLevel) => boolean;
  canAccessResource: (resource: string) => boolean;
  logAdminAction: (action: string, resource: string, details?: any) => void;
  
  // Utilities
  clearError: () => void;
  clearSecurityWarnings: () => void;
  getSessionTimeRemaining: () => number; // milliseconds
}

const ProgressiveAdminAuthContext = createContext<ProgressiveAdminAuthState | undefined>(undefined);

/**
 * Progressive admin authentication provider props
 */
interface ProgressiveAdminAuthProviderProps {
  children: ReactNode;
  backendUrl?: string;
  enableCrossAppSync?: boolean;
  enableDeviceFingerprinting?: boolean;
  enableLocationTracking?: boolean;
  sessionTimeoutMinutes?: number;
  stepUpTimeoutMinutes?: number;
  maxFailedAttempts?: number;
}

/**
 * Progressive admin authentication provider component
 */
export function ProgressiveAdminAuthProvider({
  children,
  backendUrl,
  enableCrossAppSync = true,
  enableDeviceFingerprinting = true,
  enableLocationTracking = true,
  sessionTimeoutMinutes = 60,
  stepUpTimeoutMinutes = 5,
  maxFailedAttempts = 3,
}: ProgressiveAdminAuthProviderProps) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const syncInterval = useRef<NodeJS.Timeout | null>(null);
  const sessionTimer = useRef<NodeJS.Timeout | null>(null);
  const webauthnManager = useRef<WebAuthnSecurityManager | null>(null);
  const continuousVerificationInterval = useRef<NodeJS.Timeout | null>(null);
  
  // Core state
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isInitialized, setIsInitialized] = useState(false);
  const [user, setUser] = useState<ProgressiveAdminUser | null>(null);
  const [authLevel, setAuthLevel] = useState<AuthLevel>(AuthLevel.NONE);
  const [maxAuthLevel, setMaxAuthLevel] = useState<AuthLevel>(AuthLevel.NONE);
  
  // Security state
  const [currentRisk, setCurrentRisk] = useState<SecurityRisk>(SecurityRisk.LOW);
  const [deviceTrust, setDeviceTrust] = useState<DeviceTrust>(DeviceTrust.UNTRUSTED);
  const [requiresStepUp, setRequiresStepUp] = useState(false);
  const [activeChallenge, setActiveChallenge] = useState<StepUpChallenge | null>(null);
  
  // Advanced security state
  const [zeroTrustScore, setZeroTrustScore] = useState<number>(0);
  const [biometricStatus, setBiometricStatus] = useState<BiometricStatus>({
    enabled: false,
    available: false,
    verificationLevel: 'none',
    lastVerification: null,
    biometricType: 'unknown',
    confidenceScore: 0,
  });
  const [continuousVerification, setContinuousVerification] = useState(false);
  
  // Cross-app state
  const [crossAppSession, setCrossAppSession] = useState<CrossAppSession | null>(null);
  const [frontendConnected, setFrontendConnected] = useState(false);
  
  // Error state
  const [error, setError] = useState<string | null>(null);
  const [securityWarnings, setSecurityWarnings] = useState<string[]>([]);
  
  const baseApiUrl = backendUrl || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
  
  // Initialize WebAuthn manager
  useEffect(() => {
    if (!webauthnManager.current) {
      webauthnManager.current = new WebAuthnSecurityManager(baseApiUrl);
    }
  }, [baseApiUrl]);
  
  /**
   * Generate device fingerprint
   */
  const generateDeviceFingerprint = useCallback(async (): Promise<string> => {
    if (!enableDeviceFingerprinting) {
      return 'fingerprinting-disabled';
    }
    
    const components = [
      navigator.userAgent,
      navigator.language,
      screen.width + 'x' + screen.height,
      new Date().getTimezoneOffset(),
      navigator.hardwareConcurrency,
      navigator.maxTouchPoints,
    ];
    
    const fingerprint = btoa(components.join('|'));
    return fingerprint.replace(/[+/=]/g, '').substring(0, 32);
  }, [enableDeviceFingerprinting]);
  
  /**
   * Get user location (if enabled and permitted)
   */
  const getUserLocation = useCallback(async (): Promise<string | undefined> => {
    if (!enableLocationTracking) {
      return undefined;
    }
    
    return new Promise((resolve) => {
      if (!navigator.geolocation) {
        resolve(undefined);
        return;
      }
      
      navigator.geolocation.getCurrentPosition(
        (position) => {
          const lat = position.coords.latitude.toFixed(2);
          const lon = position.coords.longitude.toFixed(2);
          resolve(`${lat},${lon}`);
        },
        () => resolve(undefined),
        { timeout: 5000, maximumAge: 300000 }
      );
    });
  }, [enableLocationTracking]);
  
  /**
   * Calculate security risk score
   */
  const calculateRiskScore = useCallback(async (context: any = {}): Promise<SecurityRisk> => {
    let riskScore = 0;
    
    // Device trust factor
    switch (deviceTrust) {
      case DeviceTrust.UNTRUSTED:
        riskScore += 3;
        break;
      case DeviceTrust.PARTIAL:
        riskScore += 1;
        break;
      default:
        break;
    }
    
    // Location factor
    if (enableLocationTracking && context.location) {
      const storedLocation = user?.lastKnownLocation;
      if (storedLocation && storedLocation !== context.location) {
        riskScore += 2; // Location change
      }
    }
    
    // Time factor (unusual login times)
    const hour = new Date().getHours();
    if (hour < 6 || hour > 22) {
      riskScore += 1; // Outside business hours
    }
    
    // Failed attempts factor
    const recentFailures = user?.loginHistory?.filter(
      attempt => !attempt.success && Date.now() - attempt.timestamp.getTime() < 3600000
    )?.length || 0;
    
    riskScore += Math.min(recentFailures, 3);
    
    // Convert to enum
    if (riskScore >= 6) return SecurityRisk.CRITICAL;
    if (riskScore >= 4) return SecurityRisk.HIGH;
    if (riskScore >= 2) return SecurityRisk.MEDIUM;
    return SecurityRisk.LOW;
  }, [deviceTrust, enableLocationTracking, user]);
  
  /**
   * Enhanced login with progressive authentication
   */
  const loginWithCredentials = useCallback(async (email: string, password: string) => {
    setIsLoading(true);
    setError(null);
    
    try {
      const deviceFingerprint = await generateDeviceFingerprint();
      const location = await getUserLocation();
      
      const loginData = {
        email,
        password,
        app_type: 'admin',
        app_id: 'admin-frontend',
        device_fingerprint: deviceFingerprint,
        location,
        client_info: {
          user_agent: navigator.userAgent,
          screen_resolution: `${screen.width}x${screen.height}`,
          timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
          language: navigator.language,
        },
      };
      
      const response = await fetch(`${baseApiUrl}/api/v1/auth/progressive-login`, {
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
      
      // Handle progressive authentication result
      await handleProgressiveAuthResult(result, { deviceFingerprint, location });
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Authentication failed';
      setError(errorMessage);
      
      // Log failed attempt
      await logLoginAttempt({
        timestamp: new Date(),
        method: 'credentials',
        success: false,
        ipAddress: 'unknown',
        userAgent: navigator.userAgent,
        riskScore: 0,
        failureReason: errorMessage,
      });
      
    } finally {
      setIsLoading(false);
    }
  }, [generateDeviceFingerprint, getUserLocation, baseApiUrl]);
  
  /**
   * Handle progressive authentication result
   */
  const handleProgressiveAuthResult = useCallback(async (
    result: any,
    context: { deviceFingerprint: string; location?: string }
  ) => {
    const {
      user: userData,
      access_token,
      auth_level,
      max_auth_level,
      requires_mfa,
      requires_step_up,
      session_id,
      global_session_id,
      device_trust_level,
      risk_score,
      cross_app_session,
    } = result;
    
    // Calculate current risk
    const currentRiskLevel = await calculateRiskScore({ location: context.location });
    
    // Create progressive user profile
    const progressiveUser: ProgressiveAdminUser = {
      id: userData.user_id,
      email: userData.email,
      displayName: userData.display_name || userData.email.split('@')[0],
      photoURL: userData.photo_url,
      
      authLevel: auth_level,
      maxAuthLevel: max_auth_level,
      currentRisk: currentRiskLevel,
      deviceTrust: device_trust_level || DeviceTrust.UNTRUSTED,
      
      provider: userData.provider || 'credentials',
      providerId: userData.provider_id || 'backend',
      tenantId: userData.tenant_id || 'default',
      
      role: userData.role,
      permissions: userData.permissions || [],
      adminAccessLevel: userData.admin_access_level || 'standard',
      
      sessionId: session_id,
      globalSessionId: global_session_id,
      lastStepUp: null,
      mfaEnabled: userData.mfa_enabled || false,
      mfaVerified: !requires_mfa,
      
      deviceFingerprint: context.deviceFingerprint,
      lastKnownLocation: context.location,
      trustedDevices: userData.trusted_devices || [],
      
      // Advanced security
      securityKeys: userData.security_keys || [],
      biometricStatus: {
        enabled: userData.biometric_enabled || false,
        available: false, // Will be checked at runtime
        verificationLevel: userData.biometric_level || 'none',
        lastVerification: null,
        biometricType: 'unknown',
        confidenceScore: 0,
      },
      behavioralProfile: null,
      zeroTrustScore: userData.trust_score || 50, // Start with neutral trust
      continuousVerification: userData.continuous_verification || false,
      
      authTime: new Date(),
      lastActivity: new Date(),
      sessionExpires: new Date(Date.now() + sessionTimeoutMinutes * 60 * 1000),
      
      emergencyAccessActive: userData.emergency_access_active || false,
      emergencyAccessExpires: userData.emergency_access_expires 
        ? new Date(userData.emergency_access_expires) 
        : null,
      
      frontendSessionId: cross_app_session?.frontend_session_id || null,
      crossAppSyncEnabled: enableCrossAppSync,
      
      loginHistory: [],
      adminActions: [],
    };
    
    // Update state
    setUser(progressiveUser);
    setAuthLevel(auth_level);
    setMaxAuthLevel(max_auth_level);
    setCurrentRisk(currentRiskLevel);
    setDeviceTrust(device_trust_level || DeviceTrust.UNTRUSTED);
    setZeroTrustScore(progressiveUser.zeroTrustScore);
    setBiometricStatus(progressiveUser.biometricStatus);
    setContinuousVerification(progressiveUser.continuousVerification);
    setIsAuthenticated(true);
    
    // Handle step-up requirements
    if (requires_step_up || requires_mfa) {
      setRequiresStepUp(true);
      const requiredLevel = requires_mfa ? AuthLevel.MFA : AuthLevel.ADMIN;
      const challenge = await createStepUpChallenge(requiredLevel, 'Login security verification');
      setActiveChallenge(challenge);
    }
    
    // Setup cross-app session
    if (cross_app_session && enableCrossAppSync) {
      setCrossAppSession(cross_app_session);
      await initializeCrossAppSync(cross_app_session);
    }
    
    // Log successful attempt
    await logLoginAttempt({
      timestamp: new Date(),
      method: 'credentials',
      success: true,
      ipAddress: 'unknown',
      userAgent: navigator.userAgent,
      location: context.location,
      riskScore: currentRiskLevel,
    });
    
    // Start session management
    startSessionManagement();
    
    console.log(`Progressive admin authentication successful - Level ${auth_level}`);
    
  }, [calculateRiskScore, sessionTimeoutMinutes, enableCrossAppSync]);
  
  /**
   * Create step-up authentication challenge
   */
  const createStepUpChallenge = useCallback(async (
    requiredLevel: AuthLevel,
    reason: string,
    context: any = {}
  ): Promise<StepUpChallenge> => {
    const challengeId = crypto.randomUUID();
    const availableMethods = [];
    
    // Determine available authentication methods based on target level
    switch (requiredLevel) {
      case AuthLevel.MFA:
        availableMethods.push('mfa_code');
        if (user?.mfaEnabled) {
          availableMethods.push('authenticator');
        }
        break;
      case AuthLevel.ADMIN:
        availableMethods.push('password');
        if (user?.mfaEnabled) {
          availableMethods.push('mfa_code');
        }
        break;
      case AuthLevel.SUPER_ADMIN:
        availableMethods.push('password', 'mfa_code');
        if (deviceTrust === DeviceTrust.FULLY_TRUSTED) {
          availableMethods.push('biometric');
        }
        break;
    }
    
    const challenge: StepUpChallenge = {
      challengeId,
      requiredLevel,
      currentLevel: authLevel,
      methods: availableMethods,
      expiresAt: new Date(Date.now() + stepUpTimeoutMinutes * 60 * 1000),
      reason,
      context,
    };
    
    return challenge;
  }, [user, deviceTrust, authLevel, stepUpTimeoutMinutes]);
  
  /**
   * Initiate step-up authentication
   */
  const initiateStepUp = useCallback(async (
    targetLevel: AuthLevel,
    reason: string,
    context: any = {}
  ): Promise<StepUpChallenge> => {
    if (!user || authLevel >= targetLevel) {
      throw new Error('Step-up authentication not required');
    }
    
    const challenge = await createStepUpChallenge(targetLevel, reason, context);
    setActiveChallenge(challenge);
    setRequiresStepUp(true);
    
    logAdminAction('step_up_initiated', 'authentication', {
      target_level: targetLevel,
      reason,
      challenge_id: challenge.challengeId,
    });
    
    return challenge;
  }, [user, authLevel, createStepUpChallenge]);
  
  /**
   * Complete step-up authentication
   */
  const completeStepUp = useCallback(async (
    challengeId: string,
    credentials: any
  ): Promise<boolean> => {
    if (!activeChallenge || activeChallenge.challengeId !== challengeId) {
      throw new Error('Invalid or expired challenge');
    }
    
    try {
      const response = await fetch(`${baseApiUrl}/api/v1/auth/complete-step-up`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user?.sessionId}`,
        },
        body: JSON.stringify({
          challenge_id: challengeId,
          credentials,
          device_fingerprint: user?.deviceFingerprint,
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Step-up authentication failed');
      }
      
      // Update authentication level
      setAuthLevel(activeChallenge.requiredLevel);
      setRequiresStepUp(false);
      setActiveChallenge(null);
      
      // Update user step-up timestamp
      if (user) {
        setUser({
          ...user,
          lastStepUp: new Date(),
          authLevel: activeChallenge.requiredLevel,
          lastActivity: new Date(),
        });
      }
      
      logAdminAction('step_up_completed', 'authentication', {
        challenge_id: challengeId,
        new_level: activeChallenge.requiredLevel,
      });
      
      console.log(`Step-up authentication successful - Level ${activeChallenge.requiredLevel}`);
      return true;
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Step-up failed';
      setError(errorMessage);
      
      logAdminAction('step_up_failed', 'authentication', {
        challenge_id: challengeId,
        error: errorMessage,
      });
      
      return false;
    }
  }, [activeChallenge, baseApiUrl, user]);
  
  /**
   * Initialize cross-app session synchronization
   */
  const initializeCrossAppSync = useCallback(async (session: CrossAppSession) => {
    if (!enableCrossAppSync) return;
    
    try {
      // Setup WebSocket or SSE connection for real-time sync
      const eventSource = new EventSource(
        `${baseApiUrl}/api/v1/session/sync-events?session_id=${session.globalSessionId}`,
        {
          withCredentials: true,
        }
      );
      
      eventSource.onmessage = (event) => {
        const data = JSON.parse(event.data);
        handleCrossAppSyncEvent(data);
      };
      
      eventSource.onerror = (error) => {
        console.error('Cross-app sync connection error:', error);
        setSecurityWarnings(prev => [...prev, 'Cross-app synchronization interrupted']);
      };
      
      // Store reference for cleanup
      (window as any).__crossAppEventSource = eventSource;
      
    } catch (error) {
      console.error('Failed to initialize cross-app sync:', error);
    }
  }, [enableCrossAppSync, baseApiUrl]);
  
  /**
   * Handle cross-app synchronization events
   */
  const handleCrossAppSyncEvent = useCallback((event: any) => {
    switch (event.type) {
      case 'session_updated':
        setCrossAppSession(prev => prev ? { ...prev, ...event.data } : null);
        break;
        
      case 'frontend_login':
        setFrontendConnected(true);
        if (user) {
          setUser({ ...user, frontendSessionId: event.data.session_id });
        }
        break;
        
      case 'frontend_logout':
        setFrontendConnected(false);
        if (user) {
          setUser({ ...user, frontendSessionId: null });
        }
        break;
        
      case 'security_alert':
        setSecurityWarnings(prev => [...prev, event.data.message]);
        break;
        
      case 'force_logout':
        logout(true);
        break;
        
      default:
        console.log('Unknown cross-app sync event:', event);
    }
  }, [user]);
  
  /**
   * Start session management timers
   */
  const startSessionManagement = useCallback(() => {
    // Clear existing timers
    if (syncInterval.current) {
      clearInterval(syncInterval.current);
    }
    if (sessionTimer.current) {
      clearTimeout(sessionTimer.current);
    }
    
    // Setup session sync interval
    if (enableCrossAppSync) {
      syncInterval.current = setInterval(async () => {
        await syncWithFrontend();
      }, 30000); // Sync every 30 seconds
    }
    
    // Setup session timeout
    const timeoutMs = sessionTimeoutMinutes * 60 * 1000;
    sessionTimer.current = setTimeout(() => {
      setError('Session expired due to inactivity');
      logout(false);
    }, timeoutMs);
    
  }, [enableCrossAppSync, sessionTimeoutMinutes]);
  
  /**
   * Utility functions
   */
  const clearError = useCallback(() => setError(null), []);
  const clearSecurityWarnings = useCallback(() => setSecurityWarnings([]), []);
  
  const hasPermission = useCallback((permission: string): boolean => {
    return user?.permissions.includes(permission) || false;
  }, [user]);
  
  const hasMinAuthLevel = useCallback((level: AuthLevel): boolean => {
    return authLevel >= level;
  }, [authLevel]);
  
  const canAccessResource = useCallback((resource: string): boolean => {
    // TODO: Implement complex resource access logic
    return hasMinAuthLevel(AuthLevel.ADMIN);
  }, [hasMinAuthLevel]);
  
  const logAdminAction = useCallback((action: string, resource: string, details?: any) => {
    if (!user) return;
    
    const adminAction: AdminAction = {
      timestamp: new Date(),
      action,
      resource,
      authLevel,
      ipAddress: 'unknown', // Would be extracted from request
      success: true,
      details,
    };
    
    // Update user's admin actions
    setUser(prev => prev ? {
      ...prev,
      adminActions: [...prev.adminActions, adminAction].slice(-100), // Keep last 100 actions
    } : null);
    
    // Send to backend for audit trail
    fetch(`${baseApiUrl}/api/v1/audit/admin-action`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${user.sessionId}`,
      },
      body: JSON.stringify(adminAction),
    }).catch(console.error);
  }, [user, authLevel, baseApiUrl]);
  
  const logLoginAttempt = useCallback(async (attempt: LoginAttempt) => {
    // Store locally
    if (user) {
      setUser(prev => prev ? {
        ...prev,
        loginHistory: [...prev.loginHistory, attempt].slice(-50), // Keep last 50 attempts
      } : null);
    }
    
    // Send to backend
    try {
      await fetch(`${baseApiUrl}/api/v1/audit/login-attempt`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(attempt),
      });
    } catch (error) {
      console.error('Failed to log login attempt:', error);
    }
  }, [user, baseApiUrl]);
  
  const getSessionTimeRemaining = useCallback((): number => {
    if (!user) return 0;
    return Math.max(0, user.sessionExpires.getTime() - Date.now());
  }, [user]);
  
  /**
   * Sync with frontend application
   */
  const syncWithFrontend = useCallback(async () => {
    if (!enableCrossAppSync || !crossAppSession) return;
    
    try {
      const response = await fetch(`${baseApiUrl}/api/v1/session/sync`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user?.sessionId}`,
        },
        body: JSON.stringify({
          global_session_id: crossAppSession.globalSessionId,
          app_id: 'admin-frontend',
          last_sync: crossAppSession.lastSync,
        }),
      });
      
      if (response.ok) {
        const syncData = await response.json();
        setCrossAppSession(prev => prev ? { ...prev, ...syncData, lastSync: new Date() } : null);
        setFrontendConnected(syncData.frontend_active || false);
      }
    } catch (error) {
      console.error('Cross-app sync failed:', error);
    }
  }, [enableCrossAppSync, crossAppSession, baseApiUrl, user]);
  
  /**
   * Logout with optional session termination
   */
  const logout = useCallback(async (terminateAllSessions = false) => {
    setIsLoading(true);
    
    try {
      if (user) {
        await fetch(`${baseApiUrl}/api/v1/auth/logout`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${user.sessionId}`,
          },
          body: JSON.stringify({
            session_id: user.sessionId,
            global_session_id: user.globalSessionId,
            terminate_all: terminateAllSessions,
          }),
        });
        
        logAdminAction('logout', 'session', { terminate_all: terminateAllSessions });
      }
      
      // Cleanup
      if (syncInterval.current) {
        clearInterval(syncInterval.current);
      }
      if (sessionTimer.current) {
        clearTimeout(sessionTimer.current);
      }
      if ((window as any).__crossAppEventSource) {
        (window as any).__crossAppEventSource.close();
      }
      
      // Reset state
      setIsAuthenticated(false);
      setUser(null);
      setAuthLevel(AuthLevel.NONE);
      setMaxAuthLevel(AuthLevel.NONE);
      setCurrentRisk(SecurityRisk.LOW);
      setDeviceTrust(DeviceTrust.UNTRUSTED);
      setRequiresStepUp(false);
      setActiveChallenge(null);
      setCrossAppSession(null);
      setFrontendConnected(false);
      
      console.log('Progressive admin logout completed');
      
    } catch (error) {
      console.error('Logout error:', error);
    } finally {
      setIsLoading(false);
    }
  }, [user, baseApiUrl, logAdminAction]);
  
  // TODO: Implement remaining methods (MFA, emergency access, etc.)
  const loginWithProvider = useCallback(async (providerId: string) => {
    setIsLoading(true);
    setError(null);
    
    try {
      const deviceFingerprint = await generateDeviceFingerprint();
      const location = await getUserLocation();
      
      const response = await fetch(`${baseApiUrl}/api/v1/auth/provider-login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          provider_id: providerId,
          app_type: 'admin',
          app_id: 'admin-frontend',
          device_fingerprint: deviceFingerprint,
          location,
          client_info: {
            user_agent: navigator.userAgent,
            screen_resolution: `${screen.width}x${screen.height}`,
            timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
            language: navigator.language,
          },
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Provider login failed');
      }
      
      // Handle authentication result
      await handleProgressiveAuthResult(result, { deviceFingerprint, location });
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Provider authentication failed';
      setError(errorMessage);
      
      await logLoginAttempt({
        timestamp: new Date(),
        method: `provider_${providerId}`,
        success: false,
        ipAddress: 'unknown',
        userAgent: navigator.userAgent,
        riskScore: 0,
        failureReason: errorMessage,
      });
    } finally {
      setIsLoading(false);
    }
  }, [generateDeviceFingerprint, getUserLocation, baseApiUrl, handleProgressiveAuthResult, logLoginAttempt]);
  
  const loginFromFrontend = useCallback(async (crossAppToken: string) => {
    setIsLoading(true);
    setError(null);
    
    try {
      const deviceFingerprint = await generateDeviceFingerprint();
      const location = await getUserLocation();
      
      const response = await fetch(`${baseApiUrl}/api/v1/auth/cross-app-login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          cross_app_token: crossAppToken,
          source_app: 'frontend',
          target_app: 'admin',
          device_fingerprint: deviceFingerprint,
          location,
          client_info: {
            user_agent: navigator.userAgent,
            screen_resolution: `${screen.width}x${screen.height}`,
            timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
            language: navigator.language,
          },
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Cross-app authentication failed');
      }
      
      // Handle progressive authentication result
      await handleProgressiveAuthResult(result, { deviceFingerprint, location });
      
      logAdminAction('cross_app_login', 'authentication', {
        source_app: 'frontend',
        success: true,
      });
      
      console.log('Cross-app authentication from frontend successful');
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Cross-app authentication failed';
      setError(errorMessage);
      
      await logLoginAttempt({
        timestamp: new Date(),
        method: 'cross_app_frontend',
        success: false,
        ipAddress: 'unknown',
        userAgent: navigator.userAgent,
        riskScore: 0,
        failureReason: errorMessage,
      });
    } finally {
      setIsLoading(false);
    }
  }, [generateDeviceFingerprint, getUserLocation, baseApiUrl, handleProgressiveAuthResult, logAdminAction, logLoginAttempt]);
  
  const enableMFA = useCallback(async (): Promise<{ qrCode: string; backupCodes: string[] }> => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    try {
      const response = await fetch(`${baseApiUrl}/api/v1/auth/mfa/setup`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          user_id: user.id,
          device_fingerprint: user.deviceFingerprint,
          setup_type: 'totp', // Time-based One-Time Password
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'MFA setup failed');
      }
      
      // Update user MFA status
      setUser(prev => prev ? { ...prev, mfaEnabled: true } : null);
      
      logAdminAction('mfa_setup_initiated', 'security', {
        setup_type: 'totp',
        device_fingerprint: user.deviceFingerprint,
      });
      
      return {
        qrCode: result.qr_code,
        backupCodes: result.backup_codes,
      };
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'MFA setup failed';
      setError(errorMessage);
      throw error;
    }
  }, [user, baseApiUrl, logAdminAction]);
  
  const verifyMFA = useCallback(async (code: string): Promise<boolean> => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    try {
      const response = await fetch(`${baseApiUrl}/api/v1/auth/mfa/verify`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          user_id: user.id,
          mfa_code: code,
          device_fingerprint: user.deviceFingerprint,
          session_id: user.sessionId,
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'MFA verification failed');
      }
      
      // Update authentication level and user status
      if (result.verification_successful) {
        setAuthLevel(AuthLevel.MFA);
        setUser(prev => prev ? {
          ...prev,
          mfaVerified: true,
          authLevel: AuthLevel.MFA,
          lastActivity: new Date(),
        } : null);
        
        // Clear step-up requirement if MFA was required
        if (requiresStepUp && activeChallenge?.requiredLevel === AuthLevel.MFA) {
          setRequiresStepUp(false);
          setActiveChallenge(null);
        }
        
        logAdminAction('mfa_verification_success', 'security', {
          new_auth_level: AuthLevel.MFA,
          device_fingerprint: user.deviceFingerprint,
        });
        
        console.log('MFA verification successful');
        return true;
      } else {
        logAdminAction('mfa_verification_failed', 'security', {
          device_fingerprint: user.deviceFingerprint,
          failure_reason: result.failure_reason,
        });
        
        setError('Invalid MFA code');
        return false;
      }
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'MFA verification failed';
      setError(errorMessage);
      
      logAdminAction('mfa_verification_error', 'security', {
        error: errorMessage,
        device_fingerprint: user?.deviceFingerprint,
      });
      
      return false;
    }
  }, [user, baseApiUrl, requiresStepUp, activeChallenge, logAdminAction]);
  
  const requestEmergencyAccess = useCallback(async (reason: string, approverContact: string): Promise<void> => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    try {
      const emergencyRequest = {
        request_id: crypto.randomUUID(),
        user_id: user.id,
        requested_level: AuthLevel.EMERGENCY,
        current_level: authLevel,
        reason,
        approver_contact: approverContact,
        device_fingerprint: user.deviceFingerprint,
        location: user.lastKnownLocation,
        timestamp: new Date().toISOString(),
        session_id: user.sessionId,
        risk_score: await calculateRiskScore(),
      };
      
      const response = await fetch(`${baseApiUrl}/api/v1/auth/emergency-access/request`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify(emergencyRequest),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Emergency access request failed');
      }
      
      // Show warning about emergency access
      setSecurityWarnings(prev => [
        ...prev,
        'Emergency access requested - Awaiting approval',
        'Emergency access is time-limited and fully audited',
        `Approval required from: ${approverContact}`,
      ]);
      
      logAdminAction('emergency_access_requested', 'security', {
        request_id: emergencyRequest.request_id,
        reason: reason.substring(0, 100), // Truncate for logging
        approver_contact: approverContact,
        current_auth_level: authLevel,
        risk_score: emergencyRequest.risk_score,
      });
      
      console.log('Emergency access request submitted successfully');
      
      // Poll for approval status
      const pollForApproval = async () => {
        try {
          const statusResponse = await fetch(
            `${baseApiUrl}/api/v1/auth/emergency-access/status/${emergencyRequest.request_id}`,
            {
              headers: {
                'Authorization': `Bearer ${user.sessionId}`,
              },
            }
          );
          
          if (statusResponse.ok) {
            const statusResult = await statusResponse.json();
            
            if (statusResult.status === 'approved') {
              setAuthLevel(AuthLevel.EMERGENCY);
              setUser(prev => prev ? {
                ...prev,
                authLevel: AuthLevel.EMERGENCY,
                emergencyAccessActive: true,
                emergencyAccessExpires: new Date(statusResult.expires_at),
                lastActivity: new Date(),
              } : null);
              
              setSecurityWarnings(prev => [
                ...prev.filter(w => !w.includes('Emergency access requested')),
                `Emergency access GRANTED - Expires: ${new Date(statusResult.expires_at).toLocaleString()}`,
                'All emergency actions are being logged and audited',
              ]);
              
              logAdminAction('emergency_access_granted', 'security', {
                request_id: emergencyRequest.request_id,
                approved_by: statusResult.approved_by,
                expires_at: statusResult.expires_at,
              });
              
            } else if (statusResult.status === 'denied') {
              setError('Emergency access request was denied');
              setSecurityWarnings(prev => 
                prev.filter(w => !w.includes('Emergency access requested'))
              );
              
              logAdminAction('emergency_access_denied', 'security', {
                request_id: emergencyRequest.request_id,
                denied_by: statusResult.denied_by,
                denial_reason: statusResult.denial_reason,
              });
            }
            
            // Continue polling if still pending
            else if (statusResult.status === 'pending') {
              setTimeout(pollForApproval, 10000); // Poll every 10 seconds
            }
          }
        } catch (pollError) {
          console.error('Failed to check emergency access status:', pollError);
        }
      };
      
      // Start polling after 5 seconds
      setTimeout(pollForApproval, 5000);
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Emergency access request failed';
      setError(errorMessage);
      
      logAdminAction('emergency_access_request_failed', 'security', {
        error: errorMessage,
        reason: reason.substring(0, 100),
        approver_contact: approverContact,
      });
      
      throw error;
    }
  }, [user, authLevel, baseApiUrl, calculateRiskScore, logAdminAction]);
  
  const refreshSession = useCallback(async (): Promise<boolean> => {
    if (!user) {
      return false;
    }
    
    try {
      const response = await fetch(`${baseApiUrl}/api/v1/auth/refresh-session`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          user_id: user.id,
          global_session_id: user.globalSessionId,
          device_fingerprint: user.deviceFingerprint,
          current_auth_level: authLevel,
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        setError(result.error || 'Session refresh failed');
        return false;
      }
      
      // Update session information
      setUser(prev => prev ? {
        ...prev,
        sessionId: result.new_session_id || prev.sessionId,
        sessionExpires: new Date(result.expires_at),
        lastActivity: new Date(),
      } : null);
      
      logAdminAction('session_refreshed', 'session', {
        new_session_id: result.new_session_id,
        expires_at: result.expires_at,
        auth_level: authLevel,
      });
      
      console.log('Session refreshed successfully');
      return true;
      
    } catch (error) {
      console.error('Session refresh error:', error);
      setError('Failed to refresh session');
      return false;
    }
  }, [user, authLevel, baseApiUrl, logAdminAction]);
  
  const elevatePermissions = useCallback(async (requiredPermissions: string[]): Promise<boolean> => {
    if (!user) {
      return false;
    }
    
    // Check if user already has all required permissions
    const hasAllPermissions = requiredPermissions.every(permission => 
      user.permissions.includes(permission)
    );
    
    if (hasAllPermissions) {
      return true;
    }
    
    try {
      // Determine required auth level for permissions
      const permissionLevels: Record<string, AuthLevel> = {
        'user.read': AuthLevel.BASIC,
        'user.write': AuthLevel.ADMIN,
        'system.read': AuthLevel.ADMIN,
        'system.write': AuthLevel.SUPER_ADMIN,
        'audit.read': AuthLevel.ADMIN,
        'audit.write': AuthLevel.SUPER_ADMIN,
        'emergency.access': AuthLevel.EMERGENCY,
      };
      
      const maxRequiredLevel = Math.max(
        ...requiredPermissions.map(p => permissionLevels[p] || AuthLevel.ADMIN)
      );
      
      // Request step-up authentication if needed
      if (authLevel < maxRequiredLevel) {
        const challenge = await initiateStepUp(
          maxRequiredLevel,
          `Permission elevation required for: ${requiredPermissions.join(', ')}`
        );
        
        setSecurityWarnings(prev => [
          ...prev,
          `Step-up authentication required for elevated permissions: ${requiredPermissions.join(', ')}`,
        ]);
        
        return false; // User needs to complete step-up
      }
      
      // Request permission elevation from backend
      const response = await fetch(`${baseApiUrl}/api/v1/auth/elevate-permissions`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          user_id: user.id,
          required_permissions: requiredPermissions,
          current_auth_level: authLevel,
          session_id: user.sessionId,
          justification: `Admin operation requiring: ${requiredPermissions.join(', ')}`,
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        setError(result.error || 'Permission elevation failed');
        return false;
      }
      
      // Update user permissions temporarily
      if (result.elevation_granted) {
        const elevatedPermissions = [...user.permissions, ...requiredPermissions];
        setUser(prev => prev ? {
          ...prev,
          permissions: [...new Set(elevatedPermissions)], // Remove duplicates
          lastActivity: new Date(),
        } : null);
        
        // Set expiration for elevated permissions
        setTimeout(() => {
          setUser(prev => prev ? {
            ...prev,
            permissions: user.permissions, // Restore original permissions
          } : null);
          
          setSecurityWarnings(prev => [
            ...prev.filter(w => !w.includes('elevated permissions')),
            'Elevated permissions have expired',
          ]);
        }, result.elevation_duration_ms || 300000); // Default 5 minutes
        
        logAdminAction('permissions_elevated', 'security', {
          elevated_permissions: requiredPermissions,
          elevation_duration_ms: result.elevation_duration_ms,
          auth_level: authLevel,
        });
        
        setSecurityWarnings(prev => [
          ...prev,
          `Elevated permissions granted: ${requiredPermissions.join(', ')}`,
          `Elevation expires in ${Math.round((result.elevation_duration_ms || 300000) / 60000)} minutes`,
        ]);
        
        return true;
      }
      
      return false;
      
    } catch (error) {
      console.error('Permission elevation error:', error);
      setError('Failed to elevate permissions');
      return false;
    }
  }, [user, authLevel, baseApiUrl, initiateStepUp, logAdminAction]);
  
  const authenticateInFrontend = useCallback(async (): Promise<string> => {
    if (!user || !crossAppSession) {
      throw new Error('User not authenticated or cross-app session not available');
    }
    
    try {
      const response = await fetch(`${baseApiUrl}/api/v1/federation/generate-frontend-token`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          source_app: 'admin',
          target_app: 'frontend',
          global_session_id: user.globalSessionId,
          user_id: user.id,
          auth_level: authLevel,
          permissions: user.permissions,
          device_fingerprint: user.deviceFingerprint,
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Frontend token generation failed');
      }
      
      logAdminAction('frontend_token_generated', 'cross_app', {
        target_app: 'frontend',
        auth_level: authLevel,
        token_expires_at: result.expires_at,
      });
      
      console.log('Frontend authentication token generated successfully');
      
      return result.cross_app_token;
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Frontend authentication failed';
      setError(errorMessage);
      
      logAdminAction('frontend_token_generation_failed', 'cross_app', {
        target_app: 'frontend',
        error: errorMessage,
      });
      
      throw error;
    }
  }, [user, crossAppSession, authLevel, baseApiUrl, logAdminAction]);
  
  const trustCurrentDevice = useCallback(async (): Promise<void> => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    try {
      const deviceInfo = {
        fingerprint: user.deviceFingerprint,
        user_agent: navigator.userAgent,
        screen_resolution: `${screen.width}x${screen.height}`,
        timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        language: navigator.language,
        platform: navigator.platform,
        location: user.lastKnownLocation,
      };
      
      const response = await fetch(`${baseApiUrl}/api/v1/auth/device/trust`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          user_id: user.id,
          device_info: deviceInfo,
          trust_level: DeviceTrust.FULLY_TRUSTED,
          session_id: user.sessionId,
          auth_level: authLevel,
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Device trust registration failed');
      }
      
      // Update device trust status
      setDeviceTrust(DeviceTrust.FULLY_TRUSTED);
      setUser(prev => prev ? {
        ...prev,
        trustedDevices: [...prev.trustedDevices, user.deviceFingerprint],
        deviceTrust: DeviceTrust.FULLY_TRUSTED,
        lastActivity: new Date(),
      } : null);
      
      // Recalculate risk score with trusted device
      const newRiskScore = await calculateRiskScore();
      setCurrentRisk(newRiskScore);
      
      logAdminAction('device_trusted', 'security', {
        device_fingerprint: user.deviceFingerprint,
        trust_level: DeviceTrust.FULLY_TRUSTED,
        previous_trust: deviceTrust,
        new_risk_score: newRiskScore,
      });
      
      setSecurityWarnings(prev => [
        ...prev,
        'Current device has been marked as fully trusted',
        'Enhanced authentication features are now available',
      ]);
      
      console.log('Device trust established successfully');
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Device trust failed';
      setError(errorMessage);
      
      logAdminAction('device_trust_failed', 'security', {
        device_fingerprint: user.deviceFingerprint,
        error: errorMessage,
      });
      
      throw error;
    }
  }, [user, authLevel, deviceTrust, baseApiUrl, calculateRiskScore, logAdminAction]);
  
  const revokeDeviceTrust = useCallback(async (deviceId: string): Promise<void> => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    try {
      const response = await fetch(`${baseApiUrl}/api/v1/auth/device/revoke-trust`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify({
          user_id: user.id,
          device_id: deviceId,
          revoke_reason: 'Admin device trust revocation',
          session_id: user.sessionId,
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Device trust revocation failed');
      }
      
      // Update trusted devices list
      setUser(prev => prev ? {
        ...prev,
        trustedDevices: prev.trustedDevices.filter(id => id !== deviceId),
        lastActivity: new Date(),
      } : null);
      
      // If current device was revoked, update trust level
      if (deviceId === user.deviceFingerprint) {
        setDeviceTrust(DeviceTrust.UNTRUSTED);
        setUser(prev => prev ? {
          ...prev,
          deviceTrust: DeviceTrust.UNTRUSTED,
        } : null);
        
        // Recalculate risk score
        const newRiskScore = await calculateRiskScore();
        setCurrentRisk(newRiskScore);
        
        setSecurityWarnings(prev => [
          ...prev,
          'Current device trust has been revoked',
          'Enhanced security measures are now active',
          'Re-authentication may be required for sensitive operations',
        ]);
      }
      
      logAdminAction('device_trust_revoked', 'security', {
        revoked_device_id: deviceId,
        is_current_device: deviceId === user.deviceFingerprint,
        remaining_trusted_devices: user.trustedDevices.filter(id => id !== deviceId).length,
      });
      
      console.log(`Device trust revoked: ${deviceId}`);
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Device trust revocation failed';
      setError(errorMessage);
      
      logAdminAction('device_trust_revocation_failed', 'security', {
        target_device_id: deviceId,
        error: errorMessage,
      });
      
      throw error;
    }
  }, [user, baseApiUrl, calculateRiskScore, logAdminAction]);
  
  const reportSuspiciousActivity = useCallback(async (details: any): Promise<void> => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    try {
      const suspiciousActivityReport = {
        report_id: crypto.randomUUID(),
        user_id: user.id,
        session_id: user.sessionId,
        timestamp: new Date().toISOString(),
        device_fingerprint: user.deviceFingerprint,
        location: user.lastKnownLocation,
        auth_level: authLevel,
        
        // Activity details
        activity_type: details.activityType || 'unknown',
        severity: details.severity || 'medium', // low, medium, high, critical
        description: details.description,
        affected_resources: details.affectedResources || [],
        potential_threat: details.potentialThreat || 'unknown',
        
        // Context information
        context: {
          ip_address: 'unknown', // Would be extracted from request
          user_agent: navigator.userAgent,
          referer: window.location.href,
          current_risk_score: currentRisk,
          recent_auth_changes: details.recentAuthChanges || false,
          unusual_location: details.unusualLocation || false,
          time_anomaly: details.timeAnomaly || false,
        },
        
        // Evidence
        evidence: details.evidence || {},
        
        // Automatic response taken
        automatic_response: details.automaticResponse || 'none',
      };
      
      const response = await fetch(`${baseApiUrl}/api/v1/security/report-suspicious-activity`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${user.sessionId}`,
        },
        body: JSON.stringify(suspiciousActivityReport),
      });
      
      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || 'Suspicious activity report failed');
      }
      
      // Handle security response
      if (result.immediate_action_required) {
        switch (result.recommended_action) {
          case 'force_logout':
            setError('Security alert: Forced logout due to suspicious activity');
            await logout(true);
            break;
            
          case 'require_step_up':
            await initiateStepUp(
              AuthLevel.SUPER_ADMIN,
              'Security verification required due to suspicious activity'
            );
            break;
            
          case 'revoke_device_trust':
            await revokeDeviceTrust(user.deviceFingerprint);
            break;
            
          case 'lock_account':
            setError('Account locked due to security concerns - Contact administrator');
            await logout(true);
            break;
        }
      }
      
      // Update security warnings
      setSecurityWarnings(prev => [
        ...prev,
        `Security report submitted: ${details.activityType}`,
        `Report ID: ${suspiciousActivityReport.report_id}`,
        ...(result.additional_warnings || []),
      ]);
      
      logAdminAction('suspicious_activity_reported', 'security', {
        report_id: suspiciousActivityReport.report_id,
        activity_type: details.activityType,
        severity: details.severity,
        immediate_action: result.recommended_action,
        threat_level: result.threat_level,
      });
      
      console.log('Suspicious activity reported successfully:', {
        reportId: suspiciousActivityReport.report_id,
        severity: details.severity,
      });
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Security report failed';
      setError(errorMessage);
      
      logAdminAction('security_report_failed', 'security', {
        activity_type: details.activityType,
        error: errorMessage,
        details: JSON.stringify(details).substring(0, 200), // Truncate
      });
      
      throw error;
    }
  }, [user, authLevel, currentRisk, baseApiUrl, logout, initiateStepUp, revokeDeviceTrust, logAdminAction]);
  
  /**
   * Register a WebAuthn hardware security key
   */
  const registerSecurityKey = useCallback(async (keyName: string): Promise<{ success: boolean; keyInfo?: SecurityKeyInfo }> => {
    if (!user || !webauthnManager.current) {
      throw new Error('User not authenticated or WebAuthn not initialized');
    }
    
    try {
      const result = await webauthnManager.current.registerWebAuthnCredential(
        user.id,
        user.email,
        user.sessionId
      );
      
      if (result.success && result.credentialId) {
        const keyInfo: SecurityKeyInfo = {
          credentialId: result.credentialId,
          keyName,
          registeredAt: new Date(),
          lastUsed: null,
          isActive: true,
          keyType: 'usb', // Could be detected from credential
          authLevel: AuthLevel.SUPER_ADMIN,
        };
        
        // Update user security keys
        setUser(prev => prev ? {
          ...prev,
          securityKeys: [...prev.securityKeys, keyInfo],
          lastActivity: new Date(),
        } : null);
        
        logAdminAction('security_key_registered', 'hardware_security', {
          key_name: keyName,
          credential_id: result.credentialId,
          auth_level: AuthLevel.SUPER_ADMIN,
        });
        
        setSecurityWarnings(prev => [
          ...prev,
          `Hardware security key "${keyName}" registered successfully`,
          'Enhanced authentication is now available',
        ]);
        
        return { success: true, keyInfo };
      }
      
      return { success: false };
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Security key registration failed';
      setError(errorMessage);
      
      logAdminAction('security_key_registration_failed', 'hardware_security', {
        key_name: keyName,
        error: errorMessage,
      });
      
      throw error;
    }
  }, [user, logAdminAction]);
  
  /**
   * Authenticate using WebAuthn hardware security key
   */
  const authenticateWithSecurityKey = useCallback(async (): Promise<{ success: boolean; authLevel?: AuthLevel }> => {
    if (!user || !webauthnManager.current) {
      throw new Error('User not authenticated or WebAuthn not initialized');
    }
    
    try {
      const result = await webauthnManager.current.authenticateWebAuthn(user.email, user.sessionId);
      
      if (result.success) {
        // Elevate authentication level
        const newAuthLevel = AuthLevel.SUPER_ADMIN;
        setAuthLevel(newAuthLevel);
        setUser(prev => prev ? {
          ...prev,
          authLevel: newAuthLevel,
          lastStepUp: new Date(),
          lastActivity: new Date(),
        } : null);
        
        // Update trust score
        const newTrustScore = Math.min(zeroTrustScore + 25, 100);
        setZeroTrustScore(newTrustScore);
        
        logAdminAction('security_key_authentication_success', 'hardware_security', {
          new_auth_level: newAuthLevel,
          trust_score_change: 25,
          new_trust_score: newTrustScore,
        });
        
        setSecurityWarnings(prev => [
          ...prev,
          'Hardware security key authentication successful',
          `Authentication level elevated to: ${AuthLevel[newAuthLevel]}`,
          `Trust score increased to: ${newTrustScore}`,
        ]);
        
        return { success: true, authLevel: newAuthLevel };
      }
      
      return { success: false };
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Security key authentication failed';
      setError(errorMessage);
      
      logAdminAction('security_key_authentication_failed', 'hardware_security', {
        error: errorMessage,
      });
      
      return { success: false };
    }
  }, [user, zeroTrustScore, logAdminAction]);
  
  /**
   * Enable biometric authentication
   */
  const enableBiometricAuth = useCallback(async (): Promise<{ success: boolean; biometricType: string }> => {
    if (!webauthnManager.current) {
      throw new Error('WebAuthn not initialized');
    }
    
    try {
      const biometricResult = await webauthnManager.current.isBiometricAvailable();
      
      if (biometricResult.isAvailable) {
        setBiometricStatus(prev => ({
          ...prev,
          enabled: true,
          available: true,
          verificationLevel: 'enhanced',
          biometricType: biometricResult.biometricType,
        }));
        
        setUser(prev => prev ? {
          ...prev,
          biometricStatus: {
            ...prev.biometricStatus,
            enabled: true,
            available: true,
            verificationLevel: 'enhanced',
            biometricType: biometricResult.biometricType,
          },
        } : null);
        
        logAdminAction('biometric_authentication_enabled', 'biometric_security', {
          biometric_type: biometricResult.biometricType,
          verification_level: 'enhanced',
        });
        
        return { success: true, biometricType: biometricResult.biometricType };
      }
      
      return { success: false, biometricType: 'unavailable' };
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Biometric setup failed';
      setError(errorMessage);
      throw error;
    }
  }, [logAdminAction]);
  
  /**
   * Verify biometric authentication
   */
  const verifyBiometric = useCallback(async (): Promise<{ verified: boolean; confidenceScore: number }> => {
    if (!user || !webauthnManager.current) {
      throw new Error('User not authenticated or WebAuthn not initialized');
    }
    
    try {
      const biometricResult = await webauthnManager.current.isBiometricAvailable();
      
      if (biometricResult.isAvailable) {
        // Simulate biometric verification (in practice would use WebAuthn API)
        const confidenceScore = Math.random() * 0.3 + 0.7; // 70-100% confidence
        
        setBiometricStatus(prev => ({
          ...prev,
          lastVerification: new Date(),
          confidenceScore,
        }));
        
        const newTrustScore = Math.min(zeroTrustScore + Math.round(confidenceScore * 10), 100);
        setZeroTrustScore(newTrustScore);
        
        logAdminAction('biometric_verification', 'biometric_security', {
          confidence_score: confidenceScore,
          trust_score_change: Math.round(confidenceScore * 10),
          new_trust_score: newTrustScore,
        });
        
        return { verified: true, confidenceScore };
      }
      
      return { verified: false, confidenceScore: 0 };
      
    } catch (error) {
      console.error('Biometric verification failed:', error);
      return { verified: false, confidenceScore: 0 };
    }
  }, [user, zeroTrustScore, logAdminAction]);
  
  /**
   * Start behavioral biometrics tracking
   */
  const startBehavioralTracking = useCallback((): void => {
    if (!webauthnManager.current) return;
    
    webauthnManager.current.startBehavioralTracking();
    
    logAdminAction('behavioral_tracking_started', 'behavioral_biometrics', {
      tracking_start: new Date().toISOString(),
    });
    
    console.log('Behavioral biometrics tracking started');
  }, [logAdminAction]);
  
  /**
   * Stop behavioral biometrics tracking
   */
  const stopBehavioralTracking = useCallback((): BehavioralBiometricsData | null => {
    if (!webauthnManager.current) return null;
    
    const behavioralData = webauthnManager.current.stopBehavioralTracking();
    
    if (behavioralData) {
      setUser(prev => prev ? {
        ...prev,
        behavioralProfile: behavioralData,
        lastActivity: new Date(),
      } : null);
      
      logAdminAction('behavioral_tracking_stopped', 'behavioral_biometrics', {
        session_duration: Date.now() - behavioralData.timestamp.getTime(),
        keystroke_samples: behavioralData.keystrokeDynamics.dwellTimes.length,
        mouse_samples: behavioralData.mouseMovement.trajectoryPattern.length,
      });
    }
    
    return behavioralData;
  }, [logAdminAction]);
  
  /**
   * Verify behavioral pattern
   */
  const verifyBehavioralPattern = useCallback(async (): Promise<{ verified: boolean; riskLevel: string }> => {
    if (!user || !webauthnManager.current) {
      return { verified: false, riskLevel: 'high' };
    }
    
    try {
      const result = await webauthnManager.current.verifyBehavioralBiometrics(user.id, user.sessionId);
      
      const trustScoreChange = result.verified ? 15 : -10;
      const newTrustScore = Math.max(0, Math.min(100, zeroTrustScore + trustScoreChange));
      setZeroTrustScore(newTrustScore);
      
      logAdminAction('behavioral_pattern_verification', 'behavioral_biometrics', {
        verification_result: result.verified,
        confidence_score: result.confidenceScore,
        risk_level: result.riskLevel,
        trust_score_change: trustScoreChange,
        new_trust_score: newTrustScore,
      });
      
      return { verified: result.verified, riskLevel: result.riskLevel };
      
    } catch (error) {
      console.error('Behavioral pattern verification failed:', error);
      return { verified: false, riskLevel: 'high' };
    }
  }, [user, zeroTrustScore, logAdminAction]);
  
  /**
   * Perform continuous verification
   */
  const performContinuousVerification = useCallback(async (): Promise<number> => {
    if (!user) return 0;
    
    try {
      let trustScore = zeroTrustScore;
      
      // Device trust factor
      switch (deviceTrust) {
        case DeviceTrust.FULLY_TRUSTED:
          trustScore = Math.min(100, trustScore + 5);
          break;
        case DeviceTrust.TRUSTED:
          trustScore = Math.min(100, trustScore + 2);
          break;
        case DeviceTrust.UNTRUSTED:
          trustScore = Math.max(0, trustScore - 5);
          break;
      }
      
      // Authentication level factor
      const authLevelBonus = authLevel * 3;
      trustScore = Math.min(100, trustScore + authLevelBonus);
      
      // Risk factor
      const riskPenalty = currentRisk * 5;
      trustScore = Math.max(0, trustScore - riskPenalty);
      
      // Session age factor
      const sessionAge = Date.now() - user.authTime.getTime();
      const sessionAgeHours = sessionAge / (1000 * 60 * 60);
      if (sessionAgeHours > 8) {
        trustScore = Math.max(0, trustScore - 10);
      }
      
      setZeroTrustScore(trustScore);
      
      logAdminAction('continuous_verification', 'zero_trust', {
        previous_trust_score: zeroTrustScore,
        new_trust_score: trustScore,
        device_trust: DeviceTrust[deviceTrust],
        auth_level: AuthLevel[authLevel],
        risk_level: SecurityRisk[currentRisk],
        session_age_hours: Math.round(sessionAgeHours * 100) / 100,
      });
      
      return trustScore;
      
    } catch (error) {
      console.error('Continuous verification failed:', error);
      return zeroTrustScore;
    }
  }, [user, zeroTrustScore, deviceTrust, authLevel, currentRisk, logAdminAction]);
  
  /**
   * Trigger trust evaluation
   */
  const triggerTrustEvaluation = useCallback(async (): Promise<{ trustScore: number; recommendations: string[] }> => {
    const currentTrustScore = await performContinuousVerification();
    const recommendations: string[] = [];
    
    if (currentTrustScore < 30) {
      recommendations.push('Immediate re-authentication required');
      recommendations.push('Enable MFA if not already active');
      recommendations.push('Verify device and location');
    } else if (currentTrustScore < 60) {
      recommendations.push('Consider step-up authentication');
      recommendations.push('Enable continuous monitoring');
      recommendations.push('Verify recent activities');
    } else if (currentTrustScore < 80) {
      recommendations.push('Enable biometric authentication');
      recommendations.push('Register hardware security key');
      recommendations.push('Review security settings');
    } else {
      recommendations.push('Trust level is excellent');
      recommendations.push('Consider enabling advanced features');
    }
    
    logAdminAction('trust_evaluation', 'zero_trust', {
      trust_score: currentTrustScore,
      recommendations_count: recommendations.length,
      evaluation_result: currentTrustScore >= 80 ? 'excellent' : currentTrustScore >= 60 ? 'good' : currentTrustScore >= 30 ? 'moderate' : 'poor',
    });
    
    return { trustScore: currentTrustScore, recommendations };
  }, [performContinuousVerification, logAdminAction]);
  
  /**
   * Remove security key
   */
  const removeSecurityKey = useCallback(async (credentialId: string): Promise<boolean> => {
    if (!user) {
      throw new Error('User not authenticated');
    }
    
    try {
      // Update user security keys
      setUser(prev => prev ? {
        ...prev,
        securityKeys: prev.securityKeys.filter(key => key.credentialId !== credentialId),
        lastActivity: new Date(),
      } : null);
      
      logAdminAction('security_key_removed', 'hardware_security', {
        credential_id: credentialId,
        remaining_keys: user.securityKeys.filter(key => key.credentialId !== credentialId).length,
      });
      
      return true;
      
    } catch (error) {
      console.error('Security key removal failed:', error);
      return false;
    }
  }, [user, logAdminAction]);
  
  // Start continuous verification if enabled
  useEffect(() => {
    if (continuousVerification && user) {
      continuousVerificationInterval.current = setInterval(() => {
        performContinuousVerification();
      }, 60000); // Every minute
      
      return () => {
        if (continuousVerificationInterval.current) {
          clearInterval(continuousVerificationInterval.current);
        }
      };
    }
  }, [continuousVerification, user, performContinuousVerification]);
  
  // Initialize authentication
  useEffect(() => {
    const initializeAuth = async () => {
      try {
        // Check for existing session
        const storedSession = localStorage.getItem('progressive_admin_session');
        if (storedSession) {
          // TODO: Validate and restore session
        }
        
        setIsInitialized(true);
      } catch (error) {
        console.error('Failed to initialize progressive admin authentication:', error);
        setError('Authentication initialization failed');
      } finally {
        setIsLoading(false);
      }
    };
    
    initializeAuth();
  }, []);
  
  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (syncInterval.current) {
        clearInterval(syncInterval.current);
      }
      if (sessionTimer.current) {
        clearTimeout(sessionTimer.current);
      }
      if ((window as any).__crossAppEventSource) {
        (window as any).__crossAppEventSource.close();
      }
    };
  }, []);
  
  // Context value
  const value: ProgressiveAdminAuthState = {
    // State
    isAuthenticated,
    isLoading,
    isInitialized,
    user,
    authLevel,
    maxAuthLevel,
    currentRisk,
    deviceTrust,
    requiresStepUp,
    activeChallenge,
    crossAppSession,
    frontendConnected,
    error,
    securityWarnings,
    
    // Authentication methods
    loginWithCredentials,
    loginWithProvider,
    loginFromFrontend,
    
    // Step-up authentication
    initiateStepUp,
    completeStepUp,
    
    // MFA operations
    enableMFA,
    verifyMFA,
    
    // Emergency access
    requestEmergencyAccess,
    
    // Session management
    refreshSession,
    elevatePermissions,
    logout,
    
    // Cross-app operations
    syncWithFrontend,
    authenticateInFrontend,
    
    // Hardware security key operations
    registerSecurityKey,
    authenticateWithSecurityKey,
    removeSecurityKey,
    
    // Biometric operations
    enableBiometricAuth,
    verifyBiometric,
    
    // Behavioral biometrics
    startBehavioralTracking,
    stopBehavioralTracking,
    verifyBehavioralPattern,
    
    // Zero-trust operations
    performContinuousVerification,
    triggerTrustEvaluation,
    
    // Security operations
    trustCurrentDevice,
    revokeDeviceTrust,
    reportSuspiciousActivity,
    
    // Utilities
    hasPermission,
    hasMinAuthLevel,
    canAccessResource,
    logAdminAction,
    clearError,
    clearSecurityWarnings,
    getSessionTimeRemaining,
  };
  
  return (
    <ProgressiveAdminAuthContext.Provider value={value}>
      {children}
    </ProgressiveAdminAuthContext.Provider>
  );
}

/**
 * Hook to use progressive admin authentication context
 */
export function useProgressiveAdminAuth(): ProgressiveAdminAuthState {
  const context = useContext(ProgressiveAdminAuthContext);
  
  if (context === undefined) {
    throw new Error('useProgressiveAdminAuth must be used within ProgressiveAdminAuthProvider');
  }
  
  return context;
}

/**
 * Hook for step-up authentication operations
 */
export function useStepUpAuth() {
  const { 
    authLevel, 
    maxAuthLevel, 
    requiresStepUp, 
    activeChallenge,
    initiateStepUp, 
    completeStepUp,
    hasMinAuthLevel 
  } = useProgressiveAdminAuth();
  
  return {
    authLevel,
    maxAuthLevel,
    requiresStepUp,
    activeChallenge,
    initiateStepUp,
    completeStepUp,
    hasMinAuthLevel,
    canElevate: (targetLevel: AuthLevel) => targetLevel <= maxAuthLevel,
  };
}

/**
 * Hook for cross-app session operations
 */
export function useCrossAppSession() {
  const {
    crossAppSession,
    frontendConnected,
    syncWithFrontend,
    authenticateInFrontend,
  } = useProgressiveAdminAuth();
  
  return {
    crossAppSession,
    frontendConnected,
    syncWithFrontend,
    authenticateInFrontend,
    isSessionActive: Boolean(crossAppSession?.globalSessionId),
  };
}

export default ProgressiveAdminAuthProvider;