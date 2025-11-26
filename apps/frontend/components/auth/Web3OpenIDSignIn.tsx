// ============================================================================
// WEB3 OPENID SIGN-IN COMPONENT
// Web3 wallet authentication page that issues OpenID Connect tokens
// ============================================================================

/**
 * CORE PRINCIPLES:
 * - Web3 wallet signing triggers OpenID token issuance
 * - No permission logic in frontend
 * - Backend validates signatures and issues tokens
 * - Simple, focused authentication flow
 */

'use client';

import React, { useState, useCallback } from 'react';
import { useRouter } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loader2, Wallet, CheckCircle, AlertCircle } from 'lucide-react';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { logger } from '@/lib/shared';
import { requestWalletChallenge, verifyWalletSignature } from '@/lib/auth/api-direct';

// Note: Import direct API temporarily, will be migrated to shared system

// Authentication step states
type AuthStep = 'connect' | 'challenge' | 'signing' | 'authenticating' | 'success' | 'error';

interface Web3OpenIDSignInProps {
  onSuccess?: () => void;
  className?: string;
}

export function Web3OpenIDSignIn({
  onSuccess,
  className = ''
}: Web3OpenIDSignInProps) {
  const { authenticateWithDirectApi } = useSharedAuth();

  const [currentStep, setCurrentStep] = useState<AuthStep>('connect');
  const [walletAddress, setWalletAddress] = useState<string>('');
  const [error, setError] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [challenge, setChallenge] = useState<{ nonce: string; message: string; wallet_address: string } | null>(null);

  // Connect wallet and request challenge
  const handleConnectWallet = useCallback(async () => {
    try {
      setError('');
      setIsLoading(true);
      setCurrentStep('connect');

      // Check if MetaMask is available
      if (typeof window.ethereum === 'undefined') {
        throw new Error('MetaMask is not installed. Please install MetaMask to continue.');
      }

      logger.info('Requesting wallet connection');

      // Request account access
      const accounts = await window.ethereum.request({
        method: 'eth_requestAccounts'
      });

      if (!accounts || accounts.length === 0) {
        throw new Error('No wallet accounts found. Please connect your wallet.');
      }

      const address = accounts[0];
      setWalletAddress(address);

      logger.info('Wallet connected, requesting challenge', { wallet_address: address });

      // Request SIWE challenge from backend
      setCurrentStep('challenge');
      const challengeResponse = await requestWalletChallenge(address);

      setChallenge({
        nonce: challengeResponse.nonce,
        message: challengeResponse.message,
        wallet_address: challengeResponse.wallet_address
      });

      logger.info('Challenge received, ready for signing');

      // Automatically proceed to signing
      await handleSignMessage(challengeResponse, address);

    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to connect wallet';
      logger.error('Wallet connection failed', { error: errorMessage });
      setError(errorMessage);
      setCurrentStep('error');
      setIsLoading(false);
    }
  }, []);

  // Sign the challenge message
  const handleSignMessage = useCallback(async (
    challengeData: { nonce: string; message: string; wallet_address: string },
    address: string
  ) => {
    try {
      setCurrentStep('signing');

      logger.info('Requesting message signature', { wallet_address: address });

      // Request signature from wallet
      const signature = await window.ethereum.request({
        method: 'personal_sign',
        params: [challengeData.message, address],
      });

      logger.info('Signature received, authenticating with backend');

      // Authenticate with backend using direct API
      setCurrentStep('authenticating');
      const result = await verifyWalletSignature({
        wallet_address: address,
        signature,
        message: challengeData.message,
        nonce: challengeData.nonce
      });

      if (result.success) {
        logger.info('🎉 Authentication successful!', {
          wallet: result.wallet_address,
          tier: result.tier_level,
          permissions: result.permissions?.length || 0,
          isNewUser: result.is_new_user
        });

        // Update SharedOpenIDWeb3Provider with authenticated user
        await authenticateWithDirectApi({
          wallet_address: result.wallet_address,
          permissions: result.permissions,
          tier_level: result.tier_level,
          is_new_user: result.is_new_user,
          access_token: result.access_token
        });

        setCurrentStep('success');
        setIsLoading(false);

        // Call onSuccess callback if provided (no redirect)
        if (onSuccess) {
          onSuccess();
        }
      } else {
        throw new Error(result.error || 'Authentication failed');
      }

    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Signature or authentication failed';
      logger.error('Authentication process failed', { error: errorMessage });
      setError(errorMessage);
      setCurrentStep('error');
      setIsLoading(false);
    }
  }, [authenticateWithDirectApi, onSuccess]);

  // Retry authentication
  const handleRetry = useCallback(() => {
    setError('');
    setChallenge(null);
    setWalletAddress('');
    setCurrentStep('connect');
  }, []);

  // Render step content
  const renderStepContent = () => {
    switch (currentStep) {
      case 'connect':
        return (
          <>
            <div className="flex flex-col items-center space-y-4">
              <Wallet className="h-12 w-12 text-blue-500" />
              <div className="text-center">
                <h3 className="text-lg font-semibold">Connect Your Wallet</h3>
                <p className="text-sm text-gray-600 mt-1">
                  Connect your Web3 wallet to sign in with EPSX
                </p>
              </div>
              <Button 
                onClick={handleConnectWallet} 
                disabled={isLoading}
                className="w-full"
              >
                {isLoading ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Connecting...
                  </>
                ) : (
                  <>
                    <Wallet className="mr-2 h-4 w-4" />
                    Connect Wallet
                  </>
                )}
              </Button>
            </div>
          </>
        );

      case 'challenge':
        return (
          <div className="flex flex-col items-center space-y-4">
            <Loader2 className="h-12 w-12 text-blue-500 animate-spin" />
            <div className="text-center">
              <h3 className="text-lg font-semibold">Preparing Authentication</h3>
              <p className="text-sm text-gray-600 mt-1">
                Generating secure challenge for {walletAddress}
              </p>
            </div>
          </div>
        );

      case 'signing':
        return (
          <div className="flex flex-col items-center space-y-4">
            <Loader2 className="h-12 w-12 text-orange-500 animate-spin" />
            <div className="text-center">
              <h3 className="text-lg font-semibold">Sign the Message</h3>
              <p className="text-sm text-gray-600 mt-1">
                Please sign the message in your wallet to continue
              </p>
              <p className="text-xs text-gray-500 mt-2">
                This signature proves you own the wallet address
              </p>
            </div>
          </div>
        );

      case 'authenticating':
        return (
          <div className="flex flex-col items-center space-y-4">
            <Loader2 className="h-12 w-12 text-green-500 animate-spin" />
            <div className="text-center">
              <h3 className="text-lg font-semibold">Authenticating</h3>
              <p className="text-sm text-gray-600 mt-1">
                Verifying your signature and issuing access tokens
              </p>
            </div>
          </div>
        );

      case 'success':
        return (
          <div className="flex flex-col items-center space-y-4">
            <CheckCircle className="h-12 w-12 text-green-500" />
            <div className="text-center">
              <h3 className="text-lg font-semibold text-green-700">Authentication Successful!</h3>
              <p className="text-sm text-gray-600 mt-1">
                Welcome to EPSX! Redirecting you now...
              </p>
            </div>
          </div>
        );

      case 'error':
        return (
          <div className="flex flex-col items-center space-y-4">
            <AlertCircle className="h-12 w-12 text-red-500" />
            <div className="text-center">
              <h3 className="text-lg font-semibold text-red-700">Authentication Failed</h3>
              <p className="text-sm text-gray-600 mt-1">
                Please try again or check your wallet connection
              </p>
            </div>
            <Button onClick={handleRetry} variant="outline" className="w-full">
              Try Again
            </Button>
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <div className={`flex items-center justify-center min-h-screen bg-gray-50 p-4 ${className}`}>
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <CardTitle className="text-2xl font-bold">EPSX Authentication</CardTitle>
          <CardDescription>
            Secure Web3 authentication powered by OpenID Connect
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {error && (
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
          
          {renderStepContent()}
          
          {walletAddress && currentStep !== 'error' && (
            <div className="bg-gray-50 p-3 rounded-lg">
              <p className="text-xs text-gray-500 mb-1">Connected Wallet:</p>
              <p className="text-sm font-mono break-all">{walletAddress}</p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

export default Web3OpenIDSignIn;