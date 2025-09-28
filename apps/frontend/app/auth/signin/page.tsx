// ============================================================================
// WEB3 OPENID SIGN-IN PAGE
// Authentication page for Web3 wallet + OpenID Connect
// ============================================================================

/**
 * AUTHENTICATION PAGE:
 * - Web3 wallet connection and signing
 * - OpenID Connect token issuance
 * - Post-authentication redirect handling
 * - Clean, focused user experience
 */

'use client';

import React, { useEffect } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useSharedAuth } from '@/shared/components/auth/SharedOpenIDWeb3Provider';
import Web3OpenIDSignIn from '@/components/auth/Web3OpenIDSignIn';
import { logger } from '@/lib/shared';

export default function SignInPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { isAuthenticated, user } = useSharedAuth();
  
  // Get return URL from query params
  const returnUrl = searchParams.get('returnUrl') || '/dashboard';

  // Redirect if already authenticated
  useEffect(() => {
    if (isAuthenticated && user) {
      logger.info('User already authenticated, redirecting', {
        wallet_address: user.wallet_address,
        return_url: returnUrl
      });
      
      router.push(returnUrl);
    }
  }, [isAuthenticated, user, router, returnUrl]);

  // Handle successful authentication
  const handleAuthSuccess = () => {
    logger.info('Authentication successful, redirecting to return URL', {
      return_url: returnUrl
    });
    
    router.push(returnUrl);
  };

  // Don't render sign-in form if already authenticated
  if (isAuthenticated) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <h2 className="text-xl font-semibold mb-2">Already Signed In</h2>
          <p className="text-gray-600">Redirecting you now...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
      <Web3OpenIDSignIn 
        redirectTo={returnUrl}
        onSuccess={handleAuthSuccess}
      />
    </div>
  );
}