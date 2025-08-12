'use client';

// Advanced OIDC Callback Handler
// Comprehensive security validation, error recovery, adaptive redirects

import React, { useEffect, useState, useCallback, useRef } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { 
  Loader2, CheckCircle, XCircle, Shield, AlertTriangle, 
  RefreshCw, ArrowLeft, ExternalLink, Clock, Zap, Info
} from 'lucide-react';
import { getOIDCClient } from '@/lib/auth/oidc-client-wrapper';
import { handleOIDCCallback, getCurrentOIDCUser } from '@/app/actions/oidc-auth';

interface CallbackState {
  stage: 'initializing' | 'validating' | 'processing' | 'success' | 'error' | 'timeout';
  progress: number;
  message: string;
  details?: string;
  canRetry: boolean;
  redirectUrl?: string;
}

interface SecurityValidation {
  stateValid: boolean;
  codePresent: boolean;
  noCsrfDetected: boolean;
  validOrigin: boolean;
  timeValid: boolean;
  details: string[];
}

interface ErrorInfo {
  code: string;
  description: string;
  recoverable: boolean;
  suggestedAction: string;
}

interface ProcessingMetrics {
  startTime: number;
  validationTime: number;
  processingTime: number;
  totalTime: number;
  attempts: number;
}

export interface OIDCCallbackHandlerProps {
  onSuccess?: (user: any) => void;
  onError?: (error: ErrorInfo) => void;
  defaultRedirectUrl?: string;
  maxRetries?: number;
  timeoutMs?: number;
  enableSecurityValidation?: boolean;
  enableMetrics?: boolean;
}

