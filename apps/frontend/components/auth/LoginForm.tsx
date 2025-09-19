'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Eye, EyeOff, Lock, Mail, CheckCircle, ExternalLink } from 'lucide-react';
import { clientConfig } from '@/config/env';

interface LoginFormProps {
  redirectTo?: string;
}

export function LoginForm({ redirectTo = '/dashboard' }: LoginFormProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState('');
  const [loginStep, setLoginStep] = useState<'ready' | 'redirecting' | 'complete'>('ready');

  /**
   * Initiate OAuth Authorization Code Flow
   * Redirects to backend OAuth authorization endpoint
   */
  const initiateOAuthFlow = async () => {
    try {
      setIsLoading(true);
      setError('');
      setLoginStep('redirecting');
      
      
      // Generate PKCE parameters and get authorization URL
      const response = await fetch('/api/auth/initiate', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          redirectTo: redirectTo
        }),
        credentials: 'include'
      });
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Failed to initiate authentication');
      }
      
      const { authorizationUrl } = await response.json();
      
      
      // Redirect to backend OAuth authorization endpoint
      window.location.href = authorizationUrl;
      
    } catch (error: any) {
      console.error('❌ OAuth initiation failed:', error);
      setError(error.message || 'Authentication failed. Please try again.');
      setLoginStep('ready');
      setIsLoading(false);
    }
  };

  const getStepDescription = () => {
    switch (loginStep) {
      case 'ready': return 'Ready to sign in'
      case 'redirecting': return 'Redirecting to secure authentication...'
      case 'complete': return 'Success! Redirecting...'
    }
  };

  return (
    <div className="space-y-6">
      {/* Authentication Status */}
      {loginStep !== 'ready' && (
        <div className="p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
          <div className="flex items-center gap-3">
            {loginStep === 'redirecting' ? (
              <div className="h-5 w-5 bg-blue-600 rounded" />
            ) : (
              <CheckCircle className="h-5 w-5 text-green-600" />
            )}
            <div>
              <h3 className="font-medium text-blue-900 dark:text-blue-100">Authentication Status</h3>
              <p className="text-sm text-blue-700 dark:text-blue-300">{getStepDescription()}</p>
            </div>
          </div>
          
          <div className="mt-3 text-xs text-blue-600 dark:text-blue-400">
            <p>✅ OAuth Authorization Code Flow</p>
            {loginStep === 'complete' && <p>✅ Secure Token Exchange</p>}
          </div>
        </div>
      )}

      {/* Error display */}
      {error && (
        <div className="rounded-lg bg-red-50 dark:bg-red-900/20 p-3 border border-red-200 dark:border-red-800">
          <p className="text-sm text-red-700 dark:text-red-400">⚠️ {error}</p>
        </div>
      )}

      {/* Main Authentication Button */}
      <Button
        onClick={initiateOAuthFlow}
        disabled={isLoading}
        className="w-full h-12 sm:h-14 text-base sm:text-lg font-semibold bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white border-0 shadow-lg rounded-xl"
      >
        {isLoading ? (
          <>
            <div className="h-4 w-4 mr-2 bg-white rounded" />
            <span>Redirecting to secure authentication...</span>
          </>
        ) : (
          <>
            <ExternalLink className="h-4 w-4 mr-2" />
            <span>Sign In with EPSX</span>
          </>
        )}
      </Button>

      {/* Features */}
      <div className="grid grid-cols-2 gap-4">
        <div className="text-center p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
          <Lock className="h-5 w-5 text-green-600 dark:text-green-400 mx-auto mb-1" />
          <p className="text-xs font-medium text-green-700 dark:text-green-300">
            Secure OAuth
          </p>
        </div>
        <div className="text-center p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
          <CheckCircle className="h-5 w-5 text-blue-600 dark:text-blue-400 mx-auto mb-1" />
          <p className="text-xs font-medium text-blue-700 dark:text-blue-300">
            No Token Exposure
          </p>
        </div>
      </div>

      {/* Security notice */}
      <div className="text-center space-y-2">
        <p className="text-xs text-muted-foreground">
          🔒 OIDC-compliant OAuth Authorization Code Flow
        </p>
        <div className="flex items-center justify-center gap-2 text-xs text-green-600 dark:text-green-400">
          <CheckCircle className="h-3 w-3" />
          <span>HttpOnly cookies • PKCE protection • Bearer tokens</span>
        </div>
      </div>
    </div>
  );
}

export default LoginForm;