'use client';

import React, { useEffect, useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Shield, ExternalLink, AlertTriangle, Loader2 } from 'lucide-react';

interface PureOIDCLoginProps {
  onSuccess?: (user: any) => void;
  redirectTo?: string;
}

export function PureOIDCLogin({ 
  onSuccess, 
  redirectTo = '/dashboard' 
}: PureOIDCLoginProps) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [error, setError] = useState<string | null>(null);
  const [isRedirecting, setIsRedirecting] = useState(false);

  // Check for error in URL parameters
  useEffect(() => {
    const errorParam = searchParams.get('error');
    if (errorParam) {
      setError(decodeURIComponent(errorParam));
    }
  }, [searchParams]);

  const handleLogin = () => {
    setIsRedirecting(true);
    setError(null);
    
    // Generate secure state parameter
    const state = generateSecureState();
    
    // Build authorization URL
    const authParams = new URLSearchParams({
      client_id: 'epsx-frontend',
      response_type: 'code',
      scope: 'openid profile email',
      redirect_uri: `${window.location.origin}/auth/callback`,
      state: state
    });
    
    // Store state for validation in callback
    sessionStorage.setItem('oidc_state', state);
    
    // Store redirect destination
    if (redirectTo) {
      sessionStorage.setItem('oidc_redirect_to', redirectTo);
    }
    
    // Redirect to backend login page
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    const loginUrl = `${backendUrl}/oauth/authorize?${authParams.toString()}`;
    
    console.log('🔐 Redirecting to backend login:', loginUrl);
    window.location.href = loginUrl;
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4">
      <Card className="w-full max-w-md">
        <CardHeader className="space-y-4">
          <div className="flex items-center gap-3">
            <Shield className="h-8 w-8 text-blue-600" />
            <div>
              <CardTitle className="text-2xl">Sign In to EPSX</CardTitle>
              <CardDescription>
                Secure access to your analytics dashboard
              </CardDescription>
            </div>
          </div>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Error display */}
          {error && (
            <Alert variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {/* Login button */}
          <Button
            onClick={handleLogin}
            disabled={isRedirecting}
            className="w-full"
            size="lg"
          >
            {isRedirecting ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Redirecting to secure login...
              </>
            ) : (
              <>
                <ExternalLink className="mr-2 h-4 w-4" />
                Continue to Secure Login
              </>
            )}
          </Button>

          {/* Security notice */}
          <div className="text-center space-y-2">
            <p className="text-sm text-muted-foreground">
              You will be redirected to our secure authentication server
            </p>
            <div className="flex items-center justify-center space-x-4 text-xs text-muted-foreground">
              <div className="flex items-center">
                <Shield className="h-3 w-3 mr-1 text-green-500" />
                <span>SSL Encrypted</span>
              </div>
              <div className="flex items-center">
                <Shield className="h-3 w-3 mr-1 text-blue-500" />
                <span>OIDC Standard</span>
              </div>
            </div>
          </div>

          {/* Info box */}
          <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
            <div className="flex">
              <div className="flex-shrink-0">
                <Shield className="h-5 w-5 text-blue-400" />
              </div>
              <div className="ml-3">
                <h3 className="text-sm font-medium text-blue-800">
                  Secure Authentication
                </h3>
                <div className="mt-2 text-sm text-blue-700">
                  <p>
                    Your credentials are processed on our secure servers and never stored in your browser.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
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

export default PureOIDCLogin;