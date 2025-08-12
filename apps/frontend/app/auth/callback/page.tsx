'use client';

import React, { useEffect, useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loader2, CheckCircle, XCircle, Shield } from 'lucide-react';

interface TokenResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
  id_token: string;
  refresh_token?: string;
  scope: string;
}

interface UserInfo {
  id: string;
  email: string;
  name?: string;
  role: string;
  permissions: string[];
}

/**
 * Pure OIDC Authentication Callback Page
 * Handles the authorization code exchange with the backend
 * 
 * Features:
 * - Pure OpenID Connect authorization code flow
 * - Secure token exchange with backend
 * - CSRF protection with state validation
 * - Clean error handling and user feedback
 */
export default function AuthCallbackPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [status, setStatus] = useState<'processing' | 'success' | 'error'>('processing');
  const [message, setMessage] = useState('Processing authentication...');
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    handleCallback();
  }, []);

  const handleCallback = async () => {
    try {
      setProgress(10);
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

      setProgress(20);
      setMessage('Verifying security parameters...');

      // Validate state parameter (CSRF protection)
      const storedState = sessionStorage.getItem('oidc_state');
      if (!storedState || state !== storedState) {
        throw new Error('Invalid state parameter - possible security issue');
      }

      setProgress(40);
      setMessage('Exchanging authorization code for tokens...');

      // Exchange authorization code for tokens
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
          client_id: 'epsx-frontend',
        }),
      });

      if (!tokenResponse.ok) {
        const errorData = await tokenResponse.text();
        console.error('Token exchange failed:', errorData);
        throw new Error(`Token exchange failed: ${tokenResponse.status}`);
      }

      setProgress(60);
      setMessage('Processing user information...');

      const tokens: TokenResponse = await tokenResponse.json();

      // Decode user information from ID token
      const userInfo = decodeIdToken(tokens.id_token);

      setProgress(80);
      setMessage('Securing session...');

      // Store tokens securely
      storeTokens(tokens);
      storeUserInfo(userInfo);

      setProgress(90);
      setMessage('Finalizing authentication...');

      // Clean up temporary data
      sessionStorage.removeItem('oidc_state');

      setProgress(100);
      setStatus('success');
      setMessage('Authentication successful!');

      console.log('✅ Authentication completed for user:', userInfo.email);

      // Get redirect destination
      const redirectTo = sessionStorage.getItem('oidc_redirect_to') || '/dashboard';
      sessionStorage.removeItem('oidc_redirect_to');

      // Redirect after a brief success display
      setTimeout(() => {
        router.push(redirectTo);
      }, 2000);

    } catch (error) {
      console.error('❌ Authentication callback failed:', error);
      
      setStatus('error');
      setMessage(error instanceof Error ? error.message : 'Authentication failed');
      setProgress(0);

      // Clean up on error
      sessionStorage.removeItem('oidc_state');
      sessionStorage.removeItem('oidc_redirect_to');
    }
  };

  const handleRetry = () => {
    router.push('/login');
  };

  const handleHome = () => {
    router.push('/');
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4">
      {/* Background pattern */}
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-purple-50 to-cyan-50" />
        <div className="absolute top-20 left-10 w-32 h-32 bg-gradient-to-br from-blue-400/10 to-purple-400/10 rounded-full animate-pulse" />
        <div className="absolute bottom-20 right-20 w-24 h-24 bg-gradient-to-br from-purple-400/10 to-cyan-400/10 rounded-full animate-bounce" />
      </div>

      <Card className="w-full max-w-md relative z-10 shadow-lg">
        <CardHeader>
          <div className="flex items-center gap-3">
            {status === 'processing' && <Loader2 className="h-6 w-6 animate-spin text-blue-500" />}
            {status === 'success' && <CheckCircle className="h-6 w-6 text-green-500" />}
            {status === 'error' && <XCircle className="h-6 w-6 text-red-500" />}
            <div>
              <CardTitle>
                {status === 'processing' && 'Authenticating...'}
                {status === 'success' && 'Success!'}
                {status === 'error' && 'Authentication Error'}
              </CardTitle>
            </div>
          </div>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Progress bar */}
          {status === 'processing' && (
            <div className="space-y-2">
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div 
                  className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${progress}%` }}
                />
              </div>
              <p className="text-sm text-center text-muted-foreground">{message}</p>
            </div>
          )}

          {/* Success message */}
          {status === 'success' && (
            <div className="text-center space-y-3">
              <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                <div className="flex items-center justify-center">
                  <Shield className="h-5 w-5 text-green-500 mr-2" />
                  <p className="text-green-800 font-medium">{message}</p>
                </div>
              </div>
              <p className="text-sm text-muted-foreground">
                Redirecting to your dashboard...
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
                <Button onClick={handleHome} variant="outline" className="flex-1">
                  Home
                </Button>
              </div>
            </div>
          )}

          {/* Security notice */}
          <div className="text-center text-xs text-muted-foreground">
            <p>🔒 Secure authentication powered by OpenID Connect</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

/**
 * Decode JWT ID token to extract user information
 */
function decodeIdToken(idToken: string): UserInfo {
  try {
    const payloadPart = idToken.split('.')[1];
    const decodedPayload = atob(payloadPart);
    const payload = JSON.parse(decodedPayload);

    return {
      id: payload.sub,
      email: payload.email || '',
      name: payload.name,
      role: payload.role || 'user',
      permissions: [], // Will be populated from access token if needed
    };
  } catch (error) {
    console.error('Failed to decode ID token:', error);
    throw new Error('Invalid authentication token');
  }
}

/**
 * Store tokens securely in sessionStorage
 */
function storeTokens(tokens: TokenResponse): void {
  // Store tokens in sessionStorage (more secure than localStorage for auth tokens)
  sessionStorage.setItem('access_token', tokens.access_token);
  sessionStorage.setItem('id_token', tokens.id_token);
  sessionStorage.setItem('token_type', tokens.token_type);
  sessionStorage.setItem('expires_in', tokens.expires_in.toString());
  sessionStorage.setItem('scope', tokens.scope);
  
  // Calculate expiration time
  const expiresAt = Date.now() + (tokens.expires_in * 1000);
  sessionStorage.setItem('expires_at', expiresAt.toString());
  
  // Store refresh token if provided
  if (tokens.refresh_token) {
    sessionStorage.setItem('refresh_token', tokens.refresh_token);
  }
}

/**
 * Store user information
 */
function storeUserInfo(userInfo: UserInfo): void {
  sessionStorage.setItem('user', JSON.stringify(userInfo));
}