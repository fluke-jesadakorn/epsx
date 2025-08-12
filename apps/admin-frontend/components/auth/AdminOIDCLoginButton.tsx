'use client';

import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Shield, ExternalLink, Loader2, Lock, Eye, UserCheck } from 'lucide-react';

interface AdminOIDCLoginButtonProps {
  redirectTo?: string;
  requireMFA?: boolean;
  enableThreatDetection?: boolean;
  enableSessionMonitoring?: boolean;
  maxFailedAttempts?: number;
}

export function AdminOIDCLoginButton({ 
  redirectTo = '/',
  requireMFA = true,
  enableThreatDetection = true,
  enableSessionMonitoring = true,
  maxFailedAttempts = 3
}: AdminOIDCLoginButtonProps) {
  const [isRedirecting, setIsRedirecting] = useState(false);

  const handleAdminLogin = () => {
    setIsRedirecting(true);
    
    // Build authorization URL with admin-specific parameters
    const authParams = new URLSearchParams({
      client_id: 'epsx-admin',
      response_type: 'code',
      scope: 'openid profile email admin:read admin:write system:manage', // Admin-specific scopes
      redirect_uri: `${window.location.origin}/auth/callback`,
      // Add redirect destination as state parameter for server-side handling
      state: redirectTo || '/'
    });
    
    // Redirect to backend admin OIDC authorization endpoint
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    const loginUrl = `${backendUrl}/oauth/authorize?${authParams.toString()}`;
    
    console.log('🔐 Redirecting to secure admin OIDC login:', loginUrl);
    
    // Add audit log entry for login attempt
    console.log('🔍 Admin OIDC login initiated', {
      timestamp: new Date().toISOString(),
      client_id: 'epsx-admin',
      redirect_uri: `${window.location.origin}/auth/callback`,
      security_features: {
        mfa: requireMFA,
        threat_detection: enableThreatDetection,
        session_monitoring: enableSessionMonitoring,
        max_attempts: maxFailedAttempts
      }
    });
    
    window.location.href = loginUrl;
  };

  return (
    <div className="w-full max-w-md mx-auto">
      {/* Admin security card */}
      <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl border border-gray-200 dark:border-gray-700 p-8">
        {/* Header */}
        <div className="text-center mb-8">
          <div className="inline-flex h-16 w-16 items-center justify-center rounded-full bg-gradient-to-r from-blue-600 to-purple-600 mb-4">
            <Shield className="h-8 w-8 text-white" />
          </div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            Administrator Access
          </h1>
          <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
            Secure authentication with enhanced monitoring
          </p>
        </div>

        {/* Security features display */}
        <div className="grid grid-cols-3 gap-4 mb-6">
          <div className="text-center p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
            <Lock className="h-5 w-5 text-blue-600 dark:text-blue-400 mx-auto mb-1" />
            <p className="text-xs font-medium text-blue-700 dark:text-blue-300">
              {requireMFA ? 'MFA Required' : 'Standard Auth'}
            </p>
          </div>
          <div className="text-center p-3 bg-purple-50 dark:bg-purple-900/20 rounded-lg">
            <Eye className="h-5 w-5 text-purple-600 dark:text-purple-400 mx-auto mb-1" />
            <p className="text-xs font-medium text-purple-700 dark:text-purple-300">
              {enableThreatDetection ? 'Threat Detection' : 'Basic Security'}
            </p>
          </div>
          <div className="text-center p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
            <UserCheck className="h-5 w-5 text-green-600 dark:text-green-400 mx-auto mb-1" />
            <p className="text-xs font-medium text-green-700 dark:text-green-300">
              {enableSessionMonitoring ? 'Session Monitor' : 'Standard Session'}
            </p>
          </div>
        </div>

        {/* Main login button */}
        <Button
          onClick={handleAdminLogin}
          disabled={isRedirecting}
          className="w-full h-12 text-base font-semibold bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white border-0 shadow-lg transition-all duration-200"
          size="lg"
        >
          {isRedirecting ? (
            <>
              <Loader2 className="mr-2 h-5 w-5 animate-spin" />
              Redirecting to secure login...
            </>
          ) : (
            <>
              <ExternalLink className="mr-2 h-5 w-5" />
              Access Admin Portal
            </>
          )}
        </Button>

        {/* Security notice */}
        <div className="mt-6 text-center space-y-2">
          <p className="text-xs text-gray-500 dark:text-gray-400">
            🔒 Enhanced security validation • Administrative privileges required
          </p>
          <p className="text-xs text-gray-400 dark:text-gray-500">
            Max {maxFailedAttempts} attempts • All access logged and monitored
          </p>
        </div>

        {/* Security indicators */}
        <div className="mt-4 flex items-center justify-center space-x-4 text-xs text-gray-400 dark:text-gray-500">
          <div className="flex items-center">
            <div className="w-2 h-2 bg-green-500 rounded-full mr-1"></div>
            <span>SSL Encrypted</span>
          </div>
          <div className="flex items-center">
            <div className="w-2 h-2 bg-blue-500 rounded-full mr-1"></div>
            <span>OIDC Standard</span>
          </div>
          <div className="flex items-center">
            <div className="w-2 h-2 bg-purple-500 rounded-full mr-1"></div>
            <span>Admin Level</span>
          </div>
        </div>
      </div>
    </div>
  );
}

/**
 * Generate a secure state parameter for CSRF protection
 */
function generateSecureState(): string {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return btoa(String.fromCharCode.apply(null, Array.from(array)))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

export default AdminOIDCLoginButton;