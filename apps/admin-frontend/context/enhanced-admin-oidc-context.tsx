'use client';

// Enhanced Admin OIDC Authentication Context
// Enterprise-grade security with privilege escalation, audit logging, and threat monitoring

import React, { 
  createContext, useContext, useState, useEffect, useCallback, 
  ReactNode, useRef, useMemo 
} from 'react';
import { useRouter } from 'next/navigation';
import { 
  getCurrentAdminOIDCUser, 
  logoutAdminOIDC,
  initiateAdminOIDCLogin,
  checkAdminPermission
} from '@/app/actions/admin-oidc-auth';

interface AdminUser {
  id: string;
  email: string;
  name?: string;
  role: string;
  permissions: string[];
  admin_level: 'moderator' | 'admin' | 'super_admin';
  tenant_id?: string;
  provider: string;
  last_activity: number;
}

interface AdminSecurityMetrics {
  loginAttempts: number;
  failedAttempts: number;
  lastLoginTime: number;
  sessionDuration: number;
  privilegeEscalations: number;
  auditEvents: number;
  riskScore: number;
  threatLevel: 'low' | 'medium' | 'high' | 'critical';
  complianceScore: number;
}

interface AdminAuditEvent {
  id: string;
  timestamp: number;
  event: string;
  user: string;
  resource?: string;
  action?: string;
  result: 'success' | 'failure' | 'blocked' | 'warning';
  riskScore: number;
  metadata: Record<string, any>;
  ipAddress?: string;
  userAgent?: string;
}

interface AdminSessionInfo {
  sessionId: string;
  startTime: number;
  lastActivity: number;
  expiryTime: number;
  maxInactivity: number;
  privilegeLevel: 'moderator' | 'admin' | 'super_admin';
  mfaVerified: boolean;
  deviceTrusted: boolean;
  locationVerified: boolean;
  complianceMode: boolean;
}

interface AdminContextState {
  isLoading: boolean;
  isAuthenticated: boolean;
  isInitialized: boolean;
  user: AdminUser | null;
  session: AdminSessionInfo | null;
  error: string | null;
  
  // Security state
  securityMetrics: AdminSecurityMetrics;
  auditEvents: AdminAuditEvent[];
  privilegeLevel: 'moderator' | 'admin' | 'super_admin';
  canEscalatePrivileges: boolean;
  requiresPrivilegeValidation: boolean;
  
  // Session state
  sessionTimeRemaining: number;
  sessionWarningActive: boolean;
  inactivityWarning: boolean;
  emergencyMode: boolean;
  
  // Compliance and monitoring
  complianceMode: boolean;
  auditingEnabled: boolean;
  threatMonitoring: boolean;
  realTimeAlerts: boolean;
}

interface AdminContextMethods {
  // Authentication
  signIn: (redirectUri?: string) => Promise<void>;
  signOut: () => Promise<void>;
  refreshSession: () => Promise<void>;
  
  // Privilege management
  escalatePrivileges: (targetLevel: 'admin' | 'super_admin', reason: string) => Promise<boolean>;
  validatePrivileges: () => Promise<boolean>;
  requestEmergencyAccess: (reason: string) => Promise<boolean>;
  
  // Session management
  extendSession: (reason?: string) => Promise<void>;
  terminateSession: (reason?: string) => Promise<void>;
  validateSession: () => Promise<boolean>;
  
  // Permission and access control
  hasPermission: (permission: string) => boolean;
  hasRole: (role: string) => boolean;
  canAccessResource: (resource: string, action?: string) => boolean;
  canPerformAction: (action: string, resource?: string) => boolean;
  
  // Audit and compliance
  logAuditEvent: (event: string, resource?: string, action?: string, metadata?: Record<string, any>) => Promise<void>;
  getAuditTrail: (filters?: AuditFilters) => AdminAuditEvent[];
  generateComplianceReport: () => Promise<ComplianceReport>;
  
  // Security operations
  reportSecurityIncident: (incident: SecurityIncident) => Promise<void>;
  blockSuspiciousActivity: (reason: string) => Promise<void>;
  enableEmergencyMode: (reason: string) => Promise<void>;
  disableEmergencyMode: () => Promise<void>;
}

interface AdminConfig {
  // Session management
  maxInactivityTime: number; // minutes
  sessionWarningTime: number; // minutes before expiry
  maxSessionDuration: number; // hours
  
