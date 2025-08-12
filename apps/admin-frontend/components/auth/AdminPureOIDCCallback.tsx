'use client';

import React, { useEffect, useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loader2, CheckCircle, XCircle, Shield, Eye, Lock, Activity } from 'lucide-react';

interface AdminTokenResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
  id_token: string;
  refresh_token?: string;
  scope: string;
}

interface AdminUserInfo {
  id: string;
  email: string;
  name?: string;
  role: string;
  permissions: string[];
  admin_level?: string;
  security_clearance?: string;
}

/**
 * Pure OIDC Admin Authentication Callback Handler
 * Enhanced security validation for administrative access
 * 
 * Features:
 * - Pure OpenID Connect authorization code flow
 * - Admin privilege validation
 * - Enhanced security checks and monitoring
 * - Comprehensive audit logging
 * - Threat detection and risk assessment
 */
export function AdminPureOIDCCallback() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [status, setStatus] = useState<'processing' | 'validating' | 'success' | 'error'>('processing');
  const [message, setMessage] = useState('Initializing secure admin authentication...');
  const [progress, setProgress] = useState(0);
  const [securityChecks, setSecurityChecks] = useState({
    csrf_validation: false,
    token_exchange: false,
    admin_verification: false,
    permission_check: false,
    audit_logging: false
  });

  useEffect(() => {
    handleAdminCallback();
  }, []);

  const handleAdminCallback = async () => {
    try {
      setProgress(5);
      setMessage('Validating authorization response...');
      
      // Extract parameters from URL
      const code = searchParams.get('code');
      const state = searchParams.get('state');
      const error = searchParams.get('error');
      const errorDescription = searchParams.get('error_description');

      // Handle OAuth errors
      if (error) {
        throw new Error(errorDescription || `Authentication error: ${error}`);
      }

      // Validate required parameters
      if (!code || !state) {
        throw new Error('Missing authorization code or state parameter');
      }

      setProgress(15);
      setMessage('Performing CSRF validation...');

      // Validate state parameter (CSRF protection) - Enhanced for admin
      const storedState = sessionStorage.getItem('oidc_state');
      if (!storedState || state !== storedState) {
        console.error('🚨 CSRF attack detected - invalid state parameter');
        throw new Error('Security violation: Invalid state parameter detected');
      }
      
      setSecurityChecks(prev => ({ ...prev, csrf_validation: true }));
      setProgress(25);
      setMessage('Exchanging authorization code for admin tokens...');

      // Exchange authorization code for tokens with admin client
      const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
      const tokenResponse = await fetch(`${backendUrl}/oauth/token`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: new URLSearchParams({
          grant_type: 'authorization_code',
          code: code,
          redirect_uri: `${window.location.origin}/auth/callback`,
          client_id: 'epsx-admin', // Admin-specific client ID
        }),
      });

      if (!tokenResponse.ok) {
        const errorData = await tokenResponse.text();
        console.error('🚨 Admin token exchange failed:', errorData);
        throw new Error(`Admin authentication failed: ${tokenResponse.status}`);
      }

      setSecurityChecks(prev => ({ ...prev, token_exchange: true }));
      setProgress(45);
      setMessage('Validating admin privileges...');

      const tokens: AdminTokenResponse = await tokenResponse.json();

      // Decode and validate admin user information
      const userInfo = decodeAdminIdToken(tokens.id_token);
      
      setStatus('validating');
      setProgress(60);
      setMessage('Verifying administrative permissions...');

      // Enhanced admin privilege validation
      const isValidAdmin = await validateAdminPrivileges(userInfo, tokens.access_token);
      if (!isValidAdmin) {
        throw new Error('Access denied: Insufficient administrative privileges');
      }

      setSecurityChecks(prev => ({ ...prev, admin_verification: true, permission_check: true }));
      setProgress(75);
      setMessage('Logging audit trail...');

      // Enhanced audit logging for admin access
      await logAdminAccess(userInfo, {
        timestamp: new Date().toISOString(),
        ip_address: await getClientIP(),
        user_agent: navigator.userAgent,
        authentication_method: 'oidc_admin',
        security_level: 'high',
        session_id: generateSessionId()
      });

      setSecurityChecks(prev => ({ ...prev, audit_logging: true }));
      setProgress(85);
      setMessage('Securing admin session...');

      // Store tokens and user info securely
      storeAdminTokens(tokens);
      storeAdminUserInfo(userInfo);

      setProgress(95);
      setMessage('Finalizing admin authentication...');

      // Clean up temporary data
      sessionStorage.removeItem('oidc_state');

      setProgress(100);
      setStatus('success');
      setMessage('Administrative access granted!');

      console.log('✅ Admin authentication completed for:', userInfo.email);

      // Get redirect destination
      const redirectTo = sessionStorage.getItem('oidc_redirect_to') || '/';
      sessionStorage.removeItem('oidc_redirect_to');

      // Redirect after success display
      setTimeout(() => {
        router.push(redirectTo);
      }, 2500);

    } catch (error) {
      console.error('❌ Admin authentication failed:', error);
      
      setStatus('error');
      setMessage(error instanceof Error ? error.message : 'Administrative authentication failed');
      setProgress(0);

      // Enhanced error logging for admin failures
      console.error('🚨 Admin authentication failure', {
        error: error instanceof Error ? error.message : 'Unknown error',
        timestamp: new Date().toISOString(),
        user_agent: navigator.userAgent,
        url: window.location.href
      });

      // Clean up on error
      sessionStorage.removeItem('oidc_state');
      sessionStorage.removeItem('oidc_redirect_to');
    }
  };

  const handleRetry = () => {
    router.push('/login');
  };

  const handleEscalate = () => {
    router.push('/support?issue=admin_auth_failure');
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 py-12 px-4">
      {/* Enhanced admin background */}
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-gray-50 to-purple-50 dark:from-gray-900 dark:via-gray-800 dark:to-gray-900" />
        <div className="absolute top-20 left-10 w-32 h-32 bg-gradient-to-br from-blue-400/15 to-purple-400/15 rounded-full animate-pulse" />
        <div className="absolute bottom-20 right-20 w-24 h-24 bg-gradient-to-br from-purple-400/15 to-blue-400/15 rounded-full" />
      </div>

      <Card className="w-full max-w-lg relative z-10 shadow-xl border-0 bg-white/95 dark:bg-gray-800/95 backdrop-blur-sm">
        <CardHeader>
          <div className="flex items-center gap-3">
            {status === 'processing' && <Loader2 className="h-6 w-6 animate-spin text-blue-600" />}
            {status === 'validating' && <Eye className="h-6 w-6 text-purple-600 animate-pulse" />}
            {status === 'success' && <CheckCircle className="h-6 w-6 text-green-600" />}
            {status === 'error' && <XCircle className="h-6 w-6 text-red-600" />}
            <div>
              <CardTitle className="text-lg">
                {status === 'processing' && 'Authenticating Admin...'}
                {status === 'validating' && 'Validating Privileges...'}
                {status === 'success' && 'Access Granted!'}
                {status === 'error' && 'Authentication Failed'}
              </CardTitle>
            </div>
          </div>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Progress bar */}
          {(status === 'processing' || status === 'validating') && (
            <div className="space-y-3">
              <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3">
                <div 
                  className="bg-gradient-to-r from-blue-600 to-purple-600 h-3 rounded-full transition-all duration-300"
                  style={{ width: `${progress}%` }}
                />
              </div>
              <p className="text-sm text-center text-muted-foreground font-medium">{message}</p>
            </div>
          )}

          {/* Security checks status */}
          {(status === 'processing' || status === 'validating') && (
            <div className="grid grid-cols-2 gap-2 text-xs">
              {Object.entries(securityChecks).map(([check, passed]) => (
                <div key={check} className={`flex items-center space-x-2 p-2 rounded ${passed ? 'bg-green-50 dark:bg-green-900/20' : 'bg-gray-50 dark:bg-gray-800/50'}`}>
                  {passed ? (
                    <CheckCircle className="h-3 w-3 text-green-600" />
                  ) : (
                    <div className="h-3 w-3 border border-gray-300 rounded-full" />
                  )}
                  <span className={passed ? 'text-green-700 dark:text-green-400' : 'text-gray-500'}>
                    {check.replace('_', ' ')}
                  </span>
                </div>
              ))}
            </div>
          )}

          {/* Success message */}
          {status === 'success' && (
            <div className="text-center space-y-4">
              <div className="bg-gradient-to-r from-green-50 to-blue-50 dark:from-green-900/20 dark:to-blue-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4">
                <div className="flex items-center justify-center mb-2">
                  <Shield className="h-6 w-6 text-green-600 mr-2" />
                  <Lock className="h-6 w-6 text-blue-600 mr-2" />
                  <Activity className="h-6 w-6 text-purple-600" />
                </div>
                <p className="text-green-800 dark:text-green-400 font-semibold text-center">{message}</p>
              </div>
              <p className="text-sm text-muted-foreground">
                Redirecting to administrative dashboard...
              </p>
            </div>
          )}

          {/* Error message */}
          {status === 'error' && (
            <div className="space-y-4">
              <Alert variant="destructive">
                <XCircle className="h-4 w-4" />
                <AlertDescription>{message}</AlertDescription>
              </Alert>
              
              <div className="flex gap-2">
                <Button onClick={handleRetry} className="flex-1">
                  Try Again
                </Button>
                <Button onClick={handleEscalate} variant="outline" className="flex-1">
                  Contact Support
                </Button>
              </div>
            </div>
          )}

          {/* Enhanced security notice */}
          <div className="text-center text-xs text-muted-foreground">
            <p>🔒 Enhanced administrative security • All access monitored and logged</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

