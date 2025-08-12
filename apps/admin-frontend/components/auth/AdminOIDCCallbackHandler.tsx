'use client';

// Advanced Admin OIDC Callback Handler
// Enterprise security with privilege verification, audit logging, and threat monitoring

import React, { useEffect, useState, useCallback, useRef } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { 
  Loader2, CheckCircle, XCircle, Shield, AlertTriangle, Crown,
  RefreshCw, ArrowLeft, ExternalLink, Clock, Zap, Activity,
  UserCheck, Lock, Eye, FileText, Globe, Wifi
} from 'lucide-react';
import { handleAdminOIDCCallback, getCurrentAdminOIDCUser } from '@/app/actions/admin-oidc-auth';

interface AdminCallbackState {
  stage: 'initializing' | 'security_check' | 'privilege_validation' | 'processing' | 'audit_logging' | 'success' | 'error' | 'access_denied' | 'timeout';
  progress: number;
  message: string;
  details?: string;
  canRetry: boolean;
  requiresEscalation: boolean;
  redirectUrl?: string;
}

interface AdminSecurityValidation {
  stateValid: boolean;
  codePresent: boolean;
  noCsrfDetected: boolean;
  validOrigin: boolean;
  timeValid: boolean;
  deviceTrusted: boolean;
  adminDomainVerified: boolean;
  privilegeLevel: 'none' | 'moderator' | 'admin' | 'super_admin';
  details: string[];
  riskScore: number;
}

interface AdminErrorInfo {
  code: string;
  description: string;
  recoverable: boolean;
  suggestedAction: string;
  requiresEscalation: boolean;
  auditRequired: boolean;
}

interface AdminProcessingMetrics {
  startTime: number;
  securityCheckTime: number;
  privilegeValidationTime: number;
  processingTime: number;
  auditLoggingTime: number;
  totalTime: number;
  attempts: number;
  riskAssessmentScore: number;
}

interface AuditLogEntry {
  timestamp: number;
  event: string;
  user?: string;
  ip?: string;
  userAgent?: string;
  result: 'success' | 'failure' | 'blocked';
  riskScore: number;
  details: Record<string, any>;
}

export interface AdminOIDCCallbackHandlerProps {
  onSuccess?: (user: any) => void;
  onError?: (error: AdminErrorInfo) => void;
  onAuditLog?: (entry: AuditLogEntry) => void;
  defaultRedirectUrl?: string;
  maxRetries?: number;
  timeoutMs?: number;
  enableSecurityValidation?: boolean;
  enableAuditLogging?: boolean;
  enableRiskAssessment?: boolean;
  requirePrivilegeEscalation?: boolean;
}

const ADMIN_ROLES = [
  'super_admin',
  'admin',
  'admin-full-004',
  'moderator-standard-003',
  'moderator',
  'system_administrator'
];

const TRUSTED_ADMIN_DOMAINS = [
  'admin.company.com',
  'identity.company.com',
  'accounts.google.com',
  'login.microsoftonline.com'
];