export function OIDCCallbackHandler({
  onSuccess,
  onError,
  defaultRedirectUrl = '/dashboard',
  maxRetries = 3,
  timeoutMs = 30000,
  enableSecurityValidation = true,
  enableMetrics = true
}: OIDCCallbackHandlerProps) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const oidcClient = getOIDCClient();
  const timeoutRef = useRef<NodeJS.Timeout>();
  const metricsRef = useRef<ProcessingMetrics>({
    startTime: Date.now(),
    validationTime: 0,
    processingTime: 0,
    totalTime: 0,
    attempts: 0
  });

  // Component state
  const [state, setState] = useState<CallbackState>({
    stage: 'initializing',
    progress: 0,
    message: 'Initializing secure authentication callback...',
    canRetry: false
  });

  const [securityValidation, setSecurityValidation] = useState<SecurityValidation>({
    stateValid: false,
    codePresent: false,
    noCsrfDetected: false,
    validOrigin: false,
    timeValid: false,
    details: []
  });

  const [errorInfo, setErrorInfo] = useState<ErrorInfo | null>(null);
  const [retryCount, setRetryCount] = useState(0);

  // Initialize callback processing
  useEffect(() => {
    processCallback();
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  // Set up timeout
  useEffect(() => {
    timeoutRef.current = setTimeout(() => {
      if (state.stage === 'initializing' || state.stage === 'validating' || state.stage === 'processing') {
        handleTimeout();
      }
    }, timeoutMs);

    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, [state.stage, timeoutMs]);

  // Main callback processing function
  const processCallback = useCallback(async () => {
    try {
      metricsRef.current.startTime = Date.now();
      metricsRef.current.attempts++;

      // Stage 1: Security Validation
      if (enableSecurityValidation) {
        setState({
          stage: 'validating',
          progress: 20,
          message: 'Validating security parameters...',
          details: 'Checking state, CSRF protection, and origin validation',
          canRetry: false
        });

        const validation = await performSecurityValidation();
        setSecurityValidation(validation);

        if (!isSecurityValidationPassed(validation)) {
          throw new Error(`Security validation failed: ${validation.details.join(', ')}`);
        }

        metricsRef.current.validationTime = Date.now() - metricsRef.current.startTime;
      }

      // Stage 2: Process OIDC Callback
      setState({
        stage: 'processing',
        progress: 60,
        message: 'Processing authentication callback...',
        details: 'Exchanging authorization code for tokens',
        canRetry: false
      });

      const code = searchParams.get('code');
      const state = searchParams.get('state');
      const tenantId = searchParams.get('tenant_id');

      if (!code || !state) {
        throw new Error('Missing required callback parameters');
      }

      // Use server action to handle the callback
      const result = await handleOIDCCallback({ code, state, tenant_id: tenantId || undefined });

      if (!result.success) {
        throw new Error(result.error || 'Authentication callback failed');
      }

      metricsRef.current.processingTime = Date.now() - metricsRef.current.startTime - metricsRef.current.validationTime;

      // Stage 3: Finalize and Redirect
      setState({
        stage: 'success',
        progress: 90,
        message: 'Authentication successful! Redirecting...',
        details: 'Finalizing session and preparing redirect',
        canRetry: false,
        redirectUrl: result.redirect_url || defaultRedirectUrl
      });

      // Get user information
      const user = await getCurrentOIDCUser();
      
      // Complete metrics
      metricsRef.current.totalTime = Date.now() - metricsRef.current.startTime;

      // Log success metrics
      if (enableMetrics) {
        console.log('🎯 OIDC Callback Metrics:', {
          totalTime: metricsRef.current.totalTime,
          validationTime: metricsRef.current.validationTime,
          processingTime: metricsRef.current.processingTime,
          attempts: metricsRef.current.attempts,
          user: user?.email
        });
      }

      // Call success callback
      if (onSuccess && user) {
        onSuccess(user);
      }

      // Final stage
      setState(prev => ({
        ...prev,
        progress: 100,
        message: 'Redirecting to application...'
      }));

      // Redirect after short delay for UX
      setTimeout(() => {
        router.replace(result.redirect_url || defaultRedirectUrl);
      }, 1500);

    } catch (error) {
      console.error('❌ OIDC callback processing failed:', error);
      handleError(error as Error);
    }
  }, [searchParams, enableSecurityValidation, enableMetrics, onSuccess, defaultRedirectUrl, router]);

  // Perform comprehensive security validation
  const performSecurityValidation = async (): Promise<SecurityValidation> => {
    const validation: SecurityValidation = {
      stateValid: false,
      codePresent: false,
      noCsrfDetected: true,
      validOrigin: false,
      timeValid: false,
      details: []
    };

    try {
      // Check for required parameters
      const code = searchParams.get('code');
      const state = searchParams.get('state');
      const error = searchParams.get('error');

      if (error) {
        validation.details.push(`OAuth error: ${error}`);
        return validation;
      }

      // Validate code presence
      if (code && code.length > 10) {
        validation.codePresent = true;
      } else {
        validation.details.push('Authorization code missing or invalid');
      }

      // Validate state parameter
      if (state && state.length > 10) {
        validation.stateValid = true;
      } else {
        validation.details.push('State parameter missing or invalid');
      }

      // Validate origin
      const referrer = document.referrer;
      const expectedOrigins = [
        process.env.NEXT_PUBLIC_BACKEND_URL,
        'http://localhost:8080',
        'https://accounts.google.com',
        'https://login.microsoftonline.com'
      ].filter(Boolean);

      if (referrer && expectedOrigins.some(origin => referrer.startsWith(origin!))) {
        validation.validOrigin = true;
      } else {
        validation.details.push(`Invalid or suspicious origin: ${referrer}`);
      }

      // Check timing (callback should happen within reasonable time)
      const sessionStartTime = localStorage.getItem('auth_start_time');
      if (sessionStartTime) {
        const elapsed = Date.now() - parseInt(sessionStartTime);
        if (elapsed < 5 * 60 * 1000) { // 5 minutes
          validation.timeValid = true;
        } else {
          validation.details.push('Callback received too late - possible replay attack');
        }
      } else {
        validation.details.push('No session start time found');
      }

      // Additional CSRF checks
      const sessionId = localStorage.getItem('oidc_session_id');
      const urlSessionId = searchParams.get('session_id');
      
      if (sessionId && urlSessionId && sessionId !== urlSessionId) {
        validation.noCsrfDetected = false;
        validation.details.push('Session ID mismatch - possible CSRF attack');
      }

    } catch (error) {
      validation.details.push(`Security validation error: ${error}`);
    }

    return validation;
  };

  // Check if security validation passed
  const isSecurityValidationPassed = (validation: SecurityValidation): boolean => {
    return validation.stateValid && 
           validation.codePresent && 
           validation.noCsrfDetected && 
           validation.validOrigin && 
           validation.timeValid;
  };

  // Handle various types of errors
  const handleError = (error: Error) => {
    const errorInfo = classifyError(error);
    setErrorInfo(errorInfo);
    
    setState({
      stage: 'error',
      progress: 0,
      message: `Authentication failed: ${errorInfo.description}`,
      details: errorInfo.suggestedAction,
      canRetry: errorInfo.recoverable && retryCount < maxRetries
    });

    if (onError) {
      onError(errorInfo);
    }
  };

  // Handle timeout scenarios
  const handleTimeout = () => {
    const timeoutError: ErrorInfo = {
      code: 'TIMEOUT',
      description: 'Authentication callback timed out',
      recoverable: true,
      suggestedAction: 'The authentication process took too long. This could be due to network issues or server problems.'
    };

    setErrorInfo(timeoutError);
    setState({
      stage: 'timeout',
      progress: 0,
      message: 'Authentication callback timed out',
      details: 'The process took longer than expected',
      canRetry: retryCount < maxRetries
    });
  };

  // Classify error types for better user experience
  const classifyError = (error: Error): ErrorInfo => {
    const message = error.message.toLowerCase();

    if (message.includes('csrf') || message.includes('state')) {
      return {
        code: 'SECURITY_ERROR',
        description: 'Security validation failed',
        recoverable: false,
        suggestedAction: 'Please return to the login page and start a new authentication session.'
      };
    }

    if (message.includes('network') || message.includes('fetch')) {
      return {
        code: 'NETWORK_ERROR',
        description: 'Network connection error',
        recoverable: true,
        suggestedAction: 'Check your internet connection and try again.'
      };
    }

    if (message.includes('token') || message.includes('invalid_grant')) {
      return {
        code: 'TOKEN_ERROR',
        description: 'Invalid or expired authorization code',
        recoverable: false,
        suggestedAction: 'The authorization code has expired. Please start a new login session.'
      };
    }

    if (message.includes('timeout')) {
      return {
        code: 'TIMEOUT_ERROR',
        description: 'Request timeout',
        recoverable: true,
        suggestedAction: 'The request took too long to complete. Please try again.'
      };
    }

    return {
      code: 'UNKNOWN_ERROR',
      description: 'Unexpected authentication error',
      recoverable: true,
      suggestedAction: 'An unexpected error occurred. Please try again or contact support if the problem persists.'
    };
  };

  // Retry the callback process
  const handleRetry = () => {
    if (retryCount < maxRetries) {
      setRetryCount(prev => prev + 1);
      setState({
        stage: 'initializing',
        progress: 0,
        message: `Retrying authentication (attempt ${retryCount + 2}/${maxRetries + 1})...`,
        canRetry: false
      });
      processCallback();
    }
  };

  // Navigate back to login
  const handleBackToLogin = () => {
    router.replace('/login');
  };

  // Get appropriate icon for current stage
  const getStageIcon = () => {
    switch (state.stage) {
      case 'success':
        return <CheckCircle className="h-8 w-8 text-green-500" />;
      case 'error':
      case 'timeout':
        return <XCircle className="h-8 w-8 text-red-500" />;
      case 'validating':
        return <Shield className="h-8 w-8 text-blue-500" />;
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
        return 'border-red-200 bg-red-50';
      case 'validating':
        return 'border-blue-200 bg-blue-50';
      default:
        return 'border-gray-200 bg-gray-50';
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-background p-4">
      <Card className={`w-full max-w-md ${getStageColor()}`}>
        <CardHeader className="text-center space-y-4">
          <div className="flex justify-center">
            {getStageIcon()}
          </div>
          <CardTitle className="text-xl">
            {state.stage === 'success' ? 'Authentication Successful' :
             state.stage === 'error' ? 'Authentication Failed' :
             state.stage === 'timeout' ? 'Request Timeout' :
             'Processing Authentication'}
          </CardTitle>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Progress indicator */}
          {(state.stage === 'initializing' || state.stage === 'validating' || state.stage === 'processing') && (
            <div className="space-y-2">
              <Progress value={state.progress} className="h-2" />
              <div className="flex items-center gap-2">
                <Zap className="h-4 w-4 text-blue-500" />
                <p className="text-sm text-muted-foreground">{state.message}</p>
              </div>
              {state.details && (
                <p className="text-xs text-muted-foreground pl-6">{state.details}</p>
              )}
            </div>
          )}

          {/* Success state */}
          {state.stage === 'success' && (
            <div className="text-center space-y-3">
              <p className="text-green-600 font-medium">{state.message}</p>
              {state.redirectUrl && (
                <div className="flex items-center justify-center gap-2 text-sm text-muted-foreground">
                  <ExternalLink className="h-4 w-4" />
                  Redirecting to {state.redirectUrl}
                </div>
              )}
              {enableMetrics && (
                <div className="text-xs text-muted-foreground">
                  Completed in {metricsRef.current.totalTime}ms
                </div>
              )}
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
                  onClick={handleBackToLogin} 
                  variant="outline" 
                  className="flex-1"
                >
                  <ArrowLeft className="mr-2 h-4 w-4" />
                  Back to Login
                </Button>
              </div>
            </div>
          )}

          {/* Security validation details */}
          {enableSecurityValidation && (state.stage === 'validating' || state.stage === 'error') && (
            <div className="space-y-2">
              <div className="flex items-center gap-2 text-sm font-medium">
                <Shield className="h-4 w-4" />
                Security Validation
              </div>
              <div className="space-y-1 pl-6">
                {[
                  { key: 'stateValid', label: 'State Parameter' },
                  { key: 'codePresent', label: 'Authorization Code' },
                  { key: 'noCsrfDetected', label: 'CSRF Protection' },
                  { key: 'validOrigin', label: 'Origin Validation' },
                  { key: 'timeValid', label: 'Timing Check' }
                ].map(({ key, label }) => (
                  <div key={key} className="flex items-center gap-2 text-xs">
                    {securityValidation[key as keyof SecurityValidation] ? (
                      <CheckCircle className="h-3 w-3 text-green-500" />
                    ) : (
                      <XCircle className="h-3 w-3 text-red-500" />
                    )}
                    <span className={securityValidation[key as keyof SecurityValidation] ? 'text-green-600' : 'text-red-600'}>
                      {label}
                    </span>
                  </div>
                ))}
              </div>
              {securityValidation.details.length > 0 && (
                <div className="text-xs text-red-600 pl-6">
                  Issues: {securityValidation.details.join(', ')}
                </div>
              )}
            </div>
          )}

          {/* Processing metrics */}
          {enableMetrics && state.stage !== 'initializing' && (
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Clock className="h-3 w-3" />
              {state.stage === 'success' ? 
                `Completed in ${metricsRef.current.totalTime}ms` :
                `Processing for ${Date.now() - metricsRef.current.startTime}ms`
              }
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}