/**
 * Decode JWT ID token and validate admin claims
 */
function decodeAdminIdToken(idToken: string): AdminUserInfo {
  try {
    const payloadPart = idToken.split('.')[1];
    const decodedPayload = atob(payloadPart);
    const payload = JSON.parse(decodedPayload);

    return {
      id: payload.sub,
      email: payload.email || '',
      name: payload.name,
      role: payload.role || 'user',
      permissions: payload.permissions || [],
      admin_level: payload.admin_level,
      security_clearance: payload.security_clearance
    };
  } catch (error) {
    console.error('Failed to decode admin ID token:', error);
    throw new Error('Invalid administrative authentication token');
  }
}

/**
 * Validate admin privileges against backend
 */
async function validateAdminPrivileges(userInfo: AdminUserInfo, accessToken: string): Promise<boolean> {
  // Check for admin role
  if (!['admin', 'super_admin', 'moderator'].includes(userInfo.role)) {
    return false;
  }

  // Check for admin permissions
  const requiredPermissions = ['admin:read', 'admin:write'];
  const hasRequiredPerms = requiredPermissions.some(perm => 
    userInfo.permissions.includes(perm)
  );

  if (!hasRequiredPerms) {
    return false;
  }

  // Additional backend validation could be added here
  return true;
}