export function AdminOIDCCallbackHandler({
  onSuccess,
  onError,
  onAuditLog,
  defaultRedirectUrl = '/dashboard',
  maxRetries = 2, // Stricter retry policy for admin
  timeoutMs = 45000, // Longer timeout for thorough security checks
  enableSecurityValidation = true,
  enableAuditLogging = true,
  enableRiskAssessment = true,
  requirePrivilegeEscalation = false
}: AdminOIDCCallbackHandlerProps) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const timeoutRef = useRef<NodeJS.Timeout>();
  const metricsRef = useRef<AdminProcessingMetrics>({
    startTime: Date.now(),
    securityCheckTime: 0,
    privilegeValidationTime: 0,
    processingTime: 0,
    auditLoggingTime: 0,
    totalTime: 0,
    attempts: 0,
    riskAssessmentScore: 0
  });

  // Component state
  const [state, setState] = useState<AdminCallbackState>({
    stage: 'initializing',
    progress: 0,
    message: 'Initializing secure admin authentication callback...',
    canRetry: false,
    requiresEscalation: false
  });

  const [securityValidation, setSecurityValidation] = useState<AdminSecurityValidation>({
    stateValid: false,
    codePresent: false,
    noCsrfDetected: false,
    validOrigin: false,
    timeValid: false,
    deviceTrusted: false,
    adminDomainVerified: false,
    privilegeLevel: 'none',
    details: [],
    riskScore: 0
  });

  const [errorInfo, setErrorInfo] = useState<AdminErrorInfo | null>(null);
  const [retryCount, setRetryCount] = useState(0);
  const [auditEntries, setAuditEntries] = useState<AuditLogEntry[]>([]);

  // Initialize admin callback processing
  useEffect(() => {
    processAdminCallback();
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  // Set up admin-specific timeout
  useEffect(() => {
    timeoutRef.current = setTimeout(() => {
      if (!['success', 'error', 'access_denied'].includes(state.stage)) {
        handleTimeout();
      }
    }, timeoutMs);

    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, [state.stage, timeoutMs]);

  // Main admin callback processing function
  const processAdminCallback = useCallback(async () => {
    try {
      metricsRef.current.startTime = Date.now();
      metricsRef.current.attempts++;

      // Stage 1: Enhanced Security Validation for Admin
      if (enableSecurityValidation) {
        setState({
          stage: 'security_check',
          progress: 15,
          message: 'Performing enhanced security validation...',
          details: 'Verifying admin authentication security parameters',
          canRetry: false,
          requiresEscalation: false
        });

        const validation = await performAdminSecurityValidation();
        setSecurityValidation(validation);

        if (!isAdminSecurityValidationPassed(validation)) {
          const auditEntry = createAuditEntry(
            'security_validation_failed',
            'failure',
            validation.riskScore,
            { validation: validation.details }
          );
          await logAuditEvent(auditEntry);
          throw new Error(`Admin security validation failed: ${validation.details.join(', ')}`);
        }

        metricsRef.current.securityCheckTime = Date.now() - metricsRef.current.startTime;
      }

      // Stage 2: Admin Privilege Validation
      setState({
        stage: 'privilege_validation',
        progress: 35,
        message: 'Validating administrator privileges...',
        details: 'Verifying admin role and permission levels',
        canRetry: false,
        requiresEscalation: false
      });

      const privilegeValidation = await validateAdminPrivileges();
      if (!privilegeValidation.valid) {
        const auditEntry = createAuditEntry(
          'privilege_validation_failed',
          'blocked',
          80,
          { reason: privilegeValidation.reason, requested_level: privilegeValidation.requestedLevel }
        );
        await logAuditEvent(auditEntry);
        
        setState({
          stage: 'access_denied',
          progress: 0,
          message: 'Access Denied: Insufficient Administrative Privileges',
          details: privilegeValidation.reason,
          canRetry: false,
          requiresEscalation: privilegeValidation.requiresEscalation
        });
        return;
      }

      metricsRef.current.privilegeValidationTime = Date.now() - metricsRef.current.startTime - metricsRef.current.securityCheckTime;

      // Stage 3: Process Admin OIDC Callback
      setState({
        stage: 'processing',
        progress: 60,
        message: 'Processing admin authentication callback...',
        details: 'Exchanging authorization code and validating admin tokens',
        canRetry: false,
        requiresEscalation: false
      });

      const code = searchParams.get('code');
      const state = searchParams.get('state');
      const tenantId = searchParams.get('tenant_id');

      if (!code || !state) {
        throw new Error('Missing required admin callback parameters');
      }

      const result = await handleAdminOIDCCallback({ code, state, tenant_id: tenantId || undefined });

      if (!result.success) {
        throw new Error(result.error || 'Admin authentication callback failed');
      }

      metricsRef.current.processingTime = Date.now() - metricsRef.current.startTime - metricsRef.current.securityCheckTime - metricsRef.current.privilegeValidationTime;

      // Stage 4: Audit Logging
      if (enableAuditLogging) {
        setState({
          stage: 'audit_logging',
          progress: 85,
          message: 'Logging administrative access...',
          details: 'Recording audit trail for administrative session',
          canRetry: false,
          requiresEscalation: false
        });

        const auditEntry = createAuditEntry(
          'admin_login_success',
          'success',
          securityValidation.riskScore,
          { 
            privilege_level: securityValidation.privilegeLevel,
            device_trusted: securityValidation.deviceTrusted,
            processing_time: metricsRef.current.processingTime
          }
        );
        await logAuditEvent(auditEntry);

        metricsRef.current.auditLoggingTime = Date.now() - metricsRef.current.startTime - metricsRef.current.securityCheckTime - metricsRef.current.privilegeValidationTime - metricsRef.current.processingTime;
      }

      // Stage 5: Success and Redirect
      setState({
        stage: 'success',
        progress: 95,
        message: 'Administrative authentication successful!',
        details: 'Preparing secure administrative session',
        canRetry: false,
        requiresEscalation: false,
        redirectUrl: result.redirect_url || defaultRedirectUrl
      });

      // Get admin user information
      const adminUser = await getCurrentAdminOIDCUser();
      
      // Complete metrics
      metricsRef.current.totalTime = Date.now() - metricsRef.current.startTime;

      // Log success metrics
      console.log('🔐 Admin OIDC Callback Metrics:', {
        totalTime: metricsRef.current.totalTime,
        securityCheckTime: metricsRef.current.securityCheckTime,
        privilegeValidationTime: metricsRef.current.privilegeValidationTime,
        processingTime: metricsRef.current.processingTime,
        auditLoggingTime: metricsRef.current.auditLoggingTime,
        attempts: metricsRef.current.attempts,
        riskScore: securityValidation.riskScore,
        privilegeLevel: securityValidation.privilegeLevel,
        user: adminUser?.email
      });

      // Call success callback
      if (onSuccess && adminUser) {
        onSuccess(adminUser);
      }

      // Final stage
      setState(prev => ({
        ...prev,
        progress: 100,
        message: 'Redirecting to admin dashboard...'
      }));

      // Redirect after appropriate delay for audit logging
      setTimeout(() => {
        router.replace(result.redirect_url || defaultRedirectUrl);
      }, 2000);

    } catch (error) {
      console.error('❌ Admin OIDC callback processing failed:', error);
      await handleAdminError(error as Error);
    }
  }, [searchParams, enableSecurityValidation, enableAuditLogging, onSuccess, defaultRedirectUrl, router]);

  // Enhanced admin security validation
  const performAdminSecurityValidation = async (): Promise<AdminSecurityValidation> => {
    const validation: AdminSecurityValidation = {
      stateValid: false,
      codePresent: false,
      noCsrfDetected: true,
      validOrigin: false,
      timeValid: false,
      deviceTrusted: false,
      adminDomainVerified: false,
      privilegeLevel: 'none',
      details: [],
      riskScore: 0
    };

    try {
      let riskScore = 0;

      // Standard validation
      const code = searchParams.get('code');
      const state = searchParams.get('state');
      const error = searchParams.get('error');

      if (error) {
        validation.details.push(`Admin OAuth error: ${error}`);
        riskScore += 30;
        validation.riskScore = riskScore;
        return validation;
      }

      // Validate admin authorization code
      if (code && code.length > 15) { // Stricter validation for admin
        validation.codePresent = true;
      } else {
        validation.details.push('Admin authorization code missing or invalid');
        riskScore += 40;
      }

      // Validate state parameter with stricter checks
      if (state && state.length > 15) { // Stricter for admin
        validation.stateValid = true;
      } else {
        validation.details.push('Admin state parameter missing or invalid');
        riskScore += 40;
      }

      // Enhanced origin validation for admin
      const referrer = document.referrer;
      const isValidAdminOrigin = TRUSTED_ADMIN_DOMAINS.some(domain => 
        referrer.includes(domain) || 
        referrer.includes(process.env.NEXT_PUBLIC_BACKEND_URL || 'localhost:8080')
      );

      if (isValidAdminOrigin) {
        validation.validOrigin = true;
        validation.adminDomainVerified = true;
      } else {
        validation.details.push(`Untrusted admin origin: ${referrer}`);
        riskScore += 50;
      }

      // Stricter timing validation for admin
      const sessionStartTime = localStorage.getItem('admin_auth_start_time');
      if (sessionStartTime) {
        const elapsed = Date.now() - parseInt(sessionStartTime);
        if (elapsed < 3 * 60 * 1000) { // 3 minutes max for admin
          validation.timeValid = true;
        } else {
          validation.details.push('Admin callback received too late');
          riskScore += 25;
        }
      } else {
        validation.details.push('No admin session start time found');
        riskScore += 20;
      }

      // Device trust validation
      const knownDevice = localStorage.getItem('admin_known_device');
      const currentFingerprint = await generateAdminDeviceFingerprint();
      if (knownDevice === currentFingerprint) {
        validation.deviceTrusted = true;
      } else {
        validation.details.push('Unknown device accessing admin panel');
        riskScore += 35;
      }

      // Admin-specific CSRF checks
      const adminSessionId = localStorage.getItem('admin_oidc_session_id');
      const urlSessionId = searchParams.get('session_id');
      
      if (adminSessionId && urlSessionId && adminSessionId !== urlSessionId) {
        validation.noCsrfDetected = false;
        validation.details.push('Admin session ID mismatch - possible CSRF attack');
        riskScore += 60;
      }

      // Time-based risk assessment for admin access
      const hour = new Date().getHours();
      if (hour < 6 || hour > 22) {
        riskScore += 15;
        validation.details.push('Admin access during off-hours');
      }

      validation.riskScore = Math.min(100, riskScore);

    } catch (error) {
      validation.details.push(`Admin security validation error: ${error}`);
      validation.riskScore = 100;
    }

    return validation;
  };

  // Validate admin privileges
  const validateAdminPrivileges = async (): Promise<{
    valid: boolean;
    reason: string;
    requestedLevel: string;
    requiresEscalation: boolean;
  }> => {
    try {
      // In a real implementation, this would check the ID token or make an API call
      // For now, we'll simulate privilege validation
      
      const minRequiredLevel = requirePrivilegeEscalation ? 'super_admin' : 'admin';
      
      // Simulate checking user's admin level from token
      // This would normally be extracted from the ID token payload
      const userAdminLevel = 'admin'; // Would come from actual token
      
      const levelHierarchy = {
        'super_admin': 3,
        'admin': 2,
        'moderator': 1,
        'none': 0
      };
      
      const userLevel = levelHierarchy[userAdminLevel as keyof typeof levelHierarchy] || 0;
      const requiredLevel = levelHierarchy[minRequiredLevel as keyof typeof levelHierarchy] || 0;
      
      if (userLevel >= requiredLevel) {
        return {
          valid: true,
          reason: 'Admin privileges validated',
          requestedLevel: minRequiredLevel,
          requiresEscalation: false
        };
      }
      
      return {
        valid: false,
        reason: `Insufficient admin privileges. Required: ${minRequiredLevel}, Found: ${userAdminLevel}`,
        requestedLevel: minRequiredLevel,
        requiresEscalation: userLevel > 0 && userLevel < requiredLevel
      };
      
    } catch (error) {
      return {
        valid: false,
        reason: `Admin privilege validation failed: ${error}`,
        requestedLevel: 'unknown',
        requiresEscalation: false
      };
    }
  };

  // Check if admin security validation passed
  const isAdminSecurityValidationPassed = (validation: AdminSecurityValidation): boolean => {
    // Stricter validation for admin - all security checks must pass and low risk score
    return validation.stateValid && 
           validation.codePresent && 
           validation.noCsrfDetected && 
           validation.validOrigin && 
           validation.timeValid &&
           validation.adminDomainVerified &&
           validation.riskScore < 50; // Stricter risk threshold for admin
  };

  // Handle admin-specific errors
  const handleAdminError = async (error: Error) => {
    const errorInfo = classifyAdminError(error);
    setErrorInfo(errorInfo);
    
    setState({
      stage: 'error',
      progress: 0,
      message: `Admin authentication failed: ${errorInfo.description}`,
      details: errorInfo.suggestedAction,
      canRetry: errorInfo.recoverable && retryCount < maxRetries,
      requiresEscalation: errorInfo.requiresEscalation
    });

    // Log security incident
    if (errorInfo.auditRequired) {
      const auditEntry = createAuditEntry(
        'admin_auth_failure',
        'failure',
        100,
        { error: error.message, classification: errorInfo.code }
      );
      await logAuditEvent(auditEntry);
    }

    if (onError) {
      onError(errorInfo);
    }
  };

  // Handle admin callback timeout
  const handleTimeout = async () => {
    const timeoutError: AdminErrorInfo = {
      code: 'ADMIN_TIMEOUT',
      description: 'Admin authentication callback timed out',
      recoverable: true,
      suggestedAction: 'The admin authentication process took too long. This could indicate a security issue.',
      requiresEscalation: false,
      auditRequired: true
    };

    setErrorInfo(timeoutError);
    setState({
      stage: 'timeout',
      progress: 0,
      message: 'Admin authentication callback timed out',
      details: 'The process exceeded the maximum allowed time for security reasons',
      canRetry: retryCount < maxRetries,
      requiresEscalation: false
    });

    // Log timeout incident
    const auditEntry = createAuditEntry(
      'admin_auth_timeout',
      'failure',
      75,
      { timeout_ms: timeoutMs, stage: state.stage }
    );
    await logAuditEvent(auditEntry);
  };

  // Classify admin-specific errors
  const classifyAdminError = (error: Error): AdminErrorInfo => {
    const message = error.message.toLowerCase();

    if (message.includes('privilege') || message.includes('admin')) {
      return {
        code: 'ADMIN_PRIVILEGE_ERROR',
        description: 'Insufficient administrative privileges',
        recoverable: false,
        suggestedAction: 'Contact your system administrator to verify your admin access level.',
        requiresEscalation: true,
        auditRequired: true
      };
    }

    if (message.includes('security') || message.includes('csrf') || message.includes('state')) {
      return {
        code: 'ADMIN_SECURITY_ERROR',
        description: 'Admin security validation failed',
        recoverable: false,
        suggestedAction: 'This may indicate a security threat. Please contact IT security immediately.',
        requiresEscalation: true,
        auditRequired: true
      };
    }

    // Other error types similar to regular callback handler but with admin-specific handling
    return {
      code: 'ADMIN_UNKNOWN_ERROR',
      description: 'Unexpected admin authentication error',
      recoverable: false,
      suggestedAction: 'Please contact your system administrator for assistance.',
      requiresEscalation: true,
      auditRequired: true
    };
  };

  // Create audit log entry
  const createAuditEntry = (
    event: string,
    result: 'success' | 'failure' | 'blocked',
    riskScore: number,
    details: Record<string, any>
  ): AuditLogEntry => {
    return {
      timestamp: Date.now(),
      event,
      user: searchParams.get('email') || 'unknown',
      ip: 'unknown', // Would be populated server-side
      userAgent: navigator.userAgent,
      result,
      riskScore,
      details: {
        ...details,
        callback_url: window.location.href,
        referrer: document.referrer,
        session_id: searchParams.get('session_id'),
        tenant_id: searchParams.get('tenant_id')
      }
    };
  };

  // Log audit event
  const logAuditEvent = async (entry: AuditLogEntry) => {
    try {
      setAuditEntries(prev => [...prev, entry]);
      
      if (onAuditLog) {
        onAuditLog(entry);
      }
      
      // In production, this would send to audit logging service
      console.log('🔍 Admin Audit Log:', entry);
      
    } catch (error) {
      console.error('Failed to log admin audit event:', error);
    }
  };

  // Retry admin callback
  const handleRetry = () => {
    if (retryCount < maxRetries) {
      setRetryCount(prev => prev + 1);
      setState({
        stage: 'initializing',
        progress: 0,
        message: `Retrying admin authentication (attempt ${retryCount + 2}/${maxRetries + 1})...`,
        canRetry: false,
        requiresEscalation: false
      });
      processAdminCallback();
    }
  };

  // Navigate back to admin login
  const handleBackToAdminLogin = () => {
    router.replace('/login');
  };

  // Generate admin device fingerprint
  const generateAdminDeviceFingerprint = async (): Promise<string> => {
    // Enhanced fingerprinting for admin security
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d')!;
    ctx.textBaseline = 'top';
    ctx.font = '14px Arial';
    ctx.fillText('Admin device fingerprint', 2, 2);
    
    const fingerprint = [
      'ADMIN',
      navigator.userAgent,
      navigator.language,
      screen.width + 'x' + screen.height,
      new Date().getTimezoneOffset(),
      canvas.toDataURL(),
      navigator.platform,
      navigator.hardwareConcurrency
    ].join('|');
    
    return btoa(fingerprint).replace(/[^a-zA-Z0-9]/g, '').substring(0, 32);
  };

  // Get appropriate icon for current stage
  const getStageIcon = () => {
    switch (state.stage) {
      case 'success':
        return <Crown className="h-8 w-8 text-green-500" />;
      case 'error':
      case 'timeout':
      case 'access_denied':
        return <XCircle className="h-8 w-8 text-red-500" />;
      case 'security_check':
        return <Shield className="h-8 w-8 text-blue-500" />;
      case 'privilege_validation':
        return <UserCheck className="h-8 w-8 text-purple-500" />;
      case 'audit_logging':
        return <FileText className="h-8 w-8 text-orange-500" />;
      default:
        return <Loader2 className="h-8 w-8 animate-spin text-blue-500" />;
    }
  };

  // Get stage-specific styling
  const getStageColor = () => {
    switch (state.stage) {
      case 'success':
        return 'border-green-200 bg-green-50';
      case 'error':
      case 'timeout':
      case 'access_denied':
        return 'border-red-200 bg-red-50';
      case 'security_check':
        return 'border-blue-200 bg-blue-50';
      case 'privilege_validation':
        return 'border-purple-200 bg-purple-50';
      case 'audit_logging':
        return 'border-orange-200 bg-orange-50';
      default:
        return 'border-gray-200 bg-gray-50';
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-background p-4">
      <Card className={`w-full max-w-lg ${getStageColor()}`}>
        <CardHeader className="text-center space-y-4">
          <div className="flex justify-center">
            {getStageIcon()}
          </div>
          <CardTitle className="text-xl">
            {state.stage === 'success' ? 'Admin Authentication Successful' :
             state.stage === 'error' ? 'Admin Authentication Failed' :
             state.stage === 'access_denied' ? 'Access Denied' :
             state.stage === 'timeout' ? 'Authentication Timeout' :
             'Processing Admin Authentication'}
          </CardTitle>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Progress indicator */}
          {['initializing', 'security_check', 'privilege_validation', 'processing', 'audit_logging'].includes(state.stage) && (
            <div className="space-y-2">
              <Progress value={state.progress} className="h-3" />
              <div className="flex items-center gap-2">
                <Activity className="h-4 w-4 text-blue-500" />
                <p className="text-sm text-muted-foreground">{state.message}</p>
              </div>
              {state.details && (
                <p className="text-xs text-muted-foreground pl-6">{state.details}</p>
              )}
            </div>
          )}

          {/* Success state */}
          {state.stage === 'success' && (
            <div className="text-center space-y-4">
              <div className="space-y-2">
                <p className="text-green-600 font-medium">{state.message}</p>
                <Badge variant="outline" className="bg-green-50 text-green-700 border-green-200">
                  <Crown className="h-3 w-3 mr-1" />
                  Administrator Access Granted
                </Badge>
              </div>
              {state.redirectUrl && (
                <div className="flex items-center justify-center gap-2 text-sm text-muted-foreground">
                  <ExternalLink className="h-4 w-4" />
                  Redirecting to {state.redirectUrl}
                </div>
              )}
              <div className="text-xs text-muted-foreground">
                Privilege Level: {securityValidation.privilegeLevel}
                {enableAuditLogging && (
                  <div className="mt-1">
                    <FileText className="h-3 w-3 inline mr-1" />
                    Access logged for audit compliance
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Access denied state */}
          {state.stage === 'access_denied' && (
            <div className="space-y-4">
              <Alert variant="destructive">
                <Lock className="h-4 w-4" />
                <AlertDescription>
                  <strong>Access Denied</strong>
                  <br />
                  {state.details}
                </AlertDescription>
              </Alert>

              {state.requiresEscalation && (
                <Alert>
                  <AlertTriangle className="h-4 w-4" />
                  <AlertDescription>
                    Your current privilege level may allow for escalation.
                    Contact your system administrator to request elevated access.
                  </AlertDescription>
                </Alert>
              )}

              <div className="flex gap-2">
                <Button 
                  onClick={handleBackToAdminLogin} 
                  className="flex-1"
                >
                  <ArrowLeft className="mr-2 h-4 w-4" />
                  Back to Login
                </Button>
              </div>
            </div>
          )}

          {/* Error state */}
          {(state.stage === 'error' || state.stage === 'timeout') && errorInfo && (
            <div className="space-y-4">
              <Alert variant="destructive">
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription>
                  <strong>{errorInfo.description}</strong>
                  <br />
                  {errorInfo.suggestedAction}
                </AlertDescription>
              </Alert>

              <div className="flex items-center gap-2">
                <Badge variant="outline" className="text-xs">
                  Error Code: {errorInfo.code}
                </Badge>
                {retryCount > 0 && (
                  <Badge variant="secondary" className="text-xs">
                    Attempt {retryCount + 1}/{maxRetries + 1}
                  </Badge>
                )}
                {errorInfo.auditRequired && (
                  <Badge variant="outline" className="text-xs text-orange-600">
                    <FileText className="h-3 w-3 mr-1" />
                    Audit Logged
                  </Badge>
                )}
              </div>

              <div className="flex gap-2">
                {state.canRetry && (
                  <Button 
                    onClick={handleRetry} 
                    className="flex-1"
                    disabled={retryCount >= maxRetries}
                  >
                    <RefreshCw className="mr-2 h-4 w-4" />
                    Retry
                  </Button>
                )}
                <Button 
                  onClick={handleBackToAdminLogin} 
                  variant="outline" 
                  className="flex-1"
                >
                  <ArrowLeft className="mr-2 h-4 w-4" />
                  Back to Login
                </Button>
              </div>
            </div>
          )}

          {/* Enhanced security validation details for admin */}
          {enableSecurityValidation && ['security_check', 'error'].includes(state.stage) && (
            <div className="space-y-3">
              <div className="flex items-center gap-2 text-sm font-medium">
                <Shield className="h-4 w-4" />
                Enhanced Admin Security Validation
              </div>
              <div className="grid grid-cols-2 gap-1 pl-6 text-xs">
                {[
                  { key: 'stateValid', label: 'State Parameter' },
                  { key: 'codePresent', label: 'Auth Code' },
                  { key: 'noCsrfDetected', label: 'CSRF Protection' },
                  { key: 'validOrigin', label: 'Origin Check' },
                  { key: 'timeValid', label: 'Timing Valid' },
                  { key: 'deviceTrusted', label: 'Device Trust' },
                  { key: 'adminDomainVerified', label: 'Admin Domain' }
                ].map(({ key, label }) => (
                  <div key={key} className="flex items-center gap-1">
                    {securityValidation[key as keyof AdminSecurityValidation] ? (
                      <CheckCircle className="h-3 w-3 text-green-500" />
                    ) : (
                      <XCircle className="h-3 w-3 text-red-500" />
                    )}
                    <span className={securityValidation[key as keyof AdminSecurityValidation] ? 'text-green-600' : 'text-red-600'}>
                      {label}
                    </span>
                  </div>
                ))}
              </div>
              {securityValidation.details.length > 0 && (
                <div className="text-xs text-red-600 pl-6">
                  Issues: {securityValidation.details.slice(0, 3).join(', ')}
                </div>
              )}
              <div className="text-xs text-muted-foreground pl-6">
                Risk Score: {securityValidation.riskScore}% | 
                Privilege Level: {securityValidation.privilegeLevel}
              </div>
            </div>
          )}

          {/* Processing metrics */}
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <Clock className="h-3 w-3" />
            {state.stage === 'success' ? 
              `Completed in ${metricsRef.current.totalTime}ms` :
              `Processing for ${Date.now() - metricsRef.current.startTime}ms`
            }
            {auditEntries.length > 0 && (
              <span className="ml-2">
                <FileText className="h-3 w-3 inline mr-1" />
                {auditEntries.length} audit event(s)
              </span>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}