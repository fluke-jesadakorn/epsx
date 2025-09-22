'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ExternalLink, Wallet } from 'lucide-react';
import { WalletConnectAuth } from './WalletConnectAuth';

interface UnifiedAuthFormProps {
  redirectTo?: string;
}

export function UnifiedAuthForm({ 
  redirectTo = '/dashboard'
}: UnifiedAuthFormProps) {
  const [isOIDCLoading, setIsOIDCLoading] = useState(false);
  const [oidcError, setOIDCError] = useState('');

  const initiateOAuthFlow = async () => {
    try {
      setIsOIDCLoading(true);
      setOIDCError('');
      
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
      window.location.href = authorizationUrl;
      
    } catch (error: any) {
      console.error('OAuth initiation failed:', error);
      setOIDCError(error.message || 'Authentication failed. Please try again.');
      setIsOIDCLoading(false);
    }
  };

  const handleWeb3AuthSuccess = (walletAddress: string) => {
    console.log('Web3 authentication successful:', walletAddress);
    window.location.href = redirectTo;
  };

  const handleWeb3AuthError = (error: string) => {
    console.error('Web3 authentication error:', error);
  };

  return (
    <Card className="w-full max-w-md mx-auto">
      <CardHeader className="text-center">
        <CardTitle className="flex items-center justify-center gap-2">
          <Wallet className="h-6 w-6 text-orange-500" />
          Connect to EPSX
        </CardTitle>
        <p className="text-sm text-muted-foreground">
          Connect your Web3 wallet to get started
        </p>
      </CardHeader>
      
      <CardContent className="space-y-6">
        {/* Web3 Authentication - Primary */}
        <div className="space-y-4">
          <WalletConnectAuth
            onAuthSuccess={handleWeb3AuthSuccess}
            onAuthError={handleWeb3AuthError}
          />
        </div>

        {/* Fallback: Traditional Auth */}
        <div className="text-center">
          <p className="text-xs text-muted-foreground mb-3">
            Don't have a Web3 wallet?
          </p>
          
          {oidcError && (
            <div className="rounded-lg bg-red-50 dark:bg-red-900/20 p-3 border border-red-200 dark:border-red-800 mb-3">
              <p className="text-sm text-red-700 dark:text-red-400">⚠️ {oidcError}</p>
            </div>
          )}

          <Button
            onClick={initiateOAuthFlow}
            disabled={isOIDCLoading}
            variant="outline"
            className="w-full"
          >
            {isOIDCLoading ? (
              <>
                <div className="h-4 w-4 mr-2 bg-current rounded animate-pulse" />
                Redirecting...
              </>
            ) : (
              <>
                <ExternalLink className="h-4 w-4 mr-2" />
                Sign In with Email
              </>
            )}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}