  // Security settings
  enableThreatMonitoring: boolean;
  enableRealTimeAuditing: boolean;
  enableComplianceMode: boolean;
  enablePrivilegeEscalation: boolean;
  
  // Risk thresholds
  maxRiskScore: number;
  criticalThreatThreshold: number;
  complianceThreshold: number;
  
  // Audit settings
  auditRetentionDays: number;
  enableRealTimeAlerts: boolean;
  auditCompressionEnabled: boolean;
}

interface AuditFilters {
  startTime?: number;
  endTime?: number;
  user?: string;
  event?: string;
  result?: string;
  minRiskScore?: number;
}

interface ComplianceReport {
  generatedAt: number;
  period: { start: number; end: number };
  summary: {
    totalEvents: number;
    securityEvents: number;
    complianceViolations: number;
    riskEvents: number;
  };
  metrics: AdminSecurityMetrics;
  violations: ComplianceViolation[];
  recommendations: string[];
}

interface ComplianceViolation {
  id: string;
  timestamp: number;
  type: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  user: string;
  remediation: string;
  status: 'open' | 'investigating' | 'resolved' | 'dismissed';
}

interface SecurityIncident {
  type: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  evidence?: Record<string, any>;
  affectedResources?: string[];
}

type AdminContextType = AdminContextState & AdminContextMethods & {
  config: Required<AdminConfig>;
};

const defaultConfig: Required<AdminConfig> = {
  maxInactivityTime: 15, // 15 minutes for admin
  sessionWarningTime: 5, // 5 minutes warning
  maxSessionDuration: 8, // 8 hours max
  enableThreatMonitoring: true,
  enableRealTimeAuditing: true,
  enableComplianceMode: true,
  enablePrivilegeEscalation: true,
  maxRiskScore: 75,
  criticalThreatThreshold: 90,
  complianceThreshold: 95,
  auditRetentionDays: 90,
  enableRealTimeAlerts: true,
  auditCompressionEnabled: true
};

const AdminContext = createContext<AdminContextType | null>(null);