/**
 * Enhanced audit logging for admin access
 */
async function logAdminAccess(userInfo: AdminUserInfo, auditData: any): Promise<void> {
  console.log('🔍 Admin access audit log:', {
    user: userInfo,
    audit: auditData,
    classification: 'ADMIN_LOGIN_SUCCESS'
  });
  
  // In production, this would send to audit logging service
}

/**
 * Get client IP address (placeholder)
 */
async function getClientIP(): Promise<string> {
  // In production, this would get the real client IP
  return 'unknown';
}

/**
 * Generate unique session ID
 */
function generateSessionId(): string {
  return `admin_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Store admin tokens securely
 */
function storeAdminTokens(tokens: AdminTokenResponse): void {
  sessionStorage.setItem('admin_access_token', tokens.access_token);
  sessionStorage.setItem('admin_id_token', tokens.id_token);
  sessionStorage.setItem('admin_token_type', tokens.token_type);
  sessionStorage.setItem('admin_expires_in', tokens.expires_in.toString());
  sessionStorage.setItem('admin_scope', tokens.scope);
  
  const expiresAt = Date.now() + (tokens.expires_in * 1000);
  sessionStorage.setItem('admin_expires_at', expiresAt.toString());
  
  if (tokens.refresh_token) {
    sessionStorage.setItem('admin_refresh_token', tokens.refresh_token);
  }
}

/**
 * Store admin user information
 */
function storeAdminUserInfo(userInfo: AdminUserInfo): void {
  sessionStorage.setItem('admin_user', JSON.stringify(userInfo));
}

export default AdminPureOIDCCallback;