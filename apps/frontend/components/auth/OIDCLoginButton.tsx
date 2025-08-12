'use client';

import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { ExternalLink, Loader2 } from 'lucide-react';

interface OIDCLoginButtonProps {
  redirectTo?: string;
}

export function OIDCLoginButton({ redirectTo = '/dashboard' }: OIDCLoginButtonProps) {
  const [isRedirecting, setIsRedirecting] = useState(false);

  const handleLogin = () => {
    setIsRedirecting(true);
    
    // Build authorization URL with redirect destination as state parameter
    const authParams = new URLSearchParams({
      client_id: 'epsx-frontend',
      response_type: 'code',
      scope: 'openid profile email',
      redirect_uri: `${window.location.origin}/auth/callback`,
      // Pass redirect destination as state for server-side handling
      state: redirectTo || '/dashboard'
    });
    
    // Redirect to backend OIDC authorization endpoint
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    const loginUrl = `${backendUrl}/oauth/authorize?${authParams.toString()}`;
    
    console.log('🔐 Redirecting to secure OIDC login:', loginUrl);
    window.location.href = loginUrl;
  };

  return (
    <div className="space-y-4">
      {/* Main login button */}
      <Button
        onClick={handleLogin}
        disabled={isRedirecting}
        className="w-full h-12 text-lg font-semibold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white border-0 shadow-lg"
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
            Continue with Secure Login
          </>
        )}
      </Button>

      {/* Security notice */}
      <div className="text-center">
        <p className="text-xs text-muted-foreground">
          🔒 Your credentials are processed securely on our servers
        </p>
      </div>
    </div>
  );
}

// Note: State parameter is now handled by the server-side callback for security

export default OIDCLoginButton;