export function EnhancedAdminOIDCProvider({ 
  children, 
  config = {} 
}: { 
  children: ReactNode; 
  config?: Partial<AdminConfig> 
}) {
  const router = useRouter();
  const finalConfig = { ...defaultConfig, ...config };
  
  // Timer references
  const sessionTimerRef = useRef<NodeJS.Timeout>();
  const auditTimerRef = useRef<NodeJS.Timeout>();
  const threatMonitorRef = useRef<NodeJS.Timeout>();
  const complianceTimerRef = useRef<NodeJS.Timeout>();
  
  // State management
  const [state, setState] = useState<AdminContextState>({
    isLoading: true,
    isAuthenticated: false,
    isInitialized: false,
    user: null,
    session: null,
    error: null,
    
    securityMetrics: {
      loginAttempts: 0,
      failedAttempts: 0,
      lastLoginTime: 0,
      sessionDuration: 0,
      privilegeEscalations: 0,
      auditEvents: 0,
      riskScore: 0,
      threatLevel: 'low',
      complianceScore: 100
    },
    
    auditEvents: [],
    privilegeLevel: 'moderator',
    canEscalatePrivileges: false,
    requiresPrivilegeValidation: false,
    
    sessionTimeRemaining: 0,
    sessionWarningActive: false,
    inactivityWarning: false,
    emergencyMode: false,
    
    complianceMode: finalConfig.enableComplianceMode,
    auditingEnabled: finalConfig.enableRealTimeAuditing,
    threatMonitoring: finalConfig.enableThreatMonitoring,
    realTimeAlerts: finalConfig.enableRealTimeAlerts
  });

  // Initialize admin authentication
  useEffect(() => {
    initializeAdminAuth();
    
    return () => {
      cleanup();
    };
  }, []);

  // Start monitoring systems
  useEffect(() => {
    if (state.isAuthenticated) {
      startSessionMonitoring();
      if (finalConfig.enableRealTimeAuditing) startAuditSystem();
      if (finalConfig.enableThreatMonitoring) startThreatMonitoring();
      if (finalConfig.enableComplianceMode) startComplianceMonitoring();
    }
    
    return () => {
      cleanup();
    };
  }, [state.isAuthenticated]);

  // Initialize admin authentication system
  const initializeAdminAuth = async () => {
    try {
      console.log('🔐 Initializing enhanced admin OIDC context...');
      
      // Check for existing admin session
      const existingUser = await getCurrentAdminOIDCUser();
      
      if (existingUser) {
        const sessionInfo = await createSessionInfo(existingUser);
        
        setState(prev => ({
          ...prev,
          isLoading: false,
          isAuthenticated: true,
          isInitialized: true,
          user: existingUser,
          session: sessionInfo,
          privilegeLevel: existingUser.admin_level,
          canEscalatePrivileges: canUserEscalatePrivileges(existingUser),
          securityMetrics: {
            ...prev.securityMetrics,
            lastLoginTime: existingUser.last_activity,
            sessionDuration: Date.now() - existingUser.last_activity
          }
        }));
        
        await logAuditEvent('admin_session_restored', undefined, undefined, {
          user: existingUser.email,
          admin_level: existingUser.admin_level
        });
        
        console.log('✅ Admin session restored:', existingUser.email, existingUser.admin_level);
      } else {
        setState(prev => ({
          ...prev,
          isLoading: false,
          isInitialized: true
        }));
      }
      
    } catch (error) {
      console.error('❌ Admin auth initialization failed:', error);
      setState(prev => ({
        ...prev,
        isLoading: false,
        isInitialized: true,
        error: error instanceof Error ? error.message : 'Admin initialization failed'
      }));
      
      await logAuditEvent('admin_init_failure', undefined, undefined, { error: error });
    }
  };

  // Create session information
  const createSessionInfo = async (user: AdminUser): Promise<AdminSessionInfo> => {
    const now = Date.now();
    const sessionDuration = finalConfig.maxSessionDuration * 60 * 60 * 1000; // hours to ms
    
    return {
      sessionId: generateSessionId(),
      startTime: user.last_activity,
      lastActivity: now,
      expiryTime: user.last_activity + sessionDuration,
      maxInactivity: finalConfig.maxInactivityTime * 60 * 1000, // minutes to ms
      privilegeLevel: user.admin_level,
      mfaVerified: false, // Would check actual MFA status
      deviceTrusted: checkDeviceTrust(),
      locationVerified: true, // Would verify actual location
      complianceMode: finalConfig.enableComplianceMode
    };
  };

  // Check if user can escalate privileges
  const canUserEscalatePrivileges = (user: AdminUser): boolean => {
    if (!finalConfig.enablePrivilegeEscalation) return false;
    
    const escalationMap = {
      'moderator': ['admin'],
      'admin': ['super_admin'],
      'super_admin': []
    };
    
    return escalationMap[user.admin_level].length > 0;
  };

  // Start session monitoring
  const startSessionMonitoring = () => {
    sessionTimerRef.current = setInterval(() => {
      if (!state.isAuthenticated || !state.session) return;
      
      const now = Date.now();
      const inactiveTime = now - state.session.lastActivity;
      const timeToExpiry = state.session.expiryTime - now;
      const warningThreshold = finalConfig.sessionWarningTime * 60 * 1000;
      
      // Check for session expiry
      if (timeToExpiry <= 0) {
        console.log('🔒 Admin session expired');
        signOut();
        return;
      }
      
      // Check for inactivity timeout
      if (inactiveTime > state.session.maxInactivity) {
        console.log('🔒 Admin session timeout due to inactivity');
        logAuditEvent('admin_session_timeout', undefined, undefined, {
          inactiveTime,
          maxInactivity: state.session.maxInactivity
        });
        signOut();
        return;
      }
      
      // Session warning
      if (timeToExpiry <= warningThreshold && !state.sessionWarningActive) {
        setState(prev => ({
          ...prev,
          sessionWarningActive: true,
          sessionTimeRemaining: timeToExpiry
        }));
      }
      
      // Inactivity warning
      const inactivityWarningThreshold = state.session.maxInactivity * 0.8; // 80% of max
      if (inactiveTime > inactivityWarningThreshold && !state.inactivityWarning) {
        setState(prev => ({ ...prev, inactivityWarning: true }));
      }
      
    }, 10000); // Check every 10 seconds for admin
  };

  // Start audit system
  const startAuditSystem = () => {
    auditTimerRef.current = setInterval(async () => {
      // Compress old audit events if enabled
      if (finalConfig.auditCompressionEnabled) {
        await compressOldAuditEvents();
      }
      
      // Calculate current security metrics
      updateSecurityMetrics();
      
    }, 60000); // Every minute
  };

  // Start threat monitoring
  const startThreatMonitoring = () => {
    threatMonitorRef.current = setInterval(async () => {
      const riskScore = calculateCurrentRiskScore();
      
      let threatLevel: 'low' | 'medium' | 'high' | 'critical' = 'low';
      if (riskScore >= finalConfig.criticalThreatThreshold) threatLevel = 'critical';
      else if (riskScore >= 70) threatLevel = 'high';
      else if (riskScore >= 40) threatLevel = 'medium';
      
      setState(prev => ({
        ...prev,
        securityMetrics: {
          ...prev.securityMetrics,
          riskScore,
          threatLevel
        }
      }));
      
      // Handle critical threats
      if (threatLevel === 'critical') {
        await handleCriticalThreat(riskScore);
      }
      
    }, 30000); // Every 30 seconds
  };

  // Start compliance monitoring
  const startComplianceMonitoring = () => {
    complianceTimerRef.current = setInterval(async () => {
      const complianceScore = calculateComplianceScore();
      
      setState(prev => ({
        ...prev,
        securityMetrics: {
          ...prev.securityMetrics,
          complianceScore
        }
      }));
      
      // Alert on compliance violations
      if (complianceScore < finalConfig.complianceThreshold) {
        await logAuditEvent('compliance_violation', undefined, undefined, {
          score: complianceScore,
          threshold: finalConfig.complianceThreshold
        });
      }
      
    }, 5 * 60000); // Every 5 minutes
  };

  // Authentication methods
  const signIn = useCallback(async (redirectUri?: string) => {
    try {
      setState(prev => ({ ...prev, error: null }));
      
      const result = await initiateAdminOIDCLogin({
        redirect_uri: redirectUri
      });
      
      if (!result.success) {
        throw new Error(result.error || 'Admin sign in failed');
      }
      
      // Redirect is handled by the server action
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Admin sign in failed';
      setState(prev => ({ ...prev, error: errorMessage }));
      
      await logAuditEvent('admin_signin_failure', undefined, undefined, { error: errorMessage });
      throw error;
    }
  }, []);

  const signOut = useCallback(async () => {
    try {
      await logAuditEvent('admin_signout', undefined, undefined, {
        sessionDuration: state.session ? Date.now() - state.session.startTime : 0,
        privilegeLevel: state.privilegeLevel
      });
      
      await logoutAdminOIDC();
      
      setState({
        isLoading: false,
        isAuthenticated: false,
        isInitialized: true,
        user: null,
        session: null,
        error: null,
        securityMetrics: {
          loginAttempts: 0,
          failedAttempts: 0,
          lastLoginTime: 0,
          sessionDuration: 0,
          privilegeEscalations: 0,
          auditEvents: 0,
          riskScore: 0,
          threatLevel: 'low',
          complianceScore: 100
        },
        auditEvents: [],
        privilegeLevel: 'moderator',
        canEscalatePrivileges: false,
        requiresPrivilegeValidation: false,
        sessionTimeRemaining: 0,
        sessionWarningActive: false,
        inactivityWarning: false,
        emergencyMode: false,
        complianceMode: finalConfig.enableComplianceMode,
        auditingEnabled: finalConfig.enableRealTimeAuditing,
        threatMonitoring: finalConfig.enableThreatMonitoring,
        realTimeAlerts: finalConfig.enableRealTimeAlerts
      });
      
    } catch (error) {
      console.error('Admin sign out error:', error);
      setState(prev => ({
        ...prev,
        error: error instanceof Error ? error.message : 'Admin sign out failed'
      }));
    }
  }, [state.session, state.privilegeLevel]);

  const refreshSession = useCallback(async () => {
    try {
      const user = await getCurrentAdminOIDCUser();
      if (user) {
        const sessionInfo = await createSessionInfo(user);
        setState(prev => ({
          ...prev,
          user,
          session: sessionInfo,
          privilegeLevel: user.admin_level,
          canEscalatePrivileges: canUserEscalatePrivileges(user)
        }));
        
        await logAuditEvent('admin_session_refreshed');
      }
    } catch (error) {
      console.error('Admin session refresh failed:', error);
      throw error;
    }
  }, []);

  // Permission and access control methods
  const hasPermission = useCallback(async (permission: string): Promise<boolean> => {
    if (!state.isAuthenticated) return false;
    
    try {
      const hasAccess = await checkAdminPermission(permission);
      
      await logAuditEvent('permission_check', permission, undefined, {
        result: hasAccess,
        user: state.user?.email,
        privilegeLevel: state.privilegeLevel
      });
      
      return hasAccess;
    } catch {
      return false;
    }
  }, [state.isAuthenticated, state.user, state.privilegeLevel]);

  const hasRole = useCallback((role: string): boolean => {
    return state.user?.role === role;
  }, [state.user]);

  const canAccessResource = useCallback(async (resource: string, action?: string): Promise<boolean> => {
    const permission = action ? `${resource}:${action}` : `${resource}:read`;
    return hasPermission(permission);
  }, [hasPermission]);

  const canPerformAction = useCallback(async (action: string, resource?: string): Promise<boolean> => {
    const permission = resource ? `${resource}:${action}` : action;
    return hasPermission(permission);
  }, [hasPermission]);

  // Audit and logging
  const logAuditEvent = useCallback(async (
    event: string, 
    resource?: string, 
    action?: string, 
    metadata: Record<string, any> = {}
  ): Promise<void> => {
    try {
      const auditEvent: AdminAuditEvent = {
        id: generateAuditId(),
        timestamp: Date.now(),
        event,
        user: state.user?.email || 'unknown',
        resource,
        action,
        result: 'success',
        riskScore: calculateEventRiskScore(event, metadata),
        metadata: {
          ...metadata,
          sessionId: state.session?.sessionId,
          privilegeLevel: state.privilegeLevel,
          userAgent: navigator.userAgent,
          timestamp: new Date().toISOString()
        },
        ipAddress: 'unknown', // Would be populated server-side
        userAgent: navigator.userAgent
      };
      
      setState(prev => ({
        ...prev,
        auditEvents: [auditEvent, ...prev.auditEvents.slice(0, 99)], // Keep last 100 events
        securityMetrics: {
          ...prev.securityMetrics,
          auditEvents: prev.securityMetrics.auditEvents + 1
        }
      }));
      
      // Send to audit service in production
      console.log('🔍 Admin Audit Event:', auditEvent);
      
    } catch (error) {
      console.error('Failed to log audit event:', error);
    }
  }, [state.user, state.session, state.privilegeLevel]);

  // Utility functions (simplified implementations)
  const generateSessionId = (): string => {
    return `admin_${Date.now()}_${Math.random().toString(36).substring(2)}`;
  };

  const generateAuditId = (): string => {
    return `audit_${Date.now()}_${Math.random().toString(36).substring(2)}`;
  };

  const checkDeviceTrust = (): boolean => {
    try {
      const trustedDevices = JSON.parse(localStorage.getItem('admin_trusted_devices') || '[]');
      const deviceId = generateDeviceId();
      return trustedDevices.includes(deviceId);
    } catch {
      return false;
    }
  };

  const generateDeviceId = (): string => {
    return btoa([
      'ADMIN',
      navigator.userAgent,
      screen.width + 'x' + screen.height,
      new Date().getTimezoneOffset()
    ].join('|')).substring(0, 32);
  };

  const calculateCurrentRiskScore = (): number => {
    let riskScore = 0;
    
    // Session age risk
    if (state.session) {
      const sessionAge = Date.now() - state.session.startTime;
      const maxAge = finalConfig.maxSessionDuration * 60 * 60 * 1000;
      riskScore += (sessionAge / maxAge) * 30;
    }
    
    // Failed attempts risk
    riskScore += state.securityMetrics.failedAttempts * 10;
    
    // Privilege level risk
    const privilegeRisk = {
      'moderator': 0,
      'admin': 10,
      'super_admin': 20
    };
    riskScore += privilegeRisk[state.privilegeLevel];
    
    // Device trust risk
    if (state.session && !state.session.deviceTrusted) {
      riskScore += 25;
    }
    
    return Math.min(100, riskScore);
  };

  const calculateEventRiskScore = (event: string, metadata: Record<string, any>): number => {
    const baseRiskScores = {
      'admin_signin': 10,
      'admin_signin_failure': 30,
      'privilege_escalation': 40,
      'emergency_access': 50,
      'compliance_violation': 60,
      'security_incident': 80
    };
    
    return baseRiskScores[event as keyof typeof baseRiskScores] || 5;
  };

  const calculateComplianceScore = (): number => {
    // Simplified compliance calculation
    let score = 100;
    
    // Deduct for high-risk events
    const highRiskEvents = state.auditEvents.filter(e => e.riskScore > 50).length;
    score -= highRiskEvents * 5;
    
    // Deduct for failed authentication attempts
    score -= state.securityMetrics.failedAttempts * 2;
    
    // Deduct for untrusted device usage
    if (state.session && !state.session.deviceTrusted) {
      score -= 10;
    }
    
    return Math.max(0, score);
  };

  const updateSecurityMetrics = () => {
    if (!state.session) return;
    
    const sessionDuration = Date.now() - state.session.startTime;
    
    setState(prev => ({
      ...prev,
      securityMetrics: {
        ...prev.securityMetrics,
        sessionDuration
      }
    }));
  };

  const compressOldAuditEvents = async () => {
    const retentionTime = finalConfig.auditRetentionDays * 24 * 60 * 60 * 1000;
    const cutoffTime = Date.now() - retentionTime;
    
    setState(prev => ({
      ...prev,
      auditEvents: prev.auditEvents.filter(event => event.timestamp > cutoffTime)
    }));
  };

  const handleCriticalThreat = async (riskScore: number) => {
    console.warn('🚨 Critical threat detected, risk score:', riskScore);
    
    await logAuditEvent('critical_threat_detected', undefined, undefined, {
      riskScore,
      threatLevel: 'critical',
      autoResponse: 'monitoring_enabled'
    });
    
    // In production, this might trigger additional security measures
    // such as requiring re-authentication, notifying security team, etc.
  };

  const cleanup = () => {
    [sessionTimerRef, auditTimerRef, threatMonitorRef, complianceTimerRef].forEach(ref => {
      if (ref.current) clearInterval(ref.current);
    });
  };

  // Placeholder implementations for remaining methods
  const escalatePrivileges = useCallback(async (targetLevel: 'admin' | 'super_admin', reason: string): Promise<boolean> => {
    await logAuditEvent('privilege_escalation_request', undefined, undefined, {
      targetLevel,
      currentLevel: state.privilegeLevel,
      reason
    });
    // Would implement actual privilege escalation logic
    return false;
  }, [state.privilegeLevel, logAuditEvent]);

  const validatePrivileges = useCallback(async (): Promise<boolean> => {
    return state.isAuthenticated && !!state.user;
  }, [state.isAuthenticated, state.user]);

  const requestEmergencyAccess = useCallback(async (reason: string): Promise<boolean> => {
    await logAuditEvent('emergency_access_request', undefined, undefined, { reason });
    return false;
  }, [logAuditEvent]);

  const extendSession = useCallback(async (reason?: string) => {
    if (state.session) {
      const newExpiryTime = Date.now() + (finalConfig.maxSessionDuration * 60 * 60 * 1000);
      setState(prev => ({
        ...prev,
        session: prev.session ? { ...prev.session, expiryTime: newExpiryTime } : null,
        sessionWarningActive: false
      }));
      
      await logAuditEvent('session_extended', undefined, undefined, { reason });
    }
  }, [state.session, logAuditEvent]);

  const terminateSession = useCallback(async (reason?: string) => {
    await logAuditEvent('session_terminated', undefined, undefined, { reason });
    await signOut();
  }, [signOut, logAuditEvent]);

  const validateSession = useCallback(async (): Promise<boolean> => {
    try {
      const user = await getCurrentAdminOIDCUser();
      return !!user;
    } catch {
      return false;
    }
  }, []);

  const getAuditTrail = useCallback((filters?: AuditFilters): AdminAuditEvent[] => {
    let events = state.auditEvents;
    
    if (filters) {
      if (filters.startTime) events = events.filter(e => e.timestamp >= filters.startTime!);
      if (filters.endTime) events = events.filter(e => e.timestamp <= filters.endTime!);
      if (filters.user) events = events.filter(e => e.user.includes(filters.user!));
      if (filters.event) events = events.filter(e => e.event.includes(filters.event!));
      if (filters.result) events = events.filter(e => e.result === filters.result);
      if (filters.minRiskScore) events = events.filter(e => e.riskScore >= filters.minRiskScore!);
    }
    
    return events;
  }, [state.auditEvents]);

  const generateComplianceReport = useCallback(async (): Promise<ComplianceReport> => {
    const now = Date.now();
    const dayMs = 24 * 60 * 60 * 1000;
    
    return {
      generatedAt: now,
      period: { start: now - (30 * dayMs), end: now },
      summary: {
        totalEvents: state.auditEvents.length,
        securityEvents: state.auditEvents.filter(e => e.riskScore > 50).length,
        complianceViolations: 0,
        riskEvents: state.auditEvents.filter(e => e.riskScore > 30).length
      },
      metrics: state.securityMetrics,
      violations: [],
      recommendations: [
        'Enable MFA for all admin accounts',
        'Implement regular access reviews',
        'Monitor privilege escalation requests',
        'Review audit logs regularly'
      ]
    };
  }, [state.auditEvents, state.securityMetrics]);

  const reportSecurityIncident = useCallback(async (incident: SecurityIncident) => {
    await logAuditEvent('security_incident_reported', undefined, undefined, {
      type: incident.type,
      severity: incident.severity,
      description: incident.description
    });
  }, [logAuditEvent]);

  const blockSuspiciousActivity = useCallback(async (reason: string) => {
    await logAuditEvent('suspicious_activity_blocked', undefined, undefined, { reason });
  }, [logAuditEvent]);

  const enableEmergencyMode = useCallback(async (reason: string) => {
    setState(prev => ({ ...prev, emergencyMode: true }));
    await logAuditEvent('emergency_mode_enabled', undefined, undefined, { reason });
  }, [logAuditEvent]);

  const disableEmergencyMode = useCallback(async () => {
    setState(prev => ({ ...prev, emergencyMode: false }));
    await logAuditEvent('emergency_mode_disabled');
  }, [logAuditEvent]);

  // Context value
  const contextValue: AdminContextType = useMemo(() => ({
    ...state,
    config: finalConfig,

    // Authentication
    signIn,
    signOut,
    refreshSession,

    // Privilege management
    escalatePrivileges,
    validatePrivileges,
    requestEmergencyAccess,

    // Session management
    extendSession,
    terminateSession,
    validateSession,

    // Permission and access control
    hasPermission: (permission: string) => hasPermission(permission).then(result => result),
    hasRole,
    canAccessResource: (resource: string, action?: string) => canAccessResource(resource, action).then(result => result),
    canPerformAction: (action: string, resource?: string) => canPerformAction(action, resource).then(result => result),

    // Audit and compliance
    logAuditEvent,
    getAuditTrail,
    generateComplianceReport,

    // Security operations
    reportSecurityIncident,
    blockSuspiciousActivity,
    enableEmergencyMode,
    disableEmergencyMode
  }), [
    state,
    finalConfig,
    signIn,
    signOut,
    refreshSession,
    escalatePrivileges,
    validatePrivileges,
    requestEmergencyAccess,
    extendSession,
    terminateSession,
    validateSession,
    hasPermission,
    hasRole,
    canAccessResource,
    canPerformAction,
    logAuditEvent,
    getAuditTrail,
    generateComplianceReport,
    reportSecurityIncident,
    blockSuspiciousActivity,
    enableEmergencyMode,
    disableEmergencyMode
  ]);

  return (
    <AdminContext.Provider value={contextValue}>
      {children}
    </AdminContext.Provider>
  );
}

// Hook to use admin context
export function useAdminAuth(): AdminContextType {
  const context = useContext(AdminContext);
  if (!context) {
    throw new Error('useAdminAuth must be used within an EnhancedAdminOIDCProvider');
  }
  return context;
}

// Hook for admin permissions
export function useAdminPermissions() {
  const { hasPermission, hasRole, canAccessResource, canPerformAction } = useAdminAuth();
  return { hasPermission, hasRole, canAccessResource, canPerformAction };
}

// Hook for audit and compliance
export function useAdminAudit() {
  const { logAuditEvent, getAuditTrail, generateComplianceReport, auditEvents } = useAdminAuth();
  return { logAuditEvent, getAuditTrail, generateComplianceReport, auditEvents };
}

// Hook for security operations
export function useAdminSecurity() {
  const { 
    securityMetrics, 
    reportSecurityIncident, 
    blockSuspiciousActivity,
    enableEmergencyMode,
    disableEmergencyMode,
    emergencyMode
  } = useAdminAuth();
  
  return { 
    securityMetrics, 
    reportSecurityIncident, 
    blockSuspiciousActivity,
    enableEmergencyMode,
    disableEmergencyMode,
    emergencyMode
  };
}